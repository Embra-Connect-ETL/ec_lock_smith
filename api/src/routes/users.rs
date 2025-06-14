/*-------------
Custom modules
--------------*/
use crate::{
    models::{ErrorResponse, SuccessResponse},
    request_guards::TokenGuard,
};
use shared::{
    models::{User, UserCredentials, UserDocument},
    repositories::{keys::KeyRepository, users::UserRepository},
    utils::auth::{authorize_user, hash_password},
};

/*-------------
3rd party modules
--------------*/
use rocket::http::{Cookie, CookieJar, Status};
use rocket::serde::json::Json;
use rocket::{delete, get, post, put, routes, State};

/*-------------
stdlib modules
--------------*/
use std::sync::Arc;

#[post("/setup", data = "<credentials>")]
pub async fn setup(
    repo: &State<Arc<UserRepository>>,
    credentials: Json<UserCredentials>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    // Check if the user already exists
    if let Ok(Some(_)) = repo.get_user_by_email(&credentials.email).await {
        return Err(Json(ErrorResponse {
            status: Status::Conflict.code,
            message: "A user with this email already exists".to_string(),
        }));
    }

    let hashed_password = match hash_password(credentials.password.clone()) {
        Ok(hash) => hash,
        Err(_e) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Something went wrong, please try again later".to_string(),
            }));
        }
    };

    let _ = match repo.create_user(&credentials.email, &hashed_password).await {
        Ok(user) => user,
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Failed to setup account".to_string(),
            }));
        }
    };

    Ok(Json(SuccessResponse {
        status: Status::Ok.code,
        message: "User registered successfully".to_string(),
    }))
}

#[post("/login", data = "<credentials>")]
pub async fn login(
    repo: &State<Arc<UserRepository>>,
    key_repo: &State<Arc<KeyRepository>>,
    credentials: Json<UserCredentials>,
    cookies: &CookieJar<'_>,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    let user_document = match repo.get_user_by_email(&credentials.email).await {
        Ok(Some(user_document)) => user_document,
        Ok(None) => {
            return Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Wrong email or password".to_string(),
            }))
        }
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Something went wrong, please try again later".to_string(),
            }))
        }
    };

    let user = User {
        id: user_document.id.to_string(),
        email: user_document.email.clone(),
        password: user_document.password.clone(),
        created_at: user_document.created_at.to_rfc3339(),
    };

    let token = match authorize_user(&user, &credentials, key_repo).await {
        Ok(token) => token,
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Wrong email or password".to_string(),
            }))
        }
    };

    // Set the token cookie (HTTP-only, secure)
    let cookie = Cookie::build(("auth_token", token.clone()))
        .http_only(true)
        .secure(false) // Switch to HTTPS
        .path("/");

    cookies.add(cookie);

    Ok(Json(SuccessResponse {
        status: Status::Ok.code,
        message: token,
    }))
}

#[post("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Json<SuccessResponse> {
    cookies.remove(Cookie::build(("auth_token", "")).path("/"));
    Json(SuccessResponse {
        status: 200,
        message: "Logged out successfully".to_string(),
    })
}

#[get("/users/<email>")]
pub async fn get_user(
    repo: &State<Arc<UserRepository>>,
    email: &str,
    token: TokenGuard,
) -> Result<Json<UserDocument>, Json<ErrorResponse>> {
    // Extract the subject (user Email) from the token
    let subject = match token.0.get_claim("sub").and_then(|sub| sub.as_str()) {
        Some(sub) => sub,
        None => {
            return Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Insufficient permissions.".to_string(),
            }))
        }
    };

    // Check if the subject matches the requested email
    if subject != email {
        return Err(Json(ErrorResponse {
            status: Status::Unauthorized.code,
            message: "You are not authorized to access this user.".to_string(),
        }));
    }

    // Fetch the user
    match repo.get_user_by_email(&email).await {
        Ok(Some(user)) => Ok(Json(user)),
        Ok(None) => Err(Json(ErrorResponse {
            status: Status::NotFound.code,
            message: "User not found.".to_string(),
        })),
        Err(_) => Err(Json(ErrorResponse {
            status: Status::InternalServerError.code,
            message: "Something went wrong, please try again later.".to_string(),
        })),
    }
}

#[put("/update/<id>", data = "<credentials>")]
pub async fn update_user(
    repo: &State<Arc<UserRepository>>,
    id: String,
    credentials: Json<UserCredentials>,
) -> Result<Json<UserDocument>, Json<ErrorResponse>> {
    // Check if the email is already in use by another user
    if let Ok(Some(existing_user)) = repo.get_user_by_email(&credentials.email).await {
        // If the email exists and it's not the user being updated
        if existing_user.id.to_string() != id {
            return Err(Json(ErrorResponse {
                status: Status::Conflict.code,
                message: "A user with this email already exists".to_string(),
            }));
        }
    }

    let hashed_password = match hash_password(credentials.password.clone()) {
        Ok(hash) => hash,
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Something went wrong, please try again later".to_string(),
            }))
        }
    };

    let user = match repo
        .update_user(&id, Some(&credentials.email), Some(&hashed_password))
        .await
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(Json(ErrorResponse {
                status: Status::NotFound.code,
                message: "User not found".to_string(),
            }))
        }
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Something went wrong, please try again later".to_string(),
            }))
        }
    };

    Ok(Json(user))
}

#[delete("/delete/user/<id>")]
pub async fn delete_user(
    repo: &State<Arc<UserRepository>>,
    id: String,
) -> Result<Json<SuccessResponse>, Json<ErrorResponse>> {
    match repo.delete_user(&id).await {
        Ok(Some(_)) => Ok(Json(SuccessResponse {
            status: Status::Ok.code,
            message: "User deleted successfully".to_string(),
        })),
        Ok(None) => {
            return Err(Json(ErrorResponse {
                status: Status::NotFound.code,
                message: "User not found".to_string(),
            }))
        }
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Something went wrong, please try again later".to_string(),
            }))
        }
    }
}

pub fn user_routes() -> Vec<rocket::Route> {
    routes![setup, login, logout, get_user, update_user, delete_user]
}
