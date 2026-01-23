use crate::internal::domain::{
    entity::User,
    error::RepositoryError,
    repository::UserRepository,
    value_object::{Id, Role},
};
use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct CreateUserUseCase {
    repository: Arc<dyn UserRepository>,
}

pub struct CreateUserCommand {
    pub id: Id,
    pub name: String,
    pub role: Role,
    pub password: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

impl CreateUserUseCase {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn execute(&self, command: CreateUserCommand) -> Result<User, CreateUserError> {
        // パスワードの空文字チェック
        if command.password.is_empty() {
            return Err(CreateUserError::EmptyPassword);
        }

        // パスワードのハッシュ化(argon2id)
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(command.password.as_bytes(), &salt)
            .map_err(|e| CreateUserError::PasswordHashError(e.to_string()))?
            .to_string();

        let user = User::create(
            command.id,
            command.name,
            command.role,
            password_hash,
            command.created_by,
            command.created_at,
        );

        self.repository
            .add(&user)
            .await
            .map_err(CreateUserError::Repository)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("パスワードは1文字以上で入力してください")]
    EmptyPassword,
    #[error("パスワードのハッシュ化に失敗しました: {0}")]
    PasswordHashError(String),
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
