//! Datenmodell und Prüfregeln für Benutzer (ANF-005).
//! Wird auf dem Server und im Browser verwendet. Passwörter gibt es seit ANF-026 nur im Portal
//! (`crate::backend::auth`).

use serde::{Deserialize, Serialize};

use crate::backend::projects::model::Level;
use crate::backend::tasks::model::TaskItem;
pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

pub const USERNAME_MAX: usize = 50;

/// Status im Formular (AK-11, AK-16).
pub const STATUSES: &[(&str, &str)] = &[("aktiv", "Aktiv"), ("gesperrt", "Gesperrt")];

/// Filter der Benutzerliste (AK-3). „alle“ ist der Standard.
pub const STATUS_FILTERS: &[(&str, &str)] = &[
    ("alle", "Alle Status"),
    ("aktiv", "Aktiv"),
    ("gesperrt", "Gesperrt"),
];

/// Sortierbare Spalten der Liste (AK-4): Schlüssel in der Webadresse → Beschriftung.
pub const SORT_COLUMNS: &[(&str, &str)] = &[
    ("benutzername", "Benutzername"),
    ("name", "Name"),
    ("email", "E-Mail"),
    ("status", "Status"),
];

/// Erlaubt: Kleinbuchstaben a–z, Ziffern, `.`, `-`, `_` (AK-13).
pub fn is_valid_username(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-' | '_'))
}

/// Fehlermeldung für einen (normalisierten) Benutzernamen oder `None`, wenn er gültig ist (AK-13).
/// Auch für die Ersteinrichtung (ANF-007 AK-21).
pub fn username_error(name: &str) -> Option<String> {
    if name.is_empty() {
        Some("Bitte einen Benutzernamen eingeben.".into())
    } else if name.chars().count() > USERNAME_MAX {
        Some(format!("Höchstens {USERNAME_MAX} Zeichen."))
    } else if !is_valid_username(name) {
        Some(
            "Erlaubt sind nur Kleinbuchstaben a–z, Ziffern sowie . - _ (keine Umlaute, keine Leerzeichen)."
                .into(),
        )
    } else {
        None
    }
}

pub fn status_label(active: bool) -> &'static str {
    if active { "Aktiv" } else { "Gesperrt" }
}

/// daisyUI-Klassen für die Status-Plakette (vollständig ausgeschrieben, damit Tailwind sie findet).
pub fn status_badge(active: bool) -> &'static str {
    if active {
        "badge badge-sm badge-success badge-soft"
    } else {
        "badge badge-sm badge-error badge-soft"
    }
}

/// „Muster Anna (anna.muster)“
pub fn display_name(name: &str, username: &str) -> String {
    format!("{name} ({username})")
}

/// Formulareingabe für Anlegen und Bearbeiten (ohne Passwort).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UserInput {
    pub benutzername: String,
    /// Kennung der Adresse (Typ „Person“)
    pub adresse_id: String,
    /// „aktiv“ oder „gesperrt“
    pub status: String,
}

impl UserInput {
    pub fn new_default() -> Self {
        Self {
            status: "aktiv".into(),
            ..Default::default()
        }
    }

    /// Leerzeichen entfernen; Benutzername in Kleinbuchstaben.
    pub fn normalized(&self) -> Self {
        Self {
            benutzername: self.benutzername.trim().to_lowercase(),
            adresse_id: self.adresse_id.trim().into(),
            status: self.status.trim().to_lowercase(),
        }
    }

    /// Prüft die normalisierte Eingabe (AK-11, AK-13). Leere Map = gültig.
    /// Ob die Adresse eine Person und noch frei ist, prüft der Server zusätzlich.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |f: &str, m: &str| {
            errors.entry(f.into()).or_insert_with(|| m.into());
        };
        if let Some(m) = username_error(&self.benutzername) {
            err("benutzername", &m);
        }
        if self.adresse_id.is_empty() {
            err("adresse_id", "Bitte eine Adresse wählen.");
        } else if self.adresse_id.parse::<i64>().is_err() {
            err("adresse_id", "Ungültige Adresse.");
        }
        if !STATUSES.iter().any(|(k, _)| *k == self.status) {
            err("status", "Bitte einen Status aus der Liste wählen.");
        }
        errors
    }

    pub fn address_id(&self) -> Option<i64> {
        self.adresse_id.parse().ok()
    }

    pub fn is_active(&self) -> bool {
        self.status == "aktiv"
    }
}

/// Benutzer mit den Angaben seiner Adresse (Detailansicht, AK-6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct User {
    pub id: i64,
    pub benutzername: String,
    pub aktiv: bool,
    /// Mit einem Zugang im Portal verknüpft – nur dann kann sich der Benutzer anmelden (ANF-026)
    pub portal_zugang: bool,
    /// Bereits formatiert (TT.MM.JJJJ HH:MM, Zeitzone Europe/Zurich)
    pub erstellt_am: String,
    pub geaendert_am: String,
    pub adresse_id: i64,
    pub adresse_nummer: String,
    /// Anzeigename der Adresse
    pub name: String,
    pub email: Option<String>,
    pub telefon: Option<String>,
    pub mobiltelefon: Option<String>,
}

impl User {
    pub fn display_name(&self) -> String {
        display_name(&self.name, &self.benutzername)
    }

    pub fn to_input(&self) -> UserInput {
        UserInput {
            benutzername: self.benutzername.clone(),
            adresse_id: self.adresse_id.to_string(),
            status: if self.aktiv { "aktiv" } else { "gesperrt" }.into(),
        }
    }

    /// Anfangswert der Adress-Auswahl beim Bearbeiten.
    pub fn address(&self) -> PickOption {
        PickOption {
            id: self.adresse_id,
            nummer: self.adresse_nummer.clone(),
            name: self.name.clone(),
        }
    }
}

/// Zeile der Benutzerliste (AK-1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct UserRow {
    pub id: i64,
    pub benutzername: String,
    pub name: String,
    pub email: Option<String>,
    pub aktiv: bool,
}

/// Projekt, Teilprojekt oder Vorgang, bei dem der Benutzer hinterlegt ist (AK-31).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AssignedElement {
    pub level: Level,
    pub id: i64,
    /// Projektnummer bzw. Nummer im Projekt, z. B. „2.3“
    pub nummer: String,
    pub name: String,
    pub projekt_id: i64,
    pub projekt_nummer: String,
    pub projekt_name: String,
}

/// Verantwortung eines Benutzers in Projekten (AK-31).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Responsibility {
    pub assigned: Vec<AssignedElement>,
    /// Offene Projektaufgaben (hinterlegt oder geerbt) und Aufgaben ohne Projekt (ANF-006 AK-18)
    pub open_tasks: Vec<TaskItem>,
}

/// Benutzer mit seiner Verantwortung – alles, was die Detailansicht braucht.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UserDetail {
    pub user: User,
    pub responsibility: Responsibility,
}

/// Meldung, wenn ein Benutzer noch in Projekten hinterlegt oder für Aufgaben ohne Projekt zuständig ist
/// und nicht gelöscht werden kann (AK-32, ANF-006 AK-19).
/// `counts`: Anzahl je Ebene (Projekt, Teilprojekt, Vorgang); `standalone`: Aufgaben ohne Projekt.
pub fn still_responsible_message(counts: &[(Level, usize)], standalone: usize) -> String {
    let mut parts: Vec<String> = counts
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|(level, n)| level.count_label(*n))
        .collect();
    if standalone > 0 {
        parts.push(format!(
            "{} ohne Projekt",
            Level::Task.count_label(standalone)
        ));
    }
    let list = if parts.is_empty() {
        "Projekte".to_string()
    } else {
        parts.join(", ")
    };
    format!(
        "Der Benutzer ist noch verantwortlich für {list} und kann nicht gelöscht werden. \
         Bitte den Benutzer sperren oder die Verantwortung neu vergeben."
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> UserInput {
        UserInput {
            benutzername: "anna.muster".into(),
            adresse_id: "4".into(),
            ..UserInput::new_default()
        }
    }

    #[test]
    fn valid_input() {
        assert!(input().normalized().validate().is_empty());
        assert!(input().is_active());
        assert_eq!(input().address_id(), Some(4));
    }

    #[test]
    fn username_is_lowercased_and_checked() {
        let i = UserInput {
            benutzername: "  Anna.Muster ".into(),
            ..input()
        };
        assert_eq!(i.normalized().benutzername, "anna.muster");
        for bad in ["anna muster", "müller", "anna@firma", "a/b"] {
            let i = UserInput {
                benutzername: bad.into(),
                ..input()
            };
            assert!(
                i.normalized().validate().contains_key("benutzername"),
                "{bad}"
            );
        }
        assert!(is_valid_username("a-b_c.9"));
        let long = UserInput {
            benutzername: "a".repeat(51),
            ..input()
        };
        assert!(long.validate()["benutzername"].contains("50"));
    }

    #[test]
    fn address_and_status_required() {
        let e = UserInput::default().normalized().validate();
        assert!(e.contains_key("benutzername"));
        assert!(e.contains_key("adresse_id"));
        assert!(e.contains_key("status"));
        let i = UserInput {
            adresse_id: "x".into(),
            ..input()
        };
        assert!(i.validate().contains_key("adresse_id"));
    }

    #[test]
    fn delete_message_lists_levels() {
        let m = still_responsible_message(
            &[
                (Level::Project, 1),
                (Level::Subproject, 0),
                (Level::Process, 2),
            ],
            0,
        );
        assert!(m.contains("1 Projekt, 2 Vorgänge und"), "{m}");
        let m = still_responsible_message(&[(Level::Project, 0)], 3);
        assert!(m.contains("für 3 Aufgaben ohne Projekt und"), "{m}");
        assert!(m.contains("sperren"));
    }
}
