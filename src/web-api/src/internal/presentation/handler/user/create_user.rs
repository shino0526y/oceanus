mod input;
mod output;

pub use self::{input::CreateUserInput, output::CreateUserOutput};

use crate::{
    AppState,
    internal::{
        application::user::create_user_use_case::{CreateUserCommand, CreateUserError},
        domain::value_object::{Id, Role, UserName},
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{Extension, Json, extract::State};
use chrono::Utc;

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUserInput,
    responses(
        (status = 200, description = "ユーザーの作成に成功", body = CreateUserOutput),
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
pub async fn create_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(input): Json<CreateUserInput>,
) -> Result<Json<CreateUserOutput>, PresentationError> {
    // バリデーション
    let id = Id::new(input.id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {e}")))?;
    let name = UserName::new(input.name)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効な名前: {e}")))?;
    let role = Role::from_i16(input.role)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なロール: {e}")))?;

    // 登録処理
    let command = CreateUserCommand {
        id,
        name,
        role,
        password: input.password,
        created_by: user.uuid(),
        created_at: Utc::now(),
    };
    let user = state
        .create_user_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            CreateUserError::EmptyPassword => {
                PresentationError::UnprocessableContent(e.to_string())
            }
            CreateUserError::Forbidden => PresentationError::Forbidden(e.to_string()),
            CreateUserError::PasswordHashError(msg) => PresentationError::InternalServerError(
                format!("パスワードのハッシュ化に失敗しました: {msg}"),
            ),
            CreateUserError::Repository(e) => PresentationError::from(e),
        })?;

    Ok(Json(CreateUserOutput::from(user)))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::{
            domain::value_object::Id,
            presentation::{handler::user::prepare_test_data, util::test_helpers},
        },
        utils::{self, make_router},
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use futures::future::JoinAll;
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者はユーザーを作成できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: CreateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "john");
        assert_eq!(output.name, "John Doe");
        assert_eq!(output.role, 2);
        let now = Utc::now();
        assert!((now - output.created_at).num_seconds().abs() < 10);
        assert_eq!(output.updated_at, output.created_at);
        // ユーザーが作成されていることの確認
        assert!(
            repos
                .user_repository
                .find_by_id(&Id::new("john").unwrap())
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn 情シスはユーザーを作成できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: CreateUserOutput = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.id, "john");
        assert_eq!(output.name, "John Doe");
        assert_eq!(output.role, 2);
        let now = Utc::now();
        assert!((now - output.created_at).num_seconds().abs() < 10);
        assert_eq!(output.updated_at, output.created_at);
        // ユーザーが作成されていることの確認
        assert!(
            repos
                .user_repository
                .find_by_id(&Id::new("john").unwrap())
                .await
                .unwrap()
                .is_some()
        );
    }

    #[tokio::test]
    async fn 情シスが管理者を作成しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 0, // Admin
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 管理者や情シスでないユーザーがユーザーを作成しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) =
            test_helpers::login(&router, "technician", "Password#1234").await;
        let input = json!({
            "id": "john",
            "name": "John Doe",
            "role": 2, // Doctor
            "password": "Password#1234",
        });
        let request = Request::builder()
            .method("POST")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::from(serde_json::to_string(&input).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn すでに存在するユーザーと競合するユーザーを作成しようとすると409エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let inputs = [
            json!({ // IDが既存ユーザーと競合
                "id": "doctor",
                "name": "John Doe",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
            json!({ // 名前が既存ユーザーと競合
                "id": "john",
                "name": "医師 太郎",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
        ];
        let requests = inputs.iter().map(|input| {
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("content-type", "application/json")
                .header("cookie", format!("session_id={session_id}"))
                .header("x-csrf-token", &csrf_token)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap()
        });

        // Act
        let responses = requests
            .map(async |req| router.clone().oneshot(req).await.unwrap())
            .collect::<JoinAll<_>>()
            .await;

        // Assert
        responses.iter().for_each(|response| {
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
            json!({ // IDのフィールドがない
                "name": "John Doe",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
            json!({ // IDが空文字
                "id": "",
                "name": "John Doe",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
            json!({ // IDに空白を含む
                "id": "john doe",
                "name": "John Doe",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
            json!({ // 名前のフィールドがない
                "id": "john",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
            json!({ // 名前が空文字
                "id": "john",
                "name": "",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
            json!({ // 名前が空白のみ
                "id": "john",
                "name": " ",
                "role": 2, // Doctor
                "password": "Password#1234",
            }),
            json!({ // ロールのフィールドがない
                "id": "john",
                "name": "John Doe",
                "password": "Password#1234",
            }),
            json!({ // ロールが負の値
                "id": "john",
                "name": "John Doe",
                "role": -1,
                "password": "Password#1234",
            }),
            json!({ // ロールが5以上の値
                "id": "john",
                "name": "John Doe",
                "role": 5,
                "password": "Password#1234",
            }),
            json!({ // パスワードのフィールドがない
                "id": "john",
                "name": "John Doe",
                "role": 2, // Doctor
            }),
            json!({ // パスワードが空文字
                "id": "john",
                "name": "John Doe",
                "role": 2, // Doctor
                "password": "",
            }),
        ];
        let requests = inputs.iter().map(|input| {
            Request::builder()
                .method("POST")
                .uri("/users")
                .header("content-type", "application/json")
                .header("cookie", format!("session_id={session_id}"))
                .header("x-csrf-token", &csrf_token)
                .body(Body::from(serde_json::to_string(&input).unwrap()))
                .unwrap()
        });

        // Act
        let responses = requests
            .map(async |req| router.clone().oneshot(req).await.unwrap())
            .collect::<JoinAll<_>>()
            .await;

        // Assert
        responses.iter().for_each(|response| {
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        });
    }
}
