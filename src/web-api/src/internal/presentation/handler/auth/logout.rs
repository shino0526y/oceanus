use crate::{AppState, internal::presentation::util::CookieHelper};
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tower_cookies::{Cookie, Cookies};
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 204, description = "ログアウトに成功"),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "auth"
)]
pub async fn logout(
    State(state): State<AppState>,
    cookies: Cookies,
) -> Result<StatusCode, LogoutError> {
    // セッションIDを取得
    let session_id = cookies
        .get(CookieHelper::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or(LogoutError::NoSession)?;

    // ログアウト処理
    state.logout_use_case.execute(&session_id).await;

    // Cookieを削除
    cookies.remove(Cookie::from(CookieHelper::SESSION_COOKIE_NAME));

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug)]
pub enum LogoutError {
    NoSession,
}

impl IntoResponse for LogoutError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            LogoutError::NoSession => (StatusCode::BAD_REQUEST, "セッションが存在しません"),
        };

        let error_response = ErrorResponse {
            error: message.to_string(),
        };
        (status, Json(error_response)).into_response()
    }
}
