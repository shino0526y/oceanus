use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Session {
    session_id: String,
    user_uuid: Uuid,
    csrf_token: String,
    expires_at: DateTime<Utc>,
}

impl Session {
    /// セッションのデフォルト有効期限（分）
    pub const DEFAULT_EXPIRY_MINUTES: i64 = 30;

    /// 新規セッションを作成する
    pub fn create(user_uuid: Uuid) -> Self {
        let session_id = Self::generate_session_id();
        let csrf_token = Self::generate_csrf_token();
        let expires_at = Utc::now() + Duration::minutes(Self::DEFAULT_EXPIRY_MINUTES);

        Self {
            session_id,
            user_uuid,
            csrf_token,
            expires_at,
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn user_uuid(&self) -> &Uuid {
        &self.user_uuid
    }

    pub fn csrf_token(&self) -> &str {
        &self.csrf_token
    }

    #[allow(dead_code)]
    pub fn expires_at(&self) -> &DateTime<Utc> {
        &self.expires_at
    }

    /// セッションが有効期限切れかどうかを判定する
    pub fn is_expired(&self) -> bool {
        Utc::now() >= self.expires_at
    }

    /// CSRFトークンを検証する
    pub fn validate_csrf_token(&self, token: &str) -> bool {
        self.csrf_token == token
    }

    /// セッションの有効期限を延長する
    pub fn extend(&mut self) {
        self.expires_at = Utc::now() + Duration::minutes(Self::DEFAULT_EXPIRY_MINUTES);
    }

    fn generate_session_id() -> String {
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
        Self::base16_encode(&bytes)
    }

    fn generate_csrf_token() -> String {
        let mut rng = rand::rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.random::<u8>()).collect();
        Self::base16_encode(&bytes)
    }

    fn base16_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }
}
