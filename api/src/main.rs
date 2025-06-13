#![allow(unused)]

use std::path::PathBuf;

use rocket::{fs::FileServer, serde::json::Json};

#[macro_use]
extern crate rocket;
extern crate crypto;
extern crate log;

mod custom_catchers;
mod db;
mod fairings;
mod models;
mod request_guards;
mod routes;

use custom_catchers::*;
use routes::users::user_routes;
use routes::vault::vault_routes;

#[get("/health")]
fn health_check() -> Json<String> {
    Json(String::from("Secrets management service is running..."))
}

#[rocket::options("/<_..>")]
fn _options() -> &'static str {
    ""
}

#[launch]
fn rocket() -> _ {
    dotenvy::dotenv().ok();

    let current_dir = std::env::current_dir().expect("Failed to get current directory");
    let public_path: PathBuf = current_dir.join("./public");

    rocket::build()
        .attach(db::init())
        .attach(fairings::CORS)
        .mount("/", routes![health_check, _options])
        .mount("/", user_routes())
        .mount("/", vault_routes())
        .mount("/", FileServer::from(public_path))
        .register(
            "/",
            catchers![
                bad_request,
                unauthorized,
                forbidden,
                not_found,
                method_not_allowed,
                request_timeout,
                conflict,
                payload_too_large,
                unsupported_media_type,
                teapot,
                too_many_requests,
                internal_error,
                bad_gateway,
                service_unavailable,
                gateway_timeout
            ],
        )
}
