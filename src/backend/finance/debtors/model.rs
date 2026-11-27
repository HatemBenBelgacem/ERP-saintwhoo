//! Debitoren – Ausgangsrechnungen und Gutschriften (ANF-018): Datentypen, Prüfregeln, Summen und die
//! Zeilen der Buchung. Wird auf dem Server und im Browser verwendet.
//!
//! Beträge und Mengen sind ganze Zahlen in kleinsten Einheiten wie im Einkauf (ANF-004): Menge in
//! Tausendsteln, Preise in Rappen, Prozente in Hundertsteln, Kurse in Millionsteln. Beträge einer
//! Gutschrift sind positiv gespeichert; die Art bestimmt die Richtung der Buchung und das Vorzeichen in
//! der Anzeige.

use serde::{Deserialize, Serialize};

use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::journal::model::{
    AccountOption, CodeOption, EffectiveLine, code_valid_on, div_round,
};
use crate::backend::projects::model::is_valid_date;
use crate::backend::purchase_orders::model::{
    UNITS, format_amount, format_fixed, format_trimmed, parse_fixed, round_to_5,
};
pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

/// Webadresse der Liste (Reiter „Debitoren“).
pub const DEBTORS_URL: &str = "/finanzen/debitoren";

/// Einträge je Seite (AK-2: ab 50 Einträgen seitenweise).
pub const PAGE_SIZE: i64 = 50;
/// Länge der Referenz des Kunden.
pub const REFERENCE_MAX: usize = 100;
/// Länge der Texte für den Druck und der Bemerkung.
pub const TEXT_MAX: usize = 2000;
/// Suchbegriff der Liste.
pub const SEARCH_MAX: usize = 100;
/// Kurse in Millionsteln (ANF-014).
pub const RATE_FACTOR: i64 = 1_000_000;

pub const KINDS: &[(&str, &str)] = &[("rechnung", "Rechnung"), ("gutschrift", "Gutschrift")];

pub const STATUSES: &[(&str, &str)] = &[
    ("entwurf", "Entwurf"),
    ("offen", "Offen"),
    ("teilweise_bezahlt", "Teilweise bezahlt"),
    ("bezahlt", "Bezahlt"),
    ("storniert", "Storniert"),
];

/// Status mit offenem Betrag: Grundlage für „überfällig“ (AK-3) und den Standardfilter (AK-2).
pub const OPEN_STATUSES: &[&str] = &["offen", "teilweise_bezahlt"];

/// Auswahl im Statusfilter: Schlüssel in der Webadresse → Beschriftung. `offen` ist die Vorgabe (AK-2).
pub const STATUS_FILTERS: &[(&str, &str)] = &[
    ("offen", "Offen und teilweise bezahlt"),
    ("alle", "Alle"),
    ("entwurf", "Entwurf"),
    ("bezahlt", "Bezahlt"),
    ("storniert", "Storniert"),
];

/// Sortierbare Spalten (AK-2): Schlüssel in der Webadresse → Beschriftung.
pub const SORT_COLUMNS: &[(&str, &str)] = &[
    ("nummer", "Nummer"),
    ("kunde", "Kunde"),
    ("projekt", "Projekt"),
    ("rechnungsdatum", "Rechnungsdatum"),
    ("faelligkeit", "Fälligkeit"),
    ("betrag", "Gesamtbetrag"),
    ("offen", "Offen"),
    ("status", "Status"),
];

/// Standardsortierung (AK-2): nach Fälligkeit.
pub const DEFAULT_SORT: &str = "faelligkeit";

fn label_in(list: &'static [(&'static str, &'static str)], key: &str) -> &'static str {
    list.iter()
        .find(|(k, _)| *k == key)
        .map_or("–", |(_, label)| label)
}

pub fn kind_label(key: &str) -> &'static str {
    label_in(KINDS, key)
}

pub fn status_label(key: &str) -> &'static str {
    label_in(STATUSES, key)
}

/// daisyUI-Klassen der Status-Plakette (vollständig ausgeschrieben, damit Tailwind sie findet).
pub fn status_badge(status: &str) -> &'static str {
    match status {
        "offen" => "badge badge-sm badge-primary badge-soft",
        "teilweise_bezahlt" => "badge badge-sm badge-warning badge-soft",
        "bezahlt" => "badge badge-sm badge-success badge-soft",
        "storniert" => "badge badge-sm badge-error badge-soft",
        _ => "badge badge-sm badge-ghost",
    }
}

/// Betrag mit Vorzeichen der Art: Gutschriften erscheinen negativ.
pub fn signed(art: &str, amount: i64) -> i64 {
    if art == "gutschrift" { -amount } else { amount }
}

/// Umrechnung in CHF mit dem Kurs in Millionsteln, auf Rappen gerundet (ANF-014).
pub fn to_chf(amount: i64, kurs: i64) -> i64 {
    div_round(
        i128::from(amount) * i128::from(kurs),
        i128::from(RATE_FACTOR),
    ) as i64
}

/// Kurs zur Anzeige, z. B. 950000 → „0.95“.
pub fn rate_text(kurs: i64) -> String {
    format_trimmed(kurs, 6)
}

/// Nummer eines Belegs, z. B. `R-2026-0001` bzw. `G-2026-0001` (Offene Frage, geklärt 2026-11-27).
pub fn number_for(art: &str, jahr: i32, laufnummer: i32) -> String {
    let prefix = if art == "gutschrift" { "G" } else { "R" };
    format!("{prefix}-{jahr}-{laufnummer:04}")
}

// ---------------------------------------------------------------------------------------------
// Positionen und Summen
// ---------------------------------------------------------------------------------------------

/// Eine Position (Werte in kleinsten Einheiten, siehe Moduldokumentation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct InvoicePosition {
    pub id: i64,
    pub bezeichnung: String,
    /// Tausendstel
    pub menge: i64,
    pub einheit: String,
    /// Rappen
    pub einzelpreis: i64,
    /// Hundertstel Prozent
    pub rabatt_prozent: i64,
    pub konto_id: i64,
    pub konto_nummer: String,
    pub konto_bezeichnung: String,
    pub mwst_code_id: Option<i64>,
    pub mwst_code: Option<String>,
    /// Satz des Codes in Hundertstel Prozent; `None` ohne Code
    pub mwst_satz: Option<i64>,
}

impl InvoicePosition {
    /// Positionsbetrag ohne MWST in Rappen = Menge × Einzelpreis × (1 − Rabatt/100), auf 0.01 gerundet
    /// (wie ANF-004).
    pub fn amount(&self) -> i64 {
        line_amount(self.menge, self.einzelpreis, self.rabatt_prozent)
    }

    pub fn to_input(&self) -> PositionInput {
        PositionInput {
            bezeichnung: self.bezeichnung.clone(),
            menge: format_trimmed(self.menge, 3),
            einheit: self.einheit.clone(),
            einzelpreis: format_fixed(self.einzelpreis, 2),
            rabatt_prozent: format_trimmed(self.rabatt_prozent, 2),
            konto_id: self.konto_id.to_string(),
            mwst_code_id: self.mwst_code_id.map(|i| i.to_string()).unwrap_or_default(),
        }
    }
}

/// Betrag einer Position in Rappen (Menge in Tausendsteln, Preis in Rappen, Rabatt in Hundertstel Prozent).
pub fn line_amount(menge: i64, einzelpreis: i64, rabatt: i64) -> i64 {
    // 10⁻³ × 10⁻² × 10⁻⁴ = 10⁻⁹ → durch 10⁷ teilen für Rappen
    let num = i128::from(menge) * i128::from(einzelpreis) * i128::from(10_000 - rabatt);
    div_round(num, 10_000_000) as i64
}

/// MWST je Code (AK-6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VatByCode {
    pub code_id: i64,
    pub code: String,
    /// Hundertstel Prozent
    pub rate: i64,
    /// Summe der Positionsbeträge mit diesem Code (Rappen)
    pub base: i64,
    /// MWST-Betrag (Rappen)
    pub vat: i64,
}

/// Summen einer Rechnung (AK-6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvoiceTotals {
    pub subtotal: i64,
    /// Nach Satz absteigend, dann nach Code
    pub vat: Vec<VatByCode>,
    /// Differenz durch die Rundung auf 0.05 (nur CHF)
    pub rounding: i64,
    pub total: i64,
}

/// Zwischensumme, MWST je Code und Gesamtbetrag wie in ANF-004; in CHF wird der Gesamtbetrag auf 0.05
/// gerundet und die Differenz als Rundung ausgewiesen.
pub fn totals(positions: &[InvoicePosition], currency: &str) -> InvoiceTotals {
    let subtotal: i64 = positions.iter().map(InvoicePosition::amount).sum();
    let mut vat: Vec<VatByCode> = Vec::new();
    for p in positions {
        let (Some(code_id), Some(rate)) = (p.mwst_code_id, p.mwst_satz) else {
            continue;
        };
        match vat.iter_mut().find(|l| l.code_id == code_id) {
            Some(line) => line.base += p.amount(),
            None => vat.push(VatByCode {
                code_id,
                code: p.mwst_code.clone().unwrap_or_default(),
                rate,
                base: p.amount(),
                vat: 0,
            }),
        }
    }
    vat.sort_by(|a, b| b.rate.cmp(&a.rate).then_with(|| a.code.cmp(&b.code)));
    for line in &mut vat {
        line.vat = div_round(i128::from(line.base) * i128::from(line.rate), 10_000) as i64;
    }
    let raw = subtotal + vat.iter().map(|l| l.vat).sum::<i64>();
    let total = if currency == "CHF" {
        round_to_5(raw)
    } else {
        raw
    };
    InvoiceTotals {
        subtotal,
        vat,
        rounding: total - raw,
        total,
    }
}

// ---------------------------------------------------------------------------------------------
// Rechnung (Kopf)
// ---------------------------------------------------------------------------------------------

/// Kopf einer Rechnung bzw. Gutschrift. Datum als `JJJJ-MM-TT`, Beträge in Rappen, Kurs in Millionsteln.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Invoice {
    pub id: i64,
    pub nummer: Option<String>,
    pub art: String,
    pub adresse_id: i64,
    pub kunde_nummer: String,
    pub kunde: String,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub projekt_name: Option<String>,
    pub bezug_rechnung_id: Option<i64>,
    pub bezug_nummer: Option<String>,
    pub status: String,
    pub rechnungsdatum: String,
    pub zahlungsbedingung_id: i64,
    pub zahlungsbedingung: String,
    pub zahlungstext: String,
    pub faelligkeit: Option<String>,
    pub skonto_bis: Option<String>,
    /// Hundertstel Prozent
    pub skonto_prozent: Option<i64>,
    pub waehrung: String,
    /// Millionstel; im Entwurf leer (wird beim Buchen bestimmt)
    pub kurs: Option<i64>,
    pub referenz_kunde: Option<String>,
    pub einleitungstext: Option<String>,
    pub schlusstext: Option<String>,
    pub bemerkung: Option<String>,
    pub gesamtbetrag: Option<i64>,
    pub gesamtbetrag_chf: Option<i64>,
    pub offener_betrag: i64,
    pub buchung_id: Option<i64>,
    /// Gegenbuchung beim Storno (AK-13)
    pub storno_buchung_id: Option<i64>,
    /// Tage seit Fälligkeit, nur bei offenen, überfälligen Belegen (AK-3)
    pub tage_ueberfaellig: Option<i32>,
    pub erstellt_am: String,
    pub geaendert_am: String,
}

impl Invoice {
    pub fn is_draft(&self) -> bool {
        self.status == "entwurf"
    }

    pub fn is_credit_note(&self) -> bool {
        self.art == "gutschrift"
    }

    /// Kopfdaten noch (teilweise) änderbar: im Entwurf alles, danach nur Texte (AK-10).
    pub fn is_editable(&self) -> bool {
        self.status != "storniert"
    }

    /// Stornieren nur offen und ohne Zahlungen (AK-13).
    pub fn can_storno(&self) -> bool {
        self.status == "offen" && self.gesamtbetrag == Some(self.offener_betrag)
    }

    /// Gutschrift aus einer gebuchten Rechnung erstellen (AK-14).
    pub fn can_credit(&self) -> bool {
        self.art == "rechnung"
            && matches!(
                self.status.as_str(),
                "offen" | "teilweise_bezahlt" | "bezahlt"
            )
    }

    /// Bezeichnung für Titel und Pfadleiste: Nummer oder „Entwurf“.
    pub fn label(&self) -> String {
        match &self.nummer {
            Some(n) => n.clone(),
            None => format!("Entwurf {}", kind_label(&self.art)),
        }
    }

    pub fn customer(&self) -> PickOption {
        PickOption {
            id: self.adresse_id,
            nummer: self.kunde_nummer.clone(),
            name: self.kunde.clone(),
        }
    }

    pub fn project(&self) -> Option<PickOption> {
        Some(PickOption {
            id: self.projekt_id?,
            nummer: self.projekt_nummer.clone()?,
            name: self.projekt_name.clone()?,
        })
    }

    pub fn to_input(&self) -> InvoiceInput {
        let s = |v: &Option<String>| v.clone().unwrap_or_default();
        InvoiceInput {
            art: self.art.clone(),
            adresse_id: self.adresse_id.to_string(),
            projekt_id: self.projekt_id.map(|i| i.to_string()).unwrap_or_default(),
            rechnungsdatum: self.rechnungsdatum.clone(),
            referenz_kunde: s(&self.referenz_kunde),
            waehrung: self.waehrung.clone(),
            zahlungsbedingung_id: self.zahlungsbedingung_id.to_string(),
            einleitungstext: s(&self.einleitungstext),
            schlusstext: s(&self.schlusstext),
            bemerkung: s(&self.bemerkung),
        }
    }
}

/// Rechnung mit Positionen (Detailansicht und Druck).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvoiceDetail {
    pub invoice: Invoice,
    pub positions: Vec<InvoicePosition>,
}

impl InvoiceDetail {
    pub fn totals(&self) -> InvoiceTotals {
        totals(&self.positions, &self.invoice.waehrung)
    }
}

/// Anschrift des Kunden für den Druck (AK-11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct CustomerAddress {
    pub anzeigename: String,
    pub zusatz: Option<String>,
    pub strasse: Option<String>,
    pub postfach: Option<String>,
    pub plz: Option<String>,
    pub ort: Option<String>,
    pub land: String,
}

impl CustomerAddress {
    /// Anschriftzeilen: Name, Zusatz, Strasse, Postfach, PLZ Ort, Land (ausser Schweiz).
    pub fn lines(&self) -> Vec<String> {
        let mut lines = vec![self.anzeigename.clone()];
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
            lines.push(crate::backend::addresses::model::country_name(&self.land).to_string());
        }
        lines
    }
}

/// Bankverbindung für den Druck (Standard-Bankkonto, ANF-015).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PrintBank {
    pub bezeichnung: String,
    pub bank: Option<String>,
    pub iban: Option<String>,
    pub bic: Option<String>,
}

/// Alles für die Druckansicht (AK-11). Firma und Bank werden beim Drucken gelesen (ANF-025 AK-9).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvoicePrint {
    pub detail: InvoiceDetail,
    pub company: Option<crate::backend::company::model::Company>,
    pub customer: CustomerAddress,
    pub bank: Option<PrintBank>,
    /// MWST-Nummer der Firma (`CHE-… MWST`), nur bei eingeschalteter MWST-Pflicht
    pub mwst_nummer: Option<String>,
}

// ---------------------------------------------------------------------------------------------
// Eingaben
// ---------------------------------------------------------------------------------------------

/// Formulareingabe für die Kopfdaten (AK-4, AK-10).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InvoiceInput {
    pub art: String,
    pub adresse_id: String,
    pub projekt_id: String,
    pub rechnungsdatum: String,
    pub referenz_kunde: String,
    pub waehrung: String,
    /// Leer = Vorgabe der Adresse bzw. Standard (AK-4)
    pub zahlungsbedingung_id: String,
    pub einleitungstext: String,
    pub schlusstext: String,
    pub bemerkung: String,
}

impl InvoiceInput {
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
            projekt_id: self.projekt_id.trim().into(),
            rechnungsdatum: self.rechnungsdatum.trim().into(),
            referenz_kunde: self.referenz_kunde.trim().into(),
            waehrung: self.waehrung.trim().to_uppercase(),
            zahlungsbedingung_id: self.zahlungsbedingung_id.trim().into(),
            einleitungstext: self.einleitungstext.trim().into(),
            schlusstext: self.schlusstext.trim().into(),
            bemerkung: self.bemerkung.trim().into(),
        }
    }

    /// Nach dem Buchen sind nur Referenz, Bemerkung und die Texte für den Druck änderbar (AK-10):
    /// alle anderen Felder kommen aus dem gespeicherten Beleg.
    pub fn restricted_to_booked(&self, current: &InvoiceInput) -> Self {
        Self {
            referenz_kunde: self.referenz_kunde.clone(),
            einleitungstext: self.einleitungstext.clone(),
            schlusstext: self.schlusstext.clone(),
            bemerkung: self.bemerkung.clone(),
            ..current.clone()
        }
    }

    /// Prüft die normalisierte Eingabe. Leere Map = gültig.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |f: &str, m: &str| {
            errors.entry(f.into()).or_insert_with(|| m.into());
        };
        if !KINDS.iter().any(|(k, _)| *k == self.art) {
            err("art", "Bitte Rechnung oder Gutschrift wählen.");
        }
        if self.adresse_id.is_empty() {
            err("adresse_id", "Bitte einen Kunden wählen.");
        } else if self.adresse_id.parse::<i64>().is_err() {
            err("adresse_id", "Ungültiger Kunde.");
        }
        if !self.projekt_id.is_empty() && self.projekt_id.parse::<i64>().is_err() {
            err("projekt_id", "Ungültiges Projekt.");
        }
        if self.rechnungsdatum.is_empty() {
            err("rechnungsdatum", "Bitte ein Rechnungsdatum eingeben.");
        } else if !is_valid_date(&self.rechnungsdatum) {
            err("rechnungsdatum", "Bitte ein gültiges Datum eingeben.");
        }
        if self.referenz_kunde.chars().count() > REFERENCE_MAX {
            err("referenz_kunde", "Höchstens 100 Zeichen.");
        }
        // Ob die Währung aktiv ist, prüft der Server mit der Datenbank (ANF-014 AK-6).
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
        for (field, value) in [
            ("einleitungstext", &self.einleitungstext),
            ("schlusstext", &self.schlusstext),
            ("bemerkung", &self.bemerkung),
        ] {
            if value.chars().count() > TEXT_MAX {
                err(field, "Höchstens 2000 Zeichen.");
            }
        }
        errors
    }

    pub fn customer_id(&self) -> Option<i64> {
        self.adresse_id.parse().ok()
    }

    pub fn project_id(&self) -> Option<i64> {
        self.projekt_id.parse().ok()
    }

    pub fn term_id(&self) -> Option<i64> {
        self.zahlungsbedingung_id.parse().ok()
    }
}

/// Formulareingabe für eine Position (AK-5).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PositionInput {
    pub bezeichnung: String,
    pub menge: String,
    pub einheit: String,
    pub einzelpreis: String,
    pub rabatt_prozent: String,
    pub konto_id: String,
    /// Leer = ohne MWST (nur ohne MWST-Pflicht erlaubt)
    pub mwst_code_id: String,
}

/// Geprüfte Werte einer Position in kleinsten Einheiten.
#[derive(Clone, Debug, PartialEq)]
pub struct PositionValues {
    pub menge: i64,
    pub einzelpreis: i64,
    pub rabatt_prozent: i64,
    pub konto_id: i64,
    pub mwst_code_id: Option<i64>,
}

impl PositionInput {
    pub fn new_default() -> Self {
        Self {
            menge: "1".into(),
            einheit: "stk".into(),
            ..Default::default()
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            bezeichnung: self.bezeichnung.trim().into(),
            menge: self.menge.trim().into(),
            einheit: self.einheit.trim().into(),
            einzelpreis: self.einzelpreis.trim().into(),
            rabatt_prozent: self.rabatt_prozent.trim().into(),
            konto_id: self.konto_id.trim().into(),
            mwst_code_id: self.mwst_code_id.trim().into(),
        }
    }

    /// Prüft die normalisierte Eingabe (AK-5). Ob Konto und Code passen, prüft `check_position`.
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
        let menge = parse_fixed(&self.menge, 3);
        match menge {
            None if self.menge.is_empty() => err("menge", "Bitte eine Menge eingeben."),
            None => err(
                "menge",
                "Bitte eine Zahl mit höchstens 3 Nachkommastellen eingeben.",
            ),
            Some(0) => err("menge", "Die Menge muss grösser als 0 sein."),
            Some(_) => {}
        }
        if !UNITS.iter().any(|(k, _)| *k == self.einheit) {
            err("einheit", "Bitte eine Einheit aus der Liste wählen.");
        }
        let einzelpreis = parse_fixed(&self.einzelpreis, 2);
        if einzelpreis.is_none() {
            err(
                "einzelpreis",
                if self.einzelpreis.is_empty() {
                    "Bitte einen Einzelpreis eingeben."
                } else {
                    "Bitte einen Betrag mit höchstens 2 Nachkommastellen eingeben (nicht negativ)."
                },
            );
        }
        let rabatt = if self.rabatt_prozent.is_empty() {
            Some(0)
        } else {
            parse_fixed(&self.rabatt_prozent, 2)
        };
        match rabatt {
            Some(r) if r <= 10_000 => {}
            _ => err(
                "rabatt_prozent",
                "Der Rabatt muss zwischen 0 und 100 % liegen.",
            ),
        }
        let konto_id = self.konto_id.parse::<i64>().ok();
        if konto_id.is_none() {
            err("konto_id", "Bitte ein Ertragskonto wählen.");
        }
        let mwst_code_id = if self.mwst_code_id.is_empty() {
            Some(None)
        } else {
            self.mwst_code_id.parse::<i64>().ok().map(Some)
        };
        if mwst_code_id.is_none() {
            err("mwst_code_id", "Bitte einen MWST-Code wählen.");
        }
        match (menge, einzelpreis, rabatt, konto_id, mwst_code_id) {
            (Some(menge), Some(einzelpreis), Some(rabatt_prozent), Some(konto_id), Some(code))
                if errors.is_empty() =>
            {
                Ok(PositionValues {
                    menge,
                    einzelpreis,
                    rabatt_prozent,
                    konto_id,
                    mwst_code_id: code,
                })
            }
            _ => Err(errors),
        }
    }
}

/// Zahlungsbedingung für die Auswahl.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TermOption {
    pub id: i64,
    pub bezeichnung: String,
    pub standard: bool,
}

/// Vorgaben für das Formular der Kopfdaten: heutiges Datum und aktive Zahlungsbedingungen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvoiceFormData {
    pub heute: String,
    pub terms: Vec<TermOption>,
}

/// Auswahl für das Positionsformular: aktive Ertragskonten, Umsatzsteuer-Codes am Rechnungsdatum,
/// Vorgabe des Ertragskontos (AK-5) und ob die MWST-Pflicht am Rechnungsdatum gilt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionFormData {
    pub konten: Vec<AccountOption>,
    pub codes: Vec<CodeOption>,
    pub vorgabe_konto_id: Option<i64>,
    pub mwst_pflicht: bool,
    pub rechnungsdatum: String,
}

/// MWST-Pflicht am Datum: eingeschaltet und Beginn erreicht (ANF-012 AK-8, AK-10).
pub fn vat_required_on(pflicht_ab: Option<&str>, datum: &str) -> bool {
    pflicht_ab.is_some_and(|ab| ab <= datum)
}

/// Vorgabe des Ertragskontos (AK-5): Systemkonto „Ertrag“, sonst Konto 3400, wenn sie aktive Ertragskonten sind.
pub fn default_income_account(konten: &[AccountOption], system: Option<i64>) -> Option<i64> {
    system
        .filter(|id| konten.iter().any(|k| k.id == *id))
        .or_else(|| konten.iter().find(|k| k.nummer == "3400").map(|k| k.id))
}

/// Prüft Ertragskonto und MWST-Code einer Position gegen die Stammdaten am Rechnungsdatum.
/// `konten`: aktive Ertragskonten, `codes`: aktive Umsatzsteuer-Codes. Fehler je Formularfeld.
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
            "Bitte ein aktives Ertragskonto wählen.".into(),
        );
    }
    if let Some(message) = code_error(mwst_code_id, codes, datum, pflicht_ab) {
        errors.insert("mwst_code_id".into(), message);
    }
    errors
}

/// Meldung zum MWST-Code einer Position am Rechnungsdatum; `None`, wenn er passt.
fn code_error(
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
            None => Some("Nur ein aktiver Umsatzsteuer-Code ist wählbar.".into()),
            Some(code) if !code_valid_on(code, datum) => Some(format!(
                "Der Code {} ist am {} nicht gültig.",
                code.code,
                german_date(datum)
            )),
            Some(_) => None,
        },
    }
}

// ---------------------------------------------------------------------------------------------
// Buchen
// ---------------------------------------------------------------------------------------------

/// Prüft alle Positionen vor dem Buchen (AK-9): Ertragskonto aktiv, MWST-Code passend zum Datum,
/// Gesamtbetrag grösser als null. Liefert die erste Meldung.
pub fn booking_check(
    positions: &[InvoicePosition],
    konten: &[AccountOption],
    codes: &[CodeOption],
    datum: &str,
    pflicht_ab: Option<&str>,
    total: i64,
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
                "Position {n}: Das Konto {} ist inaktiv oder kein Ertragskonto.",
                p.konto_nummer
            ));
        }
        if let Some(message) = code_error(p.mwst_code_id, codes, datum, pflicht_ab) {
            return Err(format!("Position {n}: {message}"));
        }
    }
    if total <= 0 {
        return Err("Der Gesamtbetrag ist null. Ein Beleg ohne Betrag wird nicht gebucht.".into());
    }
    Ok(())
}

/// Konten für die Buchung aus den Systemkonten (ANF-011).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BookingAccounts {
    pub debitoren: i64,
    /// Nur nötig, wenn eine Rundungsdifferenz entsteht
    pub rundung: Option<i64>,
}

/// Zeilen der Buchung (AK-8) und Gesamtbetrag in CHF.
///
/// Rechnung: Soll Debitoren (Gesamtbetrag), Haben Ertragskonto je Konto und Code (netto), Haben MWST-Konto
/// je Code, Rundung auf dem Konto „Rundungsdifferenzen“. Gutschrift: alle Seiten vertauscht.
/// Fremdwährung: jede Zeile einzeln in CHF umgerechnet; die Debitorenzeile ist die Summe der umgerechneten
/// Zeilen, damit Soll und Haben gleich sind.
pub fn booking_lines(
    art: &str,
    positions: &[InvoicePosition],
    totals: &InvoiceTotals,
    codes: &[CodeOption],
    accounts: BookingAccounts,
    kurs: i64,
    projekt_id: Option<i64>,
) -> Result<(Vec<EffectiveLine>, i64), String> {
    // Haben-Seite der Rechnung; bei der Gutschrift wird später alles umgedreht
    let credit = art != "gutschrift";
    // Ertrag je (Konto, Code), in der Reihenfolge der Positionen
    let mut income: Vec<(i64, Option<i64>, i64)> = Vec::new();
    for p in positions {
        match income
            .iter_mut()
            .find(|(k, c, _)| *k == p.konto_id && *c == p.mwst_code_id)
        {
            Some(entry) => entry.2 += p.amount(),
            None => income.push((p.konto_id, p.mwst_code_id, p.amount())),
        }
    }
    // (Seite Haben?, Zeile) mit Beträgen in CHF; Nullbeträge entfallen
    let mut lines: Vec<EffectiveLine> = Vec::new();
    for (konto_id, code_id, amount) in income {
        let chf = to_chf(amount, kurs);
        if chf == 0 {
            continue;
        }
        lines.push(EffectiveLine {
            soll: !credit,
            konto_id,
            betrag: chf,
            mwst_code_id: code_id,
            mwst_basis: code_id.map(|_| chf),
            ist_mwst_zeile: false,
            projekt_id,
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
            soll: !credit,
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
            // Aufgerundet: zusätzlicher Ertrag auf der Haben-Seite der Rechnung
            let on_credit_side = totals.rounding > 0;
            lines.push(EffectiveLine {
                soll: if on_credit_side { !credit } else { credit },
                konto_id,
                betrag: chf,
                mwst_code_id: None,
                mwst_basis: None,
                ist_mwst_zeile: false,
                projekt_id: None,
            });
        }
    }
    // Debitoren = Summe der Gegenseite minus Zeilen auf derselben Seite wie die Debitoren
    let gegen: i64 = lines
        .iter()
        .map(|l| {
            if l.soll != credit {
                l.betrag
            } else {
                -l.betrag
            }
        })
        .sum();
    if gegen <= 0 {
        return Err("Der Gesamtbetrag in CHF ist null. Der Beleg wird nicht gebucht.".into());
    }
    lines.insert(
        0,
        EffectiveLine {
            soll: credit,
            konto_id: accounts.debitoren,
            betrag: gegen,
            mwst_code_id: None,
            mwst_basis: None,
            ist_mwst_zeile: false,
            projekt_id,
        },
    );
    Ok((lines, gegen))
}

/// Text der Buchung im Journal, z. B. „Rechnung R-2026-0001 Muster AG“ (höchstens 200 Zeichen).
pub fn booking_text(art: &str, nummer: &str, kunde: &str) -> String {
    format!("{} {nummer} {kunde}", kind_label(art))
        .chars()
        .take(crate::backend::finance::journal::model::TEXT_MAX)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Liste
// ---------------------------------------------------------------------------------------------

/// Zeile der Liste (AK-1), der Projekt-Detailansicht (AK-15) und der Adress-Detailansicht (AK-16).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct InvoiceRow {
    pub id: i64,
    pub nummer: Option<String>,
    pub art: String,
    pub kunde: String,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub status: String,
    pub rechnungsdatum: String,
    pub faelligkeit: Option<String>,
    pub waehrung: String,
    /// Festgeschrieben beim Buchen; im Entwurf aus den Positionen berechnet
    pub gesamtbetrag: Option<i64>,
    pub offener_betrag: i64,
    pub tage_ueberfaellig: Option<i32>,
}

/// Seite der Liste.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InvoicePage {
    pub rows: Vec<InvoiceRow>,
    pub total: i64,
    pub page: i64,
}

impl InvoicePage {
    pub fn page_count(&self) -> i64 {
        ((self.total + PAGE_SIZE - 1) / PAGE_SIZE).max(1)
    }
}

/// Filter der Liste; kommt vollständig aus der Webadresse (AK-2).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InvoiceFilter {
    pub suche: String,
    pub status: String,
    pub von: String,
    pub bis: String,
    pub sortierung: String,
    /// `auf` oder `ab`
    pub richtung: String,
    pub seite: i64,
}

impl InvoiceFilter {
    /// Unbekannte Werte fallen auf die Vorgabe zurück: offene Belege, nach Fälligkeit aufsteigend (AK-2).
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
            sortierung,
            richtung,
            seite: self.seite.max(1),
        }
    }
}

/// Standardrichtung je Spalte: Nummer und Datum absteigend (neueste zuerst), sonst aufsteigend.
pub fn default_dir(sort: &str) -> &'static str {
    match sort {
        "nummer" | "rechnungsdatum" => "ab",
        _ => "auf",
    }
}

/// Offene Beträge eines Kunden je Währung (AK-16), Gutschriften abgezogen.
pub fn open_by_currency(rows: &[InvoiceRow]) -> Vec<(String, i64)> {
    let mut sums: Vec<(String, i64)> = Vec::new();
    for r in rows
        .iter()
        .filter(|r| OPEN_STATUSES.contains(&r.status.as_str()))
    {
        let value = signed(&r.art, r.offener_betrag);
        match sums.iter_mut().find(|(c, _)| *c == r.waehrung) {
            Some(entry) => entry.1 += value,
            None => sums.push((r.waehrung.clone(), value)),
        }
    }
    sums.sort_by(|a, b| a.0.cmp(&b.0));
    sums
}

/// Betrag mit Währung für Anzeigen, z. B. „CHF 1'234.50“.
pub fn money(currency: &str, cents: i64) -> String {
    format!("{currency} {}", format_amount(cents))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(
        id: i64,
        menge: i64,
        preis: i64,
        konto: i64,
        code: Option<(i64, i64)>,
    ) -> InvoicePosition {
        InvoicePosition {
            id,
            bezeichnung: format!("Position {id}"),
            menge,
            einheit: "stk".into(),
            einzelpreis: preis,
            rabatt_prozent: 0,
            konto_id: konto,
            konto_nummer: "3400".into(),
            konto_bezeichnung: "Dienstleistungserlöse".into(),
            mwst_code_id: code.map(|(id, _)| id),
            mwst_code: code.map(|(id, _)| format!("C{id}")),
            mwst_satz: code.map(|(_, satz)| satz),
        }
    }

    fn code(id: i64, satz: i64, konto_id: i64) -> CodeOption {
        CodeOption {
            id,
            code: format!("C{id}"),
            bezeichnung: "Umsatzsteuer".into(),
            satz,
            gueltig_ab: "2024-01-01".into(),
            gueltig_bis: None,
            konto_id,
        }
    }

    fn account(id: i64, nummer: &str) -> AccountOption {
        AccountOption {
            id,
            nummer: nummer.into(),
            bezeichnung: "Ertrag".into(),
            vorgabe_mwst: None,
        }
    }

    const ACCOUNTS: BookingAccounts = BookingAccounts {
        debitoren: 1100,
        rundung: Some(6990),
    };

    fn balance(lines: &[EffectiveLine]) -> (i64, i64) {
        crate::backend::finance::journal::model::totals(lines)
    }

    #[test]
    fn totals_per_code_with_rounding_in_chf() {
        // 3 × 33.33 = 99.99 mit 8.1 % → MWST 8.10, roh 108.09 → gerundet 108.10
        let positions = vec![pos(1, 3_000, 3_333, 34, Some((1, 810)))];
        let t = totals(&positions, "CHF");
        assert_eq!(t.subtotal, 9_999);
        assert_eq!(t.vat.len(), 1);
        assert_eq!(t.vat[0].vat, 810);
        assert_eq!(t.total, 10_810);
        assert_eq!(t.rounding, 1);
        // In EUR keine Rappenrundung
        let t = totals(&positions, "EUR");
        assert_eq!((t.total, t.rounding), (10_809, 0));
    }

    #[test]
    fn totals_group_codes_and_skip_positions_without_code() {
        let positions = vec![
            pos(1, 1_000, 10_000, 34, Some((1, 810))),
            pos(2, 2_000, 5_000, 34, Some((2, 260))),
            pos(3, 1_000, 5_000, 34, Some((1, 810))),
            pos(4, 1_000, 1_000, 34, None),
        ];
        let t = totals(&positions, "CHF");
        assert_eq!(t.subtotal, 26_000);
        assert_eq!(
            t.vat
                .iter()
                .map(|v| (v.code_id, v.base, v.vat))
                .collect::<Vec<_>>(),
            vec![(1, 15_000, 1_215), (2, 10_000, 260)]
        );
        assert_eq!(t.total, 27_475);
    }

    #[test]
    fn invoice_booking_is_balanced_with_debtor_on_debit() {
        let positions = vec![
            pos(1, 1_000, 10_000, 34, Some((1, 810))),
            pos(2, 1_000, 2_000, 32, Some((1, 810))),
        ];
        let t = totals(&positions, "CHF");
        let codes = vec![code(1, 810, 2200)];
        let (lines, chf) = booking_lines(
            "rechnung",
            &positions,
            &t,
            &codes,
            ACCOUNTS,
            RATE_FACTOR,
            Some(7),
        )
        .unwrap();
        assert_eq!(chf, t.total);
        let (soll, haben) = balance(&lines);
        assert_eq!(soll, haben);
        assert_eq!(lines[0].konto_id, 1100);
        assert!(lines[0].soll);
        assert_eq!(lines[0].betrag, 12_970);
        // Ertrag je Konto mit MWST-Basis, Steuerzeile auf dem Konto des Codes
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 34 && !l.soll && l.mwst_basis == Some(10_000))
        );
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 2200 && l.ist_mwst_zeile && l.betrag == 972)
        );
        // Rundung 129.72 → 129.70: Zeile auf der Sollseite
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 6990 && l.soll && l.betrag == 2)
        );
        assert!(
            lines
                .iter()
                .filter(|l| !l.ist_mwst_zeile && l.konto_id != 1100 && l.konto_id != 6990)
                .all(|l| l.projekt_id == Some(7))
        );
    }

    #[test]
    fn credit_note_swaps_sides() {
        let positions = vec![pos(1, 1_000, 10_000, 34, Some((1, 810)))];
        let t = totals(&positions, "CHF");
        let codes = vec![code(1, 810, 2200)];
        let (lines, chf) = booking_lines(
            "gutschrift",
            &positions,
            &t,
            &codes,
            ACCOUNTS,
            RATE_FACTOR,
            None,
        )
        .unwrap();
        assert_eq!(chf, 10_810);
        assert!(!lines[0].soll, "Debitoren im Haben");
        assert!(lines[1..].iter().all(|l| l.soll));
        let (soll, haben) = balance(&lines);
        assert_eq!(soll, haben);
    }

    #[test]
    fn foreign_currency_is_converted_line_by_line() {
        // EUR 1000 + 8.1 % zu 0.95 → Ertrag 950.00, MWST 76.95, Debitoren 1026.95
        let positions = vec![pos(1, 1_000, 100_000, 34, Some((1, 810)))];
        let t = totals(&positions, "EUR");
        let codes = vec![code(1, 810, 2200)];
        let (lines, chf) =
            booking_lines("rechnung", &positions, &t, &codes, ACCOUNTS, 950_000, None).unwrap();
        assert_eq!(chf, 102_695);
        let (soll, haben) = balance(&lines);
        assert_eq!(soll, haben);
    }

    #[test]
    fn missing_rounding_account_is_reported() {
        let positions = vec![pos(1, 3_000, 3_333, 34, Some((1, 810)))];
        let t = totals(&positions, "CHF");
        let accounts = BookingAccounts {
            debitoren: 1100,
            rundung: None,
        };
        let result = booking_lines(
            "rechnung",
            &positions,
            &t,
            &[code(1, 810, 2200)],
            accounts,
            RATE_FACTOR,
            None,
        );
        assert!(result.unwrap_err().contains("Rundungsdifferenzen"));
    }

    #[test]
    fn booking_check_messages() {
        let konten = vec![account(34, "3400")];
        let codes = vec![code(1, 810, 2200)];
        let ok = vec![pos(1, 1_000, 100, 34, Some((1, 810)))];
        assert!(booking_check(&ok, &konten, &codes, "2026-05-01", Some("2024-01-01"), 110).is_ok());
        assert!(booking_check(&[], &konten, &codes, "2026-05-01", None, 0).is_err());
        // Ohne MWST-Pflicht darf kein Code stehen
        let err = booking_check(&ok, &konten, &codes, "2026-05-01", None, 110).unwrap_err();
        assert!(err.starts_with("Position 1:"));
        // Mit Pflicht muss ein Code stehen
        let without = vec![pos(1, 1_000, 100, 34, None)];
        assert!(
            booking_check(
                &without,
                &konten,
                &codes,
                "2026-05-01",
                Some("2024-01-01"),
                100
            )
            .is_err()
        );
        assert!(booking_check(&without, &konten, &codes, "2026-05-01", None, 100).is_ok());
        // Inaktives Konto
        let other = vec![pos(1, 1_000, 100, 99, None)];
        assert!(
            booking_check(&other, &konten, &codes, "2026-05-01", None, 100)
                .unwrap_err()
                .contains("Ertragskonto")
        );
        // Code ausserhalb der Gültigkeit
        let mut old = code(1, 810, 2200);
        old.gueltig_bis = Some("2025-12-31".into());
        assert!(
            booking_check(&ok, &konten, &[old], "2026-05-01", Some("2024-01-01"), 110)
                .unwrap_err()
                .contains("nicht gültig")
        );
        // Betrag null
        assert!(booking_check(&without, &konten, &codes, "2026-05-01", None, 0).is_err());
    }

    #[test]
    fn default_income_account_prefers_system_account() {
        let konten = vec![account(1, "3000"), account(2, "3400")];
        assert_eq!(default_income_account(&konten, Some(1)), Some(1));
        assert_eq!(default_income_account(&konten, None), Some(2));
        // Systemkonto nicht (mehr) aktiv → 3400
        assert_eq!(default_income_account(&konten, Some(9)), Some(2));
        assert_eq!(default_income_account(&konten[..1], None), None);
    }

    #[test]
    fn position_input_validation() {
        let mut input = PositionInput {
            bezeichnung: "Beratung".into(),
            menge: "2.5".into(),
            einheit: "h".into(),
            einzelpreis: "150".into(),
            rabatt_prozent: "".into(),
            konto_id: "34".into(),
            mwst_code_id: "1".into(),
        };
        let v = input.validate().unwrap();
        assert_eq!(
            (v.menge, v.einzelpreis, v.rabatt_prozent),
            (2_500, 15_000, 0)
        );
        assert_eq!(v.mwst_code_id, Some(1));
        input.mwst_code_id = String::new();
        assert_eq!(input.validate().unwrap().mwst_code_id, None);
        input.menge = "0".into();
        input.rabatt_prozent = "101".into();
        input.konto_id = String::new();
        let errors = input.validate().unwrap_err();
        assert!(errors.contains_key("menge"));
        assert!(errors.contains_key("rabatt_prozent"));
        assert!(errors.contains_key("konto_id"));
    }

    #[test]
    fn check_position_against_master_data() {
        let konten = vec![account(34, "3400")];
        let codes = vec![code(1, 810, 2200)];
        assert!(
            check_position(
                34,
                Some(1),
                &konten,
                &codes,
                "2026-05-01",
                Some("2024-01-01")
            )
            .is_empty()
        );
        let e = check_position(35, None, &konten, &codes, "2026-05-01", Some("2024-01-01"));
        assert!(e.contains_key("konto_id") && e.contains_key("mwst_code_id"));
        assert!(
            check_position(34, None, &konten, &codes, "2023-05-01", Some("2024-01-01")).is_empty()
        );
    }

    #[test]
    fn invoice_input_validation_and_restriction() {
        let mut input = InvoiceInput::new_default("rechnung", "2026-11-27");
        let errors = input.validate();
        assert!(errors.contains_key("adresse_id"));
        input.adresse_id = "5".into();
        assert!(input.validate().is_empty());
        input.rechnungsdatum = "2026-02-30".into();
        assert!(input.validate().contains_key("rechnungsdatum"));

        let current = InvoiceInput {
            adresse_id: "5".into(),
            ..InvoiceInput::new_default("rechnung", "2026-11-01")
        };
        let changed = InvoiceInput {
            adresse_id: "9".into(),
            rechnungsdatum: "2026-12-01".into(),
            bemerkung: "Neu".into(),
            ..current.clone()
        };
        let r = changed.restricted_to_booked(&current);
        assert_eq!(r.adresse_id, "5");
        assert_eq!(r.rechnungsdatum, "2026-11-01");
        assert_eq!(r.bemerkung, "Neu");
    }

    #[test]
    fn numbers_signs_and_filter_defaults() {
        assert_eq!(number_for("rechnung", 2026, 1), "R-2026-0001");
        assert_eq!(number_for("gutschrift", 2026, 12), "G-2026-0012");
        assert_eq!(signed("gutschrift", 500), -500);
        assert_eq!(to_chf(10_000, 950_000), 9_500);
        assert_eq!(to_chf(1, 500_000), 1);
        let f = InvoiceFilter::default().normalized();
        assert_eq!(
            (
                f.status.as_str(),
                f.sortierung.as_str(),
                f.richtung.as_str(),
                f.seite
            ),
            ("offen", "faelligkeit", "auf", 1)
        );
        let f = InvoiceFilter {
            sortierung: "nummer; DROP".into(),
            von: "2026-13-01".into(),
            ..Default::default()
        }
        .normalized();
        assert_eq!((f.sortierung.as_str(), f.von.as_str()), ("faelligkeit", ""));
    }

    #[test]
    fn open_amounts_per_currency_subtract_credit_notes() {
        let row = |art: &str, status: &str, waehrung: &str, offen: i64| InvoiceRow {
            id: 1,
            nummer: None,
            art: art.into(),
            kunde: "Kunde".into(),
            projekt_id: None,
            projekt_nummer: None,
            status: status.into(),
            rechnungsdatum: "2026-01-01".into(),
            faelligkeit: None,
            waehrung: waehrung.into(),
            gesamtbetrag: Some(offen),
            offener_betrag: offen,
            tage_ueberfaellig: None,
        };
        let rows = vec![
            row("rechnung", "offen", "CHF", 1_000),
            row("gutschrift", "offen", "CHF", 300),
            row("rechnung", "teilweise_bezahlt", "EUR", 200),
            row("rechnung", "bezahlt", "CHF", 0),
            row("rechnung", "entwurf", "CHF", 0),
        ];
        assert_eq!(
            open_by_currency(&rows),
            vec![("CHF".to_string(), 700), ("EUR".to_string(), 200)]
        );
    }

    #[test]
    fn booking_text_is_limited() {
        let text = booking_text("rechnung", "R-2026-0001", &"x".repeat(300));
        assert!(text.starts_with("Rechnung R-2026-0001 x"));
        assert_eq!(text.chars().count(), 200);
    }
}
