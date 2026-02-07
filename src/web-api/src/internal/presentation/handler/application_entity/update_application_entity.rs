mod request_body;
mod response_body;

pub use self::{
    request_body::UpdateApplicationEntityRequestBody,
    response_body::UpdateApplicationEntityResponseBody,
};

use crate::{
    internal::{
        application::application_entity::update_application_entity_use_case::UpdateApplicationEntityCommand,
        domain::value_object::{HostName, Port},
        presentation::{
            error::{ErrorResponseBody, PresentationError},
            middleware::AuthenticatedUser,
        },
    },
    startup::AppState,
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
    request_body = UpdateApplicationEntityRequestBody,
    params(
        ("ae_title" = String, Path, description = "AE Title")
    ),
    responses(
        (status = 200, description = "AEの更新に成功", body = UpdateApplicationEntityResponseBody),
        (status = 400, description = "リクエストの形式が無効", body = ErrorResponseBody),
        (status = 401, description = "セッションが確立されていないか期限が切れている", body = ErrorResponseBody),
        (status = 403, description = "CSRFトークンが無効または権限がない", body = ErrorResponseBody),
        (status = 404, description = "対象のAEが見つからない", body = ErrorResponseBody),
        (status = 409, description = "競合するAEが既に存在する", body = ErrorResponseBody),
        (status = 422, description = "バリデーション失敗", body = ErrorResponseBody),
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
    Json(request_body): Json<UpdateApplicationEntityRequestBody>,
) -> Result<Json<UpdateApplicationEntityResponseBody>, PresentationError> {
    // バリデーション
    let old_title = AeValue::from_string(&ae_title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;
    let title = AeValue::from_string(&request_body.title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;
    let host = HostName::new(&request_body.host)
        .map_err(|e| PresentationError::UnprocessableContent(format!("ホスト名が不正です: {e}")))?;
    let port = Port::from_u16(request_body.port).map_err(|e| {
        PresentationError::UnprocessableContent(format!("ポート番号が不正です: {e}"))
    })?;

    // 更新処理
    let command = UpdateApplicationEntityCommand {
        old_title,
        title,
        host,
        port,
        comment: request_body.comment,
        updated_by: user.uuid(),
        updated_at: Utc::now(),
    };
    let entity = state
        .update_application_entity_use_case
        .execute(command)
        .await
        .map_err(PresentationError::from)?;

    let response_body = UpdateApplicationEntityResponseBody::from(entity);

    Ok(Json(response_body))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{
        internal::{
            domain::{
                entity::ApplicationEntity,
                value_object::{HostName, Port},
            },
            presentation::util::test_helpers,
        },
        startup,
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use chrono::{DateTime, Utc};
    use dicom_lib::core::value::value_representations::ae::AeValue;
    use futures::future::join_all;
    use serde_json::{Value, json};
    use std::str::FromStr;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn 管理者はAEを更新できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let body = json!({
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
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::OK);

        // レスポンスボディの確認
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
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let body = json!({
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
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::OK);

        // レスポンスボディの確認
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
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let body = json!({ // すでに存在するAEと同じ内容
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
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::OK);

        // レスポンスボディの確認
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
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let body = json!({
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
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn すでに存在するAEと競合する形でAEを更新しようとすると409エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        // あらかじめ別のAEを登録しておく
        repos
            .application_entity_repository
            .add(&ApplicationEntity::create(
                AeValue::from_string("OsiriX").unwrap(),
                HostName::new("192.0.2.1").unwrap(),
                Port::from_u16(104).unwrap(),
                "",
                Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
                Utc::now(),
            ))
            .await
            .unwrap();

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let bodies = [
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
        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("PUT")
                .uri("/application-entities/DCMTK")
                .header("content-type", "application/json")
                .header("cookie", format!("session_id={session_id}"))
                .header("x-csrf-token", &csrf_token)
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        });

        // Act
        let responses = join_all(requests.map(|req| router.clone().oneshot(req))).await;

        // Assert
        responses.into_iter().for_each(|res| {
            // ステータスコードの確認
            assert_eq!(res.unwrap().status(), StatusCode::CONFLICT);
        });
    }

    #[tokio::test]
    async fn リクエストボディのバリデーション違反の場合に422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let bodies = [
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
        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("PUT")
                .uri("/application-entities/DCMTK")
                .header("content-type", "application/json")
                .header("cookie", format!("session_id={session_id}"))
                .header("x-csrf-token", &csrf_token)
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        });

        // Act
        let responses = join_all(requests.map(|req| router.clone().oneshot(req))).await;

        // Assert
        responses.into_iter().for_each(|res| {
            // ステータスコードの確認
            assert_eq!(res.unwrap().status(), StatusCode::UNPROCESSABLE_ENTITY);
        });
    }
}
