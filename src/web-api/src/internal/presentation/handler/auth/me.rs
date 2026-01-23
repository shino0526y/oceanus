mod output;

pub use self::output::MeOutput;

use crate::internal::{
    domain::{entity::Session, repository::SessionRepository},
    presentation::util::CookieHelper,
};
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::sync::Arc;
use tower_cookies::Cookies;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[utoipa::path(
    get,
    path = "/me",
    responses(
        (status = 200, description = "認証済みユーザー情報を取得", body = MeOutput),
        (status = 401, description = "未認証またはセッション切れ", body = ErrorResponse),
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "auth"
)]
pub async fn me(
    cookies: Cookies,
    session_repository: Arc<dyn SessionRepository>,
) -> Result<Json<MeOutput>, MeError> {
    // CookieからセッションIDを取得
    let session_id = cookies
        .get(CookieHelper::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or(MeError::Unauthorized)?;

    // セッションを取得
    let mut session = session_repository
        .find_by_session_id(&session_id)
        .await
        .ok_or(MeError::Unauthorized)?;

    // ユーザーID（UUID）とCSRFトークンを取得
    let user_uuid = *session.user_uuid();
    let csrf_token = session.csrf_token().to_string();

    // セッションを延長
    session.extend();
    session_repository.save(session).await;

    // Cookieの有効期限も更新
    let cookie = CookieHelper::create_session_cookie(session_id, Session::DEFAULT_EXPIRY_MINUTES);
    cookies.add(cookie);

    Ok(Json(MeOutput {
        user_id: user_uuid.to_string(),
        csrf_token,
    }))
}

#[derive(Debug)]
pub enum MeError {
    Unauthorized,
}

impl IntoResponse for MeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            MeError::Unauthorized => (StatusCode::UNAUTHORIZED, "認証されていません"),
        };

        let error_response = ErrorResponse {
            error: message.to_string(),
        };
        (status, Json(error_response)).into_response()
    }
}
