use crate::{
    models::{User, UserCredentials},
    repositories::keys::KeyRepository,
};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use pasetors::{
    claims::Claims,
    keys::{AsymmetricPublicKey, AsymmetricSecretKey},
    public,
    version4::V4,
};
use sha2::{Digest, Sha256};

pub async fn decode_keys(
    repo: &KeyRepository,
) -> Result<(AsymmetricSecretKey<V4>, AsymmetricPublicKey<V4>), String> {
    let kp = repo
        .get_or_create_key_pair()
        .await
        .map_err(|e| e.to_string())?;
    let decoded_private_key = STANDARD.decode(kp.private_key).map_err(|e| e.to_string())?;
    let decoded_private_key = &decoded_private_key.as_slice();
    let private_key =
        AsymmetricSecretKey::<V4>::from(decoded_private_key).map_err(|e| e.to_string())?;
    let decoded_public_key = STANDARD.decode(kp.public_key).map_err(|e| e.to_string())?;
    let decoded_public_key = &decoded_public_key.as_slice();
    let public_key =
        AsymmetricPublicKey::<V4>::from(decoded_public_key).map_err(|e| e.to_string())?;
    Ok((private_key, public_key))
}

/*---------------------------------------------
Authorize the user via password verification.
----------------------------------------------*/

pub async fn authorize_user(
    user: &User,
    credentials: &UserCredentials,
    repo: &KeyRepository,
) -> Result<String, String> {
    if !verify(&credentials.password, &user.password).map_err(|e| e.to_string())? {
        return Err("Invalid credentials".into());
    }
    let mut claims = Claims::new().map_err(|e| e.to_string())?;
    let ecs_authentication_key = std::env::var_os("ECS_AUTHENTICATION_KEY")
        .expect("[ECS_AUTHENTICATION_KEY] must be set...")
        .into_string()
        .unwrap();

    let expiration = Utc::now() + Duration::hours(8);
    let expiration = expiration.to_rfc3339();

    let mut hasher = Sha256::new();
    hasher.update(format!("{}{}", user.email, ecs_authentication_key)); // Unique to current system
    let nonce = format!("{:x}", hasher.finalize());

    claims
        .subject(&credentials.email)
        .map_err(|e| e.to_string())?;
    claims.expiration(&expiration).map_err(|e| e.to_string())?;
    claims
        .issuer("https://www.embraconnect.com")
        .map_err(|e| e.to_string())?;
    claims
        .add_additional("nonce", nonce)
        .map_err(|e| e.to_string())?;
    // claims.add_additional("aud", vec!["https://www.embraconnect.com"]).map_err(|e|e.to_string())?;
    let (private_key, _public_key) = decode_keys(repo).await.map_err(|e| e.to_string())?;
    let token = public::sign(&private_key, &claims, None, None).map_err(|e| e.to_string())?;
    Ok(token)
}

pub fn hash_password(password: String) -> Result<String, String> {
    hash(password, DEFAULT_COST).map_err(|e| e.to_string())
}