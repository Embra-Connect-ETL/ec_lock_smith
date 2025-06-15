#![allow(unused)]
use anyhow::anyhow;
use base64::prelude::BASE64_STANDARD;
use base64::{Engine, engine::general_purpose};
use chrono::Utc;
use futures::stream::TryStreamExt;
use mongodb::{
    Client, Collection,
    bson::{doc, oid::ObjectId},
    error::Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::users::UserRepository;
use crate::models::{UserDocument, VaultDocument, VaultMetadataDocument};
use crate::repositories::quota::{self, QuotaManager};
use crate::utils::vault::{decrypt, encrypt};

#[derive(Debug)]
pub struct VaultRepository {
    collection: Collection<VaultDocument>,
    encryption_key: String,
}

impl VaultRepository {
    /// Create a new repository with a MongoDB collection and a shared SecretVault instance
    pub fn new(client: &Client, db_name: &str, collection_name: &str) -> Self {
        let collection = client
            .database(db_name)
            .collection::<VaultDocument>(collection_name);

        let encryption_key = format!(
            "{}",
            std::env::var("ECS_ENCRYPTION_KEY").expect("[ECS_ENCRYPTION_KEY] must be set")
        );

        Self {
            collection,
            encryption_key,
        }
    }

    /*-----------------
    CREATE a new secret
    --------------------*/
    pub async fn create_secret(
        &self,
        key: &str,
        value: &str,
        created_by: &str,
        user: &UserDocument,
        user_repo: &UserRepository,
    ) -> anyhow::Result<VaultDocument> {
        QuotaManager::enforce_secret_quota(self, user).await?;

        let encrypted_value = encrypt(value.as_bytes(), self.encryption_key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {:?}", e))?;
        let encoded_value = BASE64_STANDARD.encode(&encrypted_value);

        let secret = VaultDocument {
            id: ObjectId::new(),
            key: key.to_string(),
            value: encoded_value,
            created_by: user.id,
            created_at: Utc::now(),
        };

        self.collection.insert_one(&secret).await?;
        QuotaManager::update_quota_on_secret_create(user_repo, user).await?;
        Ok(secret)
    }

    /*---------------
    GET secret by id
    ---------------*/
    pub async fn get_secret_by_id(
        &self,
        id: &str,
        user: &UserDocument,
        user_repo: &UserRepository,
    ) -> anyhow::Result<Option<String>> {
        QuotaManager::enforce_request_quota(user_repo, user).await?;

        let object_id = ObjectId::parse_str(id).map_err(|_| anyhow!("Invalid ObjectId format"))?;

        let filter = doc! { "_id": object_id, "created_by": user.id };

        if let Some(secret) = self.collection.find_one(filter).await? {
            let encoded_value = BASE64_STANDARD
                .decode(&secret.value)
                .map_err(|_| anyhow!("Failed to decode secret value"))?;

            let decrypted_value = decrypt(&encoded_value, self.encryption_key.as_bytes())
                .map_err(|_| anyhow!("Failed to decrypt secret value"))?;

            Ok(Some(String::from_utf8_lossy(&decrypted_value).to_string()))
        } else {
            Ok(None)
        }
    }

    /*-----------------
    GET secret by key
    -------------------*/
    pub async fn get_secret_by_key(
        &self,
        key: &str,
        user: &UserDocument,
        user_repo: &UserRepository,
    ) -> anyhow::Result<Option<String>> {
        QuotaManager::enforce_request_quota(user_repo, user).await?;

        let filter = doc! { "key": key, "created_by": user.id };

        if let Some(secret) = self.collection.find_one(filter).await? {
            let encoded_value = BASE64_STANDARD
                .decode(&secret.value)
                .map_err(|_| anyhow!("Failed to decode secret value"))?;

            let decrypted_value = decrypt(&encoded_value, self.encryption_key.as_bytes())
                .map_err(|_| anyhow!("Failed to decrypt secret value"))?;

            Ok(Some(String::from_utf8_lossy(&decrypted_value).to_string()))
        } else {
            Ok(None)
        }
    }

    /*-----------------
    GET secret by author
    -------------------*/
    pub async fn get_secret_by_author(
        &self,
        author: &str,
        user: &UserDocument,
        user_repo: &UserRepository,
    ) -> anyhow::Result<Vec<VaultMetadataDocument>> {
        QuotaManager::enforce_request_quota(user_repo, user).await?;

        let filter = doc! { "created_by": user.id };
        let mut cursor = self.collection.find(filter).await?;

        let mut secrets = Vec::new();
        while let Some(secret) = cursor.try_next().await? {
            secrets.push(VaultMetadataDocument {
                id: secret.id,
                key: secret.key,
                created_by: author.to_string(),
                created_at: secret.created_at,
            });
        }

        Ok(secrets)
    }

    /*-------------
    DELETE a secret
    ---------------*/
    pub async fn delete_secret(
        &self,
        id: &str,
        user: UserDocument,
        user_repo: &UserRepository,
    ) -> Result<Option<String>> {
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! { "_id": object_id, "created_by": user.id };

        if let Some(secret) = self.collection.find_one_and_delete(filter).await? {
            let encoded_value = BASE64_STANDARD.decode(&secret.value).unwrap();
            let decrypted_value = decrypt(&encoded_value, &self.encryption_key.as_bytes()).unwrap();
            QuotaManager::update_quota_on_secret_delete(user_repo, &user)
                .await
                .unwrap();
            return Ok(Some(String::from_utf8_lossy(&decrypted_value).to_string()));
        }

        Ok(None)
    }

    /*-------------------------------------
    DELETE all secrets created by a user
    -------------------------------------*/
    pub async fn delete_secrets_by_user(&self, user_id: ObjectId) -> Result<u64> {
        // Delete secrets with created_by == user.id
        let filter = doc! { "created_by": user_id };
        let result = self.collection.delete_many(filter).await?;
        Ok(result.deleted_count)
    }

    /*-------------
    LIST all secrets
    ---------------*/
    pub async fn list_secrets(
        &self,
        user: &UserDocument,
        user_repo: &UserRepository,
    ) -> anyhow::Result<Vec<VaultMetadataDocument>> {
        QuotaManager::enforce_request_quota(user_repo, user).await?;

        let mut cursor = self
            .collection
            .find(doc! { "created_by": &user.id })
            .await?;

        let mut secrets = Vec::new();
        while let Some(secret) = cursor.try_next().await? {
            secrets.push(VaultMetadataDocument {
                id: secret.id,
                key: secret.key,
                created_by: user.email.clone(),
                created_at: secret.created_at,
            });
        }

        Ok(secrets)
    }

    pub async fn count_secrets_by_user(&self, user_id: ObjectId) -> Result<u32> {
        let filter = doc! { "created_by": user_id };
        let count = self.collection.count_documents(filter).await?;
        Ok(count as u32)
    }
}
