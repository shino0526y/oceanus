mod output;

pub use self::output::ListApplicationEntitiesOutputElement;

use crate::{internal::presentation::error::PresentationError, startup::AppState};
use axum::{Json, extract::State};

#[utoipa::path(
    get,
    path = "/application-entities",
    responses(
        (status = 200, description = "Application Entityの一覧の取得に成功", body = Vec<ListApplicationEntitiesOutputElement>),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "権限がありません"),
    ),
    security(
        ("session_cookie" = [])
    ),
    tag = "application-entities"
)]
pub async fn list_application_entities(
    State(state): State<AppState>,
) -> Result<Json<Vec<ListApplicationEntitiesOutputElement>>, PresentationError> {
    let output = state
        .list_application_entities_use_case
        .execute()
        .await
        .map(|entities| {
            entities
                .into_iter()
                .map(ListApplicationEntitiesOutputElement::from)
                .collect()
        })
        .map_err(PresentationError::from)?;

    Ok(Json(output))
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
    use chrono::{DateTime, Utc};
    use serde_json::Value;
    use std::str::FromStr;
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者はアプリケーションエンティティ一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/application-entities")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        let entities = body.as_array().unwrap();
        assert_eq!(entities.len(), 1);
        let entity = &entities[0];
        assert_eq!(entity["title"], "DCMTK");
        assert_eq!(entity["host"], "localhost");
        assert_eq!(entity["port"], 11112);
        assert_eq!(entity["comment"], "開発＆デバッグ用");
        let created_at = DateTime::<Utc>::from_str(entity["createdAt"].as_str().unwrap()).unwrap();
        assert_eq!(
            created_at,
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
        let updated_at = DateTime::<Utc>::from_str(entity["updatedAt"].as_str().unwrap()).unwrap();
        assert_eq!(
            updated_at,
            DateTime::<Utc>::from_str("2026-01-20T23:12:23.874+09:00").unwrap()
        );
    }

    #[tokio::test]
    async fn 情シスはアプリケーションエンティティ一覧を取得できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/application-entities")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();
        // `管理者はアプリケーションエンティティ一覧を取得できる`でレスポンスボディの中身は確認しているのでここでは件数のみ確認
        assert_eq!(body.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn 管理者でも情シスでもないユーザーがAE一覧を取得しようとすると403エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);
        let (session_id, csrf_token) =
            test_helpers::login(&router, "technician", "Password#1234").await;
        let request = Request::builder()
            .method("GET")
            .uri("/application-entities")
            .header("content-type", "application/json")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.clone().oneshot(request).await.unwrap();

        // Assert
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
