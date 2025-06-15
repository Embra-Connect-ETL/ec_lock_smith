use pasetors::{
    claims::{Claims, ClaimsValidationRules},
    public,
    token::UntrustedToken,
    version4::V4,
    Public,
};
use rocket::{
    async_trait,
    http::Status,
    request::{FromRequest, Outcome, Request},
    State,
};
use shared::repositories::keys::KeyRepository;
use shared::utils::auth::decode_keys;
use std::sync::Arc;

pub struct TokenGuard(pub Claims);

#[async_trait]
impl<'r> FromRequest<'r> for TokenGuard {
    type Error = Status;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let key_repo = match request.guard::<&State<Arc<KeyRepository>>>().await {
            Outcome::Success(state) => state,
            _ => return Outcome::Error((Status::InternalServerError, Status::InternalServerError)),
        };

        // Try to extract from Authorization header
        let header_token = request
            .headers()
            .get_one("Authorization")
            .and_then(|auth| auth.strip_prefix("Bearer ").map(str::trim));

        // Or from cookie
        let cookie_token = request.cookies().get("auth_token").map(|c| c.value());

        let token = header_token.or(cookie_token);

        let token = match token {
            Some(t) => t,
            None => return Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
        };

        let validation_rules = ClaimsValidationRules::new();

        match UntrustedToken::<Public, V4>::try_from(token) {
            Ok(untrusted_token) => match decode_keys(key_repo).await {
                Ok(kp) => {
                    match public::verify(&kp.1, &untrusted_token, &validation_rules, None, None) {
                        Ok(trusted_token) => match trusted_token.payload_claims() {
                            Some(claims) => Outcome::Success(TokenGuard(claims.clone())),
                            None => Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
                        },
                        Err(_) => Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
                    }
                }
                Err(_) => {
                    Outcome::Error((Status::InternalServerError, Status::InternalServerError))
                }
            },
            Err(_) => Outcome::Error((Status::Unauthorized, Status::Unauthorized)),
        }
    }
}
