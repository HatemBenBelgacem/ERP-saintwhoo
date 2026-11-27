//! Kreditoren – Eingangsrechnungen und Gutschriften von Lieferanten (ANF-019): Datentypen, Prüfregeln,
//! Summen mit Kontrollbetrag und die Zeilen der Buchung. Wird auf dem Server und im Browser verwendet.
//!
//! Beträge in Rappen, Kurse in Millionsteln. Die MWST steht je Position fest (aus Netto oder Brutto
//! berechnet, AK-6); gerundet wird nicht automatisch, sondern nur auf Wunsch bis 0.05 (AK-7).

use serde::{Deserialize, Serialize};

use crate::backend::finance::debtors::model::{TermOption, to_chf, vat_required_on};
use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::journal::model::{
    AccountOption, CodeOption, EffectiveLine, code_valid_on, tax_split,
};
use crate::backend::projects::model::is_valid_date;
use crate::backend::purchase_orders::model::{format_amount, format_fixed, parse_fixed};
pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

/// Webadresse der Liste (Reiter „Kreditoren“).
pub const CREDITORS_URL: &str = "/finanzen/kreditoren";

pub const PAGE_SIZE: i64 = 50;
/// Länge der Rechnungsnummer des Lieferanten.
pub const SUPPLIER_NUMBER_MAX: usize = 50;
/// Länge der Zahlungsreferenz (QR- oder Kreditorreferenz).
pub const PAYMENT_REFERENCE_MAX: usize = 35;
pub const TEXT_MAX: usize = 2000;
pub const SEARCH_MAX: usize = 100;
/// Grösste Rundungsdifferenz, die ausgeglichen werden darf (AK-7), in Rappen.
pub const ROUNDING_MAX: i64 = 5;

pub const KINDS: &[(&str, &str)] = &[("rechnung", "Rechnung"), ("gutschrift", "Gutschrift")];

pub const AMOUNT_KINDS: &[(&str, &str)] = &[
    ("netto", "Netto (ohne MWST)"),
    ("brutto", "Brutto (mit MWST)"),
];

pub const STATUS_FILTERS: &[(&str, &str)] = &[
    ("offen", "Offen und teilweise bezahlt"),
    ("alle", "Alle"),
    ("entwurf", "Entwurf"),
    ("bezahlt", "Bezahlt"),
    ("storniert", "Storniert"),
];

pub const SORT_COLUMNS: &[(&str, &str)] = &[
    ("nummer", "Nummer"),
    ("rechnungsnummer", "Rechnungs-Nr."),
    ("lieferant", "Lieferant"),
    ("auftrag", "Einkaufsauftrag"),
    ("projekt", "Projekt"),
    ("rechnungsdatum", "Rechnungsdatum"),
    ("faelligkeit", "Fälligkeit"),
    ("skonto", "Skonto bis"),
    ("betrag", "Gesamtbetrag"),
    ("offen", "Offen"),
    ("status", "Status"),
];

pub const DEFAULT_SORT: &str = "faelligkeit";

/// Interne Nummer, z. B. `K-2026-0001` (AK-10); Rechnungen und Gutschriften teilen den Zähler.
pub fn number_for(jahr: i32, laufnummer: i32) -> String {
    format!("K-{jahr}-{laufnummer:04}")
}

// ---------------------------------------------------------------------------------------------
// Positionen und Summen
// ---------------------------------------------------------------------------------------------

/// Eine Position (Beträge in Rappen).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct CreditorPosition {
    pub id: i64,
    pub bezeichnung: String,
    pub konto_id: i64,
    pub konto_nummer: String,
    pub konto_bezeichnung: String,
    pub mwst_code_id: Option<i64>,
    pub mwst_code: Option<String>,
    /// Hundertstel Prozent
    pub mwst_satz: Option<i64>,
    pub netto: i64,
    pub mwst_betrag: i64,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub einkaufsposition_id: Option<i64>,
}

impl CreditorPosition {
    pub fn brutto(&self) -> i64 {
        self.netto + self.mwst_betrag
    }

    pub fn project(&self) -> Option<PickOption> {
        Some(PickOption {
            id: self.projekt_id?,
            nummer: self.projekt_nummer.clone()?,
            name: String::new(),
        })
    }

    pub fn to_input(&self) -> PositionInput {
        PositionInput {
            bezeichnung: self.bezeichnung.clone(),
            konto_id: self.konto_id.to_string(),
            mwst_code_id: self.mwst_code_id.map(|i| i.to_string()).unwrap_or_default(),
            betrag: format_fixed(self.netto, 2),
            betragsart: "netto".into(),
            projekt_id: self.projekt_id.map(|i| i.to_string()).unwrap_or_default(),
        }
    }
}

/// MWST je Code (Summe der Positionen).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VatByCode {
    pub code_id: i64,
    pub code: String,
    pub rate: i64,
    pub base: i64,
    pub vat: i64,
}

/// Summen einer Eingangsrechnung mit Kontrollbetrag (AK-7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreditorTotals {
    pub subtotal: i64,
    pub vat: Vec<VatByCode>,
    /// Ausgeglichene Rundungsdifferenz
    pub rounding: i64,
    pub total: i64,
    /// Betrag gemäss Rechnung minus berechnetes Total; `None` ohne Kontrollbetrag
    pub difference: Option<i64>,
}

impl CreditorTotals {
    /// Buchen nur bei Differenz 0.00 (AK-7).
    pub fn is_balanced(&self) -> bool {
        self.difference == Some(0)
    }

    /// Differenz, die mit „Rundung ausgleichen“ gebucht werden darf: bis 0.05 und nicht null.
    pub fn roundable(&self) -> Option<i64> {
        let d = self.difference?;
        (d != 0 && (d + self.rounding).abs() <= ROUNDING_MAX).then_some(d + self.rounding)
    }
}

/// Summen aus den Positionen, der Rundung und dem Kontrollbetrag.
pub fn totals(
    positions: &[CreditorPosition],
    rounding: i64,
    control: Option<i64>,
) -> CreditorTotals {
    let subtotal: i64 = positions.iter().map(|p| p.netto).sum();
    let mut vat: Vec<VatByCode> = Vec::new();
    for p in positions {
        let (Some(code_id), Some(rate)) = (p.mwst_code_id, p.mwst_satz) else {
            continue;
        };
        match vat.iter_mut().find(|l| l.code_id == code_id) {
            Some(line) => {
                line.base += p.netto;
                line.vat += p.mwst_betrag;
            }
            None => vat.push(VatByCode {
                code_id,
                code: p.mwst_code.clone().unwrap_or_default(),
                rate,
                base: p.netto,
                vat: p.mwst_betrag,
            }),
        }
    }
    vat.sort_by(|a, b| b.rate.cmp(&a.rate).then_with(|| a.code.cmp(&b.code)));
    let total = subtotal + vat.iter().map(|l| l.vat).sum::<i64>() + rounding;
    CreditorTotals {
        subtotal,
        vat,
        rounding,
        total,
        difference: control.map(|c| c - total),
    }
}

/// Netto und MWST aus dem eingegebenen Betrag (AK-6): Netto → MWST aufschlagen, Brutto → herausrechnen.
pub fn split_amount(betrag: i64, satz: Option<i64>, brutto: bool) -> (i64, i64) {
    match satz {
        Some(satz) => tax_split(betrag, satz, brutto),
        None => (betrag, 0),
    }
}

// ---------------------------------------------------------------------------------------------
// Kopf
// ---------------------------------------------------------------------------------------------

/// Kopf einer Eingangsrechnung bzw. Gutschrift.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct CreditorInvoice {
    pub id: i64,
    pub nummer: Option<String>,
    pub art: String,
    pub adresse_id: i64,
    pub lieferant_nummer: String,
    pub lieferant: String,
    /// Im Entwurf aus einem Einkaufsauftrag noch leer (AK-8)
    pub rechnungsnummer: Option<String>,
    pub einkaufsauftrag_id: Option<i64>,
    pub einkaufsauftrag_nummer: Option<String>,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub projekt_name: Option<String>,
    pub status: String,
    pub rechnungsdatum: String,
    pub zahlungsbedingung_id: i64,
    pub zahlungsbedingung: String,
    pub faelligkeit: Option<String>,
    pub skonto_bis: Option<String>,
    pub skonto_prozent: Option<i64>,
    pub waehrung: String,
    pub kurs: Option<i64>,
    pub betrag_gemaess_rechnung: Option<i64>,
    pub rundung: i64,
    pub gesamtbetrag: Option<i64>,
    pub gesamtbetrag_chf: Option<i64>,
    pub offener_betrag: i64,
    pub zahlungsreferenz: Option<String>,
    pub bemerkung: Option<String>,
    pub buchung_id: Option<i64>,
    pub storno_buchung_id: Option<i64>,
    pub tage_ueberfaellig: Option<i32>,
    /// Skontofrist läuft noch (AK-3)
    pub skonto_laeuft: bool,
    pub erstellt_am: String,
    pub geaendert_am: String,
}

impl CreditorInvoice {
    pub fn is_draft(&self) -> bool {
        self.status == "entwurf"
    }

    pub fn is_editable(&self) -> bool {
        self.status != "storniert"
    }

    /// Stornieren nur offen und ohne Zahlungen (AK-14).
    pub fn can_storno(&self) -> bool {
        self.status == "offen" && self.gesamtbetrag == Some(self.offener_betrag)
    }

    /// Interne Nummer oder „Entwurf“.
    pub fn label(&self) -> String {
        match &self.nummer {
            Some(n) => n.clone(),
            None => format!(
                "Entwurf {}",
                crate::backend::finance::debtors::model::kind_label(&self.art)
            ),
        }
    }

    pub fn supplier(&self) -> PickOption {
        PickOption {
            id: self.adresse_id,
            nummer: self.lieferant_nummer.clone(),
            name: self.lieferant.clone(),
        }
    }

    pub fn order(&self) -> Option<PickOption> {
        Some(PickOption {
            id: self.einkaufsauftrag_id?,
            nummer: self.einkaufsauftrag_nummer.clone()?,
            name: self.lieferant.clone(),
        })
    }

    pub fn project(&self) -> Option<PickOption> {
        Some(PickOption {
            id: self.projekt_id?,
            nummer: self.projekt_nummer.clone()?,
            name: self.projekt_name.clone()?,
        })
    }

    pub fn to_input(&self) -> CreditorInput {
        let s = |v: &Option<String>| v.clone().unwrap_or_default();
        CreditorInput {
            art: self.art.clone(),
            adresse_id: self.adresse_id.to_string(),
            rechnungsnummer: s(&self.rechnungsnummer),
            einkaufsauftrag_id: self
                .einkaufsauftrag_id
                .map(|i| i.to_string())
                .unwrap_or_default(),
            projekt_id: self.projekt_id.map(|i| i.to_string()).unwrap_or_default(),
            rechnungsdatum: self.rechnungsdatum.clone(),
            betrag_gemaess_rechnung: self
                .betrag_gemaess_rechnung
                .map(|b| format_fixed(b, 2))
                .unwrap_or_default(),
            waehrung: self.waehrung.clone(),
            zahlungsbedingung_id: self.zahlungsbedingung_id.to_string(),
            faelligkeit: s(&self.faelligkeit),
            zahlungsreferenz: s(&self.zahlungsreferenz),
            bemerkung: s(&self.bemerkung),
        }
    }
}

/// Rechnung mit Positionen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreditorDetail {
    pub invoice: CreditorInvoice,
    pub positions: Vec<CreditorPosition>,
}

impl CreditorDetail {
    pub fn totals(&self) -> CreditorTotals {
        totals(
            &self.positions,
            self.invoice.rundung,
            self.invoice.betrag_gemaess_rechnung,
        )
    }
}

// ---------------------------------------------------------------------------------------------
// Eingaben
// ---------------------------------------------------------------------------------------------

/// Formulareingabe der Kopfdaten (AK-4, AK-12).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CreditorInput {
    pub art: String,
    pub adresse_id: String,
    pub rechnungsnummer: String,
    pub einkaufsauftrag_id: String,
    pub projekt_id: String,
    pub rechnungsdatum: String,
    pub betrag_gemaess_rechnung: String,
    pub waehrung: String,
    /// Leer = Vorgabe der Adresse bzw. Standard
    pub zahlungsbedingung_id: String,
    /// Leer = aus der Zahlungsbedingung beim Buchen
    pub faelligkeit: String,
    pub zahlungsreferenz: String,
    pub bemerkung: String,
}

impl CreditorInput {
    pub fn new_default(art: &str, heute: &str) -> Self {
        Self {
            art: if art == "gutschrift" {
                "gutschrift".into()
            } else {
                "rechnung".into()
            },
            rechnungsdatum: heute.into(),
            waehrung: "CHF".into(),
            ..Default::default()
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            art: self.art.trim().into(),
            adresse_id: self.adresse_id.trim().into(),
            rechnungsnummer: self.rechnungsnummer.trim().into(),
            einkaufsauftrag_id: self.einkaufsauftrag_id.trim().into(),
            projekt_id: self.projekt_id.trim().into(),
            rechnungsdatum: self.rechnungsdatum.trim().into(),
            betrag_gemaess_rechnung: self.betrag_gemaess_rechnung.trim().into(),
            waehrung: self.waehrung.trim().to_uppercase(),
            zahlungsbedingung_id: self.zahlungsbedingung_id.trim().into(),
            faelligkeit: self.faelligkeit.trim().into(),
            zahlungsreferenz: self
                .zahlungsreferenz
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect(),
            bemerkung: self.bemerkung.trim().into(),
        }
    }

    /// Nach dem Buchen sind nur Fälligkeit und Bemerkung änderbar (AK-12).
    pub fn restricted_to_booked(&self, current: &CreditorInput) -> Self {
        Self {
            faelligkeit: self.faelligkeit.clone(),
            bemerkung: self.bemerkung.clone(),
            ..current.clone()
        }
    }

    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |f: &str, m: &str| {
            errors.entry(f.into()).or_insert_with(|| m.into());
        };
        if !KINDS.iter().any(|(k, _)| *k == self.art) {
            err("art", "Bitte Rechnung oder Gutschrift wählen.");
        }
        if self.adresse_id.is_empty() {
            err("adresse_id", "Bitte einen Lieferanten wählen.");
        } else if self.adresse_id.parse::<i64>().is_err() {
            err("adresse_id", "Ungültiger Lieferant.");
        }
        if self.rechnungsnummer.is_empty() {
            err(
                "rechnungsnummer",
                "Bitte die Rechnungsnummer des Lieferanten eingeben.",
            );
        } else if self.rechnungsnummer.chars().count() > SUPPLIER_NUMBER_MAX {
            err("rechnungsnummer", "Höchstens 50 Zeichen.");
        }
        for (field, value, name) in [
            (
                "einkaufsauftrag_id",
                &self.einkaufsauftrag_id,
                "Einkaufsauftrag",
            ),
            ("projekt_id", &self.projekt_id, "Projekt"),
        ] {
            if !value.is_empty() && value.parse::<i64>().is_err() {
                err(field, &format!("Ungültiges {name}."));
            }
        }
        if self.rechnungsdatum.is_empty() {
            err("rechnungsdatum", "Bitte ein Rechnungsdatum eingeben.");
        } else if !is_valid_date(&self.rechnungsdatum) {
            err("rechnungsdatum", "Bitte ein gültiges Datum eingeben.");
        }
        if self.betrag_gemaess_rechnung.is_empty() {
            err(
                "betrag_gemaess_rechnung",
                "Bitte den Gesamtbetrag gemäss Rechnung eingeben.",
            );
        } else if parse_fixed(&self.betrag_gemaess_rechnung, 2).is_none() {
            err(
                "betrag_gemaess_rechnung",
                "Bitte einen Betrag mit höchstens 2 Nachkommastellen eingeben (nicht negativ).",
            );
        }
        if !crate::backend::finance::currency::model::is_code(&self.waehrung) {
            err("waehrung", "Bitte eine Währung aus der Liste wählen.");
        }
        if !self.zahlungsbedingung_id.is_empty()
            && self.zahlungsbedingung_id.parse::<i64>().is_err()
        {
            err(
                "zahlungsbedingung_id",
                "Bitte eine Zahlungsbedingung wählen.",
            );
        }
        if !self.faelligkeit.is_empty() {
            if !is_valid_date(&self.faelligkeit) {
                err("faelligkeit", "Bitte ein gültiges Datum eingeben.");
            } else if is_valid_date(&self.rechnungsdatum) && self.faelligkeit < self.rechnungsdatum
            {
                err(
                    "faelligkeit",
                    "Die Fälligkeit darf nicht vor dem Rechnungsdatum liegen.",
                );
            }
        }
        if self.zahlungsreferenz.chars().count() > PAYMENT_REFERENCE_MAX {
            err("zahlungsreferenz", "Höchstens 35 Zeichen.");
        }
        if self.bemerkung.chars().count() > TEXT_MAX {
            err("bemerkung", "Höchstens 2000 Zeichen.");
        }
        errors
    }

    pub fn supplier_id(&self) -> Option<i64> {
        self.adresse_id.parse().ok()
    }

    pub fn order_id(&self) -> Option<i64> {
        self.einkaufsauftrag_id.parse().ok()
    }

    pub fn project_id(&self) -> Option<i64> {
        self.projekt_id.parse().ok()
    }

    pub fn term_id(&self) -> Option<i64> {
        self.zahlungsbedingung_id.parse().ok()
    }

    pub fn control_amount(&self) -> Option<i64> {
        parse_fixed(&self.betrag_gemaess_rechnung, 2)
    }
}

/// Formulareingabe einer Position (AK-6).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PositionInput {
    pub bezeichnung: String,
    pub konto_id: String,
    /// Leer = ohne MWST (nur ohne MWST-Pflicht)
    pub mwst_code_id: String,
    pub betrag: String,
    /// `netto` oder `brutto`
    pub betragsart: String,
    /// Leer = Projekt der Rechnung
    pub projekt_id: String,
}

/// Geprüfte Werte einer Position (Betrag noch nicht aufgeteilt).
#[derive(Clone, Debug, PartialEq)]
pub struct PositionValues {
    pub konto_id: i64,
    pub mwst_code_id: Option<i64>,
    pub betrag: i64,
    pub brutto: bool,
    pub projekt_id: Option<i64>,
}

impl PositionInput {
    pub fn new_default() -> Self {
        Self {
            betragsart: "netto".into(),
            ..Default::default()
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            bezeichnung: self.bezeichnung.trim().into(),
            konto_id: self.konto_id.trim().into(),
            mwst_code_id: self.mwst_code_id.trim().into(),
            betrag: self.betrag.trim().into(),
            betragsart: self.betragsart.trim().into(),
            projekt_id: self.projekt_id.trim().into(),
        }
    }

    pub fn validate(&self) -> Result<PositionValues, FieldErrors> {
        let mut errors = FieldErrors::new();
        let mut err = |f: &str, m: &str| {
            errors.entry(f.into()).or_insert_with(|| m.into());
        };
        if self.bezeichnung.is_empty() {
            err("bezeichnung", "Bitte eine Bezeichnung eingeben.");
        } else if self.bezeichnung.chars().count() > 200 {
            err("bezeichnung", "Höchstens 200 Zeichen.");
        }
        let konto_id = self.konto_id.parse::<i64>().ok();
        if konto_id.is_none() {
            err("konto_id", "Bitte ein Aufwandskonto wählen.");
        }
        let code = if self.mwst_code_id.is_empty() {
            Some(None)
        } else {
            self.mwst_code_id.parse::<i64>().ok().map(Some)
        };
        if code.is_none() {
            err("mwst_code_id", "Bitte einen MWST-Code wählen.");
        }
        let betrag = parse_fixed(&self.betrag, 2);
        if betrag.is_none() {
            err(
                "betrag",
                if self.betrag.is_empty() {
                    "Bitte einen Betrag eingeben."
                } else {
                    "Bitte einen Betrag mit höchstens 2 Nachkommastellen eingeben (nicht negativ)."
                },
            );
        }
        let brutto = match self.betragsart.as_str() {
            "brutto" => true,
            "netto" | "" => false,
            _ => {
                err("betragsart", "Bitte Netto oder Brutto wählen.");
                false
            }
        };
        let projekt_id = if self.projekt_id.is_empty() {
            Some(None)
        } else {
            self.projekt_id.parse::<i64>().ok().map(Some)
        };
        if projekt_id.is_none() {
            err("projekt_id", "Ungültiges Projekt.");
        }
        match (konto_id, code, betrag, projekt_id) {
            (Some(konto_id), Some(mwst_code_id), Some(betrag), Some(projekt_id))
                if errors.is_empty() =>
            {
                Ok(PositionValues {
                    konto_id,
                    mwst_code_id,
                    betrag,
                    brutto,
                    projekt_id,
                })
            }
            _ => Err(errors),
        }
    }
}

/// Vorgaben für das Kopfformular.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreditorFormData {
    pub heute: String,
    pub terms: Vec<TermOption>,
}

/// Auswahl für das Positionsformular: aktive Aufwandskonten, Vorsteuer-Codes am Rechnungsdatum.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionFormData {
    pub konten: Vec<AccountOption>,
    pub codes: Vec<CodeOption>,
    pub vorgabe_konto_id: Option<i64>,
    pub mwst_pflicht: bool,
    pub rechnungsdatum: String,
}

/// Vorgabe des Aufwandskontos: Systemkonto „Aufwand“, sonst Konto 4000 (wenn aktive Aufwandskonten).
pub fn default_expense_account(konten: &[AccountOption], system: Option<i64>) -> Option<i64> {
    system
        .filter(|id| konten.iter().any(|k| k.id == *id))
        .or_else(|| konten.iter().find(|k| k.nummer == "4000").map(|k| k.id))
}

/// Meldung zum Vorsteuer-Code einer Position am Rechnungsdatum (ANF-012 AK-10); `None`, wenn er passt.
pub fn code_error(
    mwst_code_id: Option<i64>,
    codes: &[CodeOption],
    datum: &str,
    pflicht_ab: Option<&str>,
) -> Option<String> {
    let pflicht = vat_required_on(pflicht_ab, datum);
    match mwst_code_id {
        None if pflicht => Some("Bitte einen MWST-Code wählen (MWST-Pflicht).".into()),
        None => None,
        Some(_) if !pflicht => Some(format!(
            "Am {} gilt keine MWST-Pflicht. Bitte „ohne MWST“ wählen.",
            german_date(datum)
        )),
        Some(id) => match codes.iter().find(|c| c.id == id) {
            None => Some("Nur ein aktiver Vorsteuer-Code ist wählbar.".into()),
            Some(code) if !code_valid_on(code, datum) => Some(format!(
                "Der Code {} ist am {} nicht gültig.",
                code.code,
                german_date(datum)
            )),
            Some(_) => None,
        },
    }
}

/// Konto und Code einer Position prüfen; Fehler je Formularfeld.
pub fn check_position(
    konto_id: i64,
    mwst_code_id: Option<i64>,
    konten: &[AccountOption],
    codes: &[CodeOption],
    datum: &str,
    pflicht_ab: Option<&str>,
) -> FieldErrors {
    let mut errors = FieldErrors::new();
    if !konten.iter().any(|k| k.id == konto_id) {
        errors.insert(
            "konto_id".into(),
            "Bitte ein aktives Aufwandskonto wählen.".into(),
        );
    }
    if let Some(message) = code_error(mwst_code_id, codes, datum, pflicht_ab) {
        errors.insert("mwst_code_id".into(), message);
    }
    errors
}

// ---------------------------------------------------------------------------------------------
// Buchen
// ---------------------------------------------------------------------------------------------

/// Prüft den Entwurf vor dem Buchen (AK-7, AK-11). Liefert die erste Meldung.
pub fn booking_check(
    positions: &[CreditorPosition],
    totals: &CreditorTotals,
    konten: &[AccountOption],
    codes: &[CodeOption],
    datum: &str,
    pflicht_ab: Option<&str>,
) -> Result<(), String> {
    if positions.is_empty() {
        return Err(
            "Ohne Positionen kann nicht gebucht werden. Bitte zuerst eine Position erfassen."
                .into(),
        );
    }
    for (i, p) in positions.iter().enumerate() {
        let n = i + 1;
        if !konten.iter().any(|k| k.id == p.konto_id) {
            return Err(format!(
                "Position {n}: Das Konto {} ist inaktiv oder kein Aufwandskonto.",
                p.konto_nummer
            ));
        }
        if let Some(message) = code_error(p.mwst_code_id, codes, datum, pflicht_ab) {
            return Err(format!("Position {n}: {message}"));
        }
    }
    match totals.difference {
        None => return Err("Bitte zuerst den Betrag gemäss Rechnung eingeben.".into()),
        Some(0) => {}
        Some(d) => {
            return Err(format!(
                "Der Betrag gemäss Rechnung weicht um {} vom berechneten Total ab. Gebucht wird nur bei \
                 Differenz 0.00.",
                format_amount(d)
            ));
        }
    }
    if totals.total <= 0 {
        return Err("Der Gesamtbetrag ist null. Ein Beleg ohne Betrag wird nicht gebucht.".into());
    }
    Ok(())
}

/// Konten der Buchung aus den Systemkonten.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BookingAccounts {
    pub kreditoren: i64,
    pub rundung: Option<i64>,
}

/// Zeilen der Buchung (AK-10) und Gesamtbetrag in CHF.
///
/// Rechnung: Soll Aufwand je Position (netto, mit MWST-Basis und Projekt), Soll Vorsteuer je Code, Rundung
/// auf „Rundungsdifferenzen“, Haben Kreditoren. Gutschrift: Seiten vertauscht. Fremdwährung: jede Zeile in CHF,
/// Kreditoren als Summe.
pub fn booking_lines(
    art: &str,
    positions: &[CreditorPosition],
    totals: &CreditorTotals,
    codes: &[CodeOption],
    accounts: BookingAccounts,
    kurs: i64,
    projekt_id: Option<i64>,
) -> Result<(Vec<EffectiveLine>, i64), String> {
    // Aufwand auf der Soll-Seite der Rechnung
    let debit = art != "gutschrift";
    let mut lines: Vec<EffectiveLine> = Vec::new();
    for p in positions {
        let chf = to_chf(p.netto, kurs);
        if chf == 0 {
            continue;
        }
        lines.push(EffectiveLine {
            soll: debit,
            konto_id: p.konto_id,
            betrag: chf,
            mwst_code_id: p.mwst_code_id,
            mwst_basis: p.mwst_code_id.map(|_| chf),
            ist_mwst_zeile: false,
            projekt_id: p.projekt_id.or(projekt_id),
        });
    }
    for v in &totals.vat {
        let chf = to_chf(v.vat, kurs);
        if chf == 0 {
            continue;
        }
        let Some(code) = codes.iter().find(|c| c.id == v.code_id) else {
            return Err(format!("Der MWST-Code {} ist nicht mehr aktiv.", v.code));
        };
        lines.push(EffectiveLine {
            soll: debit,
            konto_id: code.konto_id,
            betrag: chf,
            mwst_code_id: None,
            mwst_basis: None,
            ist_mwst_zeile: true,
            projekt_id: None,
        });
    }
    if totals.rounding != 0 {
        let Some(konto_id) = accounts.rundung else {
            return Err(
                "Das Systemkonto „Rundungsdifferenzen“ fehlt (Einstellungen → Finanzen → Kontenplan → Systemkonten)."
                    .into(),
            );
        };
        let chf = to_chf(totals.rounding.abs(), kurs);
        if chf != 0 {
            // Höherer Rechnungsbetrag: zusätzlicher Aufwand auf der Seite des Aufwands
            lines.push(EffectiveLine {
                soll: if totals.rounding > 0 { debit } else { !debit },
                konto_id,
                betrag: chf,
                mwst_code_id: None,
                mwst_basis: None,
                ist_mwst_zeile: false,
                projekt_id: None,
            });
        }
    }
    let gegen: i64 = lines
        .iter()
        .map(|l| if l.soll == debit { l.betrag } else { -l.betrag })
        .sum();
    if gegen <= 0 {
        return Err("Der Gesamtbetrag in CHF ist null. Der Beleg wird nicht gebucht.".into());
    }
    lines.push(EffectiveLine {
        soll: !debit,
        konto_id: accounts.kreditoren,
        betrag: gegen,
        mwst_code_id: None,
        mwst_basis: None,
        ist_mwst_zeile: false,
        projekt_id,
    });
    Ok((lines, gegen))
}

/// Text der Buchung, z. B. „Eingangsrechnung K-2026-0001 Lieferant AG (RE-4711)“ (höchstens 200 Zeichen).
pub fn booking_text(art: &str, nummer: &str, lieferant: &str, rechnungsnummer: &str) -> String {
    let kind = if art == "gutschrift" {
        "Lieferantengutschrift"
    } else {
        "Eingangsrechnung"
    };
    format!("{kind} {nummer} {lieferant} ({rechnungsnummer})")
        .chars()
        .take(crate::backend::finance::journal::model::TEXT_MAX)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Liste
// ---------------------------------------------------------------------------------------------

/// Zeile der Liste (AK-1) und der Abschnitte in Projekt, Adresse und Einkaufsauftrag (AK-9, AK-15).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct CreditorRow {
    pub id: i64,
    pub nummer: Option<String>,
    pub art: String,
    pub rechnungsnummer: Option<String>,
    pub lieferant: String,
    pub einkaufsauftrag_id: Option<i64>,
    pub einkaufsauftrag_nummer: Option<String>,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub status: String,
    pub rechnungsdatum: String,
    pub faelligkeit: Option<String>,
    pub skonto_bis: Option<String>,
    pub skonto_laeuft: bool,
    pub waehrung: String,
    /// Festgeschrieben beim Buchen; im Entwurf aus den Positionen berechnet
    pub gesamtbetrag: Option<i64>,
    pub offener_betrag: i64,
    pub tage_ueberfaellig: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CreditorPage {
    pub rows: Vec<CreditorRow>,
    pub total: i64,
    pub page: i64,
}

impl CreditorPage {
    pub fn page_count(&self) -> i64 {
        ((self.total + PAGE_SIZE - 1) / PAGE_SIZE).max(1)
    }
}

/// Filter der Liste (AK-2), vollständig aus der Webadresse.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CreditorFilter {
    pub suche: String,
    pub status: String,
    pub von: String,
    pub bis: String,
    /// Fälligkeit bis (AK-2)
    pub faellig_bis: String,
    pub sortierung: String,
    pub richtung: String,
    pub seite: i64,
}

impl CreditorFilter {
    pub fn normalized(&self) -> Self {
        let date = |value: &str| {
            if is_valid_date(value) {
                value.to_string()
            } else {
                String::new()
            }
        };
        let sortierung = if SORT_COLUMNS.iter().any(|(k, _)| *k == self.sortierung) {
            self.sortierung.clone()
        } else {
            DEFAULT_SORT.into()
        };
        let richtung = match self.richtung.as_str() {
            "auf" | "ab" => self.richtung.clone(),
            _ => default_dir(&sortierung).into(),
        };
        Self {
            suche: self.suche.trim().chars().take(SEARCH_MAX).collect(),
            status: if STATUS_FILTERS.iter().any(|(k, _)| *k == self.status) {
                self.status.clone()
            } else {
                "offen".into()
            },
            von: date(&self.von),
            bis: date(&self.bis),
            faellig_bis: date(&self.faellig_bis),
            sortierung,
            richtung,
            seite: self.seite.max(1),
        }
    }
}

/// Standardrichtung je Spalte: Nummer und Rechnungsdatum absteigend, sonst aufsteigend.
pub fn default_dir(sort: &str) -> &'static str {
    match sort {
        "nummer" | "rechnungsdatum" => "ab",
        _ => "auf",
    }
}

/// Vergleich „bestellt / verrechnet“ eines Einkaufsauftrags (AK-9): verrechnet ist die Summe der nicht
/// stornierten Eingangsrechnungen, Gutschriften abgezogen. `true` im zweiten Wert, wenn mehr verrechnet als
/// bestellt ist.
pub fn billed_vs_ordered(rows: &[CreditorRow], bestellt: i64) -> (i64, bool) {
    let verrechnet: i64 = rows
        .iter()
        .filter(|r| r.status != "storniert")
        .map(|r| {
            crate::backend::finance::debtors::model::signed(&r.art, r.gesamtbetrag.unwrap_or(0))
        })
        .sum();
    (verrechnet, verrechnet > bestellt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::finance::debtors::model::RATE_FACTOR;

    fn pos(
        id: i64,
        netto: i64,
        code: Option<(i64, i64)>,
        projekt: Option<i64>,
    ) -> CreditorPosition {
        let mwst = code.map_or(0, |(_, satz)| split_amount(netto, Some(satz), false).1);
        CreditorPosition {
            id,
            bezeichnung: format!("Position {id}"),
            konto_id: 40,
            konto_nummer: "4000".into(),
            konto_bezeichnung: "Material".into(),
            mwst_code_id: code.map(|(id, _)| id),
            mwst_code: code.map(|(id, _)| format!("V{id}")),
            mwst_satz: code.map(|(_, s)| s),
            netto,
            mwst_betrag: mwst,
            projekt_id: projekt,
            projekt_nummer: None,
            einkaufsposition_id: None,
        }
    }

    fn code(id: i64, satz: i64) -> CodeOption {
        CodeOption {
            id,
            code: format!("V{id}"),
            bezeichnung: "Vorsteuer".into(),
            satz,
            gueltig_ab: "2024-01-01".into(),
            gueltig_bis: None,
            konto_id: 1170,
        }
    }

    fn account(id: i64, nummer: &str) -> AccountOption {
        AccountOption {
            id,
            nummer: nummer.into(),
            bezeichnung: "Aufwand".into(),
            vorgabe_mwst: None,
        }
    }

    const ACCOUNTS: BookingAccounts = BookingAccounts {
        kreditoren: 2000,
        rundung: Some(6990),
    };

    #[test]
    fn split_netto_and_brutto() {
        assert_eq!(split_amount(10_000, Some(810), false), (10_000, 810));
        assert_eq!(split_amount(10_810, Some(810), true), (10_000, 810));
        assert_eq!(split_amount(5_000, None, true), (5_000, 0));
    }

    #[test]
    fn totals_with_control_amount_and_rounding() {
        let positions = vec![pos(1, 9_999, Some((1, 810)), None)];
        // 99.99 + 8.10 = 108.09
        let t = totals(&positions, 0, Some(10_810));
        assert_eq!(t.total, 10_809);
        assert_eq!(t.difference, Some(1));
        assert!(!t.is_balanced());
        assert_eq!(t.roundable(), Some(1));
        let t = totals(&positions, 1, Some(10_810));
        assert!(t.is_balanced());
        assert_eq!(t.roundable(), None);
        // Mehr als 0.05 lässt sich nicht ausgleichen
        let t = totals(&positions, 0, Some(10_900));
        assert_eq!(t.roundable(), None);
        assert_eq!(totals(&positions, 0, None).difference, None);
    }

    #[test]
    fn invoice_booking_debits_expense_and_input_tax() {
        let positions = vec![
            pos(1, 10_000, Some((1, 810)), Some(5)),
            pos(2, 2_000, None, None),
        ];
        let t = totals(&positions, 0, Some(12_810));
        let (lines, chf) = booking_lines(
            "rechnung",
            &positions,
            &t,
            &[code(1, 810)],
            ACCOUNTS,
            RATE_FACTOR,
            Some(9),
        )
        .unwrap();
        assert_eq!(chf, 12_810);
        let (soll, haben) = crate::backend::finance::journal::model::totals(&lines);
        assert_eq!(soll, haben);
        let last = lines.last().unwrap();
        assert_eq!(
            (last.konto_id, last.soll, last.betrag),
            (2000, false, 12_810)
        );
        // Projekt der Position vor dem Projekt der Rechnung
        assert_eq!(lines[0].projekt_id, Some(5));
        assert_eq!(lines[1].projekt_id, Some(9));
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 1170 && l.ist_mwst_zeile && l.soll && l.betrag == 810)
        );
    }

    #[test]
    fn credit_note_and_rounding_sides() {
        let positions = vec![pos(1, 9_999, Some((1, 810)), None)];
        let t = totals(&positions, 1, Some(10_810));
        let (lines, chf) = booking_lines(
            "gutschrift",
            &positions,
            &t,
            &[code(1, 810)],
            ACCOUNTS,
            RATE_FACTOR,
            None,
        )
        .unwrap();
        assert_eq!(chf, 10_810);
        assert!(lines.last().unwrap().soll, "Kreditoren im Soll");
        // Rundung +0.01 bei der Gutschrift auf der Haben-Seite (wie der Aufwand)
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 6990 && !l.soll && l.betrag == 1)
        );
        let (soll, haben) = crate::backend::finance::journal::model::totals(&lines);
        assert_eq!(soll, haben);
        // Ohne Rundungskonto Meldung
        let no_rounding = BookingAccounts {
            kreditoren: 2000,
            rundung: None,
        };
        assert!(
            booking_lines(
                "rechnung",
                &positions,
                &t,
                &[code(1, 810)],
                no_rounding,
                RATE_FACTOR,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn foreign_currency_balances() {
        let positions = vec![pos(1, 100_000, Some((1, 810)), None)];
        let t = totals(&positions, 0, Some(108_100));
        let (lines, chf) = booking_lines(
            "rechnung",
            &positions,
            &t,
            &[code(1, 810)],
            ACCOUNTS,
            950_000,
            None,
        )
        .unwrap();
        assert_eq!(chf, 102_695);
        let (soll, haben) = crate::backend::finance::journal::model::totals(&lines);
        assert_eq!(soll, haben);
    }

    #[test]
    fn booking_check_requires_balance_and_valid_codes() {
        let konten = vec![account(40, "4000")];
        let codes = vec![code(1, 810)];
        let positions = vec![pos(1, 10_000, Some((1, 810)), None)];
        let ok = totals(&positions, 0, Some(10_810));
        assert!(
            booking_check(
                &positions,
                &ok,
                &konten,
                &codes,
                "2026-05-01",
                Some("2024-01-01")
            )
            .is_ok()
        );
        let off = totals(&positions, 0, Some(10_000));
        assert!(
            booking_check(
                &positions,
                &off,
                &konten,
                &codes,
                "2026-05-01",
                Some("2024-01-01")
            )
            .unwrap_err()
            .contains("-8.10")
        );
        let missing = totals(&positions, 0, None);
        assert!(
            booking_check(
                &positions,
                &missing,
                &konten,
                &codes,
                "2026-05-01",
                Some("2024-01-01")
            )
            .is_err()
        );
        assert!(
            booking_check(&positions, &ok, &konten, &codes, "2026-05-01", None)
                .unwrap_err()
                .starts_with("Position 1:")
        );
        assert!(booking_check(&[], &ok, &konten, &codes, "2026-05-01", None).is_err());
    }

    #[test]
    fn input_validation() {
        let mut input = CreditorInput::new_default("rechnung", "2026-11-27");
        let e = input.validate();
        assert!(e.contains_key("adresse_id") && e.contains_key("rechnungsnummer"));
        assert!(e.contains_key("betrag_gemaess_rechnung"));
        input.adresse_id = "3".into();
        input.rechnungsnummer = "RE-1".into();
        input.betrag_gemaess_rechnung = "108.10".into();
        assert!(input.validate().is_empty());
        input.faelligkeit = "2026-11-01".into();
        assert!(input.validate().contains_key("faelligkeit"));
        let n = CreditorInput {
            zahlungsreferenz: "21 00000 00003".into(),
            ..Default::default()
        }
        .normalized();
        assert_eq!(n.zahlungsreferenz, "210000000003");

        let current = CreditorInput {
            adresse_id: "3".into(),
            ..CreditorInput::new_default("rechnung", "2026-11-01")
        };
        let changed = CreditorInput {
            adresse_id: "9".into(),
            faelligkeit: "2026-12-31".into(),
            bemerkung: "Neu".into(),
            ..current.clone()
        }
        .restricted_to_booked(&current);
        assert_eq!(
            (
                changed.adresse_id.as_str(),
                changed.faelligkeit.as_str(),
                changed.bemerkung.as_str()
            ),
            ("3", "2026-12-31", "Neu")
        );
    }

    #[test]
    fn position_validation() {
        let input = PositionInput {
            bezeichnung: "Material".into(),
            konto_id: "40".into(),
            mwst_code_id: "".into(),
            betrag: "108.10".into(),
            betragsart: "brutto".into(),
            projekt_id: "".into(),
        };
        let v = input.validate().unwrap();
        assert!(v.brutto);
        assert_eq!(
            (v.betrag, v.mwst_code_id, v.projekt_id),
            (10_810, None, None)
        );
        let bad = PositionInput {
            betrag: "-5".into(),
            betragsart: "x".into(),
            ..input
        };
        let e = bad.validate().unwrap_err();
        assert!(e.contains_key("betrag") && e.contains_key("betragsart"));
    }

    #[test]
    fn billed_versus_ordered() {
        let row = |art: &str, status: &str, betrag: i64| CreditorRow {
            id: 1,
            nummer: None,
            art: art.into(),
            rechnungsnummer: Some("RE".into()),
            lieferant: "L".into(),
            einkaufsauftrag_id: Some(1),
            einkaufsauftrag_nummer: None,
            projekt_id: None,
            projekt_nummer: None,
            status: status.into(),
            rechnungsdatum: "2026-01-01".into(),
            faelligkeit: None,
            skonto_bis: None,
            skonto_laeuft: false,
            waehrung: "CHF".into(),
            gesamtbetrag: Some(betrag),
            offener_betrag: betrag,
            tage_ueberfaellig: None,
        };
        let rows = vec![
            row("rechnung", "offen", 800),
            row("rechnung", "storniert", 500),
            row("gutschrift", "offen", 100),
            row("rechnung", "entwurf", 400),
        ];
        assert_eq!(billed_vs_ordered(&rows, 1_000), (1_100, true));
        assert_eq!(billed_vs_ordered(&rows, 1_100), (1_100, false));
    }

    #[test]
    fn numbers_and_filter() {
        assert_eq!(number_for(2026, 7), "K-2026-0007");
        let f = CreditorFilter {
            faellig_bis: "2026-12-31".into(),
            sortierung: "skonto".into(),
            ..Default::default()
        }
        .normalized();
        assert_eq!(
            (
                f.status.as_str(),
                f.faellig_bis.as_str(),
                f.richtung.as_str()
            ),
            ("offen", "2026-12-31", "auf")
        );
        assert_eq!(
            default_expense_account(&[account(1, "4000")], None),
            Some(1)
        );
        assert!(
            booking_text("gutschrift", "K-2026-0001", "L", "G-1")
                .starts_with("Lieferantengutschrift")
        );
    }
}
