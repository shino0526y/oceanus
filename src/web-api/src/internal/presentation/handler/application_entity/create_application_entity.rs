mod input;
mod output;

pub use self::{input::CreateApplicationEntityInput, output::CreateApplicationEntityOutput};

use crate::{
    internal::{
        application::application_entity::create_application_entity_use_case::CreateApplicationEntityCommand,
        domain::value_object::{HostName, Port},
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
    startup::AppState,
};
use axum::{Extension, Json, extract::State};
use chrono::Utc;
use dicom_lib::core::value::value_representations::ae::AeValue;

#[utoipa::path(
    post,
    path = "/application-entities",
    request_body = CreateApplicationEntityInput,
    responses(
        (status = 200, description = "Application Entityの作成に成功", body = CreateApplicationEntityOutput),
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
pub async fn create_application_entity(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(input): Json<CreateApplicationEntityInput>,
) -> Result<Json<CreateApplicationEntityOutput>, PresentationError> {
    // バリデーション
    let title = AeValue::from_string(&input.title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;
    let host = HostName::new(&input.host)
        .map_err(|e| PresentationError::UnprocessableContent(format!("ホスト名が不正です: {e}")))?;
    let port = Port::from_u16(input.port).map_err(|e| {
        PresentationError::UnprocessableContent(format!("ポート番号が不正です: {e}"))
    })?;

    // 登録処理
    let command = CreateApplicationEntityCommand {
        title,
        host,
        port,
        comment: input.comment,
        created_by: user.uuid(),
        created_at: Utc::now(),
    };
    let entity = state
        .create_application_entity_use_case
        .execute(command)
        .await
        .map_err(PresentationError::from)?;

    let output = CreateApplicationEntityOutput::from(entity);

    Ok(Json(output))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{internal::presentation::util::test_helpers, startup};
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
    async fn 管理者はアプリケーションエンティティを作成できる() {
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
            .method("POST")
            .uri("/application-entities")
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
        assert_eq!(body["comment"], "");

        let created_at = DateTime::from_str(body["createdAt"].as_str().unwrap()).unwrap();
        let updated_at = DateTime::<Utc>::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        let now = Utc::now();
        assert!((now - created_at).num_seconds().abs() < 10);
        assert_eq!(created_at, updated_at);

        // リポジトリ内に正しく保存されていることの確認
        let stored = repos
            .application_entity_repository
            .find_by_title(&AeValue::from_string("OsiriX").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.host().value(), "192.0.2.1");
        assert_eq!(stored.port().value(), 104);
        assert_eq!(stored.comment(), "");
        assert_eq!(
            stored.created_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert_eq!(*stored.created_at(), created_at);
        assert_eq!(*stored.updated_at(), updated_at);
        assert!((now - *stored.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 情シスはアプリケーションエンティティを作成できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let body = json!({
            "title": "OsiriX",
            "host": "192.0.2.1",
            "port": 104,
            "comment": "",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/application-entities")
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
        assert_eq!(body["comment"], "");

        let created_at = DateTime::from_str(body["createdAt"].as_str().unwrap()).unwrap();
        let updated_at = DateTime::<Utc>::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        let now = Utc::now();
        assert!((now - created_at).num_seconds().abs() < 10);
        assert_eq!(created_at, updated_at);

        // リポジトリ内に正しく保存されていることの確認
        let stored = repos
            .application_entity_repository
            .find_by_title(&AeValue::from_string("OsiriX").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(stored.host().value(), "192.0.2.1");
        assert_eq!(stored.port().value(), 104);
        assert_eq!(stored.comment(), "");
        assert_eq!(
            stored.created_by(),
            &Uuid::parse_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap()
        );
        assert_eq!(*stored.created_at(), created_at);
        assert_eq!(*stored.updated_at(), updated_at);
        assert!((now - *stored.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 管理者でも情シスでもないユーザーがAEを作成しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) =
            test_helpers::login(&router, "technician", "Password#1234").await;
        let body = json!({
            "title": "OsiriX",
            "host": "192.0.2.1",
            "port": 104,
            "comment": "",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/application-entities")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn すでに存在するAEと競合するAEを作成しようとすると409エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let bodies = [
            json!({
                "title": "DCMTK", // タイトルが既存と競合
                "host": "192.0.2.1",
                "port": 104,
                "comment": "開発＆デバッグ用",
            }),
            json!({
                "title": "OsiriX",
                "host": "localhost", // ホスト名とポート番号の組が既存と競合
                "port": 11112,
                "comment": "開発＆デバッグ用",
            }),
        ];
        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("POST")
                .uri("/application-entities")
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
            json!({ // タイトルのフィールドがない
                "host": "192.0.2.1",
                "port": 104,
                "comment": "",
            }),
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
            json!({ // ホスト名のフィールドがない
                "title": "OsiriX",
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
            json!({ // ポート番号のフィールドがない
                "title": "OsiriX",
                "host": "192.0.2.1",
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
            json!({ // コメントのフィールドがない
                "title": "OsiriX",
                "host": "192.0.2.1",
                "port": 104,
            }),
        ];
        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("POST")
                .uri("/application-entities")
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
