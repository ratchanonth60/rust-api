use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use std::thread;

use crate::models::jwt::Claims;
use crate::models::User;
/// Hashes a password using Argon2. This is a CPU-intensive operation.
pub async fn hash_password(password: String) -> Result<String, String> {
    thread::spawn(move || {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        argon2
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|e| e.to_string())
    })
    .join()
    .unwrap()
}

pub fn verify_password(hash: &str, password: &str) -> Result<bool, String> {
    let parsed_hash = argon2::PasswordHash::new(hash).map_err(|e| e.to_string())?;
    Ok(argon2::Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn create_access_token(
    user: &User,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::time::{SystemTime, UNIX_EPOCH};

    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        + std::time::Duration::from_secs(60 * 60); // 1 hour
    let claims = Claims {
        sub: user.id,
        exp: expiration.as_secs() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}

pub fn create_refresh_token(
    user: &User,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::time::{SystemTime, UNIX_EPOCH};
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        + std::time::Duration::from_secs(60 * 60 * 24 * 7); // 7 days
    let claims = Claims {
        sub: user.id,
        exp: expiration.as_secs() as usize,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
}
pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    use jsonwebtoken::{decode, DecodingKey, Validation};
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}
