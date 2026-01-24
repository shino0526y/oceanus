use crate::{
    AppState,
    internal::{
        application::application_entity::delete_application_entity_use_case::DeleteApplicationEntityCommand,
        presentation::{error::PresentationError, middleware::AuthenticatedUser},
    },
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
