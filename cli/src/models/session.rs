use chrono::{TimeZone, Utc};
use home;
use prettytable::{Cell, Row, Table};
use reqwest::{
    Client,
    header::{AUTHORIZATION, HeaderMap, HeaderValue},
};
use std::fs;

use crate::config::API_BASE_URL;
use shared::models::{Secret, UserCredentials};

pub struct Session {
    token: Option<String>,
    client: Client,
}

impl Session {
    pub fn new() -> Self {
        Self {
            token: None,
            client: Client::new(),
        }
    }

    async fn load_token(&mut self) -> Result<(), String> {
        let Some(home_dir) = home::home_dir() else {
            return Err("Error accessing the home directory".to_owned());
        };
        let token_file = home_dir.join(".lock_smith.config");

        let token = fs::read_to_string(token_file).map_err(|e| e.to_string())?;
        self.token = Some(token.trim().to_string());
        Ok(())
    }

    fn auth_headers(&self) -> Result<HeaderMap, String> {
        let mut headers = HeaderMap::new();
        if let Some(ref token) = self.token {
            let value = format!("Bearer {}", token);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&value).map_err(|e| e.to_string())?,
            );
        }
        Ok(headers)
    }

    pub async fn create_user(&self, creds: UserCredentials) -> Result<(), String> {
        let url = format!("{}/setup", API_BASE_URL);
        self.client
            .post(url)
            .json(&creds)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn get_user(&mut self, email: &str) -> Result<(), String> {
        self.load_token().await?;
        let url = format!("{}/users/{}", API_BASE_URL, email);

        let res = self
            .client
            .get(url)
            .headers(self.auth_headers()?)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;

        let user: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;
        let id = user["_id"]["$oid"].as_str().unwrap_or("N/A");
        let email = user["email"].as_str().unwrap_or("N/A");

        // Extract the createdAt timestamp string (milliseconds since epoch)
        let created_at_millis_str = user["createdAt"]["$date"]["$numberLong"]
            .as_str()
            .unwrap_or("");

        // Convert the string milliseconds to integer, then to chrono DateTime for readable format
        let created_at = if let Ok(millis) = created_at_millis_str.parse::<i64>() {
            // Convert milliseconds to seconds and nanoseconds parts
            let secs = millis / 1000;
            let nsecs = ((millis % 1000) * 1_000_000) as u32;
            let dt = Utc.timestamp_opt(secs, nsecs).single();

            if let Some(dt) = dt {
                dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
            } else {
                "Invalid timestamp".to_string()
            }
        } else {
            "N/A".to_string()
        };

        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Id"),
            Cell::new("Email"),
            Cell::new("CreatedAt"),
        ]));

        table.add_row(Row::new(vec![
            Cell::new(id),
            Cell::new(email),
            Cell::new(&created_at),
        ]));

        table.printstd();

        Ok(())
    }

    pub async fn delete_user(&mut self, user_id: &str) -> Result<(), String> {
        self.load_token().await?;
        let url = format!("{}/delete/user/{}", API_BASE_URL, user_id);
        self.client
            .delete(url)
            .headers(self.auth_headers()?)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn create_secret(&mut self, secret: Secret) -> Result<(), String> {
        self.load_token().await?;
        let url = format!("{}/create/vault/entry", API_BASE_URL);
        self.client
            .post(url)
            .headers(self.auth_headers()?)
            .json(&secret)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn list_secrets(&mut self) -> Result<(), String> {
        self.load_token().await?;
        let url = format!("{}/retrieve/vault/entries", API_BASE_URL);
        let res = self
            .client
            .get(url)
            .headers(self.auth_headers()?)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;

        let secrets: Vec<serde_json::Value> = res.json().await.map_err(|e| e.to_string())?;

        let mut table = Table::new();
        table.add_row(Row::new(vec![
            Cell::new("Id"),
            Cell::new("Key"),
            Cell::new("Created By"),
            Cell::new("Created At"),
        ]));

        for secret in secrets {
            let id = secret["_id"]["$oid"].as_str().unwrap_or("");
            let key = secret["key"].as_str().unwrap_or("");
            let created_by = secret["created_by"].as_str().unwrap_or("");

            // Extract the createdAt timestamp string (milliseconds since epoch)
            let created_at_millis_str = secret["createdAt"]["$date"]["$numberLong"]
                .as_str()
                .unwrap_or("");

            // Convert the string milliseconds to integer, then to chrono DateTime for readable format
            let created_at = if let Ok(millis) = created_at_millis_str.parse::<i64>() {
                let secs = millis / 1000;
                let nsecs = ((millis % 1000) * 1_000_000) as u32;
                let dt = Utc.timestamp_opt(secs, nsecs).single();

                if let Some(dt) = dt {
                    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                } else {
                    "Invalid timestamp".to_string()
                }
            } else {
                "N/A".to_string()
            };

            table.add_row(Row::new(vec![
                Cell::new(id),
                Cell::new(key),
                Cell::new(created_by),
                Cell::new(&created_at),
            ]));
        }

        table.printstd();
        Ok(())
    }

    pub async fn get_secret_value(&mut self, id: &str) -> Result<String, String> {
        self.load_token().await?;
        let url = format!("{}/retrieve/vault/entries/{}", API_BASE_URL, id);

        let res = self
            .client
            .get(url)
            .headers(self.auth_headers()?)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?;

        // Deserialize the JSON string value
        let value: String = res.json().await.map_err(|e| e.to_string())?;
        Ok(value)
    }

    pub async fn delete_secret(&mut self, secret_id: &str) -> Result<String, String> {
        self.load_token().await?;
        let url = format!("{}/delete/{}", API_BASE_URL, secret_id);

        let res = self
            .client
            .delete(url)
            .headers(self.auth_headers()?)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

        let message = json["message"]
            .as_str()
            .unwrap_or("Unknown response")
            .to_string();

        let status = json["status"].as_i64().unwrap_or(200);
        if status != 200 {
            Err(message)
        } else {
            Ok(message)
        }
    }
}
