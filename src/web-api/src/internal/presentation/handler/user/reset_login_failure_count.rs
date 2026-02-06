use crate::{
    internal::{
        application::user::reset_login_failure_count_use_case::{
            ResetLoginFailureCountCommand, ResetLoginFailureCountError,
        },
        domain::value_object::Id,
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
    startup::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};

#[utoipa::path(
    delete,
    path = "/users/{id}/login-failure-count",
    params(
        ("id" = String, Path, description = "ユーザーID")
    ),
    responses(
        (status = 204, description = "ログイン失敗回数のリセットに成功"),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 404, description = "ユーザーが見つからない"),
        (status = 422, description = "バリデーションに失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "users"
)]
pub async fn reset_login_failure_count(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, PresentationError> {
    // バリデーション
    let id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {e}")))?;

    // リセット処理
    let command = ResetLoginFailureCountCommand {
        target_id: id,
        updated_by: user.uuid(),
    };
    state
        .reset_login_failure_count_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            ResetLoginFailureCountError::Repository(repo_err) => PresentationError::from(repo_err),
            ResetLoginFailureCountError::Forbidden => PresentationError::Forbidden(e.to_string()),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{
        internal::{
            domain::{entity::LoginFailureCount, value_object::Id},
            presentation::util::test_helpers,
        },
        startup,
    };
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use chrono::DateTime;
    use std::str::FromStr;
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者はユーザーのログイン失敗回数をリセットできる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        let user_uuid = *user.uuid();
        repos
            .login_failure_count_repository
            .save(&LoginFailureCount::construct(
                user_uuid,
                3,
                Some(DateTime::from_str("2026-01-28T23:01:15.295+09:00").unwrap()),
            ))
            .await
            .unwrap();

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // 失敗回数の情報自体が削除されていることの確認
        let login_failure_count = repos
            .login_failure_count_repository
            .find_by_user_uuid(&user_uuid)
            .await
            .unwrap();
        assert!(login_failure_count.is_none());
    }

    #[tokio::test]
    async fn 情シスはユーザーのログイン失敗回数をリセットできる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let user = repos
            .user_repository
            .find_by_id(&Id::new("doctor").unwrap())
            .await
            .unwrap()
            .unwrap();
        let user_uuid = *user.uuid();
        repos
            .login_failure_count_repository
            .save(&LoginFailureCount::construct(
                user_uuid,
                3,
                Some(DateTime::from_str("2026-01-28T23:01:15.295+09:00").unwrap()),
            ))
            .await
            .unwrap();

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // 失敗回数の情報自体が削除されていることの確認
        let login_failure_count = repos
            .login_failure_count_repository
            .find_by_user_uuid(&user_uuid)
            .await
            .unwrap();
        assert!(login_failure_count.is_none());
    }

    #[tokio::test]
    async fn 管理者でも情シスでもないユーザーがリセットしようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) =
            test_helpers::login(&router, "technician", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 情シスが管理者ユーザーのログイン失敗回数をリセットしようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/admin/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 存在しないユーザーのログイン失敗回数をリセットしようとすると404エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/notfound/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn パスパラメータのユーザーIDが不正な場合は422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users//login-failure-count") // 空文字
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
