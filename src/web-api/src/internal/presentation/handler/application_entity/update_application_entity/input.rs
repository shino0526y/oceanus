use serde::Deserialize;

#[derive(Deserialize)]
pub struct UpdateApplicationEntityInput {
    pub title: String,
    pub host: String,
    pub port: u16,
    pub comment: String,
}
