use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateApplicationEntityInput {
    pub title: String,
    pub host: String,
    pub port: u16,
    pub comment: String,
}
