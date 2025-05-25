use home;
use std::fs;

use ec_secrets_shared_library::{
    models::{User, UserCredentials},
    utils::auth::authorize_user,
};

use super::get_repos;

pub struct Auth {}

impl Auth {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn login(&mut self, creds: UserCredentials) -> Result<(), String> {
        let (user_repo, _, key_repo) = get_repos().await?;

        let user_doc = user_repo
            .get_user_by_email(&creds.email)
            .await
            .map_err(|error| error.to_string())?;

        if let Some(user_doc) = user_doc {
            let user = User {
                id: user_doc.id.to_string(),
                email: user_doc.email.clone(),
                password: user_doc.password.clone(),
                created_at: user_doc.created_at.to_rfc3339(),
            };

            let token = authorize_user(&user, &creds, &key_repo).await?;
            let Some(home_dir) = home::home_dir() else {
                return Err("Error acccessing the home directory".to_owned());
            };
            let token_file = home_dir.join(".lock_smith.config");
            fs::write(token_file, token).map_err(|error| error.to_string())?;
        } else {
            return Err("Invalid login credentials".to_owned());
        }

        Ok(())
    }
}
