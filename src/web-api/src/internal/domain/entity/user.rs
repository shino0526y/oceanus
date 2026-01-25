use crate::internal::domain::value_object::{Id, Role};
use chrono::{DateTime, Utc};
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Clone)]
pub struct User {
    uuid: Uuid,
    id: Id,
    name: String,
    role: Role,
    password_hash: String,
    created_by: Uuid,
    created_at: DateTime<Utc>,
    updated_by: Uuid,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn uuid(&self) -> &Uuid {
        &self.uuid
    }

    pub fn id(&self) -> &Id {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn role(&self) -> Role {
        self.role
    }

    pub fn password_hash(&self) -> &str {
        &self.password_hash
    }

    pub fn created_by(&self) -> &Uuid {
        &self.created_by
    }

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_by(&self) -> &Uuid {
        &self.updated_by
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    #[allow(clippy::too_many_arguments)]
    pub fn construct(
        uuid: Uuid,
        id: Id,
        name: impl Into<String>,
        role: Role,
        password_hash: impl Into<String>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            uuid,
            id,
            name: name.into(),
            role,
            password_hash: password_hash.into(),
            created_by,
            created_at,
            updated_by,
            updated_at,
        }
    }

    pub fn create(
        id: Id,
        name: impl Into<String>,
        role: Role,
        password_hash: impl Into<String>,
        created_by: Uuid,
        created_at: DateTime<Utc>,
    ) -> Self {
        let timestamp = Timestamp::from_unix(NoContext, created_at.timestamp_millis() as u64, 0);

        Self {
            uuid: Uuid::new_v7(timestamp),
            id,
            name: name.into(),
            role,
            password_hash: password_hash.into(),
            created_by,
            created_at,
            updated_by: created_by,
            updated_at: created_at,
        }
    }

    /// ユーザーを更新する。ただし、変更があった場合のみ更新を行う。
    ///
    /// # Returns
    /// 変更があった場合は`true`、変更がなかった場合は`false`を返す。
    pub fn update(
        &mut self,
        id: Id,
        name: impl Into<String>,
        role: Role,
        password_hash: impl Into<String>,
        updated_by: Uuid,
        updated_at: DateTime<Utc>,
    ) -> bool {
        assert!(
            updated_at >= self.created_at,
            "`updated_at`は`created_at`よりも前にはできません (created_at={}, updated_at={})",
            self.created_at,
            updated_at,
        );

        // 変更がない場合は何もしない
        let name = name.into();
        let password_hash = password_hash.into();
        if self.id == id
            && self.name == name
            && self.role == role
            && self.password_hash == password_hash
        {
            return false;
        }

        self.id = id;
        self.name = name;
        self.role = role;
        self.password_hash = password_hash;
        self.updated_by = updated_by;
        self.updated_at = updated_at;

        true
    }
}
