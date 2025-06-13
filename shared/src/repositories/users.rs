#![allow(unused)]
use chrono::{DateTime, Utc};
use futures::stream::TryStreamExt;
use mongodb::{
    Client, Collection,
    bson::{doc, oid::ObjectId},
    error::{Error, Result},
    options::ClientOptions,
};
use serde::{Deserialize, Serialize};

use crate::{models::UserDocument, utils::auth::hash_password};

#[derive(Debug)]
pub struct UserRepository {
    collection: Collection<UserDocument>,
}

impl UserRepository {
    pub fn new(client: &Client, db_name: &str, collection_name: &str) -> Self {
        let collection = client
            .database(db_name)
            .collection::<UserDocument>(collection_name);
        Self { collection }
    }

    /*-----------------
    CREATE a new user
    --------------------*/
    pub async fn create_user(&self, email: &str, password: &str) -> Result<UserDocument> {
        if let Some(_) = self.collection.find_one(doc! { "email": email }).await? {
            return Err(Error::from(std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "A user with this email already exists.",
            )));
        }

        let user = UserDocument {
            id: ObjectId::new(),
            email: email.to_string(),
            password: password.to_string(),
            created_at: Utc::now(),
        };

        self.collection.insert_one(&user).await?;

        Ok(user)
    }

    /*-------------
    GET user by id
    ---------------*/
    pub async fn get_user_by_id(&self, id: &str) -> Result<Option<UserDocument>> {
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! { "_id": object_id };
        let user = self.collection.find_one(filter).await?;
        Ok(user)
    }

    /*----------------
    GET user by email
    ---------------*/
    pub async fn get_user_by_email(&self, email: &str) -> Result<Option<UserDocument>> {
        let filter = doc! { "email": email };
        let user = self.collection.find_one(filter).await?;
        Ok(user)
    }

    /*-------------
    UPDATE a user
    -------------*/
    pub async fn update_user(
        &self,
        id: &str,
        email: Option<&str>,
        password: Option<&str>,
    ) -> Result<Option<UserDocument>> {
        let object_id = ObjectId::parse_str(id).unwrap();
        let mut update_doc = doc! {};

        if let Some(email) = email {
            update_doc.insert("email", email);
        }
        if let Some(password) = password {
            update_doc.insert("password", password);
        }

        if update_doc.is_empty() {
            return Ok(None);
        }

        let filter = doc! { "_id": object_id };
        let update = doc! { "$set": update_doc };

        let user = self.collection.find_one_and_update(filter, update).await?;
        Ok(user)
    }

    /*-------------
    DELETE a user
    ---------------*/
    pub async fn delete_user(&self, id: &str) -> Result<Option<UserDocument>> {
        let object_id = ObjectId::parse_str(id).unwrap();
        let filter = doc! { "_id": object_id };
        let user = self.collection.find_one_and_delete(filter).await?;
        Ok(user)
    }

    /*-------------
    GET all users
    ---------------*/
    pub async fn list_users(&self) -> Result<Vec<UserDocument>> {
        let filter = doc! {};
        let mut cursor = self.collection.find(filter).await?;
        let mut users = Vec::new();

        while let Some(user) = cursor.try_next().await? {
            users.push(user);
        }

        Ok(users)
    }
}
