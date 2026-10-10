use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub struct PasswordService;

impl PasswordService {
    pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
        let hash = Argon2::default()
            .hash_password(password.as_bytes())?
            .to_string();

        Ok(hash)
    }

    pub fn verify(password: &str, password_hash: &str) -> bool {
        match PasswordHash::new(&password_hash) {
            Ok(parsed_hash) => Argon2::default()
                .verify_password(password.as_bytes(), &parsed_hash)
                .is_ok(),
            Err(_) => false,
        }
    }
}