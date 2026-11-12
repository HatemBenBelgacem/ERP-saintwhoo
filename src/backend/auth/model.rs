//! Datenmodell und Regeln für die Anmeldung (ANF-007). Wird auf Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

use crate::backend::addresses::model::AddressInput;
use crate::backend::users::model::{PasswordInput, username_error};
pub use crate::forms::{FieldErrors, SaveOutcome};

/// Name des Sitzungs-Cookies.
pub const COOKIE_NAME: &str = "erp_sitzung";
/// Sitzung endet nach so vielen Stunden ohne Aktivität (AK-14).
pub const SESSION_IDLE_HOURS: i32 = 12;
/// Sitzung endet spätestens nach so vielen Tagen (AK-14).
pub const SESSION_MAX_DAYS: i32 = 7;
/// Fehlversuche je Benutzername innerhalb von `LOCK_MINUTES`, bevor gesperrt wird (AK-11).
pub const MAX_FAILED_ATTEMPTS: i64 = 5;
pub const LOCK_MINUTES: i32 = 15;

/// Pfad der Anmeldeseite.
pub const LOGIN_PATH: &str = "/anmelden";
/// Präfix der Server-Funktionen, die ohne Anmeldung erreichbar sind (AK-3).
pub const PUBLIC_API_PREFIX: &str = "/oeffentlich";

/// Feldname für Meldungen, die das ganze Anmeldeformular betreffen.
pub const FORM_ERROR: &str = "anmeldung";
pub const MSG_WRONG: &str = "Benutzername oder Passwort ist falsch.";
pub const MSG_EMPTY: &str = "Bitte Benutzername und Passwort eingeben.";
pub const MSG_LOCKED: &str = "Zu viele Fehlversuche. Bitte später erneut versuchen.";
pub const MSG_NOT_LOGGED_IN: &str = "Nicht angemeldet.";

/// Der angemeldete Benutzer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct CurrentUser {
    pub id: i64,
    pub benutzername: String,
    /// Anzeigename der Adresse
    pub name: String,
}

/// Benutzername für die Anmeldung: ohne Leerzeichen am Rand, klein geschrieben (AK-6).
pub fn normalize_username(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Webadresse der Anmeldeseite mit Rücksprungziel (AK-1). Für die Startseite ohne `zurueck`.
pub fn login_url(target: &str, expired: bool) -> String {
    let mut params = Vec::new();
    if target != "/" && !target.is_empty() {
        params.push(format!("zurueck={}", crate::forms::encode(target)));
    }
    if expired {
        params.push("abgelaufen=1".to_string());
    }
    if params.is_empty() {
        LOGIN_PATH.to_string()
    } else {
        format!("{LOGIN_PATH}?{}", params.join("&"))
    }
}

/// Ziel nach der Anmeldung: nur Adressen innerhalb der Anwendung, sonst die Startseite (AK-5).
pub fn target_after_login(zurueck: Option<&str>) -> String {
    zurueck
        .and_then(crate::forms::safe_return_url)
        .filter(|t| !t.starts_with(LOGIN_PATH))
        .unwrap_or("/")
        .to_string()
}

/// Eingabe der Ersteinrichtung (AK-21): Person und Benutzername.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FirstSetupInput {
    pub vorname: String,
    pub nachname: String,
    pub email: String,
    pub benutzername: String,
}

impl FirstSetupInput {
    /// Adresse vom Typ „Person“ (Prüfregeln aus ANF-002).
    pub fn address(&self) -> AddressInput {
        AddressInput {
            typ: "person".into(),
            vorname: self.vorname.clone(),
            nachname: self.nachname.clone(),
            email: self.email.clone(),
            ..AddressInput::new_default()
        }
        .normalized()
    }

    pub fn username(&self) -> String {
        normalize_username(&self.benutzername)
    }

    /// Prüft alle Felder (ANF-002, ANF-005). Leere Map = gültig.
    pub fn validate(&self, password: &PasswordInput) -> FieldErrors {
        let mut errors = self.address().validate();
        // Nur die Felder dieses Formulars melden
        errors.retain(|k, _| matches!(k.as_str(), "vorname" | "nachname" | "email"));
        if let Some(m) = username_error(&self.username()) {
            errors.insert("benutzername".into(), m);
        }
        errors.extend(password.validate());
        errors
    }
}

/// Wert eines Cookies aus dem `Cookie`-Kopf.
pub fn cookie_value<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    header
        .split(';')
        .filter_map(|part| part.trim().split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
        .filter(|v| !v.is_empty())
}

/// Prozent-Dekodierung eines Werts aus der Webadresse (Gegenstück zu `forms::encode`).
pub fn decode(s: &str) -> String {
    fn hex(b: u8) -> Option<u8> {
        (b as char).to_digit(16).map(|d| d as u8)
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%'
            && i + 2 < bytes.len()
            && let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2]))
        {
            out.push(h * 16 + l);
            i += 3;
            continue;
        }
        out.push(if b == b'+' { b' ' } else { b });
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Wert eines Parameters aus einem Abfrage-Text (`a=1&b=2`), dekodiert.
pub fn query_param(query: &str, name: &str) -> Option<String> {
    query
        .split('&')
        .filter_map(|p| p.split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| decode(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_url_and_target() {
        assert_eq!(login_url("/", false), "/anmelden");
        assert_eq!(
            login_url("/projekte/4?bearbeiten=projekt&id=4", true),
            "/anmelden?zurueck=%2Fprojekte%2F4%3Fbearbeiten%3Dprojekt%26id%3D4&abgelaufen=1"
        );
        assert_eq!(target_after_login(Some("/projekte/4")), "/projekte/4");
        assert_eq!(target_after_login(Some("https://boese.example.com")), "/");
        assert_eq!(target_after_login(Some("//boese.example.com")), "/");
        assert_eq!(target_after_login(Some("/anmelden")), "/");
        assert_eq!(target_after_login(None), "/");
    }

    #[test]
    fn decode_roundtrip() {
        let original = "/projekte/4?bearbeiten=projekt&id=4 Müller";
        assert_eq!(decode(&crate::forms::encode(original)), original);
        assert_eq!(decode("a%2"), "a%2");
        assert_eq!(decode("%zz"), "%zz");
        assert_eq!(
            query_param("x=1&zurueck=%2Faufgaben&y=2", "zurueck").as_deref(),
            Some("/aufgaben")
        );
        assert_eq!(query_param("x=1", "zurueck"), None);
    }

    #[test]
    fn cookie_parsing() {
        let h = "a=1; erp_sitzung=abc123; b=2";
        assert_eq!(cookie_value(h, COOKIE_NAME), Some("abc123"));
        assert_eq!(cookie_value("erp_sitzung=", COOKIE_NAME), None);
        assert_eq!(cookie_value("x_erp_sitzung=1", COOKIE_NAME), None);
    }

    #[test]
    fn username_is_normalized() {
        assert_eq!(normalize_username("  Anna.Muster "), "anna.muster");
    }

    #[test]
    fn first_setup_validation() {
        let pw = PasswordInput {
            passwort: "geheim-1234".into(),
            wiederholung: "geheim-1234".into(),
        };
        let ok = FirstSetupInput {
            vorname: "Anna".into(),
            nachname: "Muster".into(),
            email: String::new(),
            benutzername: "Anna".into(),
        };
        assert!(ok.validate(&pw).is_empty(), "{:?}", ok.validate(&pw));
        assert_eq!(ok.username(), "anna");
        let e = FirstSetupInput::default().validate(&PasswordInput::default());
        for f in ["vorname", "nachname", "benutzername", "passwort"] {
            assert!(e.contains_key(f), "{f}");
        }
        assert!(!e.contains_key("firmenname"));
    }
}
