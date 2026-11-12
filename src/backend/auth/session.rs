//! Sitzungs-Merkmal, Cookie und angemeldeter Benutzer (ANF-007, nur Server).

use argon2::password_hash::rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};

use super::model::{COOKIE_NAME, CurrentUser, MSG_NOT_LOGGED_IN, SESSION_MAX_DAYS, cookie_value};

/// Neues zufälliges Sitzungs-Merkmal (32 Byte aus dem Zufallsgenerator des Betriebssystems, hexadezimal).
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    to_hex(&bytes)
}

/// SHA-256 des Merkmals – nur dieser Wert wird in der Datenbank gespeichert (AK-13).
pub fn token_hash(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `Secure` nur, wenn das ERP über HTTPS läuft (Umgebungsvariable `SITZUNG_COOKIE_SECURE=1`).
fn secure_flag() -> &'static str {
    match std::env::var("SITZUNG_COOKIE_SECURE").as_deref() {
        Ok("1" | "true" | "ja") => "; Secure",
        _ => "",
    }
}

/// `Set-Cookie`-Wert für eine neue Sitzung (AK-13).
pub fn session_cookie(token: &str) -> String {
    let max_age = i64::from(SESSION_MAX_DAYS) * 24 * 3600;
    format!(
        "{COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={max_age}{}",
        secure_flag()
    )
}

/// `Set-Cookie`-Wert, der das Cookie im Browser löscht (AK-18).
pub fn clear_cookie() -> String {
    format!(
        "{COOKIE_NAME}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0{}",
        secure_flag()
    )
}

/// Sitzungs-Merkmal aus den Kopfzeilen der Anfrage.
pub fn token_from_headers(headers: &axum::http::HeaderMap) -> Option<String> {
    headers
        .get_all(axum::http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|h| cookie_value(h, COOKIE_NAME).map(str::to_string))
}

/// Angemeldeter Benutzer der aktuellen Anfrage (von der Middleware ermittelt) oder `None`.
/// Nur in Server-Funktionen und beim Darstellen auf dem Server verwendbar.
pub fn current_user() -> Option<CurrentUser> {
    leptos::prelude::use_context::<axum::http::request::Parts>()
        .and_then(|parts| parts.extensions.get::<CurrentUser>().cloned())
}

/// Wie `current_user`, aber mit Fehler „Nicht angemeldet.“, wenn niemand angemeldet ist.
pub fn require_user() -> Result<CurrentUser, leptos::prelude::ServerFnError> {
    current_user().ok_or_else(|| leptos::prelude::ServerFnError::new(MSG_NOT_LOGGED_IN))
}

/// Sitzungs-Merkmal der aktuellen Anfrage (für das Abmelden).
pub fn current_token() -> Option<String> {
    leptos::prelude::use_context::<axum::http::request::Parts>()
        .and_then(|parts| token_from_headers(&parts.headers))
}

/// Setzt ein Cookie in der Antwort der laufenden Server-Funktion.
pub fn set_cookie(value: &str) {
    if let (Some(response), Ok(v)) = (
        leptos::prelude::use_context::<leptos_axum::ResponseOptions>(),
        axum::http::HeaderValue::from_str(value),
    ) {
        response.append_header(axum::http::header::SET_COOKIE, v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_and_hashed() {
        let a = new_token();
        let b = new_token();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);
        let h = token_hash(&a);
        assert_eq!(h.len(), 64);
        assert_ne!(h, a);
        assert_eq!(h, token_hash(&a));
        assert_eq!(
            token_hash("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn cookie_attributes() {
        let c = session_cookie("xyz");
        assert!(c.starts_with("erp_sitzung=xyz; Path=/; HttpOnly; SameSite=Lax; Max-Age=604800"));
        assert!(clear_cookie().contains("Max-Age=0"));
    }
}
