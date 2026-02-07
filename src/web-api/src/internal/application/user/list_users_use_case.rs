use crate::internal::domain::{
    entity::User,
    error::RepositoryError,
    repository::{LoginFailureCountRepository, UserRepository},
};
use std::{collections::HashMap, sync::Arc};

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
        let user_uuid_to_login_failure_count: HashMap<_, _> = self
            .login_failure_count_repository
            .find_all()
            .await?
            .into_iter()
            .map(|f| (*f.user_uuid(), f.failure_count()))
            .collect();

        let result = users
            .into_iter()
            .map(|user| {
                let login_failure_count = user_uuid_to_login_failure_count
                    .get(user.uuid())
                    .copied()
                    .unwrap_or(0);

                UserWithLoginFailureCount {
                    user,
                    login_failure_count,
                }
            })
            .collect();

        Ok(result)
    }
}
