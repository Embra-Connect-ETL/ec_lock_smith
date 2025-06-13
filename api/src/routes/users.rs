/*-------------
Custom modules
--------------*/
use crate::models::{DeleteUserResponse, ErrorResponse, LoginResponse, SetupResponse};
use shared::{
    models::{User, UserCredentials, UserDocument},
    repositories::{keys::KeyRepository, users::UserRepository},
    utils::auth::{authorize_user, hash_password}
};

/*-------------
3rd party modules
--------------*/
use rocket::http::Status;
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
) -> Result<Json<SetupResponse>, Json<ErrorResponse>> {
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
                message: "Internal server error".to_string(),
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

    Ok(Json(SetupResponse {
        status: Status::Ok.code,
        message: "User registered successfully".to_string(),
    }))
}

#[post("/login", data = "<credentials>")]
pub async fn login(
    repo: &State<Arc<UserRepository>>,
    key_repo: &State<Arc<KeyRepository>>,
    credentials: Json<UserCredentials>,
) -> Result<Json<LoginResponse>, Json<ErrorResponse>> {
    let user_document = match repo.get_user_by_email(&credentials.email).await {
        Ok(Some(user_document)) => user_document,
        Ok(None) => {
            return Err(Json(ErrorResponse {
                status: Status::Unauthorized.code,
                message: "Invalid email or password".to_string(),
            }))
        }
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Internal server error".to_string(),
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
                message: "Invalid email or password".to_string(),
            }))
        }
    };

    Ok(Json(LoginResponse {
        status: Status::Ok.code,
        token,
    }))
}

#[get("/users")]
pub async fn list_users(
    repo: &State<Arc<UserRepository>>,
) -> Result<Json<Vec<UserDocument>>, Json<ErrorResponse>> {
    let users = match repo.list_users().await {
        Ok(users) => users,
        Err(_) => {
            return Err(Json(ErrorResponse {
                status: Status::InternalServerError.code,
                message: "Internal server error".to_string(),
            }))
        }
    };

    Ok(Json(users))
}

#[get("/users/<id>")]
pub async fn get_user(
    repo: &State<Arc<UserRepository>>,
    id: String,
) -> Result<Json<UserDocument>, Json<ErrorResponse>> {
    let user = match repo.get_user_by_id(&id).await {
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
                message: "Internal server error".to_string(),
            }))
        }
    };

    Ok(Json(user))
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
                message: "Internal server error".to_string(),
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
                message: "Internal server error".to_string(),
            }))
        }
    };

    Ok(Json(user))
}

#[delete("/delete/user/<id>")]
pub async fn delete_user(
    repo: &State<Arc<UserRepository>>,
    id: String,
) -> Result<Json<DeleteUserResponse>, Json<ErrorResponse>> {
    match repo.delete_user(&id).await {
        Ok(Some(_)) => Ok(Json(DeleteUserResponse {
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
                message: "Internal server error".to_string(),
            }))
        }
    }
}

pub fn user_routes() -> Vec<rocket::Route> {
    routes![
        setup,
        login,
        // list_users, - This endpoint will be used for administrative processes
        get_user,
        update_user,
        delete_user
    ]
}
