use chrono::{DateTime, Utc};
use uuid::Uuid;

/// ログイン失敗によるロックのしきい値
const LOGIN_FAILURE_LOCK_THRESHOLD: i16 = 5;

#[derive(Clone)]
pub struct LoginFailureCount {
    user_uuid: Uuid,
    failure_count: i16,
    last_failure_at: Option<DateTime<Utc>>,
}

impl LoginFailureCount {
    pub fn user_uuid(&self) -> &Uuid {
        &self.user_uuid
    }

    pub fn failure_count(&self) -> i16 {
        self.failure_count
    }

    pub fn last_failure_at(&self) -> Option<&DateTime<Utc>> {
        self.last_failure_at.as_ref()
    }

    /// ログインがロックされているかどうかを返す
    pub fn is_locked(&self) -> bool {
        self.failure_count >= LOGIN_FAILURE_LOCK_THRESHOLD
    }

    /// DBから復元するためのコンストラクタ
    pub fn construct(
        user_uuid: Uuid,
        failure_count: i16,
        last_failure_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            user_uuid,
            failure_count,
            last_failure_at,
        }
    }

    /// 新規作成（初期状態）
    pub fn new(user_uuid: Uuid) -> Self {
        Self {
            user_uuid,
            failure_count: 0,
            last_failure_at: None,
        }
    }

    /// ログイン失敗回数をインクリメントする
    pub fn increment(&mut self, failed_at: DateTime<Utc>) {
        self.failure_count = self.failure_count.saturating_add(1);
        self.last_failure_at = Some(failed_at);
    }

    /// ログイン失敗回数をリセットする
    pub fn reset(&mut self) {
        self.failure_count = 0;
        self.last_failure_at = None;
    }
}
