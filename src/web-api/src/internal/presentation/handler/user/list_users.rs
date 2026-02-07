mod response_body;

pub use response_body::ListUsersResponseBodyItem;

use crate::{
    internal::presentation::error::{ErrorResponseBody, PresentationError},
    startup::AppState,
};
use axum::{Json, extract::State};

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "ユーザーの一覧の取得に成功", body = Vec<ListUsersResponseBodyItem>),
        (status = 400, description = "リクエストの形式が無効", body = ErrorResponseBody),
        (status = 401, description = "セッションが確立されていないか期限が切れている", body = ErrorResponseBody),
        (status = 403, description = "権限がない", body = ErrorResponseBody),
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "users"
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<ListUsersResponseBodyItem>>, PresentationError> {
    let response_body = state
        .list_users_use_case
        .execute()
        .await
        .map(|users| {
            users
                .into_iter()
                .map(ListUsersResponseBodyItem::from)
                .collect()
        })
        .map_err(PresentationError::from)?;

    Ok(Json(response_body))
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{internal::presentation::util::test_helpers, startup};
    use axum::{
        body::{self, Body},
        http::{Request, StatusCode},
    };
    use serde_json::Value;
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者はユーザー一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

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
                3,                          // ロール
                "2026-01-24T13:26:54.695Z", // 作成日時＆更新日時
            ),
            ("doctor", "医師 太郎", 2, "2026-01-24T13:25:57.855Z"),
            ("it", "情シス 太郎", 1, "2026-01-24T13:25:34.436Z"),
            ("admin", "管理者 太郎", 0, "2026-01-20T14:10:24.332Z"),
        ];

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body.as_array().unwrap().len(), 4);
        for (i, (id, name, role, dt)) in expected.iter().enumerate() {
            let u = &body[i];
            assert_eq!(u["id"], *id);
            assert_eq!(u["name"], *name);
            assert_eq!(u["role"], *role);
            assert_eq!(u["loginFailureCount"], 0);
            assert_eq!(u["createdAt"], *dt);
            assert_eq!(u["updatedAt"], *dt);
        }
    }

    #[tokio::test]
    async fn 情シスはユーザー一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, _csrf) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        // ここでは件数のみ確認
        assert_eq!(body.as_array().unwrap().len(), 4);
    }

    #[tokio::test]
    async fn 管理者でも情シスでもないユーザーがユーザー一覧を取得しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, _csrf) = test_helpers::login(&router, "technician", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/users")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
