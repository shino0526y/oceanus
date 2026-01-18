use crate::internal::domain::repository::SessionRepository;
use std::sync::Arc;

#[allow(dead_code)]
pub struct ValidateCsrfTokenUseCase {
    session_repository: Arc<dyn SessionRepository>,
}

#[allow(dead_code)]
impl ValidateCsrfTokenUseCase {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }

    /// CSRFトークンを検証
    pub async fn execute(&self, session_id: &str, token: &str) -> bool {
        if let Some(session) = self.session_repository.find_by_session_id(session_id).await {
            session.csrf_token() == token
        } else {
            false
        }
    }
}
