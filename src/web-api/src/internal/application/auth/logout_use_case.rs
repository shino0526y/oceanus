use crate::internal::application::session::DeleteSessionUseCase;
use std::sync::Arc;

pub struct LogoutUseCase {
    delete_session_use_case: Arc<DeleteSessionUseCase>,
}

impl LogoutUseCase {
    pub fn new(delete_session_use_case: Arc<DeleteSessionUseCase>) -> Self {
        Self {
            delete_session_use_case,
        }
    }

    pub async fn execute(&self, session_id: &str) {
        self.delete_session_use_case.execute(session_id).await;
    }
}
