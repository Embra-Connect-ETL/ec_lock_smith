use shared::{
    db::connect,
    repositories::{keys::KeyRepository, users::UserRepository, vault::VaultRepository},
};

pub mod auth;
pub mod session;

pub async fn get_repos() -> Result<(UserRepository, VaultRepository, KeyRepository), String> {
    let repos = connect().await.map_err(|error| error.to_string())?;
    Ok(repos)
}
