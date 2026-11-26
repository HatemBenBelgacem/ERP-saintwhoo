//! Datenmodell und Prüfregeln für Währungen und Umrechnungskurse (ANF-014). Wird auf dem Server und im Browser verwendet.
//!
//! Kurse in Millionstel (0.96 CHF für 1 EUR = 960000), Rundung in Hundertstel (0.05 = 5).

use serde::{Deserialize, Serialize};

use crate::backend::finance::bank_account::model::flag;
use crate::backend::projects::model::is_valid_date;
use crate::backend::purchase_orders::model::{format_fixed, format_trimmed, parse_fixed};
pub use crate::forms::{FieldErrors, SaveOutcome};

pub const NAME_MAX: usize = 50;
/// Buchhaltungswährung: Alle Beträge im Journal sind in Franken (AK-1).
pub const BOOKING_CURRENCY: &str = "CHF";
/// Zulässige Rundungen (AK-2).
pub const ROUNDINGS: &[(&str, &str)] = &[("0.01", "0.01"), ("0.05", "0.05 (Rappenrundung)")];
/// Kurse haben bis zu sechs Nachkommastellen (AK-3).
pub const RATE_DECIMALS: u32 = 6;

/// `true`, wenn der Text genau drei Grossbuchstaben hat (ISO 4217, z. B. „EUR“).
pub fn is_code(code: &str) -> bool {
    code.len() == 3 && code.chars().all(|c| c.is_ascii_uppercase())
}

/// Kurs in Millionstel als Text, z. B. 960000 → „0.96“.
pub fn rate_label(kurs: i64) -> String {
    format_trimmed(kurs, RATE_DECIMALS)
}

/// Eine Währung aus der Datenbank mit dem letzten Kurs (AK-1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Currency {
    pub code: String,
    pub bezeichnung: String,
    /// Hundertstel: 1 (0.01) oder 5 (0.05)
    pub rundung: i64,
    pub buchhaltungswaehrung: bool,
    pub aktiv: bool,
    /// Letzter Kurs in Millionstel CHF
    pub letzter_kurs: Option<i64>,
    /// Datum des letzten Kurses (`JJJJ-MM-TT`)
    pub letzter_kurs_datum: Option<String>,
    /// Wird in Einkaufsaufträgen verwendet (AK-5).
    pub verwendet: bool,
}

impl Currency {
    pub fn rundung_label(&self) -> String {
        format_fixed(self.rundung, 2)
    }

    pub fn to_input(&self) -> CurrencyInput {
        CurrencyInput {
            code: self.code.clone(),
            bezeichnung: self.bezeichnung.clone(),
            rundung: self.rundung_label(),
            aktiv: flag(self.aktiv),
        }
    }
}

/// Formulareingabe für eine Währung. Der Code wird nur beim Anlegen eingegeben.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CurrencyInput {
    pub code: String,
    pub bezeichnung: String,
    pub rundung: String,
    pub aktiv: String,
}

impl CurrencyInput {
    /// Vorgabe: Rundung 0.01, aktiv.
    pub fn new_default() -> Self {
        Self {
            rundung: "0.01".into(),
            aktiv: flag(true),
            ..Default::default()
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            code: self.code.trim().to_uppercase(),
            bezeichnung: self.bezeichnung.trim().into(),
            rundung: self.rundung.trim().into(),
            aktiv: self.aktiv.trim().into(),
        }
    }

    pub fn is_active(&self) -> bool {
        !self.aktiv.is_empty()
    }

    /// Rundung in Hundertstel, nur für 0.01 und 0.05.
    pub fn rundung_value(&self) -> Option<i64> {
        match self.rundung.as_str() {
            "0.01" => Some(1),
            "0.05" => Some(5),
            _ => None,
        }
    }

    /// Prüft die (normalisierte) Eingabe (AK-2). Der Code wird nur beim Anlegen geprüft.
    pub fn validate(&self, neu: bool) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |field: &str, msg: &str| {
            errors.entry(field.into()).or_insert_with(|| msg.into());
        };

        if neu && !is_code(&self.code) {
            err("code", "Bitte drei Grossbuchstaben eingeben, z. B. EUR.");
        }
        if self.bezeichnung.is_empty() {
            err("bezeichnung", "Bitte eine Bezeichnung eingeben.");
        } else if self.bezeichnung.chars().count() > NAME_MAX {
            err("bezeichnung", "Höchstens 50 Zeichen.");
        }
        if self.rundung_value().is_none() {
            err("rundung", "Bitte 0.01 oder 0.05 wählen.");
        }
        errors
    }
}

/// Formulareingabe für einen Kurs (AK-3).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct RateInput {
    pub datum: String,
    pub kurs: String,
}

impl RateInput {
    /// Kurs in Millionstel, nur wenn er größer als 0 ist und höchstens sechs Nachkommastellen hat.
    pub fn kurs_value(&self) -> Option<i64> {
        parse_fixed(&self.kurs, RATE_DECIMALS).filter(|v| *v > 0)
    }

    /// Prüft Datum und Kurs (AK-3). Leere Map = gültig.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        if !is_valid_date(&self.datum) {
            errors.insert("datum".into(), "Bitte ein gültiges Datum eingeben.".into());
        }
        if self.kurs_value().is_none() {
            errors.insert(
                "kurs".into(),
                "Bitte einen Kurs über 0 eingeben, höchstens 6 Nachkommastellen.".into(),
            );
        }
        errors
    }
}

/// Kurs einer Währung an einem Stichtag (AK-4): der neueste Kurs mit Datum bis zum Belegdatum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExchangeRate {
    pub datum: String,
    /// Millionstel CHF
    pub kurs: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(code: &str, bezeichnung: &str, rundung: &str) -> CurrencyInput {
        CurrencyInput {
            code: code.into(),
            bezeichnung: bezeichnung.into(),
            rundung: rundung.into(),
            aktiv: flag(true),
        }
        .normalized()
    }

    #[test]
    fn code_is_three_capitals() {
        assert!(is_code("EUR"));
        assert!(!is_code("eur"));
        assert!(!is_code("EU"));
        assert!(!is_code("EURO"));
        assert!(!is_code("EU1"));
    }

    #[test]
    fn code_is_uppercased_on_normalize() {
        assert_eq!(input(" eur ", "Euro", "0.01").code, "EUR");
    }

    #[test]
    fn new_currency_needs_code_name_and_rounding() {
        assert!(input("EUR", "Euro", "0.01").validate(true).is_empty());
        assert!(
            input("", "Euro", "0.01")
                .validate(true)
                .contains_key("code")
        );
        assert!(
            input("EUR", "", "0.01")
                .validate(true)
                .contains_key("bezeichnung")
        );
        assert!(
            input("EUR", "Euro", "0.02")
                .validate(true)
                .contains_key("rundung")
        );
    }

    #[test]
    fn code_is_not_checked_on_edit() {
        assert!(input("", "Euro", "0.05").validate(false).is_empty());
    }

    #[test]
    fn rounding_values() {
        assert_eq!(input("EUR", "Euro", "0.01").rundung_value(), Some(1));
        assert_eq!(input("EUR", "Euro", "0.05").rundung_value(), Some(5));
        assert_eq!(input("EUR", "Euro", "0.1").rundung_value(), None);
    }

    #[test]
    fn rate_has_up_to_six_decimals() {
        let rate = |kurs: &str| RateInput {
            datum: "2026-11-25".into(),
            kurs: kurs.into(),
        };
        assert_eq!(rate("0.9234").kurs_value(), Some(923_400));
        assert_eq!(rate("0,9234").kurs_value(), Some(923_400));
        assert_eq!(rate("0.923456").kurs_value(), Some(923_456));
        assert_eq!(rate("0.9234567").kurs_value(), None);
        assert_eq!(rate("0").kurs_value(), None);
        assert_eq!(rate("-1").kurs_value(), None);
        assert!(rate("1.1").validate().is_empty());
        assert!(rate("0").validate().contains_key("kurs"));
    }

    #[test]
    fn rate_date_must_be_valid() {
        let r = RateInput {
            datum: "2026-13-01".into(),
            kurs: "1".into(),
        };
        assert!(r.validate().contains_key("datum"));
    }

    #[test]
    fn rate_label_trims_zeros() {
        assert_eq!(rate_label(960_000), "0.96");
        assert_eq!(rate_label(923_400), "0.9234");
        assert_eq!(rate_label(1_000_000), "1");
    }
}
