//! Zentrale Definition aller Navigationseinträge (ANF-001, AK-5).
//!
//! Neue Module werden hier ergänzt – das Layout muss dafür nicht geändert werden.

use crate::icons::Icon;

/// Ein Eintrag in der Seitenleiste.
#[derive(Clone, Copy, Debug)]
pub struct NavItem {
    pub title: &'static str,
    pub path: &'static str,
    pub icon: Option<Icon>,
    /// Kurzbeschreibung für die Karte auf der Startseite
    pub description: &'static str,
}

/// Alle Pfade unterhalb dieses Präfixes zeigen die Einstellungsnavigation (AK-9).
pub const SETTINGS_PREFIX: &str = "/einstellungen";

/// Hauptnavigation.
pub const MAIN_NAV: &[NavItem] = &[
    NavItem {
        title: "Startseite",
        path: "/",
        icon: Some(Icon::Home),
        description: "Übersicht aller Module.",
    },
    NavItem {
        title: "Adressen",
        path: "/adressen",
        icon: Some(Icon::Addresses),
        description: "Firmen und Personen: Kunden, Lieferanten und Partner an einer zentralen Stelle.",
    },
    NavItem {
        title: "Projekte",
        path: "/projekte",
        icon: Some(Icon::Projects),
        description: "Projekte mit Teilprojekten, Vorgängen und Aufgaben planen und den Fortschritt verfolgen.",
    },
    // ANF-006
    NavItem {
        title: "Aufgaben",
        path: "/aufgaben",
        icon: Some(Icon::Tasks),
        description: "Alle zugeordneten Aufgaben aus allen Projekten an einer Stelle – und Aufgaben ohne Projekt.",
    },
    NavItem {
        title: "Einkauf",
        path: "/einkauf",
        icon: Some(Icon::Purchasing),
        description: "Einkaufsaufträge bei Lieferanten erfassen, bestellen und den Wareneingang verfolgen.",
    },
];

/// Einstellungsnavigation. Der erste Eintrag ist das Ziel von „Einstellungen“ (AK-7).
pub const SETTINGS_NAV: &[NavItem] = &[
    NavItem {
        title: "Allgemein",
        path: "/einstellungen/allgemein",
        icon: Some(Icon::Sliders),
        description: "Allgemeine Einstellungen.",
    },
    // ANF-025
    NavItem {
        title: "Firma",
        path: "/einstellungen/firma",
        icon: Some(Icon::Building),
        description: "Eigene Firma: Absender aller Belege, verknüpft mit einer Adresse.",
    },
    // ANF-005
    NavItem {
        title: "Benutzer",
        path: "/einstellungen/benutzer",
        icon: Some(Icon::User),
        description: "Benutzer anlegen, sperren und ihrer Adresse zuordnen.",
    },
];

/// Einstellungsnavigation, Abschnitt „Finanzen“: Stammdaten der Finanzen (ANF-010 AK-3).
pub const SETTINGS_FINANCE_NAV: &[NavItem] = &[
    // ANF-011
    NavItem {
        title: "Kontenplan",
        path: "/einstellungen/finanzen/kontenplan",
        icon: Some(Icon::Ledger),
        description: "Konten nach KMU-Kontenrahmen und Systemkonten für automatische Buchungen.",
    },
    // ANF-012
    NavItem {
        title: "MWST",
        path: "/einstellungen/finanzen/mwst",
        icon: Some(Icon::Percent),
        description: "MWST-Codes, MWST-Pflicht und Umsatzgrenze.",
    },
    // ANF-013
    NavItem {
        title: "Geschäftsjahre",
        path: "/einstellungen/finanzen/geschaeftsjahre",
        icon: Some(Icon::Calendar),
        description: "Geschäftsjahre und Monatsperioden; gesperrte Perioden nehmen keine Buchungen mehr auf.",
    },
];

/// Ziel des Eintrags „Einstellungen“ in der Fusszeile der Seitenleiste.
pub fn settings_entry_path() -> &'static str {
    SETTINGS_NAV
        .first()
        .map_or(SETTINGS_PREFIX, |item| item.path)
}

/// Module für die Karten auf der Startseite: alle Einträge der Hauptnavigation ausser der Startseite.
pub fn modules() -> impl Iterator<Item = &'static NavItem> {
    MAIN_NAV.iter().filter(|item| item.path != "/")
}

/// `true`, wenn die aktuelle URL zum Einstellungsbereich gehört.
pub fn is_settings_path(current: &str) -> bool {
    current == SETTINGS_PREFIX || current.starts_with(&format!("{SETTINGS_PREFIX}/"))
}

/// `true`, wenn `item_path` für die aktuelle URL als aktiv markiert werden soll (AK-4).
/// Die Startseite `/` ist nur bei exakter Übereinstimmung aktiv, andere Einträge auch für Unterseiten.
pub fn is_active(item_path: &str, current: &str) -> bool {
    let current = current.trim_end_matches('/');
    let item = item_path.trim_end_matches('/');
    if item.is_empty() {
        return current.is_empty();
    }
    current == item || current.starts_with(&format!("{item}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_paths() {
        assert!(is_settings_path("/einstellungen"));
        assert!(is_settings_path("/einstellungen/allgemein"));
        assert!(!is_settings_path("/"));
        assert!(!is_settings_path("/einstellungenxyz"));
    }

    #[test]
    fn active_entries() {
        assert!(is_active("/", "/"));
        assert!(!is_active("/", "/lager"));
        assert!(is_active("/lager", "/lager"));
        assert!(is_active("/lager", "/lager/artikel/5"));
        assert!(is_active("/lager", "/lager/"));
        assert!(!is_active("/lager", "/lagerort"));
    }

    #[test]
    fn modules_exclude_home_and_have_descriptions() {
        assert!(modules().all(|m| m.path != "/" && !m.description.is_empty()));
        assert_eq!(modules().count(), MAIN_NAV.len() - 1);
    }

    #[test]
    fn settings_entry_points_to_first_settings_page() {
        assert_eq!(settings_entry_path(), "/einstellungen/allgemein");
    }
}
