use crate::{
    AppState,
    internal::{
        application::user::reset_login_failure_count_use_case::{
            ResetLoginFailureCountCommand, ResetLoginFailureCountError,
        },
        domain::value_object::Id,
        presentation::error::PresentationError,
    },
};
use axum::{
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
    Path(id): Path<String>,
) -> Result<StatusCode, PresentationError> {
    // バリデーション
    let id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;

    // リセット処理
    let command = ResetLoginFailureCountCommand { id };
    state
        .reset_login_failure_count_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            ResetLoginFailureCountError::Repository(repo_err) => PresentationError::from(repo_err),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::{
            domain::entity::LoginFailureCount,
            presentation::{handler::user::prepare_test_data, util::test_helpers},
        },
        utils::{self, make_router},
    };
    use axum::{body::Body, http::Request};
    use chrono::DateTime;
    use std::str::FromStr;
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者はユーザーのログイン失敗回数をリセットできる() {
        // Arrange
        // 事前に失敗回数を3回にセット
        let repos = prepare_test_data().await;
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
        // リクエストの準備
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // レスポンスの確認
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
        // 事前に失敗回数を3回にセット
        let repos = prepare_test_data().await;
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
        // リクエストの準備
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // レスポンスの確認
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
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "technician", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 情シスが管理者ユーザーのログイン失敗回数をリセットしようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/admin/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 存在しないユーザーのログイン失敗回数をリセットしようとすると404エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/notfound/login-failure-count")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn ユーザーIDが不正な場合は422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users//login-failure-count") // 空のIDは不正
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
