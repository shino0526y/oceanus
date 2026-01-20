use crate::internal::domain::{entity::ApplicationEntity, error::RepositoryError};
use dicom_lib::core::value::value_representations::ae::AeValue;

#[async_trait::async_trait]
pub trait ApplicationEntityRepository: Send + Sync {
    async fn find_all(&self) -> Result<Vec<ApplicationEntity>, RepositoryError>;
    async fn find_by_title(
        &self,
        title: &AeValue,
    ) -> Result<Option<ApplicationEntity>, RepositoryError>;
    async fn add(&self, entity: &ApplicationEntity) -> Result<ApplicationEntity, RepositoryError>;
    async fn update(
        &self,
        old_title: &AeValue,
        entity: &ApplicationEntity,
    ) -> Result<ApplicationEntity, RepositoryError>;
}
