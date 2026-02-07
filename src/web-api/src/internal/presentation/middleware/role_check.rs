use crate::internal::{
    domain::{repository::UserRepository, value_object::Role},
    presentation::{error::PresentationError, middleware::session_auth::AuthenticatedUser},
};
use axum::{
    body::Body, extract::Request, middleware::Next, response::IntoResponse, response::Response,
};
use std::sync::Arc;
use uuid::Uuid;

/// 管理者または情シスでなければ403を返すミドルウェア関数
pub async fn require_admin_or_it(
    request: Request<Body>,
    next: Next,
    user_repository: Arc<dyn UserRepository>,
) -> Response {
    // 認証済みユーザー情報を取得
    let user_uuid: Uuid = match request.extensions().get::<AuthenticatedUser>() {
        Some(auth) => auth.uuid(),
        None => {
            return PresentationError::Unauthorized("認証が必要です".to_string()).into_response();
        }
    };

    // ユーザーを取得
    match user_repository.find_by_uuid(&user_uuid).await {
        Ok(Some(user)) => {
            let role = user.role();
            if role == Role::Admin || role == Role::ItStaff {
                next.run(request).await
            } else {
                PresentationError::Forbidden("この操作を行う権限がありません".to_string())
                    .into_response()
            }
        }
        Ok(None) => {
            PresentationError::Unauthorized("ユーザーが見つかりません".to_string()).into_response()
        }
        Err(e) => PresentationError::InternalServerError(format!(
            "データベース処理でエラーが発生しました: {e}"
        ))
        .into_response(),
    }
}
