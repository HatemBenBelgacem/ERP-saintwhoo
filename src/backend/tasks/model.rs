//! Datenmodell und Prüfregeln für die Navigation „Aufgaben“ (ANF-006).
//! Projektaufgaben (ANF-003) und Aufgaben ohne Projekt in einer Liste. Wird auf Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

use crate::backend::projects::model::{Level, Responsible};
pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

/// Statusfilter (AK-5): Schlüssel in der Webadresse → Beschriftung. Standard ist der erste.
pub const STATUS_FILTERS: &[(&str, &str)] = &[
    ("offen", "Offen"),
    ("erledigt", "Erledigt"),
    ("alle", "Alle Status"),
];

/// Filter nach Art (AK-5). Standard ist der erste.
pub const KIND_FILTERS: &[(&str, &str)] = &[
    ("alle", "Alle Aufgaben"),
    ("mit", "Mit Projekt"),
    ("ohne", "Ohne Projekt"),
];

/// Basis-Webadresse der Navigation.
pub const BASE: &str = "/aufgaben";

/// Eine Zeile der Aufgabenliste: Projektaufgabe oder Aufgabe ohne Projekt (AK-3, AK-4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct TaskItem {
    pub id: i64,
    pub name: String,
    pub beschreibung: Option<String>,
    pub status: String,
    /// Leer bei Aufgaben ohne Projekt
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub projekt_name: Option<String>,
    /// Nummer im Projekt, z. B. „2.3.1“
    pub nummer: Option<String>,
    /// „vom Projekt“, „vom Teilprojekt 2“, „vom Vorgang 2.3“ oder „direkt zugewiesen“
    pub herkunft: String,
    /// Bereits formatiert (TT.MM.JJJJ HH:MM, Zeitzone Europe/Zurich)
    pub geaendert_am: String,
}

impl TaskItem {
    pub fn is_standalone(&self) -> bool {
        self.projekt_id.is_none()
    }

    pub fn is_done(&self) -> bool {
        self.status == "erledigt"
    }

    /// Detailansicht: im Projekt bzw. unter `/aufgaben/<kennung>` (AK-9).
    pub fn url(&self) -> String {
        if self.is_standalone() {
            task_url(self.id)
        } else {
            Level::Task.url(self.id)
        }
    }

    /// „P-2026-001 Neubau“ bzw. „ohne Projekt“
    pub fn project_label(&self) -> String {
        match (&self.projekt_nummer, &self.projekt_name) {
            (Some(n), Some(name)) => format!("{n} {name}"),
            _ => "ohne Projekt".into(),
        }
    }
}

/// Webadresse der Detailansicht einer Aufgabe ohne Projekt.
pub fn task_url(id: i64) -> String {
    format!("{BASE}/{id}")
}

/// Webadresse der Aufgabenliste eines Benutzers.
pub fn user_tasks_url(user_id: i64) -> String {
    format!("{BASE}?benutzer={user_id}")
}

/// Formulareingabe für eine Aufgabe ohne Projekt (AK-10, AK-15).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct StandaloneTaskInput {
    pub name: String,
    pub beschreibung: String,
    /// Zuständiger Benutzer (Kennung, Pflicht)
    pub benutzer_id: String,
}

impl StandaloneTaskInput {
    pub fn normalized(&self) -> Self {
        Self {
            name: self.name.trim().into(),
            beschreibung: self.beschreibung.trim().into(),
            benutzer_id: self.benutzer_id.trim().into(),
        }
    }

    /// Prüft die normalisierte Eingabe (AK-12). Ob der Benutzer aktiv ist, prüft der Server zusätzlich.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        if self.name.is_empty() {
            errors.insert("name".into(), "Bitte einen Titel eingeben.".into());
        } else if self.name.chars().count() > 200 {
            errors.insert("name".into(), "Höchstens 200 Zeichen.".into());
        }
        if self.benutzer_id.is_empty() {
            errors.insert(
                "benutzer_id".into(),
                "Bitte einen zuständigen Benutzer wählen.".into(),
            );
        } else if self.benutzer_id.parse::<i64>().is_err() {
            errors.insert("benutzer_id".into(), "Ungültiger Benutzer.".into());
        }
        errors
    }

    pub fn user_id(&self) -> Option<i64> {
        self.benutzer_id.parse().ok()
    }
}

/// Aufgabe ohne Projekt mit zuständigem Benutzer (Detailansicht, AK-14).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct StandaloneTask {
    pub id: i64,
    pub name: String,
    pub beschreibung: Option<String>,
    pub status: String,
    pub benutzer_id: i64,
    pub benutzername: String,
    pub benutzer_name: String,
    pub benutzer_aktiv: bool,
    pub erstellt_am: String,
    pub geaendert_am: String,
}

impl StandaloneTask {
    pub fn assignee(&self) -> Responsible {
        Responsible {
            id: self.benutzer_id,
            username: self.benutzername.clone(),
            name: self.benutzer_name.clone(),
            active: self.benutzer_aktiv,
        }
    }

    pub fn to_input(&self) -> StandaloneTaskInput {
        StandaloneTaskInput {
            name: self.name.clone(),
            beschreibung: self.beschreibung.clone().unwrap_or_default(),
            benutzer_id: self.benutzer_id.to_string(),
        }
    }
}

/// Ergebnis beim Aufruf von `/aufgaben/<kennung>` (AK-14, AK-17).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TaskLookup {
    /// Aufgabe ohne Projekt
    Found(StandaloneTask),
    /// Projektaufgabe – wird im Projekt angezeigt
    InProject(i64),
    Missing,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(projekt_id: Option<i64>) -> TaskItem {
        TaskItem {
            id: 7,
            name: "x".into(),
            beschreibung: None,
            status: "offen".into(),
            projekt_id,
            projekt_nummer: projekt_id.map(|_| "P-2026-001".into()),
            projekt_name: projekt_id.map(|_| "Neubau".into()),
            nummer: None,
            herkunft: String::new(),
            geaendert_am: String::new(),
        }
    }

    #[test]
    fn urls_depend_on_kind() {
        assert_eq!(item(None).url(), "/aufgaben/7");
        assert_eq!(item(Some(1)).url(), "/projekte/aufgabe/7");
        assert_eq!(item(None).project_label(), "ohne Projekt");
        assert_eq!(item(Some(1)).project_label(), "P-2026-001 Neubau");
        assert_eq!(user_tasks_url(3), "/aufgaben?benutzer=3");
    }

    #[test]
    fn title_and_assignee_required() {
        let e = StandaloneTaskInput::default().normalized().validate();
        assert!(e.contains_key("name"));
        assert!(e.contains_key("benutzer_id"));
        let ok = StandaloneTaskInput {
            name: " Offerte nachfassen ".into(),
            benutzer_id: "2".into(),
            ..Default::default()
        }
        .normalized();
        assert_eq!(ok.name, "Offerte nachfassen");
        assert!(ok.validate().is_empty());
        assert_eq!(ok.user_id(), Some(2));
        let long = StandaloneTaskInput {
            name: "a".repeat(201),
            ..ok
        };
        assert!(long.validate().contains_key("name"));
    }
}
