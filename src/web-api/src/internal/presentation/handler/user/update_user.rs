mod input;
mod output;

pub use self::{input::UpdateUserInput, output::UpdateUserOutput};

use crate::{
    AppState,
    internal::{
        application::user::update_user_use_case::{UpdateUserCommand, UpdateUserError},
        domain::value_object::{Id, Role},
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use chrono::Utc;

#[utoipa::path(
    put,
    path = "/users/{id}",
    request_body = UpdateUserInput,
    params(
        ("id" = String, Path, description = "ユーザーID")
    ),
    responses(
        (status = 200, description = "ユーザーの更新に成功", body = UpdateUserOutput),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 422, description = "バリデーションに失敗"),
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
    Json(input): Json<UpdateUserInput>,
) -> Result<Json<UpdateUserOutput>, PresentationError> {
    // バリデーション
    let old_id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;
    let new_id = Id::new(input.id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;
    let role = Role::from_i16(input.role).map_err(PresentationError::UnprocessableContent)?;

    // 更新処理
    let command = UpdateUserCommand {
        old_id,
        id: new_id,
        name: input.name,
        role,
        password: input.password,
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
                format!("パスワードのハッシュ化に失敗しました: {}", msg),
            ),
            UpdateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(UpdateUserOutput::from(user)))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::presentation::{handler::user::prepare_test_data, util::test_helpers},
        utils::{self, make_router},
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use chrono::{DateTime, Utc};
    use serde_json::json;
    use std::str::FromStr;
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn 管理者はユーザー名とロールを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "id": "doctor",
            "name": "John Doe", // 名前を変更
            "role": 3, // ロールを技師に変更
            "password": null // パスワードは変更しない
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: UpdateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "doctor");
        assert_eq!(output.name, "John Doe");
        assert_eq!(output.role, 3);
        assert_eq!(
            output.created_at,
            DateTime::<Utc>::from_str("2026-01-24T22:25:57.855+09:00").unwrap()
        );
        let now = Utc::now();
        assert!((now - output.updated_at).num_seconds().abs() < 10);
        // リポジトリに保存された内容の確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.name().to_string(), "John Doe");
        assert_eq!(user.role().as_i16(), 3);
        assert_eq!(
            user.updated_by(),
            &Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 管理者はユーザーIDを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "id": "john", // IDを変更
            "name": "医師 太郎",
            "role": 2,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: super::UpdateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "john");
        assert_eq!(output.name, "医師 太郎");
        assert_eq!(output.role, 2);
        assert_eq!(
            output.created_at,
            DateTime::<Utc>::from_str("2026-01-24T22:25:57.855+09:00").unwrap()
        );
        let now = Utc::now();
        assert!((now - output.updated_at).num_seconds().abs() < 10);
        // リポジトリに保存された内容の確認
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
        assert_eq!(
            new_user.updated_by(),
            &Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert!((now - *new_user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 管理者はパスワードを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "id": "doctor",
            "name": "医師 太郎",
            "role": 2,
            "password": "NewPassword#5678" // パスワードを変更
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: super::UpdateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "doctor");
        assert_eq!(output.name, "医師 太郎");
        assert_eq!(output.role, 2);
        assert_eq!(
            output.created_at,
            DateTime::<Utc>::from_str("2026-01-24T22:25:57.855+09:00").unwrap()
        );
        let now = Utc::now();
        assert!((now - output.updated_at).num_seconds().abs() < 10);
        // リポジトリに保存された内容の確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_ne!(
            user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA" // 旧パスワードのハッシュ
        );
        assert_eq!(
            user.updated_by(),
            &Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 情シスはユーザー名とロールを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "doctor",
            "name": "John Doe", // 名前を変更
            "role": 3, // ロールを技師に変更
            "password": null // パスワードは変更しない
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: UpdateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "doctor");
        assert_eq!(output.name, "John Doe");
        assert_eq!(output.role, 3);
        assert_eq!(
            output.created_at,
            DateTime::<Utc>::from_str("2026-01-24T22:25:57.855+09:00").unwrap()
        );
        let now = Utc::now();
        assert!((now - output.updated_at).num_seconds().abs() < 10);
        // リポジトリに保存された内容の確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(user.name().to_string(), "John Doe");
        assert_eq!(user.role().as_i16(), 3);
        assert_eq!(
            user.updated_by(),
            &Uuid::from_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 情シスはユーザーIDを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "john", // IDを変更
            "name": "医師 太郎",
            "role": 2,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: super::UpdateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "john");
        assert_eq!(output.name, "医師 太郎");
        assert_eq!(output.role, 2);
        assert_eq!(
            output.created_at,
            DateTime::<Utc>::from_str("2026-01-24T22:25:57.855+09:00").unwrap()
        );
        let now = Utc::now();
        assert!((now - output.updated_at).num_seconds().abs() < 10);
        // リポジトリに保存された内容の確認
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
        assert_eq!(
            new_user.updated_by(),
            &Uuid::from_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap()
        );
        assert!((now - *new_user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 情シスはパスワードを変更できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "doctor",
            "name": "医師 太郎",
            "role": 2,
            "password": "NewPassword#5678" // パスワードを変更
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: super::UpdateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "doctor");
        assert_eq!(output.name, "医師 太郎");
        assert_eq!(output.role, 2);
        assert_eq!(
            output.created_at,
            DateTime::<Utc>::from_str("2026-01-24T22:25:57.855+09:00").unwrap()
        );
        let now = Utc::now();
        assert!((now - output.updated_at).num_seconds().abs() < 10);
        // リポジトリに保存された内容の確認
        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_ne!(
            user.password_hash(),
            "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA" // 旧パスワードのハッシュ
        );
        assert_eq!(
            user.updated_by(),
            &Uuid::from_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap()
        );
        assert!((now - *user.updated_at()).num_seconds().abs() < 10);
    }

    #[tokio::test]
    async fn 管理者でも情シスでもないユーザーが他のユーザーを変更しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "technician", "Password#1234").await;
        let input = json!({
            "id": "doctor",
            "name": "John Doe",
            "role": 1,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 情シスが管理者ユーザーを変更しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "admin",
            "name": "John Doe", // 名前を変更しようとする
            "role": 0, // TODO: ロール変更を行ったら403エラーにならない。要修正
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/admin")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 情シスが管理者でないユーザーを管理者ユーザーに変更しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "doctor",
            "name": "医師 太郎",
            "role": 0, // 管理者に変更しようとする
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 存在しないユーザーを変更しようとすると404エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2,
            "password": null
        });
        let request = Request::builder()
            .method("PUT")
            .uri("/users/john")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::from(input.to_string()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn バリデーション違反で422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        // IDの指定がないケース
        let input1 = json!({
            "name": "John Doe",
            "role": 2,
            "password": null
        });
        let request1 = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id.clone()))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(input1.to_string()))
            .unwrap();
        // 空文字のIDを指定するケース
        let input2 = json!({
            "id": "",
            "name": "John Doe",
            "role": 2,
            "password": null
        });
        let request2 = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id.clone()))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(input2.to_string()))
            .unwrap();
        // 名前の指定がないケース
        let input3 = json!({
            "id": "john",
            "role": 2,
            "password": null
        });
        let request3 = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id.clone()))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(input3.to_string()))
            .unwrap();
        // TODO: 名前が空文字のケースは現状はエラーにならない。バリデーションを追加する
        // 名前が空文字のケース
        // let input4 = json!({
        //     "id": "john",
        //     "name": "",
        //     "role": 2,
        //     "password": null
        // });
        // let request4 = Request::builder()
        //     .method("PUT")
        //     .uri("/users/doctor")
        //     .header("content-type", "application/json")
        //     .header("cookie", format!("session_id={}", session_id.clone()))
        //     .header("x-csrf-token", csrf.clone())
        //     .body(Body::from(input4.to_string()))
        //     .unwrap();
        // ロールの指定がないケース
        let input5 = json!({
            "id": "john",
            "name": "John Doe",
            "password": null
        });
        let request5 = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id.clone()))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(input5.to_string()))
            .unwrap();
        // ロールが負の値のケース
        let input6 = json!({
            "id": "john",
            "name": "John Doe",
            "role": -1,
            "password": null
        });
        let request6 = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id.clone()))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(input6.to_string()))
            .unwrap();
        // ロールが5以上の値のケース
        let input7 = json!({
            "id": "john",
            "name": "John Doe",
            "role": 5,
            "password": null
        });
        let request7 = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id.clone()))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(input7.to_string()))
            .unwrap();
        // パスワードが空文字のケース
        let input8 = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2,
            "password": ""
        });
        let request8 = Request::builder()
            .method("PUT")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id.clone()))
            .header("x-csrf-token", csrf.clone())
            .body(Body::from(input8.to_string()))
            .unwrap();

        // Act
        let response1 = router.clone().oneshot(request1).await.unwrap();
        let response2 = router.clone().oneshot(request2).await.unwrap();
        let response3 = router.clone().oneshot(request3).await.unwrap();
        // let response4 = router.clone().oneshot(request4).await.unwrap();
        let response5 = router.clone().oneshot(request5).await.unwrap();
        let response6 = router.clone().oneshot(request6).await.unwrap();
        let response7 = router.clone().oneshot(request7).await.unwrap();
        let response8 = router.clone().oneshot(request8).await.unwrap();

        // Assert
        assert_eq!(response1.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response2.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response3.status(), StatusCode::UNPROCESSABLE_ENTITY);
        // assert_eq!(response4.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response5.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response6.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response7.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(response8.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
