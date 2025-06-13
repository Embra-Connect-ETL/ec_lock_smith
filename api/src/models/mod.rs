use rocket::response::Responder;
use serde::{Deserialize, Serialize};

/*----------
 Responses
----------*/
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSecretResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteSecretResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Deserialize, Responder, Serialize)]
pub struct AuthModuleResponse {
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SetupResponse {
    pub status: u16,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoginResponse {
    pub status: u16,
    pub token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DeleteUserResponse {
    pub status: u16,
    pub message: String,
}
