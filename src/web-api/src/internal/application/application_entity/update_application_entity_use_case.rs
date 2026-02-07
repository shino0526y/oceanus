use crate::internal::domain::{
    entity::ApplicationEntity,
    error::RepositoryError,
    repository::ApplicationEntityRepository,
    value_object::{HostName, Port},
};
use chrono::{DateTime, Utc};
use dicom_lib::core::value::value_representations::ae::AeValue;
use std::sync::Arc;
use uuid::Uuid;

pub struct UpdateApplicationEntityUseCase {
    repository: Arc<dyn ApplicationEntityRepository>,
}

pub struct UpdateApplicationEntityCommand {
    pub old_title: AeValue,

    pub title: AeValue,
    pub host: HostName,
    pub port: Port,
    pub comment: String,
    pub updated_by: Uuid,
    pub updated_at: DateTime<Utc>,
}

impl UpdateApplicationEntityUseCase {
    pub fn new(repository: Arc<dyn ApplicationEntityRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        command: UpdateApplicationEntityCommand,
    ) -> Result<ApplicationEntity, RepositoryError> {
        // エンティティを取得
        let mut entity = self
            .repository
            .find_by_title(&command.old_title)
            .await?
            .ok_or_else(|| RepositoryError::NotFound {
                resource: "AEタイトル".to_string(),
                key: command.old_title.value().to_string(),
            })?;

        // エンティティを変更し、変更があれば保存
        let is_changed = entity.update(
            command.title,
            command.host,
            command.port,
            command.comment,
            command.updated_by,
            command.updated_at,
        );
        if !is_changed {
            return Ok(entity);
        }
        self.repository.update(&command.old_title, &entity).await
    }
}
