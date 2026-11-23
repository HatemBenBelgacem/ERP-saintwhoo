//! Datenmodell und Prüfregeln der eigenen Firma (ANF-025).
//! Wird auf dem Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

use crate::backend::addresses::model::{AddressInput, country_name};
pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

pub const REGISTER_MAX: usize = 100;
pub const FOOTER_MAX: usize = 300;
/// Höchstgrösse des Logos in Bytes (AK-7).
pub const LOGO_MAX_BYTES: usize = 1024 * 1024;
/// Erlaubte Dateitypen des Logos (AK-7).
pub const LOGO_TYPES: &[&str] = &["image/png", "image/jpeg", "image/svg+xml"];
/// Webadresse, unter der der Server das Logo ausliefert.
pub const LOGO_URL: &str = "/firma/logo";

/// Gewichte der Prüfziffer nach Modulo 11 für die ersten 8 Ziffern der UID.
const UID_WEIGHTS: [u32; 8] = [5, 4, 3, 2, 7, 6, 5, 4];

/// Bringt eine UID in die Form `CHE-123.456.789`; erlaubt Eingaben wie `che123456789`
/// oder `CHE-123.456.789 MWST`. `None`, wenn es nicht genau 9 Ziffern nach „CHE“ sind.
pub fn normalize_uid(input: &str) -> Option<String> {
    let compact: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, '-' | '.'))
        .collect::<String>()
        .to_uppercase();
    let compact = compact.strip_suffix("MWST").unwrap_or(&compact);
    let digits = compact.strip_prefix("CHE")?;
    if digits.len() != 9 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(format!(
        "CHE-{}.{}.{}",
        &digits[0..3],
        &digits[3..6],
        &digits[6..9]
    ))
}

/// Prüfziffer (9. Ziffer) einer UID korrekt? Erwartet die Form `CHE-123.456.789`.
pub fn uid_checksum_ok(uid: &str) -> bool {
    let digits: Vec<u32> = uid.chars().filter_map(|c| c.to_digit(10)).collect();
    if digits.len() != 9 {
        return false;
    }
    let sum: u32 = digits[..8]
        .iter()
        .zip(UID_WEIGHTS)
        .map(|(d, w)| d * w)
        .sum();
    let check = match 11 - sum % 11 {
        11 => 0,
        10 => return false,
        c => c,
    };
    digits[8] == check
}

/// MWST-Nummer aus der UID (AK-6): `CHE-123.456.789 MWST`.
pub fn vat_number(uid: &str) -> String {
    format!("{uid} MWST")
}

/// Fehlende Teile der Postanschrift einer Adresse (AK-8). Leer = vollständig.
pub fn missing_postal_parts(
    strasse: Option<&str>,
    plz: Option<&str>,
    ort: Option<&str>,
) -> Vec<&'static str> {
    let blank = |v: Option<&str>| v.is_none_or(|s| s.trim().is_empty());
    [("Strasse", strasse), ("PLZ", plz), ("Ort", ort)]
        .into_iter()
        .filter(|(_, v)| blank(*v))
        .map(|(name, _)| name)
        .collect()
}

/// Meldung, wenn eine Adresse nicht als Firmenadresse taugt (AK-8), sonst `None`.
pub fn company_address_error(
    typ: &str,
    strasse: Option<&str>,
    plz: Option<&str>,
    ort: Option<&str>,
) -> Option<String> {
    if typ != "firma" {
        return Some("Bitte eine Adresse vom Typ „Firma“ wählen.".into());
    }
    let missing = missing_postal_parts(strasse, plz, ort);
    (!missing.is_empty()).then(|| {
        format!(
            "Die Postanschrift ist unvollständig (fehlt: {}). Bitte zuerst die Adresse ergänzen.",
            missing.join(", ")
        )
    })
}

/// Neue Firmenadresse vorbereiten (AK-4, AK-8; auch ANF-026 „Mandant definieren“): Typ „Firma“, normalisiert
/// und geprüft (ANF-002); Strasse, PLZ und Ort sind für den Absender der Belege Pflicht.
/// Liefert die Adresse und ihre Fehler (leer = gültig).
pub fn prepare_company_address(address: AddressInput) -> (AddressInput, FieldErrors) {
    let address = AddressInput {
        typ: "firma".into(),
        ..address
    }
    .normalized();
    let mut errors = address.validate();
    for (field, value) in [
        ("strasse", &address.strasse),
        ("plz", &address.plz),
        ("ort", &address.ort),
    ] {
        if value.is_empty() {
            errors
                .entry(field.into())
                .or_insert_with(|| "Für den Absender der Belege nötig.".into());
        }
    }
    (address, errors)
}

/// Dateityp eines Logos anhand des Inhalts (nicht der Dateiendung). `None` = nicht erlaubt.
pub fn detect_logo_type(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        return Some("image/png");
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("image/jpeg");
    }
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).to_lowercase();
    let head = head.trim_start_matches('\u{feff}').trim_start();
    (head.starts_with("<svg") || (head.starts_with("<?xml") && head.contains("<svg")))
        .then_some("image/svg+xml")
}

/// Formulareingabe der Firma (AK-4, AK-5). Die Adresse selbst wird im Adressstamm gepflegt.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CompanyInput {
    /// „bestehend“ = Adresse wählen, „neu“ = Adresse im selben Schritt anlegen (nur beim Erfassen)
    pub modus: String,
    pub adresse_id: String,
    pub uid: String,
    pub handelsregister: String,
    pub fusszeile: String,
}

impl CompanyInput {
    pub fn normalized(&self) -> Self {
        let uid = self.uid.trim();
        Self {
            modus: if self.modus.trim() == "neu" {
                "neu".into()
            } else {
                "bestehend".into()
            },
            adresse_id: self.adresse_id.trim().into(),
            uid: normalize_uid(uid).unwrap_or_else(|| uid.into()),
            handelsregister: self.handelsregister.trim().into(),
            fusszeile: self.fusszeile.trim().into(),
        }
    }

    pub fn creates_address(&self) -> bool {
        self.modus == "neu"
    }

    /// Prüft die normalisierte Eingabe (AK-6). Leere Map = gültig.
    /// Typ und Vollständigkeit der Adresse prüft der Server zusätzlich (AK-8).
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        if !self.creates_address() {
            if self.adresse_id.is_empty() {
                errors.insert("adresse_id".into(), "Bitte eine Adresse wählen.".into());
            } else if self.address_id().is_none() {
                errors.insert("adresse_id".into(), "Ungültige Adresse.".into());
            }
        }
        if !self.uid.is_empty() {
            if normalize_uid(&self.uid).is_none() {
                errors.insert(
                    "uid".into(),
                    "Format: CHE-123.456.789 (CHE und 9 Ziffern).".into(),
                );
            } else if !uid_checksum_ok(&self.uid) {
                errors.insert(
                    "uid".into(),
                    "Die Prüfziffer stimmt nicht – bitte die UID kontrollieren.".into(),
                );
            }
        }
        if self.handelsregister.chars().count() > REGISTER_MAX {
            errors.insert(
                "handelsregister".into(),
                format!("Höchstens {REGISTER_MAX} Zeichen."),
            );
        }
        if self.fusszeile.chars().count() > FOOTER_MAX {
            errors.insert(
                "fusszeile".into(),
                format!("Höchstens {FOOTER_MAX} Zeichen."),
            );
        }
        errors
    }

    pub fn address_id(&self) -> Option<i64> {
        self.adresse_id.parse().ok()
    }

    pub fn uid_value(&self) -> Option<&str> {
        (!self.uid.is_empty()).then_some(self.uid.as_str())
    }

    pub fn register(&self) -> Option<&str> {
        (!self.handelsregister.is_empty()).then_some(self.handelsregister.as_str())
    }

    pub fn footer(&self) -> Option<&str> {
        (!self.fusszeile.is_empty()).then_some(self.fusszeile.as_str())
    }
}

/// Die eigene Firma mit den Angaben ihrer Adresse (AK-2). Absender aller Belege (AK-9).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Company {
    pub id: i64,
    pub uid: Option<String>,
    pub handelsregister: Option<String>,
    pub fusszeile: Option<String>,
    /// Zeitpunkt der letzten Änderung (Millisekunden) – macht die Logo-Webadresse eindeutig;
    /// `None`, wenn kein Logo hinterlegt ist
    pub logo_version: Option<i64>,
    pub adresse_id: i64,
    pub adresse_nummer: String,
    pub adresse_typ: String,
    pub name: String,
    pub zusatz: Option<String>,
    pub strasse: Option<String>,
    pub postfach: Option<String>,
    pub plz: Option<String>,
    pub ort: Option<String>,
    pub land: String,
    pub telefon: Option<String>,
    pub email: Option<String>,
    pub webseite: Option<String>,
}

impl Company {
    /// Webadresse des Logos oder `None` ohne Logo.
    pub fn logo_url(&self) -> Option<String> {
        self.logo_version.map(|v| format!("{LOGO_URL}?v={v}"))
    }

    /// Anschriftzeilen für den Absenderblock (AK-3): Name, Zusatz, Strasse, Postfach,
    /// PLZ Ort, Land (ausser Schweiz).
    pub fn address_lines(&self) -> Vec<String> {
        let mut lines = vec![self.name.clone()];
        for v in [&self.zusatz, &self.strasse, &self.postfach]
            .into_iter()
            .flatten()
        {
            if !v.trim().is_empty() {
                lines.push(v.clone());
            }
        }
        let place = [self.plz.as_deref(), self.ort.as_deref()]
            .into_iter()
            .flatten()
            .filter(|s| !s.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !place.is_empty() {
            lines.push(place);
        }
        if self.land != "CH" {
            lines.push(country_name(&self.land).to_string());
        }
        lines
    }

    /// Fehlerhinweis, wenn die verknüpfte Adresse nachträglich unvollständig wurde (AK-8).
    pub fn address_problem(&self) -> Option<String> {
        company_address_error(
            &self.adresse_typ,
            self.strasse.as_deref(),
            self.plz.as_deref(),
            self.ort.as_deref(),
        )
    }

    pub fn to_input(&self) -> CompanyInput {
        CompanyInput {
            modus: "bestehend".into(),
            adresse_id: self.adresse_id.to_string(),
            uid: self.uid.clone().unwrap_or_default(),
            handelsregister: self.handelsregister.clone().unwrap_or_default(),
            fusszeile: self.fusszeile.clone().unwrap_or_default(),
        }
    }

    pub fn address_option(&self) -> PickOption {
        PickOption {
            id: self.adresse_id,
            nummer: self.adresse_nummer.clone(),
            name: self.name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uid_format_and_checksum() {
        assert_eq!(
            normalize_uid(" che 116 281 710 ").as_deref(),
            Some("CHE-116.281.710")
        );
        assert_eq!(
            normalize_uid("CHE-123.456.788 MWST").as_deref(),
            Some("CHE-123.456.788")
        );
        assert_eq!(normalize_uid("CHE-123.456.78"), None);
        assert_eq!(normalize_uid("DE-123.456.789"), None);
        assert!(uid_checksum_ok("CHE-116.281.710"));
        assert!(uid_checksum_ok("CHE-123.456.788"));
        assert!(uid_checksum_ok("CHE-100.000.006"));
        assert!(!uid_checksum_ok("CHE-123.456.789"));
        assert_eq!(vat_number("CHE-116.281.710"), "CHE-116.281.710 MWST");
    }

    #[test]
    fn input_validation() {
        let ok = CompanyInput {
            adresse_id: " 7 ".into(),
            uid: "che116281710".into(),
            ..Default::default()
        }
        .normalized();
        assert_eq!(ok.modus, "bestehend");
        assert_eq!(ok.uid, "CHE-116.281.710");
        assert!(ok.validate().is_empty(), "{:?}", ok.validate());

        let bad = CompanyInput {
            uid: "CHE-123.456.789".into(),
            fusszeile: "x".repeat(FOOTER_MAX + 1),
            ..Default::default()
        }
        .normalized();
        let errors = bad.validate();
        for f in ["adresse_id", "uid", "fusszeile"] {
            assert!(errors.contains_key(f), "{f}");
        }

        // Neue Adresse: keine Adresse wählen nötig
        let new = CompanyInput {
            modus: "neu".into(),
            ..Default::default()
        }
        .normalized();
        assert!(new.creates_address());
        assert!(new.validate().is_empty());
    }

    #[test]
    fn address_rules() {
        assert!(
            company_address_error("firma", Some("Weg 1"), Some("8000"), Some("Zürich")).is_none()
        );
        assert!(
            company_address_error("person", Some("Weg 1"), Some("8000"), Some("Zürich")).is_some()
        );
        let m = company_address_error("firma", Some(" "), None, Some("Zürich")).unwrap();
        assert!(m.contains("Strasse, PLZ"), "{m}");
    }

    #[test]
    fn logo_types() {
        assert_eq!(
            detect_logo_type(b"\x89PNG\r\n\x1a\nrest"),
            Some("image/png")
        );
        assert_eq!(
            detect_logo_type(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some("image/jpeg")
        );
        assert_eq!(
            detect_logo_type(b"  <svg xmlns=\"x\"></svg>"),
            Some("image/svg+xml")
        );
        assert_eq!(
            detect_logo_type(b"<?xml version=\"1.0\"?>\n<svg></svg>"),
            Some("image/svg+xml")
        );
        assert_eq!(detect_logo_type(b"GIF89a"), None);
        assert_eq!(detect_logo_type(b"<html></html>"), None);
    }

    #[test]
    fn sender_lines() {
        let c = Company {
            id: 1,
            uid: None,
            handelsregister: None,
            fusszeile: None,
            logo_version: None,
            adresse_id: 3,
            adresse_nummer: "A-0003".into(),
            adresse_typ: "firma".into(),
            name: "Uniqwork AG".into(),
            zusatz: None,
            strasse: Some("Bahnhofstrasse 1".into()),
            postfach: Some(String::new()),
            plz: Some("8001".into()),
            ort: Some("Zürich".into()),
            land: "CH".into(),
            telefon: None,
            email: None,
            webseite: None,
        };
        assert_eq!(
            c.address_lines(),
            ["Uniqwork AG", "Bahnhofstrasse 1", "8001 Zürich"]
        );
        assert_eq!(c.logo_url(), None);
        let de = Company {
            land: "DE".into(),
            logo_version: Some(5),
            ..c
        };
        assert_eq!(de.address_lines().last().unwrap(), "Deutschland");
        assert_eq!(de.logo_url().as_deref(), Some("/firma/logo?v=5"));
    }

    #[test]
    fn company_address_needs_postal_parts() {
        let (address, errors) = prepare_company_address(AddressInput {
            typ: "person".into(),
            firmenname: " Firma AG ".into(),
            land: "CH".into(),
            ..Default::default()
        });
        assert_eq!(address.typ, "firma");
        assert_eq!(address.firmenname, "Firma AG");
        for f in ["strasse", "plz", "ort"] {
            assert_eq!(errors[f], "Für den Absender der Belege nötig.", "{f}");
        }
        let (_, errors) = prepare_company_address(AddressInput {
            firmenname: "Firma AG".into(),
            strasse: "Bahnhofstrasse 1".into(),
            plz: "8001".into(),
            ort: "Zürich".into(),
            land: "CH".into(),
            ..Default::default()
        });
        assert!(errors.is_empty(), "{errors:?}");
    }
}
