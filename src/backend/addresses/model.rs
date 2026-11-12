//! Datenmodell und Prüfregeln für Adressen (ANF-002).
//! Wird auf dem Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

/// Einträge pro Seite in der Adressliste (AK-4).
pub const PAGE_SIZE: i64 = 50;

/// Auswählbare Länder (ISO-Code, Name). Standard ist das erste Land.
pub const COUNTRIES: &[(&str, &str)] = &[
    ("CH", "Schweiz"),
    ("DE", "Deutschland"),
    ("AT", "Österreich"),
    ("LI", "Liechtenstein"),
    ("FR", "Frankreich"),
    ("IT", "Italien"),
    ("NL", "Niederlande"),
    ("BE", "Belgien"),
    ("LU", "Luxemburg"),
    ("ES", "Spanien"),
    ("GB", "Vereinigtes Königreich"),
    ("US", "Vereinigte Staaten"),
];

pub fn country_name(code: &str) -> &str {
    COUNTRIES
        .iter()
        .find(|(c, _)| *c == code)
        .map_or(code, |(_, name)| name)
}

pub use crate::forms::{FieldErrors, SaveOutcome};

/// Formulareingabe beim Anlegen und Bearbeiten. Alle Felder als Text, wie sie aus dem Formular kommen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AddressInput {
    pub typ: String,
    #[serde(default)]
    pub anrede: String,
    #[serde(default)]
    pub firmenname: String,
    #[serde(default)]
    pub vorname: String,
    #[serde(default)]
    pub nachname: String,
    #[serde(default)]
    pub zusatz: String,
    #[serde(default)]
    pub strasse: String,
    #[serde(default)]
    pub postfach: String,
    #[serde(default)]
    pub plz: String,
    #[serde(default)]
    pub ort: String,
    #[serde(default)]
    pub land: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub telefon: String,
    #[serde(default)]
    pub mobiltelefon: String,
    #[serde(default)]
    pub webseite: String,
    #[serde(default)]
    pub notiz: String,
}

impl AddressInput {
    /// Leeres Formular für eine neue Adresse.
    pub fn new_default() -> Self {
        Self {
            typ: "firma".into(),
            land: COUNTRIES[0].0.into(),
            ..Default::default()
        }
    }

    /// Entfernt Leerzeichen am Anfang/Ende und leert bei Personen den Firmennamen.
    pub fn normalized(&self) -> Self {
        let mut n = Self {
            typ: self.typ.trim().to_lowercase(),
            anrede: self.anrede.trim().to_lowercase(),
            firmenname: self.firmenname.trim().into(),
            vorname: self.vorname.trim().into(),
            nachname: self.nachname.trim().into(),
            zusatz: self.zusatz.trim().into(),
            strasse: self.strasse.trim().into(),
            postfach: self.postfach.trim().into(),
            plz: self.plz.trim().into(),
            ort: self.ort.trim().into(),
            land: self.land.trim().to_uppercase(),
            email: self.email.trim().into(),
            telefon: self.telefon.trim().into(),
            mobiltelefon: self.mobiltelefon.trim().into(),
            webseite: self.webseite.trim().into(),
            notiz: self.notiz.trim().into(),
        };
        if n.typ == "person" {
            n.firmenname.clear();
        }
        n
    }

    /// Prüft die (normalisierte) Eingabe (AK-9, AK-11). Leere Map = gültig.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |field: &str, msg: &str| {
            errors.entry(field.into()).or_insert_with(|| msg.into());
        };

        match self.typ.as_str() {
            "firma" => {
                if self.firmenname.is_empty() {
                    err("firmenname", "Bitte einen Firmennamen eingeben.");
                }
            }
            "person" => {
                if self.vorname.is_empty() {
                    err("vorname", "Bitte einen Vornamen eingeben.");
                }
                if self.nachname.is_empty() {
                    err("nachname", "Bitte einen Nachnamen eingeben.");
                }
            }
            _ => err("typ", "Bitte „Firma“ oder „Person“ wählen."),
        }
        if !matches!(self.anrede.as_str(), "" | "herr" | "frau") {
            err("anrede", "Ungültige Anrede.");
        }
        if !COUNTRIES.iter().any(|(c, _)| *c == self.land) {
            err("land", "Bitte ein Land aus der Liste wählen.");
        }
        if !self.email.is_empty() && !is_valid_email(&self.email) {
            err(
                "email",
                "Die E-Mail-Adresse hat kein gültiges Format (z. B. name@firma.ch).",
            );
        }

        for (field, value, max) in [
            ("firmenname", &self.firmenname, 200),
            ("vorname", &self.vorname, 100),
            ("nachname", &self.nachname, 100),
            ("zusatz", &self.zusatz, 200),
            ("strasse", &self.strasse, 200),
            ("postfach", &self.postfach, 50),
            ("plz", &self.plz, 20),
            ("ort", &self.ort, 100),
            ("email", &self.email, 254),
            ("telefon", &self.telefon, 50),
            ("mobiltelefon", &self.mobiltelefon, 50),
            ("webseite", &self.webseite, 200),
        ] {
            if value.chars().count() > max {
                err(field, &format!("Höchstens {max} Zeichen."));
            }
        }
        errors
    }
}

/// Einfache Formatprüfung: genau ein @, Teil davor nicht leer, Domäne mit Punkt, keine Leerzeichen.
pub fn is_valid_email(email: &str) -> bool {
    let Some((local, domain)) = email.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && !domain.contains('@')
        && !email.chars().any(char::is_whitespace)
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains("..")
}

/// Vollständige Adresse für Detailansicht und Bearbeiten.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Address {
    pub id: i64,
    pub nummer: String,
    pub anzeigename: String,
    pub typ: String,
    pub anrede: Option<String>,
    pub firmenname: Option<String>,
    pub vorname: Option<String>,
    pub nachname: Option<String>,
    pub zusatz: Option<String>,
    pub strasse: Option<String>,
    pub postfach: Option<String>,
    pub plz: Option<String>,
    pub ort: Option<String>,
    pub land: String,
    pub email: Option<String>,
    pub telefon: Option<String>,
    pub mobiltelefon: Option<String>,
    pub webseite: Option<String>,
    pub notiz: Option<String>,
    /// Benutzer, dem die Adresse gehört (ANF-005 AK-8)
    pub benutzer_id: Option<i64>,
    pub benutzername: Option<String>,
    /// Bereits formatiert (TT.MM.JJJJ HH:MM, Zeitzone Europe/Zurich)
    pub erstellt_am: String,
    pub geaendert_am: String,
}

impl Address {
    pub fn to_input(&self) -> AddressInput {
        let s = |v: &Option<String>| v.clone().unwrap_or_default();
        AddressInput {
            typ: self.typ.clone(),
            anrede: s(&self.anrede),
            firmenname: s(&self.firmenname),
            vorname: s(&self.vorname),
            nachname: s(&self.nachname),
            zusatz: s(&self.zusatz),
            strasse: s(&self.strasse),
            postfach: s(&self.postfach),
            plz: s(&self.plz),
            ort: s(&self.ort),
            land: self.land.clone(),
            email: s(&self.email),
            telefon: s(&self.telefon),
            mobiltelefon: s(&self.mobiltelefon),
            webseite: s(&self.webseite),
            notiz: s(&self.notiz),
        }
    }
}

/// Zeile der Adressliste.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AddressRow {
    pub id: i64,
    pub nummer: String,
    pub anzeigename: String,
    pub plz: Option<String>,
    pub ort: Option<String>,
    pub land: String,
    pub email: Option<String>,
    pub telefon: Option<String>,
}

/// Eine Seite der Adressliste.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddressPage {
    pub rows: Vec<AddressRow>,
    pub total: i64,
    pub page: i64,
}

impl AddressPage {
    pub fn page_count(&self) -> i64 {
        ((self.total + PAGE_SIZE - 1) / PAGE_SIZE).max(1)
    }
}

/// Sortierbare Spalten der Liste (AK-3): Schlüssel in der Webadresse → Beschriftung.
pub const SORT_COLUMNS: &[(&str, &str)] = &[
    ("nummer", "Nummer"),
    ("name", "Name"),
    ("plz", "PLZ"),
    ("ort", "Ort"),
    ("land", "Land"),
    ("email", "E-Mail"),
    ("telefon", "Telefon"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn firma() -> AddressInput {
        AddressInput {
            firmenname: "Muster AG".into(),
            ..AddressInput::new_default()
        }
    }

    #[test]
    fn valid_company() {
        assert!(firma().normalized().validate().is_empty());
    }

    #[test]
    fn company_requires_name() {
        let e = AddressInput::new_default().normalized().validate();
        assert!(e.contains_key("firmenname"));
    }

    #[test]
    fn person_requires_first_and_last_name() {
        let input = AddressInput {
            typ: "person".into(),
            vorname: "  ".into(),
            ..AddressInput::new_default()
        };
        let e = input.normalized().validate();
        assert!(e.contains_key("vorname"));
        assert!(e.contains_key("nachname"));
        assert!(!e.contains_key("firmenname"));
    }

    #[test]
    fn person_clears_company_name() {
        let input = AddressInput {
            typ: "person".into(),
            firmenname: "X".into(),
            ..AddressInput::new_default()
        };
        assert_eq!(input.normalized().firmenname, "");
    }

    #[test]
    fn email_format() {
        assert!(is_valid_email("anna@muster.ch"));
        assert!(is_valid_email("a.b+c@sub.muster.ch"));
        for bad in [
            "anna",
            "anna@",
            "@muster.ch",
            "anna@muster",
            "an na@muster.ch",
            "a@b@c.ch",
            "a@.ch",
            "a@b..ch",
        ] {
            assert!(!is_valid_email(bad), "{bad}");
        }
        let mut f = firma();
        f.email = "falsch".into();
        assert!(f.normalized().validate().contains_key("email"));
    }

    #[test]
    fn country_must_be_in_list() {
        let mut f = firma();
        f.land = "xx".into();
        assert!(f.normalized().validate().contains_key("land"));
        f.land = "de".into();
        assert!(f.normalized().validate().is_empty());
    }

    #[test]
    fn max_length() {
        let mut f = firma();
        f.plz = "1".repeat(21);
        assert!(f.normalized().validate().contains_key("plz"));
    }

    #[test]
    fn page_count() {
        let p = |total| {
            AddressPage {
                rows: vec![],
                total,
                page: 1,
            }
            .page_count()
        };
        assert_eq!(p(0), 1);
        assert_eq!(p(50), 1);
        assert_eq!(p(51), 2);
    }
}
