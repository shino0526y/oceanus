use crate::internal::domain::{
    error::RepositoryError,
    repository::{LoginFailureCountRepository, UserRepository},
    value_object::{Id, Role},
};
use std::sync::Arc;
use uuid::Uuid;

pub struct ResetLoginFailureCountUseCase {
    user_repository: Arc<dyn UserRepository>,
    login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
}

pub struct ResetLoginFailureCountCommand {
    pub target_id: Id,
    pub updated_by: Uuid,
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
        // リセット対象のユーザーを取得
        let target_user = self
            .user_repository
            .find_by_id(&command.target_id)
            .await?
            .ok_or_else(|| {
                ResetLoginFailureCountError::Repository(RepositoryError::NotFound {
                    resource: "ユーザー".to_string(),
                    key: command.target_id.value().to_string(),
                })
            })?;

        // 情シスは管理者のログイン失敗回数をリセットできない
        let actor = self
            .user_repository
            .find_by_uuid(&command.updated_by)
            .await?
            .expect("ユーザーがログイン済みなので存在するはず");
        if actor.role() == Role::ItStaff && target_user.role() == Role::Admin {
            return Err(ResetLoginFailureCountError::Forbidden);
        }

        // ログイン失敗情報を削除
        self.login_failure_count_repository
            .delete(target_user.uuid())
            .await
            .map_err(ResetLoginFailureCountError::Repository)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ResetLoginFailureCountError {
    #[error("{0}")]
    Repository(#[from] RepositoryError),
    #[error("権限がありません")]
    Forbidden,
}
