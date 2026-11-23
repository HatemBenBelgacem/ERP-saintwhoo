//! Datenmodell und Regeln für Registrierung und Anmeldung über das Portal (ANF-007, ANF-026).
//! Wird auf Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

use crate::backend::addresses::model::is_valid_email;
use crate::backend::users::model::PasswordInput;
pub use crate::forms::{FieldErrors, SaveOutcome};

/// Name des Sitzungs-Cookies.
pub const COOKIE_NAME: &str = "erp_sitzung";
/// Sitzung endet nach so vielen Stunden ohne Aktivität (ANF-007 AK-14).
pub const SESSION_IDLE_HOURS: i32 = 12;
/// Sitzung endet spätestens nach so vielen Tagen (ANF-007 AK-14).
pub const SESSION_MAX_DAYS: i32 = 7;
/// Fehlversuche je E-Mail innerhalb von `LOCK_MINUTES`, bevor gesperrt wird (ANF-007 AK-11, ANF-026 AK-5).
pub const MAX_FAILED_ATTEMPTS: i64 = 5;
pub const LOCK_MINUTES: i32 = 15;
/// Höchstens so viele Registrierungen insgesamt innerhalb von `REGISTRATION_WINDOW_MINUTES`
/// (Schutz gegen massenhafte Registrierungen, ANF-026 offene Fragen).
pub const MAX_REGISTRATIONS: i64 = 10;
pub const REGISTRATION_WINDOW_MINUTES: i32 = 10;

/// Pfad der Anmeldeseite.
pub const LOGIN_PATH: &str = "/anmelden";
/// Pfad der Registrierung (ANF-026 AK-1).
pub const REGISTER_PATH: &str = "/registrieren";
/// Portal-Übersicht: eigene Datenbank und Mandant (ANF-026 AK-4).
pub const PORTAL_PATH: &str = "/portal";
/// Präfix der Server-Funktionen, die ohne Anmeldung erreichbar sind (ANF-007 AK-3).
pub const PUBLIC_API_PREFIX: &str = "/oeffentlich";
/// Präfix der Server-Funktionen des Portals: angemeldet, aber ohne Zugang zum ERP erreichbar (ANF-026).
pub const PORTAL_API_PREFIX: &str = "/api/portal";

/// Feldname für Meldungen, die das ganze Anmeldeformular betreffen.
pub const FORM_ERROR: &str = "anmeldung";
pub const MSG_WRONG: &str = "E-Mail oder Passwort ist falsch.";
pub const MSG_EMPTY: &str = "Bitte E-Mail und Passwort eingeben.";
pub const MSG_LOCKED: &str = "Zu viele Fehlversuche. Bitte später erneut versuchen.";
pub const MSG_NOT_LOGGED_IN: &str = "Nicht angemeldet.";
pub const MSG_NO_TENANT: &str = "Kein Mandant gewählt.";
pub const MSG_EMAIL_TAKEN: &str = "Für diese E-Mail-Adresse gibt es bereits einen Zugang.";
pub const MSG_TOO_MANY_REGISTRATIONS: &str =
    "Zurzeit gibt es zu viele Registrierungen. Bitte später erneut versuchen.";

/// Status einer Mandanten-Datenbank (Spalte `mandant.status`).
pub const TENANT_READY: &str = "bereit";

/// Mandant der Sitzung (ANF-026).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionTenant {
    pub id: i64,
    /// Leer, bis der Mandant im Modal-Fenster definiert ist (ANF-026 AK-18)
    pub firmenname: Option<String>,
    /// `wird_erstellt`, `bereit` oder `fehler`
    pub status: String,
}

impl SessionTenant {
    /// Datenbank bereit und Mandant definiert (ANF-026 AK-19).
    pub fn is_usable(&self) -> bool {
        self.status == TENANT_READY && self.firmenname.is_some()
    }
}

/// Der angemeldete Zugang aus der Portal-Datenbank (ANF-026).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CurrentUser {
    /// Kennung in `portal_benutzer`
    pub portal_id: i64,
    pub email: String,
    /// „Vorname Nachname“
    pub name: String,
    pub ist_administrator: bool,
    /// Gewählter Mandant; leer, solange es keine Datenbank gibt
    pub mandant: Option<SessionTenant>,
    /// Benutzer in der Mandanten-Datenbank (ANF-026 AK-15); kommt mit dem Erstellen der Datenbank
    pub benutzer_id: Option<i64>,
}

impl CurrentUser {
    /// Darf das ERP verwenden: Datenbank bereit und Mandant definiert (ANF-026 AK-19).
    pub fn erp_access(&self) -> bool {
        self.mandant.as_ref().is_some_and(SessionTenant::is_usable)
    }
}

/// E-Mail als Anmeldename: ohne Leerzeichen am Rand, klein geschrieben (ANF-026 AK-2, AK-3).
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
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

/// Ziel nach der Anmeldung: nur Adressen innerhalb der Anwendung, sonst die Startseite (ANF-007 AK-5).
pub fn target_after_login(zurueck: Option<&str>) -> String {
    zurueck
        .and_then(crate::forms::safe_return_url)
        .filter(|t| !t.starts_with(LOGIN_PATH) && !t.starts_with(REGISTER_PATH))
        .unwrap_or("/")
        .to_string()
}

/// Eingabe der Registrierung (ANF-026 AK-2). Das Passwort kommt getrennt als `PasswordInput`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RegistrationInput {
    pub vorname: String,
    pub nachname: String,
    pub email: String,
}

impl RegistrationInput {
    /// Leerzeichen am Rand entfernen, E-Mail klein schreiben.
    pub fn normalized(&self) -> Self {
        Self {
            vorname: self.vorname.trim().into(),
            nachname: self.nachname.trim().into(),
            email: normalize_email(&self.email),
        }
    }

    /// Prüft die (normalisierte) Eingabe und das Passwort (ANF-005-Regeln). Leere Map = gültig.
    pub fn validate(&self, password: &PasswordInput) -> FieldErrors {
        let mut errors = FieldErrors::new();
        if self.vorname.is_empty() {
            errors.insert("vorname".into(), "Bitte einen Vornamen eingeben.".into());
        }
        if self.nachname.is_empty() {
            errors.insert("nachname".into(), "Bitte einen Nachnamen eingeben.".into());
        }
        if self.email.is_empty() {
            errors.insert("email".into(), "Bitte eine E-Mail-Adresse eingeben.".into());
        } else if !is_valid_email(&self.email) {
            errors.insert(
                "email".into(),
                "Die E-Mail-Adresse hat kein gültiges Format (z. B. name@firma.ch).".into(),
            );
        }
        for (field, value, max) in [
            ("vorname", &self.vorname, 100),
            ("nachname", &self.nachname, 100),
            ("email", &self.email, 254),
        ] {
            if value.chars().count() > max {
                errors.insert(field.into(), format!("Höchstens {max} Zeichen."));
            }
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
        assert_eq!(target_after_login(Some("/registrieren")), "/");
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
    fn email_is_normalized() {
        assert_eq!(
            normalize_email("  Anna.Muster@Firma.CH "),
            "anna.muster@firma.ch"
        );
    }

    #[test]
    fn registration_validation() {
        let pw = PasswordInput {
            passwort: "geheim-1234".into(),
            wiederholung: "geheim-1234".into(),
        };
        let ok = RegistrationInput {
            vorname: " Anna ".into(),
            nachname: "Muster".into(),
            email: " Anna@Firma.CH ".into(),
        }
        .normalized();
        assert_eq!(ok.vorname, "Anna");
        assert_eq!(ok.email, "anna@firma.ch");
        assert!(ok.validate(&pw).is_empty(), "{:?}", ok.validate(&pw));

        let e = RegistrationInput::default().validate(&PasswordInput::default());
        for f in ["vorname", "nachname", "email", "passwort"] {
            assert!(e.contains_key(f), "{f}");
        }
        let bad = RegistrationInput {
            email: "keine-adresse".into(),
            ..ok.clone()
        };
        assert!(bad.validate(&pw).contains_key("email"));
        let long = RegistrationInput {
            vorname: "x".repeat(101),
            ..ok
        };
        assert_eq!(long.validate(&pw)["vorname"], "Höchstens 100 Zeichen.");
    }

    #[test]
    fn erp_access_needs_ready_and_defined_tenant() {
        let mut user = CurrentUser {
            portal_id: 1,
            email: "anna@firma.ch".into(),
            name: "Anna Muster".into(),
            ist_administrator: true,
            mandant: None,
            benutzer_id: None,
        };
        assert!(!user.erp_access());
        user.mandant = Some(SessionTenant {
            id: 1,
            firmenname: None,
            status: TENANT_READY.into(),
        });
        assert!(!user.erp_access(), "Mandant noch nicht definiert");
        user.mandant = Some(SessionTenant {
            id: 1,
            firmenname: Some("Firma AG".into()),
            status: "wird_erstellt".into(),
        });
        assert!(!user.erp_access(), "Datenbank noch nicht bereit");
        user.mandant = Some(SessionTenant {
            id: 1,
            firmenname: Some("Firma AG".into()),
            status: TENANT_READY.into(),
        });
        assert!(user.erp_access());
    }
}
