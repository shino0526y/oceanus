mod input;
mod output;

pub use self::{input::LoginInput, output::LoginOutput};

use crate::{
    AppState,
    internal::{
        application::auth::{AuthenticationError, LoginCommand},
        domain::{entity::Session, value_object::Id},
        presentation::util::CookieHelper,
    },
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
