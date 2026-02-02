pub mod create_application_entity;
pub mod delete_application_entity;
pub mod list_application_entities;
pub mod update_application_entity;

pub use self::{
    create_application_entity::create_application_entity,
    delete_application_entity::delete_application_entity,
    list_application_entities::list_application_entities,
    update_application_entity::update_application_entity,
};

#[cfg(test)]
async fn prepare_test_data() -> crate::utils::Repositories {
    use crate::{
        internal::{
            domain::{
                entity::{ApplicationEntity, User},
                repository::{ApplicationEntityRepository, UserRepository},
                value_object::{HostName, Id, Port, Role, UserName},
            },
            infrastructure::repository::{TestApplicationEntityRepository, TestUserRepository},
        },
        utils,
    };
    use chrono::DateTime;
    use dicom_lib::core::value::value_representations::ae::AeValue;
    use std::{str::FromStr, sync::Arc};
    use uuid::Uuid;

    let user_repository = Arc::new(TestUserRepository::new());
    user_repository.add(&User::construct(
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        Id::new("admin").unwrap(),
        UserName::new("管理者 太郎").unwrap(),
        Role::Admin,
        "$argon2id$v=19$m=19456,t=2,p=1$Zf/xy2I09QAEAvKnXga60w$arwk9jM50i/6RAjgZ2+N6fiRq0WWJFX3GmngTw+n34Y",
        Uuid::from_str("00000000-0000-7000-8000-000000000000").unwrap(),
        DateTime::from_str("2026-01-20T23:10:24.332+09:00").unwrap(),
        Uuid::from_str("00000000-0000-7000-8000-000000000000").unwrap(),
        DateTime::from_str("2026-01-20T23:10:24.332+09:00").unwrap(),
    )).await.unwrap();
    user_repository.add(&User::construct(
        Uuid::from_str("4922356e-d6a0-7083-8e18-93b7a023c328").unwrap(),
        Id::new("it").unwrap(),
        UserName::new("情シス 太郎").unwrap(),
        Role::ItStaff,
        "$argon2id$v=19$m=19456,t=2,p=1$20Tk1g6xZ9BdBDcrKqWy1A$//ZKdw5sFbvtSwtbgnBapb3u1r112qUBz6QVG3JuzzU",
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:34.436+09:00").unwrap(),
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:25:34.436+09:00").unwrap(),
    )).await.unwrap();
    user_repository.add(&User::construct(
        Uuid::from_str("49223a37-7e58-717c-b222-754550659249").unwrap(),
        Id::new("technician").unwrap(),
        UserName::new("技師 太郎").unwrap(),
        Role::Technician,
        "$argon2id$v=19$m=19456,t=2,p=1$HLCrMDHifn55j/Kq5M6t0g$lIVsN8r8+osWzQmU6n5khyRZk8TNeB9/qn4NeULwVfI",
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:26:54.695+09:00").unwrap(),
        Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
        DateTime::from_str("2026-01-24T22:26:54.695+09:00").unwrap(),
    )).await.unwrap();
    let application_entity_repository = Arc::new(TestApplicationEntityRepository::new());
    application_entity_repository
        .add(&ApplicationEntity::construct(
            Uuid::from_str("019bdbbf-e0c2-7c24-8c21-5132ac857f26").unwrap(),
            AeValue::from_string("DCMTK").unwrap(),
            HostName::new("localhost").unwrap(),
            Port::from_u16(11112).unwrap(),
            "開発＆デバッグ用",
            Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
            DateTime::from_str("2026-01-20T23:12:23.874+09:00").unwrap(),
            Uuid::from_str("019bdbbe-0dcc-7474-8b43-95b89ca8b4fd").unwrap(),
            DateTime::from_str("2026-01-20T23:12:23.874+09:00").unwrap(),
        ))
        .await
        .unwrap();

    let mut repos = utils::Repositories::new_for_test();
    repos.user_repository = user_repository;
    repos.application_entity_repository = application_entity_repository;

    repos
}
