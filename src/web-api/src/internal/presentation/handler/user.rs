pub mod create_user;
pub mod delete_user;
pub mod list_users;
pub mod reset_login_failure_count;
pub mod update_user;

pub use self::{
    create_user::create_user, delete_user::delete_user, list_users::list_users,
    reset_login_failure_count::reset_login_failure_count, update_user::update_user,
};

#[cfg(test)]
async fn prepare_test_data() -> crate::utils::Repositories {
    use crate::{
        internal::{
            domain::{
                entity::User,
                repository::UserRepository,
                value_object::{Id, Role},
            },
            infrastructure::repository::TestUserRepository,
        },
        utils,
    };
    use chrono::DateTime;
    use std::{str::FromStr, sync::Arc};
    use uuid::Uuid;

    let admin = User::construct(
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        Id::new("admin").unwrap(),
        "管理者 太郎",
        Role::Admin,
        "$argon2id$v=19$m=19456,t=2,p=1$Zf/xy2I09QAEAvKnXga60w$arwk9jM50i/6RAjgZ2+N6fiRq0WWJFX3GmngTw+n34Y",
        Uuid::from_str("00000000-0000-7000-8000-000000000000").unwrap(),
        DateTime::from_str("2026-01-20T23:10:24.332+09:00").unwrap(),
        Uuid::from_str("00000000-0000-7000-8000-000000000000").unwrap(),
        DateTime::from_str("2026-01-20T23:10:24.332+09:00").unwrap(),
    );
    let it_staff = User::construct(
        Uuid::from_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap(),
        Id::new("it").unwrap(),
        "情シス 太郎",
        Role::ItStaff,
        "$argon2id$v=19$m=19456,t=2,p=1$20Tk1g6xZ9BdBDcrKqWy1A$//ZKdw5sFbvtSwtbgnBapb3u1r112qUBz6QVG3JuzzU",
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:34.436+09:00").unwrap(),
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:34.436+09:00").unwrap(),
    );
    let doctor = User::construct(
        Uuid::from_str("492236d4-2f18-76ab-a82f-84e29fcf92f8").unwrap(),
        Id::new("doctor").unwrap(),
        "医師 太郎",
        Role::Doctor,
        "$argon2id$v=19$m=19456,t=2,p=1$1E/vEPPwrHBsW1fLuzdUVQ$1sAIm/nnFMIyc1IBuKW8+6KcdyHtdzjHCv7ae8lG6sA",
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:57.855+09:00").unwrap(),
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:57.855+09:00").unwrap(),
    );
    let technician = User::construct(
        Uuid::from_str("49223a37-7e58-717c-b222-754550659249").unwrap(),
        Id::new("technician").unwrap(),
        "技師 太郎",
        Role::Technician,
        "$argon2id$v=19$m=19456,t=2,p=1$HLCrMDHifn55j/Kq5M6t0g$lIVsN8r8+osWzQmU6n5khyRZk8TNeB9/qn4NeULwVfI",
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:26:54.695+09:00").unwrap(),
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:26:54.695+09:00").unwrap(),
    );

    let user_repository = Arc::new(TestUserRepository::new());
    user_repository.add(&admin).await.unwrap();
    user_repository.add(&it_staff).await.unwrap();
    user_repository.add(&doctor).await.unwrap();
    user_repository.add(&technician).await.unwrap();

    let mut repos = utils::Repositories::new_for_test();
    repos.user_repository = user_repository;

    repos
}
