use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use std::thread;

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

