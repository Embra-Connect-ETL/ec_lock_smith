use crate::config::API_BASE_URL;
use home;
use reqwest::Client;
use serde::Deserialize;
use shared::models::UserCredentials;
use std::fs;
use std::path::PathBuf;

pub struct Auth;

impl Auth {
    pub fn new() -> Self {
        Self
    }

    pub async fn login(&mut self, creds: UserCredentials) -> Result<(), String> {
        let client = Client::new();

        let res = client
            .post(format!("{}/login", API_BASE_URL))
            .json(&creds)
            .send()
            .await
            .map_err(|e| format!("Failed to send login request: {e:?}"))?;

        if !res.status().is_success() {
            return Err(format!("Login failed: {}", res.status()));
        }

        // Deserialize message/token field
        #[derive(Deserialize)]
        struct LoginResponse {
            message: String,
        }

        let login_response: LoginResponse = res
            .json()
            .await
            .map_err(|e| format!("Failed to parse response JSON: {e:?}"))?;

        let token = login_response.message;

        let Some(home_dir) = home::home_dir() else {
            return Err("Error accessing the home directory".to_owned());
        };

        let token_file: PathBuf = home_dir.join(".lock_smith.config");
        fs::write(token_file, token).map_err(|error| error.to_string())?;

        Ok(())
    }
}
