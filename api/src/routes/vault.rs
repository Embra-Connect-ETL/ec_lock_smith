/*-------------
Custom modules
--------------*/
use crate::models::*;
use crate::request_guards::TokenGuard;
use shared::models::{Secret, VaultMetadataDocument};
use shared::repositories::users::UserRepository;
use shared::repositories::vault::VaultRepository;

/*-------------
3rd party modules
--------------*/
use log::{error, info};
use rocket::http::Status;
use rocket::serde::json::Json;
use rocket::{delete, get, post, routes, State};

/*-------------
stdlib modules
--------------*/
use std::sync::Arc;

/*---------------------
 Create a vault entry
---------------------*/
#[post("/create/vault/entry", data = "<secret>")]
pub async fn create_secret(
    vault_repo: &State<Arc<VaultRepository>>,
    user_repo: &State<Arc<UserRepository>>,
    secret: Json<Secret>,
    claims: TokenGuard,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    if let Some(created_by) = claims.0.get_claim("sub") {
        if let Some(created_by) = created_by.as_str() {
            match vault_repo
                .create_secret(&secret.key, &secret.value, created_by, user_repo)
                .await
            {
                Ok(_) => {
                    info!("Vault entry created successfully.");
                    Ok(Json(SuccessResponse {
                        status: Status::Ok.code,
                        message: "Vault entry created successfully".to_string(),
                    }))
                }
                Err(e) => {
                    error!("Failed to create vault entry: {:?}", e);
                    Err(Json(ErrorResponse {
                        status: Status::InternalServerError.code,
                        message: "Failed to create vault entry".to_string(),
                    }))
                }
            }
        } else {
            Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Insufficient Permissions".to_string(),
            }))
        }
    } else {
        Err(Json(ErrorResponse {
            status: Status::Unauthorized.code,
            message: "Insufficient Permissions".to_string(),
        }))
    }
}

/*--------------------------
 Retrieve all vault entries
---------------------------*/
#[get("/retrieve/vault/entries")]
pub async fn list_entries(
    vault_repo: &State<Arc<VaultRepository>>,
    user_repo: &State<Arc<UserRepository>>,
    token: TokenGuard,
) -> Result<Json<Vec<VaultMetadataDocument>>, Json<ErrorResponse>> {
    if let Some(subject) = token.0.get_claim("sub") {
        if let Some(subject) = subject.as_str() {
            let user = user_repo
                .get_user_by_email(subject)
                .await
                .unwrap()
                .ok_or_else(|| anyhow::anyhow!("User not found: {}", subject))
                .unwrap();

            match vault_repo.list_secrets(subject, user).await {
                Ok(entries) => {
                    info!("Successfully retrieved {} vault entries.", entries.len());
                    Ok(Json(entries)) // Always return an array, even if empty
                }
                Err(_) => {
                    error!("Failed to retrieve vault entries.");
                    Err(Json(ErrorResponse {
                        status: Status::InternalServerError.code,
                        message: "Failed to retrieve vault entries.".to_string(),
                    }))
                }
            }
        } else {
            Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Insufficient Permissions".to_string(),
            }))
        }
    } else {
        Err(Json(ErrorResponse {
            status: Status::Unauthorized.code,
            message: "Insufficient Permissions".to_string(),
        }))
    }
}

/*-----------------------------
 Retrieve a vault entry by id
------------------------------*/
#[get("/retrieve/vault/entries/<id>")]
pub async fn get_entry(
    vault_repo: &State<Arc<VaultRepository>>,
    user_repo: &State<Arc<UserRepository>>,
    id: &str,
    token: TokenGuard,
) -> Result<Json<String>, Json<ErrorResponse>> {
    if id.trim().is_empty() {
        error!("Invalid request: Provided ID is empty.");
        return Err(Json(ErrorResponse {
            status: Status::BadRequest.code,
            message: "Invalid ID provided.".to_string(),
        }));
    }

    if let Some(subject) = token.0.get_claim("sub") {
        if let Some(subject) = subject.as_str() {
            let user = user_repo
                .get_user_by_email(subject)
                .await
                .unwrap()
                .ok_or_else(|| anyhow::anyhow!("User not found: {}", subject))
                .unwrap();

            match vault_repo.get_secret_by_id(&id, user.id).await {
                Ok(Some(entry)) => {
                    info!("Successfully retrieved vault entry with ID: {}", id);
                    Ok(Json(entry))
                }
                Ok(None) => {
                    error!("Vault entry not found with ID: {}", id);
                    Err(Json(ErrorResponse {
                        status: Status::NotFound.code,
                        message: "Vault entry not found.".to_string(),
                    }))
                }
                Err(e) => {
                    error!(
                        "Failed to retrieve vault entry by ID: {}. Error: {:?}",
                        id, e
                    );
                    Err(Json(ErrorResponse {
                        status: Status::InternalServerError.code,
                        message: "Failed to retrieve vault entry.".to_string(),
                    }))
                }
            }
        } else {
            Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Insufficient Permissions".to_string(),
            }))
        }
    } else {
        Err(Json(ErrorResponse {
            status: Status::Unauthorized.code,
            message: "Insufficient Permissions".to_string(),
        }))
    }
}

/*-----------------------------
 Retrieve a vault entry by key
------------------------------*/
#[get("/retrieve/vault/entry/key/<key>")]
pub async fn get_entry_by_key(
    vault_repo: &State<Arc<VaultRepository>>,
    user_repo: &State<Arc<UserRepository>>,
    key: &str,
    token: TokenGuard,
) -> Result<Json<String>, Json<ErrorResponse>> {
    if key.trim().is_empty() {
        error!("Invalid request: Provided Key is empty.");
        return Err(Json(ErrorResponse {
            status: Status::BadRequest.code,
            message: "Invalid Key provided.".to_string(),
        }));
    }

    if let Some(subject) = token.0.get_claim("sub") {
        if let Some(subject) = subject.as_str() {
            let user = user_repo
                .get_user_by_email(subject)
                .await
                .unwrap()
                .ok_or_else(|| anyhow::anyhow!("User not found: {}", subject))
                .unwrap();

            match vault_repo.get_secret_by_key(&key, user.id).await {
                Ok(Some(entry)) => {
                    info!("Successfully retrieved vault entry with Key: {}", key);
                    Ok(Json(entry))
                }
                Ok(None) => {
                    error!("Vault entry not found with Key: {}", key);
                    Err(Json(ErrorResponse {
                        status: Status::NotFound.code,
                        message: "Vault entry not found.".to_string(),
                    }))
                }
                Err(e) => {
                    error!(
                        "Failed to retrieve vault entry by Key: {}. Error: {:?}",
                        key, e
                    );
                    Err(Json(ErrorResponse {
                        status: Status::InternalServerError.code,
                        message: "Failed to retrieve vault entry.".to_string(),
                    }))
                }
            }
        } else {
            Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Insufficient Permissions".to_string(),
            }))
        }
    } else {
        Err(Json(ErrorResponse {
            status: Status::Unauthorized.code,
            message: "Insufficient Permissions".to_string(),
        }))
    }
}

/*---------------------------------
 Retrieve a vault entry by author
----------------------------------*/
#[get("/retrieve/vault/entry/<created_by>")]
pub async fn get_entry_by_author(
    vault_repo: &State<Arc<VaultRepository>>,
    user_repo: &State<Arc<UserRepository>>,
    created_by: &str,
    token: TokenGuard,
) -> Result<Json<Vec<VaultMetadataDocument>>, Json<ErrorResponse>> {
    match token.0.get_claim("sub").and_then(|sub| sub.as_str()) {
        Some(subject) => match user_repo.get_user_by_email(subject).await {
            Ok(Some(user)) => match vault_repo.get_secret_by_author(created_by, user.id).await {
                Ok(secrets) if !secrets.is_empty() => {
                    info!(
                        "Successfully retrieved {} vault entries for author: {}",
                        secrets.len(),
                        subject
                    );
                    Ok(Json(secrets))
                }
                Ok(_) => {
                    error!("No vault entries found for author: {}", subject);
                    Err(Json(ErrorResponse {
                        status: Status::NotFound.code,
                        message: "No vault entries found.".to_string(),
                    }))
                }
                Err(e) => {
                    error!(
                        "Failed to retrieve vault entries for author: {}. Error: {:?}",
                        subject, e
                    );
                    Err(Json(ErrorResponse {
                        status: Status::InternalServerError.code,
                        message: "Failed to retrieve vault entries.".to_string(),
                    }))
                }
            },
            Ok(None) => Err(Json(ErrorResponse {
                status: Status::NotFound.code,
                message: format!("User not found: {}", subject),
            })),
            Err(_) => Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Failed to retrieve user.".to_string(),
            })),
        },
        None => Err(Json(ErrorResponse {
            status: Status::Unauthorized.code,
            message: "Insufficient Permissions".to_string(),
        })),
    }
}

/*---------------------
 Delete a vault entry
----------------------*/
#[delete("/delete/<id>")]
pub async fn delete_entry(
    vault_repo: &State<Arc<VaultRepository>>,
    user_repo: &State<Arc<UserRepository>>,
    id: &str,
    token: TokenGuard,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    if id.trim().is_empty() || id.contains(char::is_whitespace) {
        error!("Invalid request: Provided ID '{}' is invalid.", id);
        return Err(Json(ErrorResponse {
            status: Status::BadRequest.code,
            message: "Invalid ID provided for deletion.".to_string(),
        }));
    }

    if let Some(subject) = token.0.get_claim("sub") {
        if let Some(subject) = subject.as_str() {
            let user = user_repo
                .get_user_by_email(subject)
                .await
                .unwrap()
                .ok_or_else(|| anyhow::anyhow!("User not found: {}", subject))
                .unwrap();

            match vault_repo.delete_secret(&id, user).await {
                Ok(Some(_)) => {
                    info!("Successfully deleted vault entry with ID: {}", id);
                    Ok(Json(SuccessResponse {
                        status: Status::Ok.code,
                        message: "Vault entry deleted successfully.".to_string(),
                    }))
                }
                Ok(None) => {
                    error!("Vault entry not found for deletion with ID: {}", id);
                    Err(Json(ErrorResponse {
                        status: Status::NotFound.code,
                        message: "Vault entry not found.".to_string(),
                    }))
                }
                Err(e) => {
                    error!(
                        "Failed to delete vault entry with ID: {}. Error: {:?}",
                        id, e
                    );
                    Err(Json(ErrorResponse {
                        status: Status::InternalServerError.code,
                        message: "Failed to delete vault entry.".to_string(),
                    }))
                }
            }
        } else {
            Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Insufficient Permissions".to_string(),
            }))
        }
    } else {
        Err(Json(ErrorResponse {
            status: Status::Unauthorized.code,
            message: "Insufficient Permissions".to_string(),
        }))
    }
}

pub fn vault_routes() -> Vec<rocket::Route> {
    routes![
        create_secret,
        list_entries,
        get_entry,
        get_entry_by_author,
        get_entry_by_key,
        delete_entry
    ]
}
