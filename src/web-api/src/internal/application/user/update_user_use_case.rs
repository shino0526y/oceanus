mod update_user_command;

pub use update_user_command::UpdateUserCommand;

use crate::internal::domain::{
    entity::User, error::RepositoryError, repository::UserRepository, value_object::Id,
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::Utc;
use std::sync::Arc;

pub struct UpdateUserUseCase {
    repository: Arc<dyn UserRepository>,
}

impl UpdateUserUseCase {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(
        &self,
        old_id: &Id,
        command: UpdateUserCommand,
    ) -> Result<User, UpdateUserError> {
        let mut entity = self.repository.find_by_id(old_id).await?.ok_or_else(|| {
            UpdateUserError::Repository(RepositoryError::NotFound {
                resource: "ユーザー".to_string(),
                key: old_id.value().to_string(),
            })
        })?;

        // パスワードのハッシュ化
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(command.password.as_bytes(), &salt)
            .map_err(|e| UpdateUserError::PasswordHashError(e.to_string()))?
            .to_string();

        entity.update(
            command.id,
            command.name,
            command.role,
            password_hash,
            Utc::now(),
        );

        self.repository
            .update(old_id, &entity)
            .await
            .map_err(UpdateUserError::Repository)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateUserError {
    #[error("パスワードのハッシュ化に失敗しました: {0}")]
    PasswordHashError(String),
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
