mod create_application_entity;
mod list_application_entities;
mod update_application_entity;

use self::{
    create_application_entity::{CreateApplicationEntityInput, CreateApplicationEntityOutput},
    list_application_entities::ListApplicationEntitiesOutputElement,
    update_application_entity::{UpdateApplicationEntityInput, UpdateApplicationEntityOutput},
};
use crate::{
    AppState,
    internal::{
        application::application_entity::{
            create_application_entity_use_case::CreateApplicationEntityCommand,
            update_application_entity_use_case::UpdateApplicationEntityCommand,
        },
        domain::value_object::Port, presentation::error::PresentationError,
    },
};
use axum::{Json, extract::State, extract::Path};
use dicom_lib::core::value::value_representations::ae::AeValue;

pub async fn list_application_entities(
    State(state): State<AppState>,
) -> Result<Json<Vec<ListApplicationEntitiesOutputElement>>, PresentationError> {
    let output = state
        .list_application_entities_use_case
        .list_application_entities()
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

pub async fn create_application_entity(
    State(state): State<AppState>,
    Json(payload): Json<CreateApplicationEntityInput>,
) -> Result<Json<CreateApplicationEntityOutput>, PresentationError> {
    let command = CreateApplicationEntityCommand {
        title: AeValue::from_string(&payload.title)
            .map_err(|e| PresentationError::BadRequest(format!("AEタイトルが不正です: {e}")))?,
        host: payload.host,
        port: Port::from_u16(payload.port)
            .map_err(|e| PresentationError::BadRequest(format!("ポート番号が不正です: {e}")))?,
        comment: payload.comment,
    };

    let entity = state
        .create_application_entity_use_case
        .create_application_entity(command)
        .await
        .map_err(PresentationError::from)?;

    let output = CreateApplicationEntityOutput::from(entity);

    Ok(Json(output))
}

pub async fn update_application_entity(
    State(state): State<AppState>,
    Path(ae_title): Path<String>,
    Json(payload): Json<UpdateApplicationEntityInput>,
) -> Result<Json<UpdateApplicationEntityOutput>, PresentationError> {
    let command = UpdateApplicationEntityCommand {
        title: AeValue::from_string(&payload.title)
            .map_err(|e| PresentationError::BadRequest(format!("AEタイトルが不正です: {e}")))?,
        host: payload.host,
        port: Port::from_u16(payload.port)
            .map_err(|e| PresentationError::BadRequest(format!("ポート番号が不正です: {e}")))?,
        comment: payload.comment,
    };

    let entity = state
        .update_application_entity_use_case
        .update_application_entity(&ae_title, command)
        .await
        .map_err(PresentationError::from)?;

    let output = UpdateApplicationEntityOutput::from(entity);

    Ok(Json(output))
}
