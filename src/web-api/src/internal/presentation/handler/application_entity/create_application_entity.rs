mod input;
mod output;

pub use self::{input::CreateApplicationEntityInput, output::CreateApplicationEntityOutput};

use crate::{
    AppState,
    internal::{
        application::application_entity::create_application_entity_use_case::CreateApplicationEntityCommand,
        domain::value_object::Port, presentation::error::PresentationError,
    },
};
use axum::{Json, extract::State};
use dicom_lib::core::value::value_representations::ae::AeValue;

#[utoipa::path(
    post,
    path = "/application-entities",
    request_body = CreateApplicationEntityInput,
    responses(
        (status = 200, description = "Application Entityの作成に成功", body = CreateApplicationEntityOutput),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効"),
        (status = 422, description = "バリデーション失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "application-entities"
)]
pub async fn create_application_entity(
    State(state): State<AppState>,
    Json(payload): Json<CreateApplicationEntityInput>,
) -> Result<Json<CreateApplicationEntityOutput>, PresentationError> {
    let command = CreateApplicationEntityCommand {
        title: AeValue::from_string(&payload.title).map_err(|e| {
            PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
        })?,
        host: payload.host,
        port: Port::from_u16(payload.port).map_err(|e| {
            PresentationError::UnprocessableContent(format!("ポート番号が不正です: {e}"))
        })?,
        comment: payload.comment,
    };

    let entity = state
        .create_application_entity_use_case
        .execute(command)
        .await
        .map_err(PresentationError::from)?;

    let output = CreateApplicationEntityOutput::from(entity);

    Ok(Json(output))
}
