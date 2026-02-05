mod input;
mod output;

pub use self::{input::LoginInput, output::LoginOutput};

use crate::{
    internal::{
        application::auth::{AuthenticationError, LoginCommand},
        domain::{entity::Session, value_object::Id},
        presentation::util::CookieHelper,
    },
    startup::AppState,
};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tower_cookies::Cookies;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[utoipa::path(
    post,
    path = "/login",
    request_body = LoginInput,
    responses(
        (status = 200, description = "ログインに成功", body = LoginOutput),
        (status = 401, description = "認証に失敗", body = ErrorResponse),
        (status = 422, description = "バリデーション失敗", body = ErrorResponse),
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    cookies: Cookies,
    Json(input): Json<LoginInput>,
) -> Result<Json<LoginOutput>, LoginError> {
    // バリデーション
    let user_id = Id::new(&input.user_id).map_err(|e| LoginError::Validation {
        message: format!("無効なユーザーID: {e}"),
    })?;

    // ログイン処理
    let command = LoginCommand {
        user_id: user_id.clone(),
        password: input.password,
    };
    let (session_id, csrf_token, role) = state.login_use_case.execute(command).await?;

    // Cookie設定
    let cookie = CookieHelper::create_session_cookie(session_id, Session::DEFAULT_EXPIRY_MINUTES);
    cookies.add(cookie);

    Ok(Json(LoginOutput {
        user_id: user_id.value().to_string(),
        csrf_token,
        role,
    }))
}

#[derive(Debug)]
pub enum LoginError {
    Validation { message: String },
    Authentication(AuthenticationError),
}

impl From<AuthenticationError> for LoginError {
    fn from(err: AuthenticationError) -> Self {
        LoginError::Authentication(err)
    }
}

impl IntoResponse for LoginError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            LoginError::Validation { message } => (StatusCode::UNPROCESSABLE_ENTITY, message),
            LoginError::Authentication(err) => {
                let status = match err {
                    AuthenticationError::InvalidCredentials => StatusCode::UNAUTHORIZED,
                    AuthenticationError::Locked => StatusCode::FORBIDDEN,
                    AuthenticationError::Other { .. } => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (status, err.to_string())
            }
        };

        let error_response = ErrorResponse { error: message };
        (status, Json(error_response)).into_response()
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::startup;
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use chrono::Utc;
    use futures::future::JoinAll;
    use serde_json::{Value, json};
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn 正しいIDとパスワードでログインできる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let body = json!({
            "userId": "doctor",
            "password": "Password#1234"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["userId"], "doctor");
        assert!(!body["csrfToken"].as_str().unwrap().is_empty());
        assert_eq!(body["role"], 2); // doctor
    }

    #[tokio::test]
    async fn 存在しないユーザーIDの場合は401エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let body = json!({
            "userId": "notfound",
            "password": "Password#1234"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn パスワードが間違っている場合は401エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let body = json!({
            "userId": "doctor",
            "password": "wrongpassword"
        });
        let request = Request::builder()
            .method("POST")
            .uri("/login")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn ログイン失敗回数が5回に達したらユーザーがロックされ403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let wrong_body = json!({
            "userId": "doctor",
            "password": "wrongpassword"
        });
        let correct_body = json!({
            "userId": "doctor",
            "password": "Password#1234"
        });
        let wrong_request = || {
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&wrong_body).unwrap()))
                .unwrap()
        };
        let correct_request = || {
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&correct_body).unwrap()))
                .unwrap()
        };

        // Act
        // 5回連続で間違ったパスワードを送信
        let response1 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response2 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response3 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response4 = router.clone().oneshot(wrong_request()).await.unwrap();
        let response5 = router.clone().oneshot(wrong_request()).await.unwrap();
        // その後、正しいパスワードでログインを試みる
        let response6 = router.clone().oneshot(correct_request()).await.unwrap();
        // またパスワードを間違える
        let response7 = router.clone().oneshot(wrong_request()).await.unwrap();

        // Assert
        // 4回失敗までは401エラー
        assert_eq!(response1.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response2.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response3.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(response4.status(), StatusCode::UNAUTHORIZED);
        // 5回目でロック（403エラー）
        assert_eq!(response5.status(), StatusCode::FORBIDDEN);
        // ロック後は正しいパスワードでも403エラー
        assert_eq!(response6.status(), StatusCode::FORBIDDEN);
        // ロック後に再度間違ったパスワードを送っても403エラー。ログイン失敗回数は増える
        assert_eq!(response7.status(), StatusCode::FORBIDDEN);
        // ログイン失敗回数がリポジトリに反映されていることを確認
        let login_failure_count = repos
            .login_failure_count_repository
            .find_by_user_uuid(&Uuid::parse_str("492236d4-2f18-76ab-a82f-84e29fcf92f8").unwrap())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(login_failure_count.failure_count(), 6); // 合計6回の失敗
        let last_failure_at = login_failure_count.last_failure_at().unwrap();
        let duration = Utc::now().signed_duration_since(*last_failure_at);
        assert!(duration.num_seconds() < 10);
    }

    #[tokio::test]
    async fn バリデーション違反の場合は422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let bodies = [
            json!({ // フィールドなし
                "password": "Password#1234"
            }),
            json!({ // 空文字
                "userId": "",
                "password": "Password#1234"
            }),
        ];
        let requests = bodies.iter().map(|body| {
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap()
        });

        // Act
        let responses = requests
            .map(async |request| router.clone().oneshot(request).await.unwrap())
            .collect::<JoinAll<_>>()
            .await;

        // Assert
        responses.iter().for_each(|response| {
            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        });
    }
}
