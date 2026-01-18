use crate::internal::{
    application::session::ExtendSessionUseCase, domain::entity::Session,
    presentation::util::CookieHelper,
};
use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tower_cookies::Cookies;

const CSRF_TOKEN_HEADER: &str = "X-CSRF-Token";

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
            return StatusCode::UNAUTHORIZED.into_response();
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
                    return StatusCode::BAD_REQUEST.into_response();
                }
            },
            None => {
                // CSRFトークンがない場合は403
                return StatusCode::FORBIDDEN.into_response();
            }
        };

        // セッションとCSRFトークンを検証し、セッションを延長
        match extend_session_use_case
            .execute_with_csrf_validation(&session_id, csrf_token)
            .await
        {
            Ok(true) => {
                // セッション有効期限を延長したので、Cookieも更新
                let cookie = CookieHelper::create_session_cookie(
                    session_id,
                    Session::DEFAULT_EXPIRY_MINUTES,
                );
                cookies.add(cookie);

                next.run(request).await
            }
            Ok(false) => {
                // セッションが見つからないか期限切れ
                StatusCode::UNAUTHORIZED.into_response()
            }
            Err(_) => {
                // CSRFトークンが不正
                StatusCode::FORBIDDEN.into_response()
            }
        }
    } else {
        // GETなどの参照系リクエストではCSRFトークン不要
        // セッションを検証し延長する
        if extend_session_use_case.execute(&session_id).await {
            // セッション有効期限を延長したので、Cookieも更新
            let cookie =
                CookieHelper::create_session_cookie(session_id, Session::DEFAULT_EXPIRY_MINUTES);
            cookies.add(cookie);

            next.run(request).await
        } else {
            // セッションが見つからないか期限切れ
            StatusCode::UNAUTHORIZED.into_response()
        }
    }
}
