mod input;
mod output;

pub use self::{input::UpdateApplicationEntityInput, output::UpdateApplicationEntityOutput};

use crate::{
    AppState,
    internal::{
        application::application_entity::update_application_entity_use_case::UpdateApplicationEntityCommand,
        domain::value_object::{HostName, Port},
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use chrono::Utc;
use dicom_lib::core::value::value_representations::ae::AeValue;

#[utoipa::path(
    put,
    path = "/application-entities/{ae_title}",
    request_body = UpdateApplicationEntityInput,
    params(
        ("ae_title" = String, Path, description = "AE Title")
    ),
    responses(
        (status = 200, description = "Application Entityの更新に成功", body = UpdateApplicationEntityOutput),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 422, description = "バリデーション失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "application-entities"
)]
pub async fn update_application_entity(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(ae_title): Path<String>,
    Json(input): Json<UpdateApplicationEntityInput>,
) -> Result<Json<UpdateApplicationEntityOutput>, PresentationError> {
    // バリデーション
    let old_title = AeValue::from_string(&ae_title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;
    let title = AeValue::from_string(&input.title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;
    let host = HostName::new(&input.host)
        .map_err(|e| PresentationError::UnprocessableContent(format!("ホスト名が不正です: {e}")))?;
    let port = Port::from_u16(input.port).map_err(|e| {
        PresentationError::UnprocessableContent(format!("ポート番号が不正です: {e}"))
    })?;

    // 更新処理
    let command = UpdateApplicationEntityCommand {
        old_title,
        title,
        host,
        port,
        comment: input.comment,
        updated_by: user.uuid(),
        updated_at: Utc::now(),
    };
    let entity = state
        .update_application_entity_use_case
        .execute(command)
        .await
        .map_err(PresentationError::from)?;

    let output = UpdateApplicationEntityOutput::from(entity);

    Ok(Json(output))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use crate::{
        internal::{
            domain::{
                entity::ApplicationEntity,
                value_object::{HostName, Port},
            },
            presentation::{handler::application_entity::prepare_test_data, util::test_helpers},
        },
        utils::{self, make_router},
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use chrono::{DateTime, Utc};
    use dicom_lib::core::value::value_representations::ae::AeValue;
    use futures::future::JoinAll;
    use serde_json::{Value, json};
    use std::str::FromStr;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn 管理者はAEを更新できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "title": "OsiriX",
            "host": "192.0.2.1",
            "port": 104,
            "comment": "DCMTKから変更しました",
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/application-entities/DCMTK")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // レスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["title"], "OsiriX");
        assert_eq!(body["host"], "192.0.2.1");
        assert_eq!(body["port"], 104);
        assert_eq!(body["comment"], "DCMTKから変更しました");
        let created_at = DateTime::<Utc>::from_str(body["createdAt"].as_str().unwrap()).unwrap();
        assert_eq!(
            created_at,
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
        let updated_at = DateTime::<Utc>::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        let now = Utc::now();
        assert!((now - updated_at).num_seconds().abs() < 10);
        // リポジトリの確認
        let stored = repos
            .application_entity_repository
            .find_by_title(&AeValue::from_string("OsiriX").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.host().value(), "192.0.2.1");
        assert_eq!(stored.port().value(), 104);
        assert_eq!(stored.comment(), "DCMTKから変更しました");
        assert_eq!(
            *stored.created_at(),
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
        assert_eq!(*stored.updated_at(), updated_at);
    }

    #[tokio::test]
    async fn 情シスはAEを更新できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "title": "OsiriX",
            "host": "192.0.2.1",
            "port": 104,
            "comment": "DCMTKから変更しました",
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/application-entities/DCMTK")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // レスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["title"], "OsiriX");
        assert_eq!(body["host"], "192.0.2.1");
        assert_eq!(body["port"], 104);
        assert_eq!(body["comment"], "DCMTKから変更しました");
        let created_at = DateTime::<Utc>::from_str(body["createdAt"].as_str().unwrap()).unwrap();
        assert_eq!(
            created_at,
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
        let updated_at = DateTime::<Utc>::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        let now = Utc::now();
        assert!((now - updated_at).num_seconds().abs() < 10);
        // リポジトリの確認
        let stored = repos
            .application_entity_repository
            .find_by_title(&AeValue::from_string("OsiriX").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.host().value(), "192.0.2.1");
        assert_eq!(stored.port().value(), 104);
        assert_eq!(stored.comment(), "DCMTKから変更しました");
        assert_eq!(
            *stored.created_at(),
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
        assert_eq!(*stored.updated_at(), updated_at);
    }

    #[tokio::test]
    async fn 内容に変更がない場合は更新されない() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({ // すでに存在するAEと同じ内容
            "title": "DCMTK",
            "host": "localhost",
            "port": 11112,
            "comment": "開発＆デバッグ用",
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/application-entities/DCMTK")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // レスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        let updated_at = DateTime::<Utc>::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        assert_eq!(
            updated_at,
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
        // リポジトリの確認
        let stored = repos
            .application_entity_repository
            .find_by_title(&AeValue::from_string("DCMTK").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            *stored.updated_at(),
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
    }

    #[tokio::test]
    async fn 存在しないAEを更新しようとすると404エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "title": "OsiriX",
            "host": "192.0.2.1",
            "port": 104,
            "comment": "",
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/application-entities/OsiriX")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn すでに存在するAEと競合する形でAEを更新しようとすると409エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        // あらかじめ別のAEを登録しておく
        repos
            .application_entity_repository
            .add(&ApplicationEntity::create(
                AeValue::from_string("OsiriX").unwrap(),
                HostName::new("192.0.2.1").unwrap(),
                Port::from_u16(104).unwrap(),
                "",
                Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
                Utc::now(),
            ))
            .await
            .unwrap();
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let inputs = [
            json!({ // タイトルが既存と競合
                "title": "OsiriX",
                "host": "192.0.2.2",
                "port": 11112,
                "comment": "",
            }),
            json!({ // ホスト名とポート番号の組が既存と競合
                "title": "DCMTK",
                "host": "192.0.2.1",
                "port": 104,
                "comment": "",
            }),
        ];
        let requests = inputs.iter().map(|input| {
            Request::builder()
                .method("PUT")
                .uri("/application-entities/DCMTK")
                .header("content-type", "application/json")
                .header("cookie", format!("session_id={session_id}"))
                .header("x-csrf-token", &csrf_token)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap()
        });

        // Act
        let responses = requests
            .map(|request| router.clone().oneshot(request))
            .collect::<JoinAll<_>>()
            .await;

        // Assert
        responses.into_iter().for_each(|result| {
            let response = result.unwrap();
            assert_eq!(response.status(), StatusCode::CONFLICT);
        });
    }

    #[tokio::test]
    async fn リクエストボディのバリデーション違反の場合に422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let inputs = [
            json!({ // タイトルが空文字
                "title": "",
                "host": "192.0.2.1",
                "port": 104,
                "comment": "",
            }),
            json!({ // タイトルが長すぎる(17文字)
                "title": "12345678901234567",
                "host": "192.0.2.1",
                "port": 104,
                "comment": "",
            }),
            json!({ // ホスト名が空文字
                "title": "OsiriX",
                "host": "",
                "port": 104,
                "comment": "",
            }),
            json!({ // ホスト名に不正な文字が含まれる
                "title": "OsiriX",
                "host": "invalid_host_name!",
                "port": 104,
                "comment": "",
            }),
            json!({ // ポート番号が不正(0)
                "title": "OsiriX",
                "host": "192.0.2.1",
                "port": 0,
                "comment": "",
            }),
            json!({ // ポート番号が不正(65536)
                "title": "OsiriX",
                "host": "192.0.2.1",
                "port": 65536,
                "comment": "",
            }),
        ];
        let requests = inputs.iter().map(|input| {
            Request::builder()
                .method("PUT")
                .uri("/application-entities/DCMTK")
                .header("content-type", "application/json")
                .header("cookie", format!("session_id={session_id}"))
                .header("x-csrf-token", &csrf_token)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap()
        });

        // Act
        let responses = requests
            .map(|request| router.clone().oneshot(request))
            .collect::<JoinAll<_>>()
            .await;

        // Assert
        responses.into_iter().for_each(|result| {
            let response = result.unwrap();
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        });
    }
}
