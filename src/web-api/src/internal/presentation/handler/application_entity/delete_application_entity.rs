use crate::{
    internal::{
        application::application_entity::delete_application_entity_use_case::DeleteApplicationEntityCommand,
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
    startup::AppState,
};
use axum::{
    Extension,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use dicom_lib::core::value::value_representations::ae::AeValue;

#[utoipa::path(
    delete,
    path = "/application-entities/{ae_title}",
    params(
        ("ae_title" = String, Path, description = "AE Title")
    ),
    responses(
        (status = 204, description = "Application Entityの削除に成功"),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 404, description = "Application Entityが見つからない"),
        (status = 422, description = "バリデーション失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "application-entities"
)]
pub async fn delete_application_entity(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(ae_title): Path<String>,
) -> Result<StatusCode, PresentationError> {
    // バリデーション
    let title = AeValue::from_string(&ae_title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;

    // 削除処理
    let deleted_at = Utc::now();
    let command = DeleteApplicationEntityCommand {
        title,
        deleted_by: user.uuid(),
        deleted_at,
    };
    state
        .delete_application_entity_use_case
        .execute(command)
        .await
        .map_err(PresentationError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::super::prepare_test_data;
    use crate::{internal::presentation::util::test_helpers, startup};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use dicom_lib::core::value::value_representations::ae::AeValue;
    use tower::ServiceExt;

    #[tokio::test]
    async fn 管理者はアプリケーションエンティティを削除できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/application-entities/DCMTK")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // リポジトリから削除されていることの確認
        let stored = repos
            .application_entity_repository
            .find_by_title(&AeValue::from_string("DCMTK").unwrap())
            .await
            .unwrap();
        assert!(stored.is_none());
    }

    #[tokio::test]
    async fn 情シスはアプリケーションエンティティを削除できる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "it", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/application-entities/DCMTK")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::NO_CONTENT);

        // リポジトリから削除されていることの確認
        let stored = repos
            .application_entity_repository
            .find_by_title(&AeValue::from_string("DCMTK").unwrap())
            .await
            .unwrap();
        assert!(stored.is_none());
    }

    #[tokio::test]
    async fn 存在しないアプリケーションエンティティを削除しようとすると404エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        let request = Request::builder()
            .method("DELETE")
            .uri("/application-entities/OsiriX")
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn 不正なAEタイトルの場合に422エラーになる() {
        // Arrange
        let repos = prepare_test_data().await;
        let state = startup::make_state(&repos);
        let router = startup::make_router(state, &repos);

        let (session_id, csrf_token) = test_helpers::login(&router, "admin", "Password#1234").await;
        // AEタイトルは16文字以内である必要がある。17文字のタイトルを指定する。
        let invalid_title = "A".repeat(17);
        let request = Request::builder()
            .method("DELETE")
            .uri(format!("/application-entities/{invalid_title}"))
            .header("cookie", format!("session_id={session_id}"))
            .header("x-csrf-token", &csrf_token)
            .body(Body::empty())
            .unwrap();

        // Act
        let response = router.oneshot(request).await.unwrap();

        // Assert
        // ステータスコードの確認
        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }
}
