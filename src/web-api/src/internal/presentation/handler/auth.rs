pub mod login;
pub mod logout;
pub mod me;

pub use self::{login::login, logout::logout, me::me};

#[cfg(test)]
pub(crate) async fn prepare_test_data() -> crate::utils::Repositories {
    use crate::{
        internal::{
            domain::{
                entity::User,
                repository::UserRepository,
                value_object::{Id, Role, UserName},
            },
            infrastructure::repository::TestUserRepository,
        },
        utils,
    };
    use chrono::DateTime;
    use std::{str::FromStr, sync::Arc};
    use uuid::Uuid;

    let user_repository = Arc::new(TestUserRepository::new());
    user_repository.add(&User::construct(
        Uuid::from_str("492236d4-2f18-76ab-a82f-84e29fcf92f8").unwrap(),
        Id::new("doctor").unwrap(),
        UserName::new("医師 太郎").unwrap(),
        Role::Doctor,
        "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA",
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:57.855+09:00").unwrap(),
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:57.855+09:00").unwrap(),
    )).await.unwrap();

    let mut repos = utils::Repositories::new_for_test();
    repos.user_repository = user_repository;

    repos
}
