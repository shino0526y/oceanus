mod update_application_entity_command;

pub use update_application_entity_command::UpdateApplicationEntityCommand;

use crate::internal::domain::{
    entity::ApplicationEntity, error::RepositoryError, repository::ApplicationEntityRepository,
};
use chrono::Utc;
use std::sync::Arc;

pub struct UpdateApplicationEntityUseCase {
    repository: Arc<dyn ApplicationEntityRepository>,
}

impl UpdateApplicationEntityUseCase {
    pub fn new(repository: Arc<dyn ApplicationEntityRepository>) -> Self {
        Self { repository }
    }

    pub async fn update_application_entity(
        &self,
        old_title: &str,
        command: UpdateApplicationEntityCommand,
    ) -> Result<ApplicationEntity, RepositoryError> {
        // エンティティを取得
        let mut entity = self
            .repository
            .find_by_title(old_title)
            .await?
            .ok_or_else(|| RepositoryError::NotFound {
                resource: "AEタイトル".to_string(),
                key: old_title.to_string(),
            })?;

        // エンティティを変更
        entity.update(
            command.title,
            command.host,
            command.port,
            command.comment,
            Utc::now(),
        );

        // 変更されたエンティティを保存
        self.repository.update(old_title, &entity).await
    }
}
