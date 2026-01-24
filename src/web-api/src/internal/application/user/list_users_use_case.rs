use crate::internal::domain::{
    entity::User, error::RepositoryError, repository::{LoginFailureCountRepository, UserRepository},
};
use std::sync::Arc;

pub struct ListUsersUseCase {
    user_repository: Arc<dyn UserRepository>,
    login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
}

/// ユーザー情報とログイン失敗情報を含む出力DTO
pub struct UserWithLoginFailureCount {
    pub user: User,
    pub login_failure_count: i16,
}

impl ListUsersUseCase {
    pub fn new(
        user_repository: Arc<dyn UserRepository>,
        login_failure_count_repository: Arc<dyn LoginFailureCountRepository>,
    ) -> Self {
        Self {
            user_repository,
            login_failure_count_repository,
        }
    }

    pub async fn execute(&self) -> Result<Vec<UserWithLoginFailureCount>, RepositoryError> {
        let users = self.user_repository.find_all().await?;

        let mut result = Vec::with_capacity(users.len());
        for user in users {
            let login_failure_count = self
                .login_failure_count_repository
                .find_by_user_uuid(user.uuid())
                .await?
                .map(|lf| lf.failure_count())
                .unwrap_or(0);

            result.push(UserWithLoginFailureCount {
                user,
                login_failure_count,
            });
        }

        Ok(result)
    }
}
