mod output;

pub use self::output::ListApplicationEntitiesOutputElement;

use crate::{AppState, internal::presentation::error::PresentationError};
use axum::{Json, extract::State};

#[utoipa::path(
    get,
    path = "/application-entities",
    responses(
        (status = 200, description = "Application Entityの一覧の取得に成功", body = Vec<ListApplicationEntitiesOutputElement>),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効"),
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
