use crate::internal::domain::{entity::Session, repository::SessionRepository};
use std::sync::Arc;

pub struct CreateSessionUseCase {
    session_repository: Arc<dyn SessionRepository>,
}

impl CreateSessionUseCase {
    pub fn new(session_repository: Arc<dyn SessionRepository>) -> Self {
        Self { session_repository }
    }

    /// セッションを作成し、セッションIDとCSRFトークンを返す
    pub async fn execute(&self, user_id: String) -> (String, String) {
        let session = Session::create(user_id);
        let session_id = session.session_id().to_string();
        let csrf_token = session.csrf_token().to_string();
        self.session_repository.save(session).await;
        (session_id, csrf_token)
    }
}
