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
    use chrono::{DateTime, Utc};
    use std::str::FromStr;
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者はユーザー一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, _csrf) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let expected = [
            (
                "technician",
                "技師 太郎",
                3,                               // ロール
                "2026-01-24T22:26:54.695+09:00", // 作成日時＆更新日時
            ),
            ("doctor", "医師 太郎", 2, "2026-01-24T22:25:57.855+09:00"),
            ("it", "情シス 太郎", 1, "2026-01-24T22:25:34.436+09:00"),
            ("admin", "管理者 太郎", 0, "2026-01-20T23:10:24.332+09:00"),
        ];

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: Vec<ListUsersOutputElement> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(output.len(), 4);
        for (i, (id, name, role, dt_str)) in expected.iter().enumerate() {
            let u = &output[i];
            assert_eq!(u.id, *id);
            assert_eq!(u.name, *name);
            assert_eq!(u.role, *role);
            assert_eq!(u.login_failure_count, 0);
            let dt = DateTime::<Utc>::from_str(dt_str).unwrap();
            assert_eq!(u.created_at, dt);
            assert_eq!(u.updated_at, dt);
        }
    }

    #[tokio::test]
    async fn 情シスはユーザー一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, _csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let output: Vec<ListUsersOutputElement> = serde_json::from_slice(&bytes).unwrap();
        // `管理者はユーザー一覧を取得できる`でアウトプットの中身は確認しているのでここでは件数のみ確認
        assert_eq!(output.len(), 4);
    }

    #[tokio::test]
    async fn 管理者でも情シスでもないユーザーがユーザー一覧を取得しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let app_state = utils::make_app_state(&repos);
        let router = make_router(app_state, &repos);
        let (session_id, _csrf) = test_helpers::login(&router, "technician", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
