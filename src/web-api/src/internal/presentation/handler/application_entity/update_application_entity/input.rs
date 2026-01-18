use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct UpdateApplicationEntityInput {
    pub title: String,
    pub host: String,
    pub port: u16,
    pub comment: String,
}
