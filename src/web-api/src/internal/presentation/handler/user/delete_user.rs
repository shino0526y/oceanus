use crate::{
    AppState,
    internal::{
        application::user::delete_user_use_case::{DeleteUserCommand, DeleteUserError},
        domain::value_object::Id,
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id" = String, Path, description = "ユーザーID")
    ),
    responses(
        (status = 204, description = "ユーザーの削除に成功"),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 404, description = "ユーザーが見つからない"),
        (status = 422, description = "バリデーションに失敗、または自分自身を削除しようとした"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(id): Path<String>,
) -> Result<StatusCode, PresentationError> {
    // バリデーション
    let id = Id::new(id)
        .map_err(|e| PresentationError::UnprocessableContent(format!("無効なID: {}", e)))?;

    // 削除処理
    let deleted_at = Utc::now();
    let command = DeleteUserCommand {
        id,
        deleted_by: user.uuid(),
        deleted_at,
    };
    state
        .delete_user_use_case
        .execute(command)
        .await
        .map_err(|e| match e {
            DeleteUserError::CannotDeleteSelf => {
                PresentationError::UnprocessableContent(e.to_string())
            }
            DeleteUserError::Forbidden => PresentationError::Forbidden(e.to_string()),
            DeleteUserError::Repository(repo_err) => PresentationError::from(repo_err),
        })?;

    Ok(StatusCode::NO_CONTENT)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::presentation::{handler::user::prepare_test_data, util::test_helpers},
        utils::{self, make_router},
    };
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者は他のユーザーを削除できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        // ユーザーが削除されていることの確認
        assert!(
            repos
                .user_repository
                .find_by_id(&Id::new("doctor").unwrap())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn 情シスは他の管理者でないユーザーを削除できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf.clone())
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        // HTTPレスポンスの確認
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        // ユーザーが削除されていることの確認
        assert!(
            repos
                .user_repository
                .find_by_id(&Id::new("doctor").unwrap())
                .await
                .unwrap()
                .is_none()
        );
    }

    #[tokio::test]
    async fn 情シスが管理者を削除しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/admin")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 管理者や情シスでないユーザーがユーザーを削除しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "technician", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/doctor")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn 存在しないユーザーを削除しようとすると404エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/john_doe")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn ユーザーIDを指定せず削除しようとすると405エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .header("x-csrf-token", csrf)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn 自分自身を削除しようとすると422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/users/admin")
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
