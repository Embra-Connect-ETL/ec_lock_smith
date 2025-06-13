#![allow(unused)]
use shared::db::connect;
use rocket::fairing::AdHoc;
use std::sync::Arc;

/*-------------
Custom modules
---------------*/
use shared::repositories::{
    keys::KeyRepository, users::UserRepository, vault::VaultRepository,
};

pub fn init() -> AdHoc {
    AdHoc::on_ignite(
        "Establish connection with Database cluster",
        |rocket| async {
            match connect().await {
                Ok((user_repository, vault_repository, key_repository)) => rocket
                    .manage(Arc::new(user_repository))
                    .manage(Arc::new(vault_repository))
                    .manage(Arc::new(key_repository)),
                Err(error) => {
                    panic!("Cannot connect to instance:: {:?}", error)
                }
            }
        },
    )
}
