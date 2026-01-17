use crate::internal::domain::value_object::{Id, Role};
use chrono::{DateTime, Utc};

pub struct User {
    id: Id,
    name: String,
    role: Role,
    password_hash: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        id: Id,
        name: impl Into<String>,
        role: Role,
        password_hash: impl Into<String>,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            role,
            password_hash: password_hash.into(),
            created_at,
            updated_at,
        }
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

    pub fn created_at(&self) -> &DateTime<Utc> {
        &self.created_at
    }

    pub fn updated_at(&self) -> &DateTime<Utc> {
        &self.updated_at
    }

    pub fn update(
        &mut self,
        id: Id,
        name: impl Into<String>,
        role: Role,
        password_hash: impl Into<String>,
        updated_at: DateTime<Utc>,
    ) {
        assert!(
            updated_at >= self.created_at,
            "`updated_at`は`created_at`よりも前にはできません (created_at={}, updated_at={})",
            self.created_at,
            updated_at,
        );

        self.id = id;
        self.name = name.into();
        self.role = role;
        self.password_hash = password_hash.into();
        self.updated_at = updated_at;
    }
}
