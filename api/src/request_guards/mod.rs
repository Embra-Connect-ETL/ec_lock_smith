use shared::utils::auth::decode_keys;
use pasetors::{
    claims::{Claims, ClaimsValidationRules},
    public,
    token::UntrustedToken,
    version4::V4,
    Public,
};
use rocket::async_trait;
use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request, State,
};
use std::sync::Arc;

use shared::repositories::keys::KeyRepository;

pub struct TokenGuard(pub Claims);

#[async_trait]
impl<'r> FromRequest<'r> for TokenGuard {
    type Error = Status;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key_repo = match request.guard::<&State<Arc<KeyRepository>>>().await {
            Outcome::Success(state) => state,
            _ => return Outcome::Forward(Status::InternalServerError),
        };

        let auth_header = request.headers().get_one("Authorization");

        match auth_header {
            Some(token) if token.starts_with("Bearer ") => {
                let token = token.trim_start_matches("Bearer ").trim();
                let validation_rules = ClaimsValidationRules::new();
                if let Ok(untrusted_token) = UntrustedToken::<Public, V4>::try_from(token) {
                    if let Ok(kp) = decode_keys(key_repo).await {
                        if let Ok(trusted_token) =
                            public::verify(&kp.1, &untrusted_token, &validation_rules, None, None)
                        {
                            if let Some(claims) = trusted_token.payload_claims() {
                                Outcome::Success(TokenGuard(claims.clone()))
                            } else {
                                Outcome::Error((Status::Unauthorized, Status::Unauthorized))
                            }
                        } else {
                            Outcome::Error((
                                Status::InternalServerError,
                                Status::InternalServerError,
                            ))
                        }
                    } else {
                        Outcome::Error((Status::InternalServerError, Status::InternalServerError))
                    }
                } else {
                    Outcome::Error((Status::InternalServerError, Status::InternalServerError))
                }
            }
            _ => Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
        }
    }
}
