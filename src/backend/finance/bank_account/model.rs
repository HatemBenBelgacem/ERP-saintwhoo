//! Datenmodell und Prüfregeln für eigene Bankkonten (ANF-015). Wird auf dem Server und im Browser verwendet.
//!
//! IBAN und QR-IBAN werden ohne Leerzeichen gespeichert und in Vierergruppen angezeigt (AK-3, AK-4).

use serde::{Deserialize, Serialize};

pub use crate::forms::{FieldErrors, SaveOutcome};

pub const NAME_MAX: usize = 100;
pub const BANK_MAX: usize = 100;
/// Höchstlänge einer IBAN ohne Leerzeichen (ISO 13616).
pub const IBAN_MAX: usize = 34;
pub const BIC_MAX: usize = 11;

/// Art eines Bankkontos (AK-1).
pub const KINDS: &[(&str, &str)] = &[("bank", "Bank"), ("post", "Post"), ("kasse", "Kasse")];

const MSG_IBAN: &str = "Die IBAN ist ungültig. Bitte Länderkennung, Länge und Ziffern prüfen.";
const MSG_QR_IBAN: &str = "Die QR-IBAN muss eine Schweizer oder liechtensteiner IBAN mit Institutsnummer 30000 bis 31999 sein.";

pub fn kind_label(key: &str) -> &'static str {
    KINDS
        .iter()
        .find(|(k, _)| *k == key)
        .map_or("–", |(_, l)| l)
}

/// Leerzeichen entfernen und Grossbuchstaben machen (so werden IBAN und BIC gespeichert).
pub fn compact(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

/// Schreibt eine IBAN in Vierergruppen: „CH93 0076 2011 6238 5295 7“.
pub fn group_iban(iban: &str) -> String {
    let chars: Vec<char> = iban.chars().collect();
    chars
        .chunks(4)
        .map(|group| group.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Prüft eine IBAN nach ISO 13616: Länderkennung, Länge und Prüfziffer (Modulo 97).
/// Schweizer und liechtensteiner IBAN haben 21 Zeichen. `None` = gültig.
pub fn iban_error(raw: &str) -> Option<&'static str> {
    let iban = compact(raw);
    let b = iban.as_bytes();
    if b.len() < 15 || b.len() > IBAN_MAX || !b.iter().all(|c| c.is_ascii_alphanumeric()) {
        return Some(MSG_IBAN);
    }
    if !(b[0].is_ascii_uppercase()
        && b[1].is_ascii_uppercase()
        && b[2].is_ascii_digit()
        && b[3].is_ascii_digit())
    {
        return Some(MSG_IBAN);
    }
    if (iban.starts_with("CH") || iban.starts_with("LI")) && b.len() != 21 {
        return Some("Eine IBAN in der Schweiz und in Liechtenstein hat 21 Zeichen.");
    }
    // Modulo 97: die ersten vier Zeichen ans Ende, Buchstaben werden zu Zahlen (A = 10 … Z = 35)
    let rearranged = format!("{}{}", &iban[4..], &iban[..4]);
    let mut rest: u32 = 0;
    for c in rearranged.chars() {
        rest = match c.to_digit(10) {
            Some(d) => (rest * 10 + d) % 97,
            None => (rest * 100 + (c as u32 - 'A' as u32 + 10)) % 97,
        };
    }
    (rest != 1).then_some(MSG_IBAN)
}

/// Prüft eine QR-IBAN: gültige Schweizer oder liechtensteiner IBAN mit Institutsnummer 30000 bis 31999 (AK-4).
pub fn qr_iban_error(raw: &str) -> Option<&'static str> {
    let iban = compact(raw);
    let institut = iban.get(4..9).and_then(|s| s.parse::<u32>().ok());
    let ok = iban_error(&iban).is_none()
        && (iban.starts_with("CH") || iban.starts_with("LI"))
        && institut.is_some_and(|i| (30000..=31999).contains(&i));
    (!ok).then_some(MSG_QR_IBAN)
}

/// BIC: 8 oder 11 Zeichen; die ersten sechs sind Buchstaben, danach Buchstaben oder Ziffern.
pub fn bic_ok(bic: &str) -> bool {
    let b = bic.as_bytes();
    (b.len() == 8 || b.len() == BIC_MAX)
        && b[..6].iter().all(|c| c.is_ascii_uppercase())
        && b[6..].iter().all(|c| c.is_ascii_alphanumeric())
}

/// Ein Bankkonto aus der Datenbank, mit Konto im Kontenplan (AK-1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BankAccount {
    pub id: i64,
    pub bezeichnung: String,
    pub art: String,
    pub bank: Option<String>,
    pub iban: Option<String>,
    pub qr_iban: Option<String>,
    pub bic: Option<String>,
    pub konto_id: i64,
    pub konto_nummer: String,
    pub konto_bezeichnung: String,
    pub standard: bool,
    pub aktiv: bool,
}

impl BankAccount {
    pub fn to_input(&self) -> BankAccountInput {
        let s = |v: &Option<String>| v.clone().unwrap_or_default();
        BankAccountInput {
            bezeichnung: self.bezeichnung.clone(),
            art: self.art.clone(),
            bank: s(&self.bank),
            iban: s(&self.iban),
            qr_iban: s(&self.qr_iban),
            bic: s(&self.bic),
            konto_id: self.konto_id.to_string(),
            standard: flag(self.standard),
            aktiv: flag(self.aktiv),
        }
    }
}

/// Kennzeichen aus Kontrollkästchen: „1“ = angehakt.
pub fn flag(on: bool) -> String {
    if on { "1".into() } else { String::new() }
}

/// Formulareingabe für ein Bankkonto. Alle Felder als Text, wie sie aus dem Formular kommen.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BankAccountInput {
    pub bezeichnung: String,
    pub art: String,
    pub bank: String,
    pub iban: String,
    pub qr_iban: String,
    pub bic: String,
    pub konto_id: String,
    pub standard: String,
    pub aktiv: String,
}

impl BankAccountInput {
    /// Leeres Formular: Art „Bank“, aktiv.
    pub fn new_default() -> Self {
        Self {
            art: "bank".into(),
            aktiv: flag(true),
            ..Default::default()
        }
    }

    /// Leerzeichen entfernen; IBAN und BIC kompakt speichern. Bei der Kasse entfallen Bank und Kontodaten (AK-3).
    pub fn normalized(&self) -> Self {
        let art = self.art.trim().to_lowercase();
        let kasse = art == "kasse";
        let keep = |s: &str| {
            if kasse {
                String::new()
            } else {
                s.trim().to_string()
            }
        };
        let kompakt = |s: &str| if kasse { String::new() } else { compact(s) };
        Self {
            bezeichnung: self.bezeichnung.trim().into(),
            art,
            bank: keep(&self.bank),
            iban: kompakt(&self.iban),
            qr_iban: kompakt(&self.qr_iban),
            bic: kompakt(&self.bic),
            konto_id: self.konto_id.trim().into(),
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

    /// Kennung des Kontos im Kontenplan, sofern gültig.
    pub fn konto(&self) -> Option<i64> {
        self.konto_id.parse().ok()
    }

    /// Prüft die (normalisierte) Eingabe ohne Datenbank (AK-2, AK-3, AK-4). Leere Map = gültig.
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
        if !KINDS.iter().any(|(k, _)| *k == self.art) {
            err("art", "Bitte Bank, Post oder Kasse wählen.");
        }
        if self.bank.chars().count() > BANK_MAX {
            err("bank", "Höchstens 100 Zeichen.");
        }
        if self.art != "kasse" {
            if self.iban.is_empty() {
                err("iban", "Bei Bank und Post ist die IBAN Pflicht.");
            } else if let Some(msg) = iban_error(&self.iban) {
                err("iban", msg);
            }
        }
        if !self.qr_iban.is_empty()
            && let Some(msg) = qr_iban_error(&self.qr_iban)
        {
            err("qr_iban", msg);
        }
        if !self.bic.is_empty() && !bic_ok(&self.bic) {
            err("bic", "Der BIC hat 8 oder 11 Zeichen, z. B. ZKBKCHZZ80A.");
        }
        if self.konto().is_none() {
            err("konto_id", "Bitte ein Konto aus dem Kontenplan wählen.");
        }
        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iban_accepts_valid_swiss_and_german() {
        assert_eq!(iban_error("CH93 0076 2011 6238 5295 7"), None);
        assert_eq!(iban_error("ch9300762011623852957"), None);
        assert_eq!(iban_error("DE89 3704 0044 0532 0130 00"), None);
    }

    #[test]
    fn iban_rejects_bad_check_digits_and_length() {
        assert!(iban_error("CH93 0076 2011 6238 5295 8").is_some());
        assert!(iban_error("CH93 0076 2011 6238 5295").is_some());
        assert!(iban_error("DE89 3704 0044 0532 0130 0").is_some());
        assert!(iban_error("1234567890123456").is_some());
    }

    #[test]
    fn swiss_iban_needs_21_characters() {
        // Prüfziffer stimmt, aber 22 Zeichen
        assert!(iban_error("CH930076201162385295700").is_some());
    }

    #[test]
    fn qr_iban_needs_institute_number() {
        // Institutsnummer 30000 bis 31999 (Stelle 5 bis 9)
        assert_eq!(qr_iban_error("CH44 3199 9123 0008 8901 2"), None);
        assert!(qr_iban_error("CH93 0076 2011 6238 5295 7").is_some());
        assert!(qr_iban_error("DE89 3704 0044 0532 0130 00").is_some());
    }

    #[test]
    fn bic_format() {
        assert!(bic_ok("ZKBKCHZZ"));
        assert!(bic_ok("ZKBKCHZZ80A"));
        assert!(!bic_ok("ZKBKCH"));
        assert!(!bic_ok("zkbkchzz"));
    }

    #[test]
    fn group_iban_in_fours() {
        assert_eq!(
            group_iban("CH9300762011623852957"),
            "CH93 0076 2011 6238 5295 7"
        );
    }

    #[test]
    fn cash_box_drops_bank_data() {
        let input = BankAccountInput {
            bezeichnung: " Kasse ".into(),
            art: "Kasse".into(),
            iban: "CH93 0076 2011 6238 5295 7".into(),
            konto_id: "3".into(),
            aktiv: flag(true),
            ..Default::default()
        }
        .normalized();
        assert_eq!(input.bezeichnung, "Kasse");
        assert_eq!(input.art, "kasse");
        assert!(input.iban.is_empty());
        assert!(input.validate().is_empty());
    }

    #[test]
    fn bank_needs_iban_and_konto() {
        let errors = BankAccountInput {
            bezeichnung: "ZKB".into(),
            art: "bank".into(),
            aktiv: flag(true),
            ..Default::default()
        }
        .normalized()
        .validate();
        assert!(errors.contains_key("iban"));
        assert!(errors.contains_key("konto_id"));
        assert!(!errors.contains_key("bezeichnung"));
    }
}
