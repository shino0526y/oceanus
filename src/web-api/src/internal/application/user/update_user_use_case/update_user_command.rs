use crate::internal::domain::value_object::{Id, Role};

pub struct UpdateUserCommand {
    pub id: Id,
    pub name: String,
    pub role: Role,
    pub password: String,
}

impl UpdateUserCommand {
    pub fn new(id: Id, name: String, role: Role, password: String) -> Self {
        Self {
            id,
            name,
            role,
            password,
        }
    }
}
