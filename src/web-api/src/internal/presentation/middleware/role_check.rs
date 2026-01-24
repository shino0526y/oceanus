use crate::internal::domain::{repository::UserRepository, value_object::Role};
use axum::{
    body::Body, extract::Request, http::StatusCode, middleware::Next, response::IntoResponse,
    response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::internal::presentation::middleware::session_auth::AuthenticatedUser;

/// 管理者または情シスでなければ403を返すミドルウェア関数
pub async fn require_admin_or_it(
    request: Request<Body>,
    next: Next,
    user_repository: Arc<dyn UserRepository>,
) -> Response {
    // 認証済みユーザー情報を取得
    let user_uuid: Uuid = match request.extensions().get::<AuthenticatedUser>() {
        Some(auth) => auth.uuid(),
        None => return StatusCode::UNAUTHORIZED.into_response(),
    };

    // ユーザーを取得
    match user_repository.find_by_uuid(&user_uuid).await {
        Ok(Some(user)) => {
            let role = user.role();
            if role == Role::Admin || role == Role::ItStaff {
                next.run(request).await
            } else {
                StatusCode::FORBIDDEN.into_response()
            }
        }
        Ok(None) => StatusCode::UNAUTHORIZED.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
