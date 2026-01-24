mod input;
mod output;

pub use self::{input::UpdateApplicationEntityInput, output::UpdateApplicationEntityOutput};

use crate::{
    AppState,
    internal::{
        application::application_entity::update_application_entity_use_case::UpdateApplicationEntityCommand,
        domain::value_object::Port,
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
};
use axum::{
    Extension, Json,
    extract::{Path, State},
};
use chrono::Utc;
use dicom_lib::core::value::value_representations::ae::AeValue;

#[utoipa::path(
    put,
    path = "/application-entities/{ae_title}",
    request_body = UpdateApplicationEntityInput,
    params(
        ("ae_title" = String, Path, description = "AE Title")
    ),
    responses(
        (status = 200, description = "Application Entityの更新に成功", body = UpdateApplicationEntityOutput),
        (status = 401, description = "セッションが確立されていない"),
        (status = 403, description = "CSRFトークンが無効または権限がありません"),
        (status = 422, description = "バリデーション失敗"),
    ),
    security(
        ("session_cookie" = []),
        ("csrf_token" = [])
    ),
    tag = "application-entities"
)]
pub async fn update_application_entity(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(ae_title): Path<String>,
    Json(payload): Json<UpdateApplicationEntityInput>,
) -> Result<Json<UpdateApplicationEntityOutput>, PresentationError> {
    // バリデーション
    let old_title = AeValue::from_string(&ae_title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;
    let title = AeValue::from_string(&payload.title).map_err(|e| {
        PresentationError::UnprocessableContent(format!("AEタイトルが不正です: {e}"))
    })?;
    let port = Port::from_u16(payload.port).map_err(|e| {
        PresentationError::UnprocessableContent(format!("ポート番号が不正です: {e}"))
    })?;

    // 更新処理
    let command = UpdateApplicationEntityCommand {
        old_title,
        title,
        host: payload.host,
        port,
        comment: payload.comment,
        updated_by: user.uuid(),
        updated_at: Utc::now(),
    };
    let entity = state
        .update_application_entity_use_case
        .execute(command)
        .await
        .map_err(PresentationError::from)?;

    let output = UpdateApplicationEntityOutput::from(entity);

    Ok(Json(output))
}
