use crate::internal::{
    application::session::ExtendSessionUseCase,
    domain::entity::Session,
    presentation::{error::PresentationError, util::CookieHelper},
};
use axum::{
    body::Body,
    extract::Request,
    http::Method,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_cookies::Cookies;
use uuid::Uuid;

const CSRF_TOKEN_HEADER: &str = "X-CSRF-Token";

/// 認証済みユーザーのUUIDを格納する構造体
#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedUser(pub Uuid);

impl AuthenticatedUser {
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

/// セッション認証ミドルウェア
pub async fn session_auth_middleware(
    cookies: Cookies,
    extend_session_use_case: Arc<ExtendSessionUseCase>,
    request: Request<Body>,
    next: Next,
) -> Response {
    // CookieからセッションIDを取得
    let session_id = match cookies.get(CookieHelper::SESSION_COOKIE_NAME) {
        Some(cookie) => cookie.value().to_string(),
        None => {
            // セッションIDがない場合は401
            return PresentationError::Unauthorized("セッションが見つかりません".to_string())
                .into_response();
        }
    };

    // POST, PUT, DELETEなどの変更系リクエストではCSRFトークンを検証
    let method = request.method();
    if matches!(
        method,
        &Method::POST | &Method::PUT | &Method::DELETE | &Method::PATCH
    ) {
        // CSRFトークンをヘッダーから取得
        let csrf_token = match request.headers().get(CSRF_TOKEN_HEADER) {
            Some(header_value) => match header_value.to_str() {
                Ok(token) => token,
                Err(_) => {
                    return PresentationError::BadRequest("不正なCSRFトークンです".to_string())
                        .into_response();
                }
            },
            None => {
                // CSRFトークンがない場合は403
                return PresentationError::Forbidden("CSRFトークンがありません".to_string())
                    .into_response();
            }
        };

        // セッションとCSRFトークンを検証し、セッションを延長
        match extend_session_use_case
            .execute_with_csrf_validation(&session_id, csrf_token)
            .await
        {
            Ok(Some(user_uuid)) => {
                // セッション有効期限を延長したので、Cookieも更新
                let cookie = CookieHelper::create_session_cookie(
                    session_id,
                    Session::DEFAULT_EXPIRY_MINUTES,
                );
                cookies.add(cookie);

                // 認証済みユーザー情報をリクエストに追加
                let mut request = request;
                request
                    .extensions_mut()
                    .insert(AuthenticatedUser(user_uuid));

                next.run(request).await
            }
            Ok(None) => {
                // セッションが見つからないか期限切れ
                PresentationError::Unauthorized("セッションが期限切れか見つかりません".to_string())
                    .into_response()
            }
            Err(_) => {
                // CSRFトークンが不正
                PresentationError::Forbidden("不正なCSRFトークンです".to_string()).into_response()
            }
        }
    } else {
        // GETなどの参照系リクエストではCSRFトークン不要
        // セッションを検証し延長する
        if let Some(user_uuid) = extend_session_use_case.execute(&session_id).await {
            // セッション有効期限を延長したので、Cookieも更新
            let cookie =
                CookieHelper::create_session_cookie(session_id, Session::DEFAULT_EXPIRY_MINUTES);
            cookies.add(cookie);

            // 認証済みユーザー情報をリクエストに追加
            let mut request = request;
            request
                .extensions_mut()
                .insert(AuthenticatedUser(user_uuid));

            next.run(request).await
        } else {
            // セッションが見つからないか期限切れ
            PresentationError::Unauthorized("セッションが期限切れか見つかりません".to_string())
                .into_response()
        }
    }
}
