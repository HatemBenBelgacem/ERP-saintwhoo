//! Datenmodell und Prüfregeln für Projekte (ANF-003).
//! Hierarchie: Projekt → Teilprojekt → Vorgang → Aufgabe. Wird auf Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

/// Einträge pro Seite in der Projektliste (AK-3).
pub const PAGE_SIZE: i64 = 50;

/// Ebene in der Projekt-Hierarchie. In Webadressen und Formularen deutsch geschrieben.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Level {
    #[serde(rename = "projekt")]
    Project,
    #[serde(rename = "teilprojekt")]
    Subproject,
    #[serde(rename = "vorgang")]
    Process,
    #[serde(rename = "aufgabe")]
    Task,
}

impl Level {
    /// Ebene aus dem Schlüssel in der Webadresse („projekt“, „teilprojekt“, „vorgang“, „aufgabe“).
    pub fn from_key(key: &str) -> Option<Level> {
        [
            Level::Project,
            Level::Subproject,
            Level::Process,
            Level::Task,
        ]
        .into_iter()
        .find(|l| l.key() == key)
    }

    pub fn key(self) -> &'static str {
        match self {
            Level::Project => "projekt",
            Level::Subproject => "teilprojekt",
            Level::Process => "vorgang",
            Level::Task => "aufgabe",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Level::Project => "Projekt",
            Level::Subproject => "Teilprojekt",
            Level::Process => "Vorgang",
            Level::Task => "Aufgabe",
        }
    }

    pub fn label_plural(self) -> &'static str {
        match self {
            Level::Project => "Projekte",
            Level::Subproject => "Teilprojekte",
            Level::Process => "Vorgänge",
            Level::Task => "Aufgaben",
        }
    }

    /// „1 Vorgang“ / „3 Vorgänge“
    pub fn count_label(self, n: usize) -> String {
        format!(
            "{n} {}",
            if n == 1 {
                self.label()
            } else {
                self.label_plural()
            }
        )
    }

    /// Beschriftung der Schaltfläche zum Anlegen (AK-8).
    pub fn new_label(self) -> &'static str {
        match self {
            Level::Project => "Neues Projekt",
            Level::Subproject => "Neues Teilprojekt",
            Level::Process => "Neuer Vorgang",
            Level::Task => "Neue Aufgabe",
        }
    }

    pub fn child(self) -> Option<Level> {
        match self {
            Level::Project => Some(Level::Subproject),
            Level::Subproject => Some(Level::Process),
            Level::Process => Some(Level::Task),
            Level::Task => None,
        }
    }

    pub fn parent(self) -> Option<Level> {
        match self {
            Level::Project => None,
            Level::Subproject => Some(Level::Project),
            Level::Process => Some(Level::Subproject),
            Level::Task => Some(Level::Process),
        }
    }

    /// Webadresse der Detailansicht.
    pub fn url(self, id: i64) -> String {
        match self {
            Level::Project => format!("/projekte/{id}"),
            other => format!("/projekte/{}/{id}", other.key()),
        }
    }

    pub fn statuses(self) -> &'static [(&'static str, &'static str)] {
        if self == Level::Task {
            TASK_STATUSES
        } else {
            PROJECT_STATUSES
        }
    }

    pub fn default_status(self) -> &'static str {
        if self == Level::Task {
            "offen"
        } else {
            "aktiv"
        }
    }

    /// Farbiger Streifen links (als Schatten, funktioniert auch in Tabellenzellen).
    /// Klassen vollständig ausgeschrieben, damit Tailwind sie findet.
    pub fn stripe_class(self) -> &'static str {
        match self {
            Level::Project => "shadow-[inset_4px_0_0_var(--color-level-project)]",
            Level::Subproject => "shadow-[inset_4px_0_0_var(--color-level-subproject)]",
            Level::Process => "shadow-[inset_4px_0_0_var(--color-level-process)]",
            Level::Task => "shadow-[inset_4px_0_0_var(--color-level-task)]",
        }
    }

    /// Plakette mit dem Namen der Ebene.
    pub fn badge_class(self) -> &'static str {
        match self {
            Level::Project => {
                "badge badge-sm border-0 font-medium bg-level-project/15 text-level-project-ink"
            }
            Level::Subproject => {
                "badge badge-sm border-0 font-medium bg-level-subproject/15 text-level-subproject-ink"
            }
            Level::Process => {
                "badge badge-sm border-0 font-medium bg-level-process/15 text-level-process-ink"
            }
            Level::Task => {
                "badge badge-sm border-0 font-medium bg-level-task/15 text-level-task-ink"
            }
        }
    }

    /// Leichter Hintergrund für Zeilen dieser Ebene.
    pub fn tint_class(self) -> &'static str {
        match self {
            Level::Project => "bg-level-project/5",
            Level::Subproject => "bg-level-subproject/5",
            Level::Process => "bg-level-process/5",
            Level::Task => "",
        }
    }

    /// Wofür ein hier hinterlegter Benutzer verantwortlich ist (ANF-005 AK-25 bis AK-27).
    pub fn responsibility_scope(self) -> &'static str {
        match self {
            Level::Project => {
                "Gilt für das Projekt und alle Teilprojekte, Vorgänge und Aufgaben darin."
            }
            Level::Subproject => "Gilt für das Teilprojekt und alle Vorgänge und Aufgaben darin.",
            Level::Process => "Gilt für den Vorgang und alle Aufgaben darin.",
            Level::Task => "",
        }
    }

    /// Feldbezeichnung für den Namen.
    pub fn name_label(self) -> &'static str {
        if self == Level::Task { "Titel" } else { "Name" }
    }

    /// Feldbezeichnung für das Enddatum.
    pub fn end_label(self) -> &'static str {
        if self == Level::Task {
            "Fällig am"
        } else {
            "Ende"
        }
    }
}

pub const PROJECT_STATUSES: &[(&str, &str)] = &[
    ("geplant", "Geplant"),
    ("aktiv", "Aktiv"),
    ("pausiert", "Pausiert"),
    ("abgeschlossen", "Abgeschlossen"),
    ("abgebrochen", "Abgebrochen"),
];

pub const TASK_STATUSES: &[(&str, &str)] = &[("offen", "Offen"), ("erledigt", "Erledigt")];

pub fn status_label(status: &str) -> &str {
    PROJECT_STATUSES
        .iter()
        .chain(TASK_STATUSES)
        .find(|(k, _)| *k == status)
        .map_or(status, |(_, l)| l)
}

/// daisyUI-Klassen für die Status-Plakette (vollständig ausgeschrieben, damit Tailwind sie findet).
pub fn status_badge(status: &str) -> &'static str {
    match status {
        "aktiv" => "badge badge-sm badge-primary badge-soft",
        "erledigt" | "abgeschlossen" => "badge badge-sm badge-success badge-soft",
        "pausiert" => "badge badge-sm badge-warning badge-soft",
        "abgebrochen" => "badge badge-sm badge-error badge-soft",
        _ => "badge badge-sm badge-ghost",
    }
}

/// ISO-Datum (JJJJ-MM-TT) → TT.MM.JJJJ; leer → „–“.
pub fn format_date(iso: Option<&str>) -> String {
    match iso {
        Some(d) if d.len() == 10 => format!("{}.{}.{}", &d[8..10], &d[5..7], &d[0..4]),
        Some(d) => d.to_string(),
        None => "–".into(),
    }
}

/// Prüft ein ISO-Datum JJJJ-MM-TT auf Format und gültigen Kalendertag.
pub fn is_valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let (Ok(y), Ok(m), Ok(d)) = (
        s[0..4].parse::<u32>(),
        s[5..7].parse::<u32>(),
        s[8..10].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days = match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return false,
    };
    (1900..=2999).contains(&y) && (1..=days).contains(&d)
}

/// Formulareingabe für alle Ebenen. Felder, die eine Ebene nicht hat, bleiben leer.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ElementInput {
    pub name: String,
    pub beschreibung: String,
    pub status: String,
    pub start: String,
    pub ende: String,
    /// nur Projekt: Kunde (Kennung der Adresse)
    pub adresse_id: String,
    /// Projekt, Teilprojekt, Vorgang: verantwortlicher Benutzer (Kennung, ANF-005 AK-24)
    pub benutzer_id: String,
}

impl ElementInput {
    pub fn new_default(level: Level) -> Self {
        Self {
            status: level.default_status().into(),
            ..Default::default()
        }
    }

    /// Leerzeichen entfernen; Felder, die die Ebene nicht hat, leeren; Aufwand mit Punkt.
    pub fn normalized(&self, level: Level) -> Self {
        let mut n = Self {
            name: self.name.trim().into(),
            beschreibung: self.beschreibung.trim().into(),
            status: self.status.trim().into(),
            start: self.start.trim().into(),
            ende: self.ende.trim().into(),
            adresse_id: self.adresse_id.trim().into(),
            benutzer_id: self.benutzer_id.trim().into(),
        };
        if level != Level::Project {
            n.adresse_id.clear();
        }
        // Aufgaben haben nur Titel und Beschreibung; erledigt/offen wird über das Kontrollkästchen gesetzt.
        // Einen verantwortlichen Benutzer erhalten sie nur über die Ebenen darüber (ANF-005 AK-24).
        if level == Level::Task {
            n.benutzer_id.clear();
            n.start.clear();
            n.ende.clear();
            n.status = "offen".into();
        }
        n
    }

    /// Prüft die normalisierte Eingabe (AK-10). Leere Map = gültig.
    pub fn validate(&self, level: Level) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |f: &str, m: &str| {
            errors.entry(f.into()).or_insert_with(|| m.into());
        };

        if self.name.is_empty() {
            err(
                "name",
                if level == Level::Task {
                    "Bitte einen Titel eingeben."
                } else {
                    "Bitte einen Namen eingeben."
                },
            );
        } else if self.name.chars().count() > 200 {
            err("name", "Höchstens 200 Zeichen.");
        }
        if !level.statuses().iter().any(|(k, _)| *k == self.status) {
            err("status", "Bitte einen Status aus der Liste wählen.");
        }
        for field in ["start", "ende"] {
            let v = if field == "start" {
                &self.start
            } else {
                &self.ende
            };
            if !v.is_empty() && !is_valid_date(v) {
                err(field, "Bitte ein gültiges Datum eingeben.");
            }
        }
        if is_valid_date(&self.start) && is_valid_date(&self.ende) && self.ende < self.start {
            err(
                "ende",
                &format!("{} darf nicht vor dem Start liegen.", level.end_label()),
            );
        }
        if level == Level::Project
            && !self.adresse_id.is_empty()
            && self.adresse_id.parse::<i64>().is_err()
        {
            err("adresse_id", "Ungültige Adresse.");
        }
        if !self.benutzer_id.is_empty() && self.benutzer_id.parse::<i64>().is_err() {
            err("benutzer_id", "Ungültiger Benutzer.");
        }
        errors
    }

    pub fn customer_id(&self) -> Option<i64> {
        self.adresse_id.parse().ok()
    }

    /// Verantwortlicher Benutzer (Kennung) oder `None`.
    pub fn responsible_id(&self) -> Option<i64> {
        self.benutzer_id.parse().ok()
    }
}

/// Benutzer, der bei einem Projekt, Teilprojekt oder Vorgang als verantwortlich hinterlegt ist (ANF-005).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Responsible {
    pub id: i64,
    pub username: String,
    /// Anzeigename der Adresse des Benutzers
    pub name: String,
    pub active: bool,
}

impl Responsible {
    /// Aus den Spalten einer Abfrage (alle leer = niemand hinterlegt).
    pub fn from_columns(
        id: Option<i64>,
        username: Option<String>,
        name: Option<String>,
        active: Option<bool>,
    ) -> Option<Self> {
        Some(Self {
            id: id?,
            username: username?,
            name: name.unwrap_or_default(),
            active: active.unwrap_or(false),
        })
    }

    /// „Muster Anna (anna.muster)“
    pub fn display(&self) -> String {
        format!("{} ({})", self.name, self.username)
    }

    /// Anfangswert für die Benutzer-Auswahl im Formular.
    pub fn pick_option(&self) -> PickOption {
        PickOption {
            id: self.id,
            nummer: self.username.clone(),
            name: self.name.clone(),
        }
    }

    /// Webadresse der Detailansicht des Benutzers.
    pub fn url(&self) -> String {
        format!("/einstellungen/benutzer/{}", self.id)
    }
}

/// Ein verantwortlicher Benutzer mit Herkunft der Verantwortung (ANF-005 AK-28),
/// z. B. „vom Projekt“, „vom Teilprojekt 2“ oder „hier hinterlegt“.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResponsibleEntry {
    pub user: Responsible,
    pub origin: String,
}

/// Projektkopf.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Project {
    pub id: i64,
    pub nummer: String,
    pub name: String,
    pub beschreibung: Option<String>,
    pub status: String,
    pub start: Option<String>,
    pub ende: Option<String>,
    pub adresse_id: Option<i64>,
    pub kunde_nummer: Option<String>,
    pub kunde: Option<String>,
    /// Verantwortlicher Benutzer (ANF-005)
    pub benutzer_id: Option<i64>,
    pub benutzername: Option<String>,
    pub benutzer_name: Option<String>,
    pub benutzer_aktiv: Option<bool>,
    pub erstellt_am: String,
    pub geaendert_am: String,
}

impl Project {
    pub fn responsible(&self) -> Option<Responsible> {
        Responsible::from_columns(
            self.benutzer_id,
            self.benutzername.clone(),
            self.benutzer_name.clone(),
            self.benutzer_aktiv,
        )
    }

    pub fn to_input(&self) -> ElementInput {
        ElementInput {
            name: self.name.clone(),
            beschreibung: self.beschreibung.clone().unwrap_or_default(),
            status: self.status.clone(),
            start: self.start.clone().unwrap_or_default(),
            ende: self.ende.clone().unwrap_or_default(),
            adresse_id: self.adresse_id.map(|i| i.to_string()).unwrap_or_default(),
            benutzer_id: self.benutzer_id.map(|i| i.to_string()).unwrap_or_default(),
        }
    }

    pub fn customer(&self) -> Option<PickOption> {
        Some(PickOption {
            id: self.adresse_id?,
            nummer: self.kunde_nummer.clone()?,
            name: self.kunde.clone()?,
        })
    }
}

/// Teilprojekt, Vorgang oder Aufgabe mit allen untergeordneten Elementen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Node {
    pub id: i64,
    pub level: Level,
    /// Nummer innerhalb des Projekts, z. B. „2.3.1“ (AK-9)
    pub number: String,
    pub name: String,
    pub beschreibung: Option<String>,
    pub status: String,
    pub start: Option<String>,
    pub ende: Option<String>,
    /// Hier hinterlegter verantwortlicher Benutzer (nur Teilprojekt und Vorgang, ANF-005)
    pub responsible: Option<Responsible>,
    pub erstellt_am: String,
    pub geaendert_am: String,
    pub children: Vec<Node>,
}

impl Node {
    /// (erledigte Aufgaben, alle Aufgaben) – bei einer Aufgabe sie selbst (AK-5).
    pub fn task_stats(&self) -> (usize, usize) {
        if self.level == Level::Task {
            (usize::from(self.status == "erledigt"), 1)
        } else {
            task_stats(&self.children)
        }
    }

    pub fn to_input(&self) -> ElementInput {
        ElementInput {
            name: self.name.clone(),
            beschreibung: self.beschreibung.clone().unwrap_or_default(),
            status: self.status.clone(),
            start: self.start.clone().unwrap_or_default(),
            ende: self.ende.clone().unwrap_or_default(),
            adresse_id: String::new(),
            benutzer_id: self
                .responsible
                .as_ref()
                .map(|r| r.id.to_string())
                .unwrap_or_default(),
        }
    }

    pub fn title(&self) -> String {
        format!("{} {}", self.number, self.name)
    }
}

pub fn task_stats(nodes: &[Node]) -> (usize, usize) {
    nodes
        .iter()
        .map(Node::task_stats)
        .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
}

/// Anzahl untergeordneter Elemente je Ebene, z. B. für den Löschdialog (AK-14).
pub fn descendant_summary(nodes: &[Node]) -> String {
    fn walk(nodes: &[Node], counts: &mut [usize; 3]) {
        for n in nodes {
            let i = match n.level {
                Level::Subproject => 0,
                Level::Process => 1,
                _ => 2,
            };
            counts[i] += 1;
            walk(&n.children, counts);
        }
    }
    let mut counts = [0; 3];
    walk(nodes, &mut counts);
    [Level::Subproject, Level::Process, Level::Task]
        .into_iter()
        .zip(counts)
        .filter(|(_, n)| *n > 0)
        .map(|(l, n)| l.count_label(n))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Ein Projekt mit vollständiger Struktur.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectTree {
    pub project: Project,
    pub children: Vec<Node>,
}

impl ProjectTree {
    /// Pfad vom Teilprojekt bis zum gesuchten Element (für Pfadleiste und Detailansicht).
    pub fn find_path(&self, level: Level, id: i64) -> Option<Vec<&Node>> {
        fn walk<'a>(nodes: &'a [Node], level: Level, id: i64, path: &mut Vec<&'a Node>) -> bool {
            for n in nodes {
                path.push(n);
                if (n.level == level && n.id == id) || walk(&n.children, level, id, path) {
                    return true;
                }
                path.pop();
            }
            false
        }
        let mut path = Vec::new();
        walk(&self.children, level, id, &mut path).then_some(path)
    }

    /// Pfadleiste bis einschliesslich `path` (letzter Eintrag mit Verweis, wenn `link_last`).
    pub fn breadcrumbs(&self, path: &[&Node], link_last: bool) -> Vec<(String, Option<String>)> {
        let mut items = vec![
            ("Projekte".to_string(), Some("/projekte".to_string())),
            (
                self.project.nummer.clone(),
                Some(Level::Project.url(self.project.id)),
            ),
        ];
        for n in path {
            items.push((n.title(), Some(n.level.url(n.id))));
        }
        if !link_last && let Some(last) = items.last_mut() {
            last.1 = None;
        }
        items
    }

    /// Alle verantwortlichen Benutzer des Elements am Ende von `path` (leer = das Projekt selbst),
    /// von oben nach unten (ANF-005 AK-25 bis AK-28). Die Verantwortung wird nicht gespeichert, sondern
    /// hier aus der Hierarchie ermittelt (AK-29). Derselbe Benutzer erscheint nur einmal, mit der obersten Ebene.
    pub fn responsible_for(&self, path: &[&Node]) -> Vec<ResponsibleEntry> {
        let viewed = path.len();
        let sources = std::iter::once((self.project.responsible(), "vom Projekt".to_string()))
            .chain(path.iter().map(|n| {
                (
                    n.responsible.clone(),
                    format!("vom {} {}", n.level.label(), n.number),
                )
            }));
        let mut entries: Vec<ResponsibleEntry> = Vec::new();
        for (i, (user, origin)) in sources.enumerate() {
            let Some(user) = user else { continue };
            if entries.iter().any(|e| e.user.id == user.id) {
                continue;
            }
            let origin = if i == viewed {
                "hier hinterlegt".to_string()
            } else {
                origin
            };
            entries.push(ResponsibleEntry { user, origin });
        }
        entries
    }
}

/// Zeile der Projektliste.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProjectRow {
    pub id: i64,
    pub nummer: String,
    pub name: String,
    pub kunde: Option<String>,
    pub status: String,
    pub start: Option<String>,
    pub ende: Option<String>,
    /// Verantwortlicher Benutzer (ANF-005 AK-30)
    pub benutzer_id: Option<i64>,
    pub benutzername: Option<String>,
    pub benutzer_name: Option<String>,
    pub benutzer_aktiv: Option<bool>,
    /// Struktur des Projekts (Teilprojekte → Vorgänge → Aufgaben) für die Übersicht
    #[cfg_attr(feature = "ssr", sqlx(skip))]
    pub children: Vec<Node>,
}

impl ProjectRow {
    pub fn responsible(&self) -> Option<Responsible> {
        Responsible::from_columns(
            self.benutzer_id,
            self.benutzername.clone(),
            self.benutzer_name.clone(),
            self.benutzer_aktiv,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectPage {
    pub rows: Vec<ProjectRow>,
    pub total: i64,
    pub page: i64,
}

impl ProjectPage {
    pub fn page_count(&self) -> i64 {
        ((self.total + PAGE_SIZE - 1) / PAGE_SIZE).max(1)
    }
}

/// Sortierbare Spalten (AK-2): Schlüssel in der Webadresse → Beschriftung.
pub const SORT_COLUMNS: &[(&str, &str)] = &[
    ("nummer", "Nummer"),
    ("name", "Name"),
    ("kunde", "Kunde"),
    ("status", "Status"),
    ("start", "Start"),
    ("ende", "Ende"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn node(level: Level, status: &str, children: Vec<Node>) -> Node {
        Node {
            id: 1,
            level,
            number: "1".into(),
            name: "x".into(),
            beschreibung: None,
            status: status.into(),
            start: None,
            ende: None,
            responsible: None,
            erstellt_am: String::new(),
            geaendert_am: String::new(),
            children,
        }
    }

    fn user(id: i64, name: &str) -> Responsible {
        Responsible {
            id,
            username: name.to_lowercase(),
            name: name.into(),
            active: true,
        }
    }

    fn tree_with(project_user: Option<Responsible>, children: Vec<Node>) -> ProjectTree {
        ProjectTree {
            project: Project {
                id: 1,
                nummer: "P-2026-001".into(),
                name: "Neubau".into(),
                beschreibung: None,
                status: "aktiv".into(),
                start: None,
                ende: None,
                adresse_id: None,
                kunde_nummer: None,
                kunde: None,
                benutzer_id: project_user.as_ref().map(|u| u.id),
                benutzername: project_user.as_ref().map(|u| u.username.clone()),
                benutzer_name: project_user.as_ref().map(|u| u.name.clone()),
                benutzer_aktiv: project_user.as_ref().map(|u| u.active),
                erstellt_am: String::new(),
                geaendert_am: String::new(),
            },
            children,
        }
    }

    /// Beispiel aus ANF-005: Anna beim Projekt, Ben bei Teilprojekt 2, Carla bei Vorgang 2.3.
    fn example_tree() -> ProjectTree {
        let mut task_21 = node(Level::Task, "offen", vec![]);
        task_21.id = 21;
        task_21.number = "2.1.1".into();
        let mut task_23 = node(Level::Task, "offen", vec![]);
        task_23.id = 23;
        task_23.number = "2.3.1".into();
        let mut proc_21 = node(Level::Process, "aktiv", vec![task_21]);
        proc_21.id = 21;
        proc_21.number = "2.1".into();
        let mut proc_23 = node(Level::Process, "aktiv", vec![task_23]);
        proc_23.id = 23;
        proc_23.number = "2.3".into();
        proc_23.responsible = Some(user(3, "Carla"));
        let mut sp_1 = node(Level::Subproject, "aktiv", vec![]);
        sp_1.id = 1;
        let mut sp_2 = node(Level::Subproject, "aktiv", vec![proc_21, proc_23]);
        sp_2.id = 2;
        sp_2.number = "2".into();
        sp_2.responsible = Some(user(2, "Ben"));
        tree_with(Some(user(1, "Anna")), vec![sp_1, sp_2])
    }

    fn names_and_origins(entries: &[ResponsibleEntry]) -> Vec<(String, String)> {
        entries
            .iter()
            .map(|e| (e.user.name.clone(), e.origin.clone()))
            .collect()
    }

    #[test]
    fn dates() {
        assert!(is_valid_date("2026-02-28"));
        assert!(is_valid_date("2028-02-29"));
        assert!(!is_valid_date("2026-02-29"));
        assert!(!is_valid_date("2026-13-01"));
        assert!(!is_valid_date("26-01-01"));
        assert!(!is_valid_date("2026-1-01"));
        assert_eq!(format_date(Some("2026-11-03")), "03.11.2026");
        assert_eq!(format_date(None), "–");
    }

    #[test]
    fn name_required_and_end_after_start() {
        let mut i = ElementInput::new_default(Level::Project);
        assert!(i.validate(Level::Project).contains_key("name"));
        i.name = "Neubau".into();
        i.start = "2026-05-01".into();
        i.ende = "2026-04-30".into();
        let e = i.normalized(Level::Project).validate(Level::Project);
        assert_eq!(e.len(), 1, "{e:?}");
        assert!(e["ende"].contains("nicht vor dem Start"));
        i.ende = "2026-05-01".into();
        assert!(
            i.normalized(Level::Project)
                .validate(Level::Project)
                .is_empty()
        );
    }

    #[test]
    fn task_has_only_title_and_description() {
        let i = ElementInput {
            name: " Pläne zeichnen ".into(),
            beschreibung: "x".into(),
            status: "erledigt".into(),
            start: "2026-01-01".into(),
            ende: "kein Datum".into(),
            ..Default::default()
        };
        let n = i.normalized(Level::Task);
        assert_eq!(n.name, "Pläne zeichnen");
        assert_eq!(n.beschreibung, "x");
        assert!(n.start.is_empty() && n.ende.is_empty());
        assert_eq!(n.status, "offen");
        assert!(n.validate(Level::Task).is_empty());
        assert!(
            ElementInput::default()
                .normalized(Level::Task)
                .validate(Level::Task)
                .contains_key("name")
        );
        assert!(Level::Process.statuses().iter().any(|(k, _)| *k == "aktiv"));
    }

    #[test]
    fn normalization_clears_foreign_fields() {
        let i = ElementInput {
            adresse_id: "5".into(),
            ..Default::default()
        };
        assert!(i.normalized(Level::Process).adresse_id.is_empty());
        assert_eq!(i.normalized(Level::Project).adresse_id, "5");
    }

    #[test]
    fn stats_and_summary() {
        let tasks = vec![
            node(Level::Task, "erledigt", vec![]),
            node(Level::Task, "offen", vec![]),
            node(Level::Task, "erledigt", vec![]),
        ];
        let sp = vec![node(
            Level::Subproject,
            "aktiv",
            vec![node(Level::Process, "aktiv", tasks)],
        )];
        assert_eq!(task_stats(&sp), (2, 3));
        assert_eq!(
            descendant_summary(&sp),
            "1 Teilprojekt, 1 Vorgang, 3 Aufgaben"
        );
        assert_eq!(descendant_summary(&[]), "");
    }

    #[test]
    fn responsibility_is_inherited_downwards() {
        let t = example_tree();
        let o = |a: &str, b: &str| (a.to_string(), b.to_string());

        // Projekt selbst (AK-25)
        assert_eq!(
            names_and_origins(&t.responsible_for(&[])),
            vec![o("Anna", "hier hinterlegt")]
        );
        // Teilprojekt 1 ohne eigenen Benutzer: nur Anna (AK-26: Ben gilt nicht für andere Teilprojekte)
        let path = t.find_path(Level::Subproject, 1).unwrap();
        assert_eq!(
            names_and_origins(&t.responsible_for(&path)),
            vec![o("Anna", "vom Projekt")]
        );
        // Aufgabe 2.1.1: Anna und Ben (AK-27: Carla gilt nicht für andere Vorgänge)
        let path = t.find_path(Level::Task, 21).unwrap();
        assert_eq!(
            names_and_origins(&t.responsible_for(&path)),
            vec![o("Anna", "vom Projekt"), o("Ben", "vom Teilprojekt 2")]
        );
        // Aufgabe 2.3.1: alle drei (AK-28)
        let path = t.find_path(Level::Task, 23).unwrap();
        assert_eq!(
            names_and_origins(&t.responsible_for(&path)),
            vec![
                o("Anna", "vom Projekt"),
                o("Ben", "vom Teilprojekt 2"),
                o("Carla", "vom Vorgang 2.3")
            ]
        );
        // Vorgang 2.3 selbst
        let path = t.find_path(Level::Process, 23).unwrap();
        assert_eq!(
            t.responsible_for(&path).last().map(|e| e.origin.as_str()),
            Some("hier hinterlegt")
        );
    }

    #[test]
    fn same_user_on_several_levels_appears_once_with_top_level() {
        let mut sp = node(Level::Subproject, "aktiv", vec![]);
        sp.responsible = Some(user(1, "Anna"));
        let t = tree_with(Some(user(1, "Anna")), vec![sp]);
        let path = t.find_path(Level::Subproject, 1).unwrap();
        let entries = t.responsible_for(&path);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].origin, "vom Projekt");
        assert!(tree_with(None, vec![]).responsible_for(&[]).is_empty());
    }

    #[test]
    fn task_has_no_own_responsible() {
        let i = ElementInput {
            name: "x".into(),
            benutzer_id: "4".into(),
            ..Default::default()
        };
        assert!(i.normalized(Level::Task).benutzer_id.is_empty());
        assert_eq!(i.normalized(Level::Process).responsible_id(), Some(4));
        let bad = ElementInput {
            benutzer_id: "abc".into(),
            ..i
        };
        assert!(
            bad.normalized(Level::Project)
                .validate(Level::Project)
                .contains_key("benutzer_id")
        );
    }

    #[test]
    fn urls() {
        assert_eq!(Level::Project.url(4), "/projekte/4");
        assert_eq!(Level::Process.url(7), "/projekte/vorgang/7");
        assert_eq!(Level::from_key("vorgang"), Some(Level::Process));
        assert_eq!(Level::from_key("x"), None);
    }
}
