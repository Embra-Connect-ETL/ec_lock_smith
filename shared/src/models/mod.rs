use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/*------------
 Encryption Keys models
-------------*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyPairDocument {
    pub private_key: String,
    pub public_key: String,
    #[serde(
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime",
        rename = "createdAt"
    )]
    pub created_at: DateTime<Utc>,
}

/*------------
 User models
-------------*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserDocument {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub email: String,
    pub password: String,
    #[serde(
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime",
        rename = "createdAt"
    )]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: String,
    pub email: String,
    pub password: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct UserCredentials {
    pub email: String,
    pub password: String,
}

/*------------
 Vault models
-------------*/
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VaultDocument {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub key: String,
    pub value: String,
    pub created_by: String,
    #[serde(
        with = "bson::serde_helpers::chrono_datetime_as_bson_datetime",
        rename = "createdAt"
    )]
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema, Clone)]
pub struct Vault {
    #[serde(rename = "_id")]
    pub id: String,
    pub key: String,
    pub value: String,
    pub created_by: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Secret {
    pub key: String,
    pub value: String,
}
