use crate::internal::domain::{entity::Session, repository::SessionRepository};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct InMemorySessionRepository {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl InMemorySessionRepository {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait::async_trait]
impl SessionRepository for InMemorySessionRepository {
    async fn save(&self, session: Session) {
        self.sessions
            .write()
            .unwrap()
            .insert(session.session_id().to_string(), session);
    }

    async fn find_by_session_id(&self, session_id: &str) -> Option<Session> {
        let (session, is_expired) = {
            let sessions = self.sessions.read().unwrap();
            match sessions.get(session_id) {
                Some(session) => (Some(session.clone()), session.is_expired()),
                None => (None, false),
            }
        };

        if is_expired {
            self.sessions.write().unwrap().remove(session_id);
            return None;
        }

        session
    }

    async fn delete(&self, session_id: &str) {
        self.sessions.write().unwrap().remove(session_id);
    }

    async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.sessions.write().unwrap();
        sessions.retain(|_, session| !session.is_expired());
    }
}

#[cfg(test)]
pub struct TestSessionRepository {
    inner: Arc<RwLock<HashMap<String, Session>>>,
}

#[cfg(test)]
impl TestSessionRepository {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
#[async_trait::async_trait]
impl SessionRepository for TestSessionRepository {
    async fn save(&self, session: Session) {
        self.inner
            .write()
            .unwrap()
            .insert(session.session_id().to_string(), session);
    }

    async fn find_by_session_id(&self, session_id: &str) -> Option<Session> {
        let (session, is_expired) = {
            let sessions = self.inner.read().unwrap();
            match sessions.get(session_id) {
                Some(session) => (Some(session.clone()), session.is_expired()),
                None => (None, false),
            }
        };

        if is_expired {
            self.inner.write().unwrap().remove(session_id);
            return None;
        }

        session
    }

    async fn delete(&self, session_id: &str) {
        self.inner.write().unwrap().remove(session_id);
    }

    async fn cleanup_expired_sessions(&self) {
        let mut sessions = self.inner.write().unwrap();
        sessions.retain(|_, session| !session.is_expired());
    }
}
