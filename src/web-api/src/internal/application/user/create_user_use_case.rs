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
use std::sync::Arc;

pub struct CreateUserUseCase {
    repository: Arc<dyn UserRepository>,
}

impl CreateUserUseCase {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub async fn create_user(
        &self,
        id: Id,
        name: impl Into<String>,
        role: Role,
        password: impl Into<String>,
    ) -> Result<User, CreateUserError> {
        // パスワードのハッシュ化(argon2id)
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.into().as_bytes(), &salt)
            .map_err(|e| CreateUserError::PasswordHashError(e.to_string()))?
            .to_string();

        let now = chrono::Utc::now();

        let user = User::new(id, name, role, password_hash, now, now);

        self.repository
            .add(user)
            .await
            .map_err(CreateUserError::Repository)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateUserError {
    #[error("パスワードのハッシュ化に失敗しました: {0}")]
    PasswordHashError(String),
    #[error("{0}")]
    Repository(#[from] RepositoryError),
}
