use crate::internal::domain::{
    error::RepositoryError,
    repository::{LoginFailureCountRepository, UserRepository},
    value_object::Id,
};
use std::sync::Arc;

pub struct ResetLoginFailureCountUseCase {
    user_repository: Arc<dyn UserRepository>,
    login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
}

pub struct ResetLoginFailureCountCommand {
    pub id: Id,
}

impl ResetLoginFailureCountUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
    ) -> Self {
        Self {
            user_repository,
            login_failure_count_repository,
        }
    }

    pub async fn execute(
        &self,
        command: ResetLoginFailureCountCommand,
    ) -> Result<(), ResetLoginFailureCountError> {
        // ユーザーを取得（存在確認）
        let user = self
            .user_repository
            .find_by_id(&command.id)
            .await?
            .ok_or_else(|| {
                ResetLoginFailureCountError::Repository(RepositoryError::NotFound {
                    resource: "ユーザー".to_string(),
                    key: command.id.value().to_string(),
                })
            })?;

        // ログイン失敗情報を削除
        self.login_failure_count_repository
            .delete(user.uuid())
            .await
            .map_err(ResetLoginFailureCountError::Repository)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResetLoginFailureCountError {
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
