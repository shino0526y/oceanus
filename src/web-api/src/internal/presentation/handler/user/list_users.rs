mod output;

pub use output::ListUsersOutputElement;

use crate::{AppState, internal::presentation::error::PresentationError};
use axum::{Json, extract::State};

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "ユーザーの一覧の取得に成功", body = Vec<ListUsersOutputElement>),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "権限がありません"),
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "users"
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<ListUsersOutputElement>>, PresentationError> {
    let output = state
        .list_users_use_case
        .execute()
        .await
        .map(|users| {
            users
                .into_iter()
                .map(ListUsersOutputElement::from)
                .collect()
        })
        .map_err(PresentationError::from)?;

    Ok(Json(output))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        internal::presentation::{handler::user::prepare_test_data, util::test_helpers},
        utils::{self, make_router},
    };
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn list_users_正常系_管理者はユーザー一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, _csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap();
        let users: Vec<ListUsersOutputElement> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(users.len(), 4);
        let technician = &users[0];
        assert_eq!(technician.id, "technician");
        assert_eq!(technician.name, "技師 太郎");
        assert_eq!(technician.role, 3);
        assert_eq!(technician.login_failure_count, 0);
        let doctor = &users[1];
        assert_eq!(doctor.id, "doctor");
        assert_eq!(doctor.name, "医師 太郎");
        assert_eq!(doctor.role, 2);
        assert_eq!(doctor.login_failure_count, 0);
        let it = &users[2];
        assert_eq!(it.id, "it");
        assert_eq!(it.name, "情シス 太郎");
        assert_eq!(it.role, 1);
        assert_eq!(it.login_failure_count, 0);
        let admin = &users[3];
        assert_eq!(admin.id, "admin");
        assert_eq!(admin.name, "管理者 太郎");
        assert_eq!(admin.role, 0);
        assert_eq!(admin.login_failure_count, 0);
    }

    #[tokio::test]
    async fn list_users_正常系_情シスはユーザー一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, _csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap();
        let users: Vec<ListUsersOutputElement> = serde_json::from_slice(&bytes).unwrap();
        // `list_users_正常系_管理者はユーザー一覧を取得できる`でアウトプットの中身は確認しているのでここでは件数のみ確認
        assert_eq!(users.len(), 4);
    }

    #[tokio::test]
    async fn list_users_準正常系_認証されていないと401エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn list_users_準正常系_権限がないユーザーは403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, _csrf) = test_helpers::login(&router, "technician", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={}", session_id))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
