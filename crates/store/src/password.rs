//! Password hashing with argon2id (PHC strings).

use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

use crate::{Error, Result};

pub fn hash(password: &str) -> Result<String> {
    Argon2::default()
        .hash_password(password.as_bytes())
        .map(|h| h.to_string())
        .map_err(|e| Error::Password(e.to_string()))
}

pub fn verify(password: &str, hash: &str) -> bool {
    PasswordHash::new(hash).is_ok_and(|parsed| {
        Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn hash_then_verify() {
        let h = super::hash("hunter2").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(super::verify("hunter2", &h));
        assert!(!super::verify("hunter3", &h));
    }
}
