mod request_body;
mod response_body;

pub use self::{request_body::UpdateUserRequestBody, response_body::UpdateUserResponseBody};

use crate::{
    internal::{
        application::user::update_user_use_case::{UpdateUserCommand, UpdateUserError},
        domain::value_object::{Id, Role, UserName},
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

#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUserRequestBody,
    params(
        ("id" = String, Path, description = "ユーザーID")
    ),
    responses(
        (status = 200, description = "ユーザーの更新に成功", body = UpdateUserResponseBody),
        (status = 400, description = "リクエストの形式が無効", body = ErrorResponseBody),
        (status = 401, description = "セッションが確立されていないか期限が切れている", body = ErrorResponseBody),
        (status = 403, description = "CSRFトークンが無効または権限がない", body = ErrorResponseBody),
        (status = 404, description = "対象のユーザーが見つからない", body = ErrorResponseBody),
        (status = 409, description = "ユーザーIDが既に存在しています", body = ErrorResponseBody),
        (status = 422, description = "バリデーションに失敗", body = ErrorResponseBody),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
    Json(request_body): Json<UpdateUserRequestBody>,
) -> Result<Json<UpdateUserResponseBody>, PresentationError> {
    // バリデーション
    let old_id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {e}")))?;
    let new_id = Id::new(request_body.id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {e}")))?;
    let name = UserName::new(request_body.name)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効な名前: {e}")))?;
    let role = Role::from_i16(request_body.role)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なロール: {e}")))?;

    // 更新処理
    let command = UpdateUserCommand {
        old_id,
        id: new_id,
        name,
        role,
        password: request_body.password,
        updated_by: user.uuid(),
        updated_at: Utc::now(),
    };
    let user = state
        .update_user_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            UpdateUserError::EmptyPassword => {
                PresentationError::UnprocessableContent(e.to_string())
            }
            UpdateUserError::Forbidden => PresentationError::Forbidden(e.to_string()),
            UpdateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {msg}"),
            ),
            UpdateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(UpdateUserResponseBody::from(user)))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{
        internal::{domain::value_object::Id, presentation::util::test_helpers},
        startup,
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use chrono::{DateTime, Utc};
    use futures::future::join_all;
    use serde_json::{Value, json};
    use std::str::FromStr;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn 管理者はユーザー名とロールを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let body = json!({
            "id": "doctor",
            "name": "John Doe", // 名前を変更
            "role": 3, // ロールを技師に変更
            "password": null // パスワードは変更しない
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["id"], "doctor");
        assert_eq!(body["name"], "John Doe");
        assert_eq!(body["role"], 3);
        assert_eq!(body["createdAt"], "2026-01-24T13:25:57.855Z");
        let now = Utc::now();
        let updated_at = DateTime::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        assert!((now - updated_at).num_seconds().abs() < 10);

        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.id().value(), "doctor");
        assert_eq!(user.name().value(), "John Doe");
        assert_eq!(user.role().as_i16(), 3);
        assert_eq!(
            user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA"
        );
        assert_eq!(
            user.created_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert_eq!(
            user.created_at().to_rfc3339(),
            "2026-01-24T13:25:57.855+00:00"
        );
        assert_eq!(
            user.updated_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 管理者はユーザーIDを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let body = json!({
            "id": "john", // IDを変更
            "name": "医師 太郎",
            "role": 2,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["id"], "john");
        assert_eq!(body["name"], "医師 太郎");
        assert_eq!(body["role"], 2);
        assert_eq!(body["createdAt"], "2026-01-24T13:25:57.855Z");
        let now = Utc::now();
        let updated_at = DateTime::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        assert!((now - updated_at).num_seconds().abs() < 10);

        let old_user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap();
        assert!(old_user.is_none());

        let new_user = repos
            .user_repository
            .find_by_id(&Id::new("john").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(new_user.id().value(), "john");
        assert_eq!(new_user.name().value(), "医師 太郎");
        assert_eq!(new_user.role().as_i16(), 2);
        assert_eq!(
            new_user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA"
        );
        assert_eq!(
            new_user.created_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert_eq!(
            new_user.created_at().to_rfc3339(),
            "2026-01-24T13:25:57.855+00:00"
        );
        assert_eq!(
            new_user.updated_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert!((now - *new_user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 管理者はパスワードを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let body = json!({
            "id": "doctor",
            "name": "医師 太郎",
            "role": 2,
            "password": "NewPassword#5678" // パスワードを変更
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["id"], "doctor");
        assert_eq!(body["name"], "医師 太郎");
        assert_eq!(body["role"], 2);
        assert_eq!(body["createdAt"], "2026-01-24T13:25:57.855Z");
        let now = Utc::now();
        let updated_at = DateTime::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        assert!((now - updated_at).num_seconds().abs() < 10);

        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.id().value(), "doctor");
        assert_eq!(user.name().value(), "医師 太郎");
        assert_eq!(user.role().as_i16(), 2);
        assert_ne!(
            user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA"
        );
        assert_eq!(
            user.created_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert_eq!(
            user.created_at().to_rfc3339(),
            "2026-01-24T13:25:57.855+00:00"
        );
        assert_eq!(
            user.updated_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 情シスはユーザー名とロールを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let body = json!({
            "id": "doctor",
            "name": "John Doe", // 名前を変更
            "role": 3, // ロールを技師に変更
            "password": null // パスワードは変更しない
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["id"], "doctor");
        assert_eq!(body["name"], "John Doe");
        assert_eq!(body["role"], 3);
        assert_eq!(body["createdAt"], "2026-01-24T13:25:57.855Z");
        let now = Utc::now();
        let updated_at = DateTime::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        assert!((now - updated_at).num_seconds().abs() < 10);

        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.id().value(), "doctor");
        assert_eq!(user.name().value(), "John Doe");
        assert_eq!(user.role().as_i16(), 3);
        assert_eq!(
            user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA"
        );
        assert_eq!(
            user.created_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert_eq!(
            user.created_at().to_rfc3339(),
            "2026-01-24T13:25:57.855+00:00"
        );
        assert_eq!(
            user.updated_by(),
            &Uuid::parse_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 情シスはユーザーIDを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let body = json!({
            "id": "john", // IDを変更
            "name": "医師 太郎",
            "role": 2,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["id"], "john");
        assert_eq!(body["name"], "医師 太郎");
        assert_eq!(body["role"], 2);
        assert_eq!(body["createdAt"], "2026-01-24T13:25:57.855Z");
        let now = Utc::now();
        let updated_at = DateTime::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        assert!((now - updated_at).num_seconds().abs() < 10);

        let old_user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap();
        assert!(old_user.is_none());

        let new_user = repos
            .user_repository
            .find_by_id(&Id::new("john").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(new_user.id().value(), "john");
        assert_eq!(new_user.name().value(), "医師 太郎");
        assert_eq!(new_user.role().as_i16(), 2);
        assert_eq!(
            new_user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA"
        );
        assert_eq!(
            new_user.created_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert_eq!(
            new_user.created_at().to_rfc3339(),
            "2026-01-24T13:25:57.855+00:00"
        );
        assert_eq!(
            new_user.updated_by(),
            &Uuid::parse_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap()
        );
        assert!((now - *new_user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 情シスはパスワードを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let body = json!({
            "id": "doctor",
            "name": "医師 太郎",
            "role": 2,
            "password": "NewPassword#5678" // パスワードを変更
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);

        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["id"], "doctor");
        assert_eq!(body["name"], "医師 太郎");
        assert_eq!(body["role"], 2);
        assert_eq!(body["createdAt"], "2026-01-24T13:25:57.855Z");
        let now = Utc::now();
        let updated_at = DateTime::from_str(body["updatedAt"].as_str().unwrap()).unwrap();
        assert!((now - updated_at).num_seconds().abs() < 10);

        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.id().value(), "doctor");
        assert_eq!(user.name().value(), "医師 太郎");
        assert_eq!(user.role().as_i16(), 2);
        assert_ne!(
            user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA"
        );
        assert_eq!(
            user.created_by(),
            &Uuid::parse_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert_eq!(
            user.created_at().to_rfc3339(),
            "2026-01-24T13:25:57.855+00:00"
        );
        assert_eq!(
            user.updated_by(),
            &Uuid::parse_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 管理者でも情シスでもないユーザーが他のユーザーを変更しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) =
            test_helpers::login(&router, "technician", "Password#1234").await;
        let body = json!({
            "id": "doctor",
            "name": "John Doe",
            "role": 1,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // リポジトリが変更されていないことを確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.name().value(), "医師 太郎");
        assert_eq!(user.role().as_i16(), 2);
    }

    #[tokio::test]
    async fn 情シスが管理者ユーザーを変更しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let bodies = [
            json!({
                "id": "admin",
                "name": "John Doe", // 名前を変更しようとする
                "role": 0,
                "password": null
            }),
            json!({
                "id": "admin",
                "name": "管理者 太郎",
                "role": 1, // 管理者を情シスに格下げしようとする
                "password": null
            }),
            json!({
                "id": "admin",
                "name": "管理者 太郎",
                "role": 0,
                "password": "NewPassword#5678" // 管理者のパスワードを変更しようとする
            }),
        ];
        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("PUT")
                .uri("/users/admin")
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
            assert_eq!(res.unwrap().status(), StatusCode::FORBIDDEN);
        });

        // リポジトリが変更されていないことを確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("admin").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.role().as_i16(), 0);
        assert_eq!(user.name().value(), "管理者 太郎");
    }

    #[tokio::test]
    async fn 情シスが管理者でないユーザーを管理者ユーザーに変更しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let body = json!({
            "id": "doctor",
            "name": "医師 太郎",
            "role": 0, // 管理者に変更しようとする
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        // リポジトリが変更されていないことを確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.role().as_i16(), 2);
    }

    #[tokio::test]
    async fn 存在しないユーザーを変更しようとすると404エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let body = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/john")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn すでに存在するユーザーと競合する内容でユーザーを更新しようとすると409エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let bodies = [
            json!({ // IDが既存ユーザーと競合
                "id": "technician", // 既存の技師ユーザーと同じID
                "name": "John Doe",
                "role": 3,
                "password": "Password#1234",
            }),
            json!({ // 名前が既存ユーザーと競合
                "id": "john",
                "name": "技師 太郎", // 既存の技師ユーザーと同じ名前
                "role": 3,
                "password": "Password#1234",
            }),
        ];

        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("PUT")
                .uri("/users/doctor") // 既存の医師ユーザーを更新しようとする
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
            assert_eq!(res.unwrap().status(), StatusCode::CONFLICT);
        });

        // リポジトリが変更されていないことを確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.name().value(), "医師 太郎");
    }

    #[tokio::test]
    async fn リクエストボディのバリデーション違反の場合に422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let bodies = [
            json!({ // IDのフィールドがない
                "name": "John Doe",
                "role": 2,
                "password": null,
            }),
            json!({ // IDが空文字
                "id": "",
                "name": "John Doe",
                "role": 2,
                "password": null,
            }),
            json!({ // IDに空白を含む
                "id": "john doe",
                "name": "John Doe",
                "role": 2,
                "password": null,
            }),
            json!({ // 名前のフィールドがない
                "id": "john",
                "role": 2,
                "password": null,
            }),
            json!({ // 名前が空文字
                "id": "john",
                "name": "",
                "role": 2,
                "password": null,
            }),
            json!({ // 名前が空白のみ
                "id": "john",
                "name": " ",
                "role": 2,
                "password": null,
            }),
            json!({ // ロールのフィールドがない
                "id": "john",
                "name": "John Doe",
                "password": null,
            }),
            json!({ // ロールが負の値
                "id": "john",
                "name": "John Doe",
                "role": -1,
                "password": null,
            }),
            json!({ // ロールが5以上の値
                "id": "john",
                "name": "John Doe",
                "role": 5,
                "password": null,
            }),
            json!({ // パスワードが空文字
                "id": "john",
                "name": "John Doe",
                "role": 2,
                "password": "",
            }),
        ];

        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("PUT")
                .uri("/users/doctor")
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
            assert_eq!(res.unwrap().status(), StatusCode::UNPROCESSABLE_ENTITY);
        });
    }
}
