//! Passwort-Hashes mit Argon2id (ANF-005 AK-21, nur Server).
//!
//! Gespeichert wird nur der Hash im PHC-Format (`$argon2id$v=19$…`), der Verfahren, Parameter
//! und ein zufälliges Salz enthält. Das Passwort selbst wird nirgends abgelegt.

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Hash für ein neues Passwort (Argon2id, Standardparameter, zufälliges Salz).
/// Rechenintensiv – in Server-Funktionen über `spawn_blocking` aufrufen.
pub fn hash(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string())
}

/// `true`, wenn das Passwort zum gespeicherten Hash passt (für die spätere Anmeldung).
pub fn verify(password: &str, stored_hash: &str) -> bool {
    PasswordHash::new(stored_hash).is_ok_and(|h| {
        Argon2::default()
            .verify_password(password.as_bytes(), &h)
            .is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify() {
        let h = hash("geheim-1234").unwrap();
        assert!(h.starts_with("$argon2id$"));
        assert!(!h.contains("geheim-1234"));
        assert!(verify("geheim-1234", &h));
        assert!(!verify("geheim-1235", &h));
        assert!(!verify("geheim-1234", "kein hash"));
        // Gleiches Passwort, anderes Salz → anderer Hash
        assert_ne!(h, hash("geheim-1234").unwrap());
    }
}
