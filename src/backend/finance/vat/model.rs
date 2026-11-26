//! Datenmodell und Prüfregeln für MWST-Codes, MWST-Einstellungen und die Umsatzgrenze (ANF-012).
//! Wird auf dem Server und im Browser verwendet.
//!
//! Beträge in Rappen (ganze Zahlen), Sätze in Hundertstel Prozent (810 = 8.1 %).

use serde::{Deserialize, Serialize};

use crate::backend::projects::model::is_valid_date;
use crate::backend::purchase_orders::model::{format_fixed, format_trimmed, parse_fixed};
pub use crate::forms::{FieldErrors, SaveOutcome};

pub const CODE_MAX: usize = 10;
pub const NAME_MAX: usize = 100;
pub const FORM_MAX: usize = 10;
/// Höchstsatz in Hundertstel Prozent (100 %).
pub const RATE_MAX: i64 = 10_000;
/// Vorgabe der Umsatzgrenze: CHF 100 000.00 in Rappen (AK-11).
pub const DEFAULT_LIMIT: i64 = 10_000_000;
/// Ab diesem Anteil der Umsatzgrenze erscheint der Hinweis (AK-11).
pub const WARN_PERCENT: i128 = 80;
/// Gültig ab für die Standard-Codes: die Sätze 8.1 / 2.6 / 3.8 % gelten seit 1. Januar 2024.
pub const STANDARD_VALID_FROM: &str = "2024-01-01";

/// Art eines Codes (AK-3).
pub const KINDS: &[(&str, &str)] = &[("umsatzsteuer", "Umsatzsteuer"), ("vorsteuer", "Vorsteuer")];

/// Abrechnungsmethode (AK-8).
pub const METHODS: &[(&str, &str)] = &[
    ("effektiv", "Effektiv"),
    ("saldosteuersatz", "Saldosteuersatz"),
];

/// Abrechnungsart (AK-8).
pub const ACCOUNTING: &[(&str, &str)] = &[
    ("vereinbart", "Vereinbart (Sollbesteuerung)"),
    ("vereinnahmt", "Vereinnahmt (Istbesteuerung)"),
];

/// Abrechnungsperiode (AK-8).
pub const PERIODS: &[(&str, &str)] = &[("quartal", "Quartalsweise"), ("halbjahr", "Halbjährlich")];

pub fn kind_label(kind: &str) -> &'static str {
    KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or("–", |(_, l)| l)
}

fn label_in(list: &'static [(&'static str, &'static str)], key: &str) -> &'static str {
    list.iter().find(|(k, _)| *k == key).map_or("–", |(_, l)| l)
}

pub fn method_label(key: &str) -> &'static str {
    label_in(METHODS, key)
}

pub fn accounting_label(key: &str) -> &'static str {
    label_in(ACCOUNTING, key)
}

pub fn period_label(key: &str) -> &'static str {
    label_in(PERIODS, key)
}

/// Kontoart, die ein MWST-Konto haben muss: Umsatzsteuer auf Passivkonten, Vorsteuer auf Aktivkonten (AK-4).
pub fn konto_kind_for(art: &str) -> &'static str {
    if art == "umsatzsteuer" {
        "passiv"
    } else {
        "aktiv"
    }
}

/// Systemkonto (ANF-011), das zur Art passt und im Formular vorgeschlagen wird (AK-4).
pub fn system_role_for(art: &str) -> &'static str {
    if art == "umsatzsteuer" {
        "umsatzsteuer"
    } else {
        "vorsteuer"
    }
}

/// „8.1 %“
pub fn percent_label(satz: i64) -> String {
    format!("{} %", format_trimmed(satz, 2))
}

/// Prozent-Eingabe in Hundertstel Prozent, z. B. „8.1“ → 810 (höchstens 2 Nachkommastellen, 0 bis 100).
pub fn parse_rate(s: &str) -> Option<i64> {
    parse_fixed(s, 2).filter(|r| *r <= RATE_MAX)
}

/// Standard-Code der Schweizer MWST (AK-2). Das Konto ist das Systemkonto der Art, außer `konto_nummer` ist gesetzt.
#[derive(Clone, Copy, Debug)]
pub struct StandardCode {
    pub code: &'static str,
    pub bezeichnung: &'static str,
    pub art: &'static str,
    /// Hundertstel Prozent
    pub satz: i64,
    pub konto_nummer: Option<&'static str>,
}

pub const STANDARD_CODES: &[StandardCode] = &[
    StandardCode {
        code: "UN81",
        bezeichnung: "Umsatzsteuer Normalsatz",
        art: "umsatzsteuer",
        satz: 810,
        konto_nummer: None,
    },
    StandardCode {
        code: "UR26",
        bezeichnung: "Umsatzsteuer reduzierter Satz",
        art: "umsatzsteuer",
        satz: 260,
        konto_nummer: None,
    },
    StandardCode {
        code: "UB38",
        bezeichnung: "Umsatzsteuer Beherbergung",
        art: "umsatzsteuer",
        satz: 380,
        konto_nummer: None,
    },
    StandardCode {
        code: "VM81",
        bezeichnung: "Vorsteuer Material/Dienstleistungen 8.1 %",
        art: "vorsteuer",
        satz: 810,
        konto_nummer: None,
    },
    StandardCode {
        code: "VI81",
        bezeichnung: "Vorsteuer Investitionen/übriger Betriebsaufwand 8.1 %",
        art: "vorsteuer",
        satz: 810,
        konto_nummer: Some("1171"),
    },
    StandardCode {
        code: "UEX",
        bezeichnung: "Steuerbefreit / Export",
        art: "umsatzsteuer",
        satz: 0,
        konto_nummer: None,
    },
];

/// Ein MWST-Code aus der Datenbank, mit Konto und der Angabe, ob er schon verwendet wird (AK-5, AK-7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct VatCode {
    pub id: i64,
    pub code: String,
    pub bezeichnung: String,
    pub art: String,
    /// Hundertstel Prozent
    pub satz: i64,
    pub gueltig_ab: String,
    pub gueltig_bis: Option<String>,
    pub konto_id: i64,
    pub konto_nummer: String,
    pub konto_bezeichnung: String,
    pub formularziffer: Option<String>,
    pub aktiv: bool,
    /// Wird in Einkaufspositionen (und später in Rechnungen und Buchungen) verwendet.
    pub verwendet: bool,
}

impl VatCode {
    /// Gilt der Code am Datum `datum` (`JJJJ-MM-TT`)?
    pub fn is_valid_on(&self, datum: &str) -> bool {
        self.gueltig_ab.as_str() <= datum
            && self.gueltig_bis.as_deref().is_none_or(|bis| datum <= bis)
    }

    /// „VM81 · 8.1 %“ für Auswahllisten (die Bezeichnung steht auf der MWST-Seite).
    pub fn choice_label(&self) -> String {
        format!("{} · {}", self.code, percent_label(self.satz))
    }

    pub fn to_input(&self) -> VatCodeInput {
        VatCodeInput {
            code: self.code.clone(),
            bezeichnung: self.bezeichnung.clone(),
            art: self.art.clone(),
            satz: format_fixed(self.satz, 2),
            gueltig_ab: self.gueltig_ab.clone(),
            gueltig_bis: self.gueltig_bis.clone().unwrap_or_default(),
            konto_id: self.konto_id.to_string(),
            formularziffer: self.formularziffer.clone().unwrap_or_default(),
            aktiv: if self.aktiv {
                "1".into()
            } else {
                String::new()
            },
        }
    }
}

/// Einstellungen der MWST (eine Zeile, AK-8, AK-11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct VatSettings {
    pub mwst_pflichtig: bool,
    pub mwst_pflichtig_ab: Option<String>,
    /// Rappen
    pub umsatzgrenze: i64,
    pub methode: String,
    pub abrechnungsart: String,
    pub periodizitaet: String,
}

impl Default for VatSettings {
    fn default() -> Self {
        Self {
            mwst_pflichtig: false,
            mwst_pflichtig_ab: None,
            umsatzgrenze: DEFAULT_LIMIT,
            methode: "effektiv".into(),
            abrechnungsart: "vereinbart".into(),
            periodizitaet: "quartal".into(),
        }
    }
}

impl VatSettings {
    pub fn to_input(&self) -> VatSettingsInput {
        VatSettingsInput {
            pflichtig: if self.mwst_pflichtig {
                "1".into()
            } else {
                String::new()
            },
            pflichtig_ab: self.mwst_pflichtig_ab.clone().unwrap_or_default(),
            umsatzgrenze: format_fixed(self.umsatzgrenze, 2),
            methode: self.methode.clone(),
            abrechnungsart: self.abrechnungsart.clone(),
            periodizitaet: self.periodizitaet.clone(),
        }
    }
}

/// Alles, was die MWST-Seite zeigt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VatOverview {
    pub settings: VatSettings,
    pub codes: Vec<VatCode>,
    /// UID der Firma (ANF-025); ohne UID kann die Pflicht nicht eingeschaltet werden (AK-8)
    pub uid: Option<String>,
    /// Massgebender Umsatz aus dem Buchungsjournal (AK-11)
    pub umsatz: Umsatz,
}

/// Gebuchte Erlöse (Kontenklasse 3) in Rappen: im laufenden Geschäftsjahr und in den letzten 12 Monaten.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Umsatz {
    pub geschaeftsjahr: i64,
    pub letzte_12_monate: i64,
}

impl Umsatz {
    /// Massgebend ist der höhere der beiden Werte (ANF-012 AK-11).
    pub fn massgebend(&self) -> i64 {
        self.geschaeftsjahr.max(self.letzte_12_monate)
    }
}

/// Formulareingabe für einen MWST-Code (AK-3).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VatCodeInput {
    pub code: String,
    pub bezeichnung: String,
    pub art: String,
    /// Prozent als Text, z. B. „8.1“
    pub satz: String,
    pub gueltig_ab: String,
    pub gueltig_bis: String,
    pub konto_id: String,
    pub formularziffer: String,
    /// Kontrollkästchen: nicht leer = aktiv
    pub aktiv: String,
}

/// Geprüfte Werte eines MWST-Codes, bereit zum Speichern.
#[derive(Clone, Debug, PartialEq)]
pub struct CodeValues {
    pub code: String,
    pub bezeichnung: String,
    pub art: String,
    pub satz: i64,
    pub gueltig_ab: String,
    pub gueltig_bis: Option<String>,
    pub konto_id: i64,
    pub formularziffer: Option<String>,
    pub aktiv: bool,
}

impl VatCodeInput {
    pub fn new_default() -> Self {
        Self {
            art: "umsatzsteuer".into(),
            aktiv: "1".into(),
            ..Default::default()
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            code: self.code.trim().to_uppercase(),
            bezeichnung: self.bezeichnung.trim().into(),
            art: self.art.trim().to_lowercase(),
            satz: self.satz.trim().into(),
            gueltig_ab: self.gueltig_ab.trim().into(),
            gueltig_bis: self.gueltig_bis.trim().into(),
            konto_id: self.konto_id.trim().into(),
            formularziffer: self.formularziffer.trim().into(),
            aktiv: self.aktiv.trim().into(),
        }
    }

    /// Prüft die normalisierte Eingabe und liefert die Werte zum Speichern (AK-3, AK-4, AK-6).
    /// Die Konto-Kontoart und die Verwendung prüft der Server zusätzlich.
    pub fn check(&self) -> Result<CodeValues, FieldErrors> {
        let mut errors = FieldErrors::new();
        let mut err = |field: &str, message: &str| {
            errors.entry(field.into()).or_insert_with(|| message.into());
        };

        if self.code.is_empty() {
            err("code", "Bitte einen Code eingeben.");
        } else if !is_valid_code(&self.code) {
            err(
                "code",
                "Höchstens 10 Zeichen: Grossbuchstaben, Ziffern, „-“ oder „_“.",
            );
        }
        if self.bezeichnung.is_empty() {
            err("bezeichnung", "Bitte eine Bezeichnung eingeben.");
        } else if self.bezeichnung.chars().count() > NAME_MAX {
            err("bezeichnung", "Höchstens 100 Zeichen.");
        }
        if !KINDS.iter().any(|(k, _)| *k == self.art) {
            err("art", "Bitte eine Art wählen.");
        }
        let satz = parse_rate(&self.satz);
        if satz.is_none() {
            err(
                "satz",
                "Bitte einen Satz von 0 bis 100 % eingeben (höchstens 2 Nachkommastellen).",
            );
        }
        if self.gueltig_ab.is_empty() {
            err("gueltig_ab", "Bitte ein Datum eingeben.");
        } else if !is_valid_date(&self.gueltig_ab) {
            err("gueltig_ab", "Bitte ein gültiges Datum eingeben.");
        }
        if !self.gueltig_bis.is_empty() {
            if !is_valid_date(&self.gueltig_bis) {
                err("gueltig_bis", "Bitte ein gültiges Datum eingeben.");
            } else if is_valid_date(&self.gueltig_ab) && self.gueltig_bis < self.gueltig_ab {
                err(
                    "gueltig_bis",
                    "Das Ende liegt vor dem Beginn der Gültigkeit.",
                );
            }
        }
        let konto_id = self.konto_id.parse::<i64>().ok();
        if konto_id.is_none() {
            err("konto_id", "Bitte ein Konto wählen.");
        }
        if self.formularziffer.chars().count() > FORM_MAX {
            err("formularziffer", "Höchstens 10 Zeichen.");
        }

        match (satz, konto_id) {
            (Some(satz), Some(konto_id)) if errors.is_empty() => Ok(CodeValues {
                code: self.code.clone(),
                bezeichnung: self.bezeichnung.clone(),
                art: self.art.clone(),
                satz,
                gueltig_ab: self.gueltig_ab.clone(),
                gueltig_bis: (!self.gueltig_bis.is_empty()).then(|| self.gueltig_bis.clone()),
                konto_id,
                formularziffer: (!self.formularziffer.is_empty())
                    .then(|| self.formularziffer.clone()),
                aktiv: !self.aktiv.is_empty(),
            }),
            _ => Err(errors),
        }
    }
}

/// Grossbuchstaben, Ziffern, „-“ und „_“, höchstens 10 Zeichen.
pub fn is_valid_code(code: &str) -> bool {
    (1..=CODE_MAX).contains(&code.len())
        && code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

/// Prüft, ob das Konto zur Art passt (AK-4): gleiche Kontoart und aktiv.
pub fn konto_error(art: &str, konto_art: &str, aktiv: bool) -> Option<String> {
    if !aktiv {
        Some("Das Konto ist inaktiv.".into())
    } else if konto_art != konto_kind_for(art) {
        Some(format!(
            "Für {} ist ein Konto der Kontoart {} nötig.",
            kind_label(art),
            if konto_kind_for(art) == "passiv" {
                "Passiv"
            } else {
                "Aktiv"
            }
        ))
    } else {
        None
    }
}

/// Ein verwendeter Code behält Satz und Art (AK-5). Gibt die Fehler der geänderten Felder zurück.
pub fn locked_change_errors(existing: &VatCode, new: &CodeValues) -> FieldErrors {
    let mut errors = FieldErrors::new();
    if !existing.verwendet {
        return errors;
    }
    if existing.satz != new.satz {
        errors.insert(
            "satz".into(),
            "Der Satz kann nicht mehr geändert werden, weil der Code verwendet wird. \
             Den Code mit „gültig bis“ beenden und einen neuen anlegen."
                .into(),
        );
    }
    if existing.art != new.art {
        errors.insert(
            "art".into(),
            "Die Art kann nicht mehr geändert werden, weil der Code verwendet wird.".into(),
        );
    }
    errors
}

/// Eingabe der MWST-Einstellungen (AK-8, AK-11).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct VatSettingsInput {
    /// Kontrollkästchen: nicht leer = MWST-pflichtig
    pub pflichtig: String,
    pub pflichtig_ab: String,
    pub umsatzgrenze: String,
    pub methode: String,
    pub abrechnungsart: String,
    pub periodizitaet: String,
}

impl VatSettingsInput {
    pub fn normalized(&self) -> Self {
        Self {
            pflichtig: self.pflichtig.trim().into(),
            pflichtig_ab: self.pflichtig_ab.trim().into(),
            umsatzgrenze: self.umsatzgrenze.trim().into(),
            methode: self.methode.trim().into(),
            abrechnungsart: self.abrechnungsart.trim().into(),
            periodizitaet: self.periodizitaet.trim().into(),
        }
    }

    /// Prüft die Eingabe und liefert die Einstellungen. `uid_vorhanden`: die Firma hat eine UID (AK-8).
    pub fn check(&self, uid_vorhanden: bool) -> Result<VatSettings, FieldErrors> {
        let mut errors = FieldErrors::new();
        let pflichtig = !self.pflichtig.is_empty();
        if pflichtig {
            if self.pflichtig_ab.is_empty() {
                errors.insert(
                    "pflichtig_ab".into(),
                    "Bitte das Beginndatum eingeben.".into(),
                );
            } else if !is_valid_date(&self.pflichtig_ab) {
                errors.insert(
                    "pflichtig_ab".into(),
                    "Bitte ein gültiges Datum eingeben.".into(),
                );
            }
            if !uid_vorhanden {
                errors.insert(
                    "pflichtig".into(),
                    "Ohne UID der Firma kann die MWST-Pflicht nicht eingeschaltet werden. \
                     Die UID steht unter Einstellungen → Firma."
                        .into(),
                );
            }
        }
        let umsatzgrenze = parse_fixed(&self.umsatzgrenze, 2).filter(|v| *v > 0);
        if umsatzgrenze.is_none() {
            errors.insert(
                "umsatzgrenze".into(),
                "Bitte einen Betrag über 0 eingeben (höchstens 2 Nachkommastellen).".into(),
            );
        }
        if !METHODS.iter().any(|(k, _)| *k == self.methode) {
            errors.insert("methode".into(), "Bitte eine Methode wählen.".into());
        }
        if !ACCOUNTING.iter().any(|(k, _)| *k == self.abrechnungsart) {
            errors.insert(
                "abrechnungsart".into(),
                "Bitte eine Abrechnungsart wählen.".into(),
            );
        }
        if !PERIODS.iter().any(|(k, _)| *k == self.periodizitaet) {
            errors.insert(
                "periodizitaet".into(),
                "Bitte eine Periodizität wählen.".into(),
            );
        }
        match umsatzgrenze {
            Some(limit) if errors.is_empty() => Ok(VatSettings {
                mwst_pflichtig: pflichtig,
                mwst_pflichtig_ab: pflichtig.then(|| self.pflichtig_ab.clone()),
                umsatzgrenze: limit,
                methode: self.methode.clone(),
                abrechnungsart: self.abrechnungsart.clone(),
                periodizitaet: self.periodizitaet.clone(),
            }),
            _ => Err(errors),
        }
    }
}

/// Gilt die MWST-Pflicht an `datum`? Ja, wenn sie eingeschaltet ist und das Beginndatum erreicht ist (AK-10).
pub fn pflicht_gilt_am(settings: &VatSettings, datum: &str) -> bool {
    settings.mwst_pflichtig
        && settings
            .mwst_pflichtig_ab
            .as_deref()
            .is_some_and(|ab| ab <= datum)
}

/// Stand der Umsatzgrenze (AK-11).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LimitStage {
    Unauffaellig,
    /// ab 80 % (`warning`)
    Hinweis,
    /// ab 100 % (`error`)
    Erreicht,
}

/// Stufe für einen Umsatz in Rappen gegenüber der Umsatzgrenze in Rappen.
pub fn limit_stage(umsatz: i64, grenze: i64) -> LimitStage {
    let (u, g) = (i128::from(umsatz), i128::from(grenze));
    if u >= g {
        LimitStage::Erreicht
    } else if u * 100 >= g * WARN_PERCENT {
        LimitStage::Hinweis
    } else {
        LimitStage::Unauffaellig
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(satz: i64, verwendet: bool) -> VatCode {
        VatCode {
            id: 1,
            code: "VM81".into(),
            bezeichnung: "Vorsteuer".into(),
            art: "vorsteuer".into(),
            satz,
            gueltig_ab: "2024-01-01".into(),
            gueltig_bis: None,
            konto_id: 5,
            konto_nummer: "1170".into(),
            konto_bezeichnung: "Vorsteuer".into(),
            formularziffer: None,
            aktiv: true,
            verwendet,
        }
    }

    fn valid_input() -> VatCodeInput {
        VatCodeInput {
            code: " vm81 ".into(),
            bezeichnung: " Vorsteuer Material ".into(),
            art: "Vorsteuer".into(),
            satz: "8.1".into(),
            gueltig_ab: "2024-01-01".into(),
            konto_id: "5".into(),
            aktiv: "1".into(),
            ..Default::default()
        }
    }

    #[test]
    fn valid_code_input() {
        let values = valid_input().normalized().check().unwrap();
        assert_eq!(values.code, "VM81");
        assert_eq!(values.art, "vorsteuer");
        assert_eq!(values.satz, 810);
        assert_eq!(values.gueltig_bis, None);
        assert_eq!(values.formularziffer, None);
        assert!(values.aktiv);
    }

    #[test]
    fn invalid_code_input_reports_fields() {
        let bad = VatCodeInput {
            code: "ZU LANG FUER DEN CODE".into(),
            bezeichnung: String::new(),
            art: "anderes".into(),
            satz: "101".into(),
            gueltig_ab: "2024-02-30".into(),
            gueltig_bis: "2023-12-31".into(),
            konto_id: "x".into(),
            ..Default::default()
        }
        .normalized();
        let errors = bad.check().unwrap_err();
        for field in [
            "code",
            "bezeichnung",
            "art",
            "satz",
            "gueltig_ab",
            "konto_id",
        ] {
            assert!(errors.contains_key(field), "{field} fehlt in {errors:?}");
        }
        let early = VatCodeInput {
            gueltig_ab: "2025-01-01".into(),
            gueltig_bis: "2024-12-31".into(),
            ..valid_input()
        };
        assert!(early.check().unwrap_err().contains_key("gueltig_bis"));
    }

    #[test]
    fn code_characters() {
        assert!(is_valid_code("UN81"));
        assert!(is_valid_code("A-1_2"));
        assert!(!is_valid_code(""));
        assert!(!is_valid_code("un81"));
        assert!(!is_valid_code("ÄÖ"));
        assert!(!is_valid_code("ABCDEFGHIJK"));
    }

    #[test]
    fn validity_by_date() {
        let c = code(810, false);
        assert!(c.is_valid_on("2024-01-01"));
        assert!(!c.is_valid_on("2023-12-31"));
        let ended = VatCode {
            gueltig_bis: Some("2025-06-30".into()),
            ..code(810, false)
        };
        assert!(ended.is_valid_on("2025-06-30"));
        assert!(!ended.is_valid_on("2025-07-01"));
    }

    #[test]
    fn konto_must_match_kind() {
        assert_eq!(konto_error("vorsteuer", "aktiv", true), None);
        assert!(
            konto_error("vorsteuer", "passiv", true)
                .unwrap()
                .contains("Aktiv")
        );
        assert_eq!(konto_error("umsatzsteuer", "passiv", true), None);
        assert!(konto_error("umsatzsteuer", "passiv", false).is_some());
    }

    #[test]
    fn used_codes_keep_rate_and_kind() {
        let used = code(810, true);
        let same = valid_input().normalized().check().unwrap();
        assert!(locked_change_errors(&used, &same).is_empty());
        let changed = CodeValues {
            satz: 260,
            ..same.clone()
        };
        assert!(locked_change_errors(&used, &changed).contains_key("satz"));
        let other_kind = CodeValues {
            art: "umsatzsteuer".into(),
            ..same.clone()
        };
        assert!(locked_change_errors(&used, &other_kind).contains_key("art"));
        // Unbenutzte Codes dürfen alles ändern
        assert!(locked_change_errors(&code(810, false), &changed).is_empty());
    }

    #[test]
    fn settings_need_uid_and_start_date() {
        let input = VatSettingsInput {
            pflichtig: "1".into(),
            umsatzgrenze: "100000".into(),
            methode: "effektiv".into(),
            abrechnungsart: "vereinbart".into(),
            periodizitaet: "quartal".into(),
            ..Default::default()
        };
        let no_uid = input.check(false).unwrap_err();
        assert!(no_uid.contains_key("pflichtig"));
        assert!(no_uid.contains_key("pflichtig_ab"));
        let ok = VatSettingsInput {
            pflichtig_ab: "2027-01-01".into(),
            ..input.clone()
        }
        .check(true)
        .unwrap();
        assert!(ok.mwst_pflichtig);
        assert_eq!(ok.umsatzgrenze, DEFAULT_LIMIT);
        assert_eq!(ok.mwst_pflichtig_ab.as_deref(), Some("2027-01-01"));
        // ausgeschaltet: kein Datum und keine UID nötig, Beginn wird verworfen
        let off = VatSettingsInput {
            pflichtig: String::new(),
            pflichtig_ab: "2027-01-01".into(),
            ..input
        }
        .check(false)
        .unwrap();
        assert!(!off.mwst_pflichtig);
        assert_eq!(off.mwst_pflichtig_ab, None);
    }

    #[test]
    fn limit_stages() {
        let grenze = DEFAULT_LIMIT;
        assert_eq!(limit_stage(0, grenze), LimitStage::Unauffaellig);
        assert_eq!(limit_stage(7_999_999, grenze), LimitStage::Unauffaellig);
        assert_eq!(limit_stage(8_000_000, grenze), LimitStage::Hinweis);
        assert_eq!(limit_stage(9_999_999, grenze), LimitStage::Hinweis);
        assert_eq!(limit_stage(10_000_000, grenze), LimitStage::Erreicht);
        assert_eq!(limit_stage(20_000_000, grenze), LimitStage::Erreicht);
    }

    #[test]
    fn standard_codes_are_consistent() {
        let mut names: Vec<&str> = STANDARD_CODES.iter().map(|c| c.code).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), STANDARD_CODES.len());
        for c in STANDARD_CODES {
            assert!(is_valid_code(c.code), "{}", c.code);
            assert!(KINDS.iter().any(|(k, _)| *k == c.art), "{}", c.code);
            assert!((0..=RATE_MAX).contains(&c.satz), "{}", c.code);
            assert!(c.bezeichnung.chars().count() <= NAME_MAX);
        }
        let rates: Vec<i64> = STANDARD_CODES.iter().map(|c| c.satz).collect();
        assert!(
            rates.contains(&810)
                && rates.contains(&260)
                && rates.contains(&380)
                && rates.contains(&0)
        );
    }

    #[test]
    fn pflicht_starts_at_begin_date() {
        let on = VatSettings {
            mwst_pflichtig: true,
            mwst_pflichtig_ab: Some("2027-01-01".into()),
            ..Default::default()
        };
        assert!(!pflicht_gilt_am(&on, "2026-12-31"));
        assert!(pflicht_gilt_am(&on, "2027-01-01"));
        assert!(!pflicht_gilt_am(&VatSettings::default(), "2030-01-01"));
    }

    #[test]
    fn labels_and_parsing() {
        assert_eq!(parse_rate("8.1"), Some(810));
        assert_eq!(parse_rate("100"), Some(10_000));
        assert_eq!(parse_rate("100.01"), None);
        assert_eq!(parse_rate("8.123"), None);
        assert_eq!(percent_label(260), "2.6 %");
        assert_eq!(system_role_for("umsatzsteuer"), "umsatzsteuer");
        assert_eq!(konto_kind_for("vorsteuer"), "aktiv");
        assert_eq!(code(810, false).to_input().satz, "8.10");
    }
}
