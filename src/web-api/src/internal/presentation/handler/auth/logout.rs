use crate::{
    internal::presentation::{
        error::{ErrorResponseBody, PresentationError},
        util::CookieHelper,
    },
    startup::AppState,
};
use axum::{extract::State, http::StatusCode};
use tower_cookies::Cookies;

#[utoipa::path(
    post,
    path = "/logout",
    responses(
        (status = 204, description = "ログアウトに成功"),
        (status = 400, description = "リクエストの形式が無効", body = ErrorResponseBody),
        (status = 401, description = "セッションが確立されていないか期限が切れている", body = ErrorResponseBody),
        (status = 403, description = "CSRFトークンが無効", body = ErrorResponseBody),
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
) -> Result<StatusCode, PresentationError> {
    // セッションIDを取得
    let session_id = cookies
        .get(CookieHelper::SESSION_COOKIE_NAME)
        .map(|c| c.value().to_string())
        .ok_or(PresentationError::BadRequest(
            "セッションが存在しません".to_string(),
        ))?;

    // ログアウト処理
    state.logout_use_case.execute(&session_id).await;

    // Cookieを削除
    cookies.remove(CookieHelper::delete_session_cookie());

    Ok(StatusCode::NO_CONTENT)
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
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        // 事前にセッションを作成してリポジトリに保存
        let user_uuid = Uuid::parse_str("492236d4-2f18-76ab-a82f-84e29fcf92f8").unwrap();
        let session = Session::create(user_uuid);
        let session_id = session.session_id().to_string();
        let csrf_token = session.csrf_token().to_string();
        repos.session_repository.save(session).await;

        let request = Request::builder()
            .method("POST")
            .uri("/logout")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // リポジトリからセッションが削除されていることを確認
        let session = repos
            .session_repository
            .find_by_session_id(&session_id)
            .await;
        assert!(session.is_none());
    }
}
