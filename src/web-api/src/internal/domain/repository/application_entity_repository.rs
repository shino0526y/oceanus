use crate::internal::domain::{entity::ApplicationEntity, error::RepositoryError};

#[async_trait::async_trait]
pub trait ApplicationEntityRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<ApplicationEntity>, RepositoryError>;
    async fn add(&self, entity: &ApplicationEntity) -> Result<ApplicationEntity, RepositoryError>;
}
