use crate::{internal::presentation::util::CookieHelper, startup::AppState};
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

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{internal::domain::entity::Session, startup};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use uuid::Uuid;

    #[tokio::test]
    async fn ログインしている場合はログアウトできる() {
        // Arrange
        // 事前にセッションを作成しておく
        let user_uuid = Uuid::parse_str("492236d4-2f18-76ab-a82f-84e29fcf92f8").unwrap();
        let session = Session::create(user_uuid);
        let session_id = session.session_id().to_string();
        let csrf_token = session.csrf_token().to_string();
        // リクエストの準備
        let repos = prepare_test_data().await;
        repos.session_repository.save(session).await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let request = Request::builder()
            .method("POST")
            .uri("/logout")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // レスポンスの確認
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        // リポジトリからセッションが削除されていることを確認
        let session = repos
            .session_repository
            .find_by_session_id(&session_id)
            .await;
        assert!(session.is_none());
    }
}
