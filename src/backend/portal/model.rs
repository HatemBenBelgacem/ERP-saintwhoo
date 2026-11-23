//! Regeln für das Erstellen einer Mandanten-Datenbank (ANF-026). Wird auf Server und im Browser verwendet.

use crate::backend::users::model::{USERNAME_MAX, is_valid_username};

/// Höchstens so viele Datenbanken je Administrator (ANF-026 AK-14, entschieden: eine).
pub const MAX_DATABASES_PER_ADMIN: usize = 1;

/// Status einer Mandanten-Datenbank (Spalte `mandant.status`).
pub const STATUS_CREATING: &str = "wird_erstellt";
pub const STATUS_READY: &str = crate::backend::auth::model::TENANT_READY;
pub const STATUS_FAILED: &str = "fehler";

pub const MSG_NOT_ADMIN: &str = "Nur Administratoren können eine Datenbank erstellen.";
pub const MSG_LIMIT: &str = "Sie haben bereits eine Datenbank.";
pub const MSG_RUNNING: &str = "Ihre Datenbank wird bereits erstellt.";
pub const MSG_FAILED: &str = "Die Datenbank konnte nicht erstellt werden. Bitte erneut versuchen.";
pub const MSG_NOT_READY: &str = "Ihre Datenbank ist noch nicht bereit.";
pub const MSG_ALREADY_DEFINED: &str =
    "Der Mandant ist bereits definiert. Änderungen erfolgen im ERP unter Einstellungen › Firma.";

/// Was beim Klick auf „Datenbank erstellen“ geschieht (ANF-026 AK-11, AK-13, AK-14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreationDecision {
    /// Neuen Mandanten anlegen
    New,
    /// Fehlgeschlagenen Mandanten (Kennung) erneut erstellen (AK-13)
    Retry(i64),
    /// Eine Erstellung läuft bereits (Doppelklick, AK-14)
    AlreadyRunning,
    /// Grenze „eine Datenbank pro Administrator“ erreicht (AK-14)
    LimitReached,
}

/// Entscheidung aus den bestehenden Mandanten des Administrators: (Kennung, Status).
pub fn creation_decision(existing: &[(i64, &str)]) -> CreationDecision {
    if existing.iter().any(|(_, s)| *s == STATUS_CREATING) {
        return CreationDecision::AlreadyRunning;
    }
    let ready = existing.iter().filter(|(_, s)| *s == STATUS_READY).count();
    if ready >= MAX_DATABASES_PER_ADMIN {
        return CreationDecision::LimitReached;
    }
    match existing.iter().find(|(_, s)| *s == STATUS_FAILED) {
        Some((id, _)) => CreationDecision::Retry(*id),
        None => CreationDecision::New,
    }
}

/// Name einer Mandanten-Datenbank, wie ihn die Portal-Datenbank erzeugt: `uq_m_` und mindestens
/// 6 Ziffern (ANF-026 AK-12). Nur solche Namen dürfen in SQL-Befehle wie `CREATE DATABASE`.
pub fn is_valid_database_name(name: &str) -> bool {
    name.strip_prefix("uq_m_").is_some_and(|digits| {
        digits.len() >= 6 && digits.len() <= 19 && digits.chars().all(|c| c.is_ascii_digit())
    })
}

/// Benutzername des ersten Benutzers in der Mandanten-Datenbank aus der E-Mail (ANF-026 AK-15):
/// Teil vor dem @, klein, unerlaubte Zeichen als Punkt, höchstens 50 Zeichen; sonst `admin`.
pub fn username_from_email(email: &str) -> String {
    let local = email.split('@').next().unwrap_or_default().to_lowercase();
    let mut name: String = local
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_') {
                c
            } else {
                '.'
            }
        })
        .take(USERNAME_MAX)
        .collect();
    name = name.trim_matches('.').to_string();
    if is_valid_username(&name) {
        name
    } else {
        "admin".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision() {
        use CreationDecision::*;
        assert_eq!(creation_decision(&[]), New);
        assert_eq!(creation_decision(&[(3, STATUS_FAILED)]), Retry(3));
        assert_eq!(creation_decision(&[(3, STATUS_CREATING)]), AlreadyRunning);
        assert_eq!(creation_decision(&[(3, STATUS_READY)]), LimitReached);
        assert_eq!(
            creation_decision(&[(1, STATUS_READY), (2, STATUS_FAILED)]),
            LimitReached
        );
        assert_eq!(
            creation_decision(&[(1, STATUS_FAILED), (2, STATUS_CREATING)]),
            AlreadyRunning
        );
    }

    #[test]
    fn database_names() {
        assert!(is_valid_database_name("uq_m_000001"));
        assert!(is_valid_database_name("uq_m_1234567"));
        assert!(!is_valid_database_name("uq_m_00001"));
        assert!(!is_valid_database_name("uq_m_00000a"));
        assert!(!is_valid_database_name(
            "uq_m_000001\"; DROP DATABASE x; --"
        ));
        assert!(!is_valid_database_name("uniqwork_portal"));
        assert!(!is_valid_database_name(""));
        assert!(!is_valid_database_name(&format!("uq_m_{}", "1".repeat(20))));
    }

    #[test]
    fn usernames_from_email() {
        assert_eq!(username_from_email("anna.muster@firma.ch"), "anna.muster");
        assert_eq!(username_from_email("Anna+Test@firma.ch"), "anna.test");
        assert_eq!(username_from_email("jörg@firma.ch"), "j.rg");
        assert_eq!(username_from_email("++@firma.ch"), "admin");
        assert_eq!(
            username_from_email(&format!("{}@x.ch", "a".repeat(80))).len(),
            USERNAME_MAX
        );
    }
}
