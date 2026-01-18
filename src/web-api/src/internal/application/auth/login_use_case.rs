use crate::internal::application::{
    auth::{AuthenticateUserUseCase, authenticate_user_use_case::AuthenticationError},
    session::CreateSessionUseCase,
};
use crate::internal::domain::value_object::Id;
use std::sync::Arc;

pub struct LoginUseCase {
    authenticate_user_use_case: Arc<AuthenticateUserUseCase>,
    create_session_use_case: Arc<CreateSessionUseCase>,
}

impl LoginUseCase {
    pub fn new(
        authenticate_user_use_case: Arc<AuthenticateUserUseCase>,
        create_session_use_case: Arc<CreateSessionUseCase>,
    ) -> Self {
        Self {
            authenticate_user_use_case,
            create_session_use_case,
        }
    }

    pub async fn execute(
        &self,
        user_id: &Id,
        password: &str,
    ) -> Result<(String, String), AuthenticationError> {
        // 認証
        let user_id = self
            .authenticate_user_use_case
            .execute(user_id, password)
            .await?;

        // セッション確立
        let (session_id, csrf_token) = self.create_session_use_case.execute(user_id).await;

        Ok((session_id, csrf_token))
    }
}
