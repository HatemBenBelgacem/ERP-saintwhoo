//! Datenmodell und Prüfregeln für Zahlungsbedingungen (ANF-016). Wird auf dem Server und im Browser verwendet.
//!
//! Skonto-Prozente in Hundertstel (2.5 % = 250). Die Fälligkeit wird erst beim Buchen von Rechnungen
//! berechnet (ANF-018); hier gibt es nur die Vorgaben.

use serde::{Deserialize, Serialize};

use crate::backend::finance::bank_account::model::flag;
use crate::backend::purchase_orders::model::{format_trimmed, parse_fixed};
pub use crate::forms::{FieldErrors, SaveOutcome};

pub const NAME_MAX: usize = 100;
/// Länge des Rechnungstextes; er wird auf die Rechnung gedruckt (AK-1).
pub const TEXT_MAX: usize = 250;
/// Höchste Zahlungsfrist in Tagen (AK-2).
pub const DAYS_MAX: i64 = 365;

/// Skonto als (Skontofrist in Tagen, Skonto in Hundertstel Prozent).
pub type Skonto = Option<(i64, i64)>;

/// Vorschlag für den Rechnungstext, z. B. „Zahlbar innert 10 Tagen mit 2 % Skonto oder innert 30 Tagen netto“ (AK-3).
pub fn suggest_text(frist: i64, skonto: Skonto) -> String {
    match skonto {
        Some((0, prozent)) => format!(
            "Zahlbar sofort mit {} % Skonto oder innert {frist} Tagen netto",
            format_trimmed(prozent, 2)
        ),
        Some((skontofrist, prozent)) => format!(
            "Zahlbar innert {skontofrist} Tagen mit {} % Skonto oder innert {frist} Tagen netto",
            format_trimmed(prozent, 2)
        ),
        None if frist == 0 => "Zahlbar sofort netto".into(),
        None => format!("Zahlbar innert {frist} Tagen netto"),
    }
}

/// Kurzform für Listen: „10 Tage 2 % Skonto, 30 Tage netto“ bzw. „30 Tage netto“.
pub fn summary(frist: i64, skonto: Skonto) -> String {
    match skonto {
        Some((skontofrist, prozent)) => format!(
            "{skontofrist} Tage {} % Skonto, {frist} Tage netto",
            format_trimmed(prozent, 2)
        ),
        None if frist == 0 => "Sofort netto".into(),
        None => format!("{frist} Tage netto"),
    }
}

/// Eine Zahlungsbedingung aus der Datenbank (AK-1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PaymentTerm {
    pub id: i64,
    pub bezeichnung: String,
    pub zahlungsfrist_tage: i64,
    pub skontofrist_tage: Option<i64>,
    /// Hundertstel Prozent
    pub skonto_prozent: Option<i64>,
    pub rechnungstext: String,
    pub standard: bool,
    pub aktiv: bool,
    /// Von mindestens einer Adresse als Vorgabe gewählt (AK-7).
    pub verwendet: bool,
}

impl PaymentTerm {
    pub fn skonto(&self) -> Skonto {
        match (self.skontofrist_tage, self.skonto_prozent) {
            (Some(frist), Some(prozent)) => Some((frist, prozent)),
            _ => None,
        }
    }

    pub fn summary(&self) -> String {
        summary(self.zahlungsfrist_tage, self.skonto())
    }

    pub fn to_input(&self) -> PaymentTermInput {
        PaymentTermInput {
            bezeichnung: self.bezeichnung.clone(),
            zahlungsfrist: self.zahlungsfrist_tage.to_string(),
            skontofrist: self
                .skontofrist_tage
                .map(|v| v.to_string())
                .unwrap_or_default(),
            skonto: self
                .skonto_prozent
                .map(|v| format_trimmed(v, 2))
                .unwrap_or_default(),
            rechnungstext: self.rechnungstext.clone(),
            standard: flag(self.standard),
            aktiv: flag(self.aktiv),
        }
    }
}

/// Formulareingabe für eine Zahlungsbedingung. Alle Felder als Text, wie sie aus dem Formular kommen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaymentTermInput {
    pub bezeichnung: String,
    pub zahlungsfrist: String,
    pub skontofrist: String,
    pub skonto: String,
    pub rechnungstext: String,
    pub standard: String,
    pub aktiv: String,
}

impl PaymentTermInput {
    /// Vorgabe: 30 Tage netto, aktiv, mit dem passenden Rechnungstext.
    pub fn new_default() -> Self {
        Self {
            zahlungsfrist: "30".into(),
            rechnungstext: suggest_text(30, None),
            aktiv: flag(true),
            ..Default::default()
        }
    }

    /// Leerzeichen entfernen; Dezimalkomma im Skonto wird zum Punkt.
    pub fn normalized(&self) -> Self {
        Self {
            bezeichnung: self.bezeichnung.trim().into(),
            zahlungsfrist: self.zahlungsfrist.trim().into(),
            skontofrist: self.skontofrist.trim().into(),
            skonto: self.skonto.trim().replace(',', "."),
            rechnungstext: self.rechnungstext.trim().into(),
            standard: self.standard.trim().into(),
            aktiv: self.aktiv.trim().into(),
        }
    }

    pub fn is_standard(&self) -> bool {
        !self.standard.is_empty()
    }

    pub fn is_active(&self) -> bool {
        !self.aktiv.is_empty()
    }

    /// Zahlungsfrist und Skonto aus der Eingabe; `None`, wenn etwas fehlt oder ungültig ist.
    pub fn terms(&self) -> Option<(i64, Skonto)> {
        let frist = self.zahlungsfrist.parse::<i64>().ok()?;
        if self.skontofrist.is_empty() && self.skonto.is_empty() {
            return Some((frist, None));
        }
        let skontofrist = self.skontofrist.parse::<i64>().ok()?;
        let prozent = parse_fixed(&self.skonto, 2)?;
        Some((frist, Some((skontofrist, prozent))))
    }

    /// Prüft die (normalisierte) Eingabe (AK-2). Leere Map = gültig.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |field: &str, msg: &str| {
            errors.entry(field.into()).or_insert_with(|| msg.into());
        };

        if self.bezeichnung.is_empty() {
            err("bezeichnung", "Bitte eine Bezeichnung eingeben.");
        } else if self.bezeichnung.chars().count() > NAME_MAX {
            err("bezeichnung", "Höchstens 100 Zeichen.");
        }

        let frist = match self.zahlungsfrist.parse::<i64>() {
            Ok(f) if (0..=DAYS_MAX).contains(&f) => Some(f),
            _ => {
                err(
                    "zahlungsfrist",
                    "Bitte eine ganze Zahl von 0 bis 365 eingeben.",
                );
                None
            }
        };

        match (self.skontofrist.is_empty(), self.skonto.is_empty()) {
            (true, true) => {}
            (false, false) => {
                match self.skontofrist.parse::<i64>() {
                    Ok(tage) if tage >= 0 && frist.is_none_or(|f| tage < f) => {}
                    _ => err(
                        "skontofrist",
                        "Bitte eine ganze Zahl eingeben, kleiner als die Zahlungsfrist.",
                    ),
                }
                match parse_fixed(&self.skonto, 2) {
                    Some(prozent) if prozent > 0 && prozent < 10_000 => {}
                    _ => err(
                        "skonto",
                        "Skonto über 0 und unter 100 %, höchstens 2 Nachkommastellen.",
                    ),
                }
            }
            (true, false) => err("skontofrist", "Skontofrist und Skonto % gemeinsam angeben."),
            (false, true) => err("skonto", "Skontofrist und Skonto % gemeinsam angeben."),
        }

        if self.rechnungstext.is_empty() {
            err("rechnungstext", "Bitte einen Rechnungstext eingeben.");
        } else if self.rechnungstext.chars().count() > TEXT_MAX {
            err("rechnungstext", "Höchstens 250 Zeichen.");
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(frist: &str, skontofrist: &str, skonto: &str) -> PaymentTermInput {
        PaymentTermInput {
            bezeichnung: "Test".into(),
            zahlungsfrist: frist.into(),
            skontofrist: skontofrist.into(),
            skonto: skonto.into(),
            rechnungstext: "Zahlbar".into(),
            ..Default::default()
        }
        .normalized()
    }

    #[test]
    fn suggestion_matches_example() {
        assert_eq!(
            suggest_text(30, Some((10, 200))),
            "Zahlbar innert 10 Tagen mit 2 % Skonto oder innert 30 Tagen netto"
        );
        assert_eq!(suggest_text(30, None), "Zahlbar innert 30 Tagen netto");
        assert_eq!(suggest_text(0, None), "Zahlbar sofort netto");
        assert_eq!(
            suggest_text(30, Some((10, 250))),
            "Zahlbar innert 10 Tagen mit 2.5 % Skonto oder innert 30 Tagen netto"
        );
    }

    #[test]
    fn summary_for_lists() {
        assert_eq!(summary(30, None), "30 Tage netto");
        assert_eq!(
            summary(30, Some((10, 200))),
            "10 Tage 2 % Skonto, 30 Tage netto"
        );
    }

    #[test]
    fn skonto_pair_is_all_or_nothing() {
        assert!(input("30", "", "").validate().is_empty());
        assert!(input("30", "10", "2").validate().is_empty());
        assert!(input("30", "10", "").validate().contains_key("skonto"));
        assert!(input("30", "", "2").validate().contains_key("skontofrist"));
    }

    #[test]
    fn skontofrist_must_be_shorter_than_zahlungsfrist() {
        assert!(
            input("30", "30", "2")
                .validate()
                .contains_key("skontofrist")
        );
        assert!(input("30", "29", "2").validate().is_empty());
    }

    #[test]
    fn skonto_between_zero_and_hundred() {
        assert!(input("30", "10", "0").validate().contains_key("skonto"));
        assert!(input("30", "10", "100").validate().contains_key("skonto"));
        assert!(input("30", "10", "2.123").validate().contains_key("skonto"));
        assert!(input("30", "10", "2,5").validate().is_empty());
    }

    #[test]
    fn days_range() {
        assert!(input("365", "", "").validate().is_empty());
        assert!(
            input("366", "", "")
                .validate()
                .contains_key("zahlungsfrist")
        );
        assert!(input("-1", "", "").validate().contains_key("zahlungsfrist"));
        assert!(
            input("zehn", "", "")
                .validate()
                .contains_key("zahlungsfrist")
        );
    }

    #[test]
    fn text_is_required() {
        let mut i = input("30", "", "");
        i.rechnungstext.clear();
        assert!(i.validate().contains_key("rechnungstext"));
    }
}
