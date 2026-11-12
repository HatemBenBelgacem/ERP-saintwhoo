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
}

/// Alle Pfade unterhalb dieses Präfixes zeigen die Einstellungsnavigation (AK-9).
pub const SETTINGS_PREFIX: &str = "/einstellungen";

/// Hauptnavigation.
pub const MAIN_NAV: &[NavItem] = &[
    NavItem {
        title: "Startseite",
        path: "/",
        icon: Some(Icon::Home),
    },
    NavItem {
        title: "Adressen",
        path: "/adressen",
        icon: Some(Icon::Addresses),
    },
    NavItem {
        title: "Projekte",
        path: "/projekte",
        icon: Some(Icon::Projects),
    },
    NavItem {
        title: "Einkauf",
        path: "/einkauf",
        icon: Some(Icon::Purchasing),
    },
];

/// Einstellungsnavigation. Der erste Eintrag ist das Ziel von „Einstellungen“ (AK-7).
pub const SETTINGS_NAV: &[NavItem] = &[NavItem {
    title: "Allgemein",
    path: "/einstellungen/allgemein",
    icon: Some(Icon::Sliders),
}];

/// Ziel des Eintrags „Einstellungen“ in der Fusszeile der Seitenleiste.
pub fn settings_entry_path() -> &'static str {
    SETTINGS_NAV
        .first()
        .map_or(SETTINGS_PREFIX, |item| item.path)
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
    fn settings_entry_points_to_first_settings_page() {
        assert_eq!(settings_entry_path(), "/einstellungen/allgemein");
    }
}
