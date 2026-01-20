use crate::internal::domain::repository::SessionRepository;
use std::sync::Arc;

pub struct ValidateCsrfTokenUseCase {
    session_repository: Arc<dyn SessionRepository>,
}

pub struct ValidateCsrfTokenCommand {
    pub session_id: String,
    pub token: String,
}

#[allow(dead_code)]
impl ValidateCsrfTokenUseCase {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }

    /// CSRFトークンを検証
    pub async fn execute(&self, command: ValidateCsrfTokenCommand) -> bool {
        if let Some(session) = self
            .session_repository
            .find_by_session_id(&command.session_id)
            .await
        {
            session.csrf_token() == command.token
        } else {
            false
        }
    }
}
