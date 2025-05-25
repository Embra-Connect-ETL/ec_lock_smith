use home;
use prettytable::{Cell, Row, Table};
use std::fs;

use ec_secrets_shared_library::{
    models::{Secret, UserCredentials},
    repositories::{users::UserRepository, vault::VaultRepository},
    utils::auth::{decode_keys, hash_password},
};
use pasetors::{
    Public,
    claims::{Claims, ClaimsValidationRules},
    public,
    token::UntrustedToken,
    version4::V4,
};

use super::get_repos;

pub struct Session {
    claims: Option<Claims>,
    user_repo: Option<UserRepository>,
    vault_repo: Option<VaultRepository>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            claims: None,
            user_repo: None,
            vault_repo: None,
        }
    }

    async fn validate_session(&mut self) -> Result<(), String> {
        let Some(home_dir) = home::home_dir() else {
            return Err("Error acccessing the home directory".to_owned());
        };
        let token_file = home_dir.join(".lock_smith.config");

        let token = fs::read_to_string(token_file).map_err(|error| error.to_string())?;

        let untrusted_token = UntrustedToken::<Public, V4>::try_from(token.as_str())
            .map_err(|error| error.to_string())?;

        let (user_repo, vault_repo, key_repo) = get_repos().await?;

        let keys = decode_keys(&key_repo).await?;

        let validation_rules = ClaimsValidationRules::new();

        let trusted_token =
            public::verify(&keys.1, &untrusted_token, &validation_rules, None, None)
                .map_err(|error| error.to_string())?;

        let Some(claims) = trusted_token.payload_claims() else {
            return Err("Token has no payload".to_owned());
        };

        self.user_repo = Some(user_repo);
        self.claims = Some(claims.clone());
        self.vault_repo = Some(vault_repo);

        Ok(())
    }

    pub async fn get_users(&mut self, id: Option<&str>) -> Result<(), String> {
        let _ = &self.validate_session().await?;

        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Id"),
            Cell::new("Email"),
            Cell::new("CreatedAt"),
        ]));


        let Some(user_repo) = &self.user_repo else {
            return Err("failed to connect to the database".to_owned());
        };

        if let Some(id) = id {
            let _ = user_repo
                .get_user_by_id(id)
                .await
                .map_err(|error| error.to_string())?
                .map(|user| {
                    table.add_row(Row::new(vec![
                        Cell::new(user.id.to_string().as_str()),
                        Cell::new(user.email.as_str()),
                        Cell::new(user.created_at.to_string().as_str()),
                    ]));
                });
        } else {
            let users = user_repo.list_users().await.map_err(|error| {
                println!("Error: {error:?}");
                error.to_string()
            })?;

            users.iter().for_each(|user| {
                table.add_row(Row::new(vec![
                    Cell::new(user.id.to_string().as_str()),
                    Cell::new(user.email.as_str()),
                    Cell::new(user.created_at.to_string().as_str()),
                ]));
            });
        }
        table.printstd();
        Ok(())
    }

    pub async fn delete_user(&mut self, id: Option<&str>) -> Result<(), String> {
        let _ = &self.validate_session().await?;

        let Some(user_repo) = &self.user_repo else {
            return Err("failed to connect to the database".to_owned());
        };

        let Some(id) = id else {
            return Err("Please provide an id for the account to delete".to_owned());
        };

        let _ = user_repo
            .delete_user(id)
            .await
            .map_err(|error| error.to_string())?;

        Ok(())
    }

    pub async fn create_user(&mut self, creds: UserCredentials) -> Result<(), String> {
        let _ = &self.validate_session().await?;

        let Some(user_repo) = &self.user_repo else {
            return Err("failed to connect to the database".to_owned());
        };

        let hashed_pwd = hash_password(creds.password)?;
        let _ = user_repo
            .create_user(&creds.email, &hashed_pwd)
            .await
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub async fn create_secret(&mut self, secret: Secret) -> Result<(), String> {
        let _ = &self.validate_session().await?;

        let Some(vault_repo) = &self.vault_repo else {
            return Err("failed to connect to the database".to_owned());
        };

        let Some(claims) = &self.claims else {
            return Err("Session invalid. Please login.".to_owned());
        };

        let Some(created_by) = claims.get_claim("sub") else {
            return Err("".to_owned());
        };

        let _ = vault_repo
            .create_secret(&secret.key, &secret.value, created_by.to_string().as_str())
            .await
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub async fn list_secrets(&mut self, id: Option<&str>) -> Result<(), String> {
        let _ = &self.validate_session().await?;

        let Some(vault_repo) = &self.vault_repo else {
            return Err("failed to connect to the database".to_owned());
        };

        let Some(claims) = &self.claims else {
            return Err("Session invalid. Please login.".to_owned());
        };

        let mut table = Table::new();

        let Some(created_by) = claims.get_claim("sub") else {
            return Err("".to_owned());
        };

        if let Some(id) = id {
            table.add_row(Row::new(vec![Cell::new("Id"), Cell::new("Secret")]));
            let Some(secret) = vault_repo
                .get_secret_by_id(id, created_by.to_string().as_str())
                .await
                .map_err(|error| error.to_string())?
            else {
                return Err("Invalid secret id".to_owned());
            };
            table.add_row(Row::new(vec![Cell::new(id), Cell::new(secret.as_str())]));
        } else {
            table.add_row(Row::new(vec![
                Cell::new("Id"),
                Cell::new("Key"),
                Cell::new("Value"),
            ]));
            let secrets = vault_repo
                .list_secrets(created_by.to_string().as_str())
                .await
                .map_err(|error| error.to_string())?;
            if secrets.len() <= 0 {
                return Err("No Secrets created yet".to_owned());
            }
            secrets.iter().for_each(|secret| {
                table.add_row(Row::new(vec![
                    Cell::new(secret.id.to_string().as_str()),
                    Cell::new(secret.key.as_str()),
                    Cell::new(secret.value.as_str()),
                ]));
            });
        }
        table.printstd();
        Ok(())
    }

    pub async fn delete_secret(&mut self, id: &str) -> Result<(), String> {
        let _ = &self.validate_session().await?;

        let Some(vault_repo) = &self.vault_repo else {
            return Err("failed to connect to the database".to_owned());
        };

        let Some(claims) = &self.claims else {
            return Err("Session invalid. Please login.".to_owned());
        };

        let Some(created_by) = claims.get_claim("sub") else {
            return Err("".to_owned());
        };

        let _ = vault_repo
            .delete_secret(id, created_by.to_string().as_str())
            .await
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}
