//! Datenmodell, Prüfregeln und Betragsrechnung für Einkaufsaufträge (ANF-004).
//! Wird auf Server und im Browser verwendet.
//!
//! Beträge und Mengen werden als ganze Zahlen in kleinsten Einheiten geführt (keine Gleitkommazahl, AK-25):
//! Menge in Tausendsteln, Preise in Rappen (Hundertsteln), Prozente in Hundertsteln.

use serde::{Deserialize, Serialize};

use crate::backend::projects::model::is_valid_date;
pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

/// Einträge pro Seite in der Liste (AK-3).
pub const PAGE_SIZE: i64 = 50;

/// Status eines Einkaufsauftrags.
pub const STATUSES: &[(&str, &str)] = &[
    ("entwurf", "Entwurf"),
    ("bestellt", "Bestellt"),
    ("teilweise_geliefert", "Teilweise geliefert"),
    ("geliefert", "Geliefert"),
    ("storniert", "Storniert"),
];

/// Offene Status – Standardfilter der Liste (AK-2) und Grundlage für „überfällig“ (AK-5).
pub const OPEN_STATUSES: &[&str] = &["entwurf", "bestellt", "teilweise_geliefert"];

/// Auswahl im Statusfilter der Liste: Schlüssel in der Webadresse → Beschriftung.
pub const STATUS_FILTERS: &[(&str, &str)] = &[
    ("offen", "Offen"),
    ("alle", "Alle"),
    ("entwurf", "Entwurf"),
    ("bestellt", "Bestellt"),
    ("teilweise_geliefert", "Teilweise geliefert"),
    ("geliefert", "Geliefert"),
    ("storniert", "Storniert"),
];

// Währungen kommen aus den Einstellungen (ANF-014), nicht aus einer festen Liste.

pub const UNITS: &[(&str, &str)] = &[
    ("stk", "Stk"),
    ("h", "h"),
    ("kg", "kg"),
    ("m", "m"),
    ("m2", "m²"),
    ("m3", "m³"),
    ("l", "l"),
    ("pauschal", "Pauschal"),
];

/// Auswahlwert im Formular für eine Position, die noch den festen Satz vor ANF-012 trägt (AK-9):
/// der Satz bleibt unverändert.
pub const ALT_VAT: &str = "alt";

/// Auswahl des MWST-Codes einer Position (ANF-012 AK-9).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum VatSelection {
    /// Bisheriger Satz ohne Code bleibt erhalten
    Keep,
    /// Ohne MWST (Satz 0, kein Code)
    Ohne,
    /// MWST-Code (Kennung aus `mwst_code`)
    Code(i64),
}

/// Zu speichernder Stand des MWST-Codes und Satz einer Position (Hundertstel Prozent).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VatWrite {
    pub code_id: Option<i64>,
    pub satz: i64,
}

/// Grösste erlaubte Menge bzw. grösster Preis (passt zu NUMERIC(12,…) in der Datenbank).
const MAX_INTEGER_DIGITS: usize = 9;

pub fn status_label(status: &str) -> &str {
    STATUSES
        .iter()
        .find(|(k, _)| *k == status)
        .map_or(status, |(_, l)| l)
}

/// daisyUI-Klassen für die Status-Plakette (vollständig ausgeschrieben, damit Tailwind sie findet).
pub fn status_badge(status: &str) -> &'static str {
    match status {
        "bestellt" => "badge badge-sm badge-primary badge-soft",
        "teilweise_geliefert" => "badge badge-sm badge-warning badge-soft",
        "geliefert" => "badge badge-sm badge-success badge-soft",
        "storniert" => "badge badge-sm badge-error badge-soft",
        _ => "badge badge-sm badge-ghost",
    }
}

pub fn unit_label(unit: &str) -> &str {
    UNITS
        .iter()
        .find(|(k, _)| *k == unit)
        .map_or(unit, |(_, l)| l)
}

// ---------------------------------------------------------------------------------------------
// Festkomma-Zahlen
// ---------------------------------------------------------------------------------------------

/// Liest eine Dezimalzahl mit höchstens `decimals` Nachkommastellen als ganze Zahl in kleinsten Einheiten.
/// Erlaubt Punkt oder Komma als Dezimalzeichen und ' als Tausendertrennzeichen (z. B. „1'234.50“).
/// Negative Zahlen werden nicht angenommen.
pub fn parse_fixed(s: &str, decimals: u32) -> Option<i64> {
    let s: String = s
        .trim()
        .chars()
        .filter(|c| !matches!(c, '\'' | '’' | ' '))
        .collect();
    let (int, frac) = match s.split_once(['.', ',']) {
        Some((i, f)) => (i, f),
        None => (s.as_str(), ""),
    };
    if (int.is_empty() && frac.is_empty())
        || int.len() > MAX_INTEGER_DIGITS
        || frac.len() > decimals as usize
        || !int.chars().chain(frac.chars()).all(|c| c.is_ascii_digit())
    {
        return None;
    }
    let int: i64 = if int.is_empty() { 0 } else { int.parse().ok()? };
    let mut frac_value: i64 = if frac.is_empty() {
        0
    } else {
        frac.parse().ok()?
    };
    for _ in frac.len()..decimals as usize {
        frac_value *= 10;
    }
    Some(int * 10_i64.pow(decimals) + frac_value)
}

/// Schreibt eine Festkomma-Zahl mit genau `decimals` Nachkommastellen (für Formularfelder), z. B. „12.50“.
pub fn format_fixed(value: i64, decimals: u32) -> String {
    let factor = 10_i64.pow(decimals);
    let sign = if value < 0 { "-" } else { "" };
    let v = value.abs();
    if decimals == 0 {
        return format!("{sign}{v}");
    }
    format!(
        "{sign}{}.{:0width$}",
        v / factor,
        v % factor,
        width = decimals as usize
    )
}

/// Wie `format_fixed`, aber ohne überflüssige Nullen am Ende: 2.500 → „2.5“, 3.000 → „3“.
pub fn format_trimmed(value: i64, decimals: u32) -> String {
    let s = format_fixed(value, decimals);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// Betrag in Rappen zur Anzeige mit Tausendertrennzeichen: 123456 → „1'234.56“.
pub fn format_amount(cents: i64) -> String {
    let s = format_fixed(cents, 2);
    let (sign, s) = s.strip_prefix('-').map_or(("", s.as_str()), |r| ("-", r));
    let (int, frac) = s.split_once('.').unwrap_or((s, "00"));
    let mut grouped = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push('\'');
        }
        grouped.push(c);
    }
    format!("{sign}{grouped}.{frac}")
}

/// Ganzzahlige Division mit kaufmännischer Rundung (ab 0.5 aufrunden, symmetrisch um 0).
fn div_round(num: i128, den: i128) -> i128 {
    let half = den / 2;
    if num >= 0 {
        (num + half) / den
    } else {
        (num - half) / den
    }
}

/// Rundet Rappen auf 5 Rappen (Rappenrundung CHF).
pub fn round_to_5(cents: i64) -> i64 {
    div_round(cents as i128, 5) as i64 * 5
}

// ---------------------------------------------------------------------------------------------
// Positionen und Summen
// ---------------------------------------------------------------------------------------------

/// Eine Position eines Einkaufsauftrags (Werte in kleinsten Einheiten, siehe Moduldokumentation).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Position {
    pub id: i64,
    pub bezeichnung: String,
    pub artikelnummer_lieferant: Option<String>,
    /// Tausendstel
    pub menge: i64,
    pub einheit: String,
    /// Rappen
    pub einzelpreis: i64,
    /// Hundertstel Prozent
    pub rabatt_prozent: i64,
    /// Hundertstel Prozent
    pub mwst_satz: i64,
    /// MWST-Code, aus dem der Satz stammt; `None` bei „ohne MWST“ und bei Positionen vor ANF-012
    pub mwst_code_id: Option<i64>,
    /// Tausendstel
    pub gelieferte_menge: i64,
}

impl Position {
    /// Positionsbetrag ohne MWST in Rappen = Menge × Einzelpreis × (1 − Rabatt/100), auf 0.01 gerundet.
    pub fn amount(&self) -> i64 {
        // Menge (10⁻³) × Preis (10⁻²) × (10000 − Rabatt in 10⁻⁴) ergibt 10⁻⁹ → durch 10⁷ teilen für Rappen
        let num =
            self.menge as i128 * self.einzelpreis as i128 * (10_000 - self.rabatt_prozent) as i128;
        div_round(num, 10_000_000) as i64
    }

    pub fn is_fully_delivered(&self) -> bool {
        self.gelieferte_menge >= self.menge
    }

    /// Mehr geliefert als bestellt (AK-18).
    pub fn is_over_delivered(&self) -> bool {
        self.gelieferte_menge > self.menge
    }

    pub fn to_input(&self) -> PositionInput {
        PositionInput {
            bezeichnung: self.bezeichnung.clone(),
            artikelnummer_lieferant: self.artikelnummer_lieferant.clone().unwrap_or_default(),
            menge: format_trimmed(self.menge, 3),
            einheit: self.einheit.clone(),
            einzelpreis: format_fixed(self.einzelpreis, 2),
            rabatt_prozent: format_trimmed(self.rabatt_prozent, 2),
            mwst_code: self.vat_choice(),
        }
    }

    /// Auswahl im Formular: Code-Kennung, leer für „ohne MWST“ oder „alt“ für einen Satz ohne Code.
    pub fn vat_choice(&self) -> String {
        match self.mwst_code_id {
            Some(id) => id.to_string(),
            None if self.mwst_satz == 0 => String::new(),
            None => ALT_VAT.into(),
        }
    }
}

/// MWST-Anteil für einen Satz.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VatLine {
    /// Hundertstel Prozent
    pub rate: i64,
    /// Summe der Positionsbeträge mit diesem Satz (Rappen)
    pub base: i64,
    /// MWST-Betrag (Rappen)
    pub vat: i64,
}

/// Summen eines Einkaufsauftrags (AK-7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Totals {
    pub subtotal: i64,
    /// Nach Satz absteigend
    pub vat: Vec<VatLine>,
    /// Differenz durch Rappenrundung (nur CHF)
    pub rounding: i64,
    pub total: i64,
}

/// Berechnet Zwischensumme, MWST je Satz und Gesamtbetrag. In CHF wird der Gesamtbetrag auf 0.05 gerundet.
pub fn totals(positions: &[Position], currency: &str) -> Totals {
    let subtotal: i64 = positions.iter().map(Position::amount).sum();
    let mut vat: Vec<VatLine> = Vec::new();
    for p in positions {
        match vat.iter_mut().find(|l| l.rate == p.mwst_satz) {
            Some(line) => line.base += p.amount(),
            None => vat.push(VatLine {
                rate: p.mwst_satz,
                base: p.amount(),
                vat: 0,
            }),
        }
    }
    vat.sort_by_key(|l| std::cmp::Reverse(l.rate));
    for line in &mut vat {
        // Basis (10⁻²) × Satz (10⁻⁴ als Anteil) → durch 10⁴ teilen
        line.vat = div_round(line.base as i128 * line.rate as i128, 10_000) as i64;
    }
    let raw = subtotal + vat.iter().map(|l| l.vat).sum::<i64>();
    let total = if currency == "CHF" {
        round_to_5(raw)
    } else {
        raw
    };
    Totals {
        subtotal,
        vat,
        rounding: total - raw,
        total,
    }
}

/// Status nach dem Erfassen von Liefermengen (AK-17): alles geliefert → „geliefert“,
/// etwas geliefert → „teilweise_geliefert“, sonst „bestellt“.
pub fn delivery_status(positions: &[Position]) -> &'static str {
    if !positions.is_empty() && positions.iter().all(Position::is_fully_delivered) {
        "geliefert"
    } else if positions.iter().any(|p| p.gelieferte_menge > 0) {
        "teilweise_geliefert"
    } else {
        "bestellt"
    }
}

// ---------------------------------------------------------------------------------------------
// Einkaufsauftrag
// ---------------------------------------------------------------------------------------------

/// Kopfdaten eines Einkaufsauftrags.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Order {
    pub id: i64,
    pub nummer: String,
    pub adresse_id: i64,
    pub lieferant_nummer: String,
    pub lieferant: String,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub projekt_name: Option<String>,
    pub status: String,
    /// ISO-Datum JJJJ-MM-TT
    pub bestelldatum: Option<String>,
    pub liefertermin: Option<String>,
    pub referenz_lieferant: Option<String>,
    pub waehrung: String,
    pub lieferadresse: Option<String>,
    pub bemerkung: Option<String>,
    /// Liefertermin überschritten und noch offen (AK-5)
    pub ueberfaellig: bool,
    pub erstellt_am: String,
    pub geaendert_am: String,
}

impl Order {
    /// Kopfdaten, Positionen und Lieferant frei änderbar (AK-13).
    pub fn is_draft(&self) -> bool {
        self.status == "entwurf"
    }

    /// Bestellt, aber noch nicht vollständig geliefert: Wareneingang möglich, nur wenige Felder änderbar (AK-15, AK-16).
    pub fn is_ordered(&self) -> bool {
        matches!(self.status.as_str(), "bestellt" | "teilweise_geliefert")
    }

    /// Kopfdaten noch (teilweise) bearbeitbar.
    pub fn is_editable(&self) -> bool {
        self.is_draft() || self.is_ordered()
    }

    /// Stornieren nur im Entwurf oder bestellt, solange nichts geliefert ist (AK-19).
    pub fn can_cancel(&self, positions: &[Position]) -> bool {
        self.is_draft()
            || (self.status == "bestellt" && positions.iter().all(|p| p.gelieferte_menge == 0))
    }

    pub fn supplier(&self) -> PickOption {
        PickOption {
            id: self.adresse_id,
            nummer: self.lieferant_nummer.clone(),
            name: self.lieferant.clone(),
        }
    }

    pub fn project(&self) -> Option<PickOption> {
        Some(PickOption {
            id: self.projekt_id?,
            nummer: self.projekt_nummer.clone()?,
            name: self.projekt_name.clone()?,
        })
    }

    pub fn to_input(&self) -> OrderInput {
        OrderInput {
            adresse_id: self.adresse_id.to_string(),
            projekt_id: self.projekt_id.map(|i| i.to_string()).unwrap_or_default(),
            bestelldatum: self.bestelldatum.clone().unwrap_or_default(),
            liefertermin: self.liefertermin.clone().unwrap_or_default(),
            referenz_lieferant: self.referenz_lieferant.clone().unwrap_or_default(),
            waehrung: self.waehrung.clone(),
            lieferadresse: self.lieferadresse.clone().unwrap_or_default(),
            bemerkung: self.bemerkung.clone().unwrap_or_default(),
        }
    }
}

/// Einkaufsauftrag mit allen Positionen (Detailansicht).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderDetail {
    pub order: Order,
    pub positions: Vec<Position>,
}

impl OrderDetail {
    pub fn totals(&self) -> Totals {
        totals(&self.positions, &self.order.waehrung)
    }
}

/// Formulareingabe für die Kopfdaten (AK-8 bis AK-10, AK-13, AK-15).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct OrderInput {
    pub adresse_id: String,
    pub projekt_id: String,
    pub bestelldatum: String,
    pub liefertermin: String,
    pub referenz_lieferant: String,
    pub waehrung: String,
    pub lieferadresse: String,
    pub bemerkung: String,
}

impl OrderInput {
    pub fn new_default() -> Self {
        Self {
            waehrung: "CHF".into(),
            ..Default::default()
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            adresse_id: self.adresse_id.trim().into(),
            projekt_id: self.projekt_id.trim().into(),
            bestelldatum: self.bestelldatum.trim().into(),
            liefertermin: self.liefertermin.trim().into(),
            referenz_lieferant: self.referenz_lieferant.trim().into(),
            waehrung: self.waehrung.trim().to_uppercase(),
            lieferadresse: self.lieferadresse.trim().into(),
            bemerkung: self.bemerkung.trim().into(),
        }
    }

    /// Nach dem Bestellen sind nur Liefertermin, Referenz und Bemerkung änderbar (AK-15):
    /// alle anderen Felder werden aus dem gespeicherten Auftrag übernommen.
    pub fn restricted_to_ordered(&self, current: &OrderInput) -> Self {
        Self {
            liefertermin: self.liefertermin.clone(),
            referenz_lieferant: self.referenz_lieferant.clone(),
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

        if self.adresse_id.is_empty() {
            err("adresse_id", "Bitte einen Lieferanten wählen.");
        } else if self.adresse_id.parse::<i64>().is_err() {
            err("adresse_id", "Ungültiger Lieferant.");
        }
        if !self.projekt_id.is_empty() && self.projekt_id.parse::<i64>().is_err() {
            err("projekt_id", "Ungültiges Projekt.");
        }
        if !self.bestelldatum.is_empty() && !is_valid_date(&self.bestelldatum) {
            err("bestelldatum", "Bitte ein gültiges Datum eingeben.");
        }
        if !self.liefertermin.is_empty() && !is_valid_date(&self.liefertermin) {
            err("liefertermin", "Bitte ein gültiges Datum eingeben.");
        }
        if is_valid_date(&self.bestelldatum)
            && is_valid_date(&self.liefertermin)
            && self.liefertermin < self.bestelldatum
        {
            err(
                "liefertermin",
                "Der Liefertermin darf nicht vor dem Bestelldatum liegen.",
            );
        }
        if self.referenz_lieferant.chars().count() > 100 {
            err("referenz_lieferant", "Höchstens 100 Zeichen.");
        }
        // Ob die Währung aktiv ist, prüft der Server mit der Datenbank (ANF-014 AK-6).
        if !crate::backend::finance::currency::model::is_code(&self.waehrung) {
            err("waehrung", "Bitte eine Währung aus der Liste wählen.");
        }
        errors
    }

    pub fn supplier_id(&self) -> Option<i64> {
        self.adresse_id.parse().ok()
    }

    pub fn project_id(&self) -> Option<i64> {
        self.projekt_id.parse().ok()
    }
}

/// Formulareingabe für eine Position (AK-11, AK-12).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PositionInput {
    pub bezeichnung: String,
    pub artikelnummer_lieferant: String,
    pub menge: String,
    pub einheit: String,
    pub einzelpreis: String,
    pub rabatt_prozent: String,
    /// Auswahl: Kennung eines MWST-Codes, leer für „ohne MWST“ oder „alt“ (siehe `ALT_VAT`)
    pub mwst_code: String,
}

/// Geprüfte Werte einer Position in kleinsten Einheiten.
#[derive(Clone, Debug, PartialEq)]
pub struct PositionValues {
    pub menge: i64,
    pub einzelpreis: i64,
    pub rabatt_prozent: i64,
    pub mwst: VatSelection,
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
            artikelnummer_lieferant: self.artikelnummer_lieferant.trim().into(),
            menge: self.menge.trim().into(),
            einheit: self.einheit.trim().into(),
            einzelpreis: self.einzelpreis.trim().into(),
            rabatt_prozent: self.rabatt_prozent.trim().into(),
            mwst_code: self.mwst_code.trim().into(),
        }
    }

    /// Prüft die normalisierte Eingabe und liefert die Werte in kleinsten Einheiten (AK-11).
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
        if self.artikelnummer_lieferant.chars().count() > 50 {
            err("artikelnummer_lieferant", "Höchstens 50 Zeichen.");
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
        let mwst = match self.mwst_code.as_str() {
            "" => Some(VatSelection::Ohne),
            ALT_VAT => Some(VatSelection::Keep),
            other => other.parse::<i64>().ok().map(VatSelection::Code),
        };
        if mwst.is_none() {
            err("mwst_code", "Bitte einen MWST-Code wählen.");
        }

        match (menge, einzelpreis, rabatt, mwst) {
            (Some(menge), Some(einzelpreis), Some(rabatt_prozent), Some(mwst))
                if errors.is_empty() =>
            {
                Ok(PositionValues {
                    menge,
                    einzelpreis,
                    rabatt_prozent,
                    mwst,
                })
            }
            _ => Err(errors),
        }
    }
}

/// Gelieferte Menge prüfen (AK-16): Zahl ≥ 0 mit höchstens 3 Nachkommastellen, in Tausendsteln.
pub fn parse_delivered(s: &str) -> Result<i64, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Bitte eine Menge eingeben (0, wenn nichts geliefert wurde).".into());
    }
    parse_fixed(s, 3).ok_or_else(|| {
        "Bitte eine Zahl ab 0 mit höchstens 3 Nachkommastellen eingeben.".to_string()
    })
}

// ---------------------------------------------------------------------------------------------
// Liste
// ---------------------------------------------------------------------------------------------

/// Zeile der Liste (AK-1) bzw. der Einkaufsaufträge eines Projekts (AK-22).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct OrderRow {
    pub id: i64,
    pub nummer: String,
    pub lieferant: String,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub status: String,
    pub bestelldatum: Option<String>,
    pub liefertermin: Option<String>,
    pub waehrung: String,
    pub ueberfaellig: bool,
    /// Gesamtbetrag in Rappen (aus den Positionen berechnet)
    #[cfg_attr(feature = "ssr", sqlx(skip))]
    pub total: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrderPage {
    pub rows: Vec<OrderRow>,
    pub total: i64,
    pub page: i64,
}

impl OrderPage {
    pub fn page_count(&self) -> i64 {
        ((self.total + PAGE_SIZE - 1) / PAGE_SIZE).max(1)
    }
}

/// Sortierbare Spalten (AK-2): Schlüssel in der Webadresse → Beschriftung.
pub const SORT_COLUMNS: &[(&str, &str)] = &[
    ("nummer", "Nummer"),
    ("lieferant", "Lieferant"),
    ("projekt", "Projekt"),
    ("bestelldatum", "Bestelldatum"),
    ("liefertermin", "Liefertermin"),
    ("status", "Status"),
    ("betrag", "Gesamtbetrag"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(menge: i64, preis: i64, rabatt: i64, mwst: i64) -> Position {
        Position {
            id: 1,
            bezeichnung: "x".into(),
            artikelnummer_lieferant: None,
            menge,
            einheit: "stk".into(),
            einzelpreis: preis,
            rabatt_prozent: rabatt,
            mwst_satz: mwst,
            mwst_code_id: None,
            gelieferte_menge: 0,
        }
    }

    #[test]
    fn parse_numbers() {
        assert_eq!(parse_fixed("12.5", 2), Some(1250));
        assert_eq!(parse_fixed("12,50", 2), Some(1250));
        assert_eq!(parse_fixed("1'234.05", 2), Some(123_405));
        assert_eq!(parse_fixed("3", 3), Some(3000));
        assert_eq!(parse_fixed(".5", 3), Some(500));
        assert_eq!(parse_fixed("0.125", 3), Some(125));
        assert_eq!(parse_fixed("0.1234", 3), None);
        assert_eq!(parse_fixed("-1", 2), None);
        assert_eq!(parse_fixed("1e3", 2), None);
        assert_eq!(parse_fixed("", 2), None);
        assert_eq!(parse_fixed(".", 2), None);
        assert_eq!(parse_fixed("1234567890", 2), None);
    }

    #[test]
    fn format_numbers() {
        assert_eq!(format_fixed(1250, 2), "12.50");
        assert_eq!(format_fixed(5, 2), "0.05");
        assert_eq!(format_trimmed(2500, 3), "2.5");
        assert_eq!(format_trimmed(3000, 3), "3");
        assert_eq!(format_trimmed(810, 2), "8.1");
        assert_eq!(format_amount(123_456_789), "1'234'567.89");
        assert_eq!(format_amount(99), "0.99");
        assert_eq!(format_amount(-100_000), "-1'000.00");
    }

    #[test]
    fn position_amount_with_discount() {
        // 2.5 h × 120.00 = 300.00
        assert_eq!(pos(2500, 12_000, 0, 810).amount(), 30_000);
        // 3 × 19.99 − 10 % = 53.973 → 53.97
        assert_eq!(pos(3000, 1999, 1000, 810).amount(), 5397);
        // 1 × 0.05 − 50 % = 0.025 → 0.03 (kaufmännisch)
        assert_eq!(pos(1000, 5, 5000, 0).amount(), 3);
        // 100 % Rabatt
        assert_eq!(pos(1000, 5000, 10_000, 810).amount(), 0);
    }

    #[test]
    fn totals_with_vat_per_rate_and_rounding() {
        let positions = vec![
            pos(1000, 10_000, 0, 810), // 100.00 zu 8.1 %
            pos(1000, 5_033, 0, 260),  // 50.33 zu 2.6 %
            pos(2000, 1_000, 0, 810),  // 20.00 zu 8.1 %
        ];
        let t = totals(&positions, "CHF");
        assert_eq!(t.subtotal, 17_033);
        assert_eq!(
            t.vat,
            vec![
                VatLine {
                    rate: 810,
                    base: 12_000,
                    vat: 972
                },
                VatLine {
                    rate: 260,
                    base: 5_033,
                    vat: 131
                },
            ]
        );
        // 170.33 + 9.72 + 1.31 = 181.36 → 181.35
        assert_eq!((t.total, t.rounding), (18_135, -1));
        let eur = totals(&positions, "EUR");
        assert_eq!((eur.total, eur.rounding), (18_136, 0));
        assert_eq!(totals(&[], "CHF").total, 0);
    }

    #[test]
    fn status_from_deliveries() {
        let mut a = pos(2000, 100, 0, 810);
        let mut b = pos(1000, 100, 0, 810);
        assert_eq!(delivery_status(&[a.clone(), b.clone()]), "bestellt");
        a.gelieferte_menge = 1000;
        assert_eq!(
            delivery_status(&[a.clone(), b.clone()]),
            "teilweise_geliefert"
        );
        a.gelieferte_menge = 2000;
        assert_eq!(
            delivery_status(&[a.clone(), b.clone()]),
            "teilweise_geliefert"
        );
        b.gelieferte_menge = 1500;
        assert!(b.is_over_delivered());
        assert_eq!(delivery_status(&[a, b]), "geliefert");
    }

    #[test]
    fn position_validation() {
        let mut i = PositionInput::new_default();
        i.bezeichnung = " Schrauben ".into();
        i.einzelpreis = "0.35".into();
        let v = i.normalized().validate().unwrap();
        assert_eq!(
            v,
            PositionValues {
                menge: 1000,
                einzelpreis: 35,
                rabatt_prozent: 0,
                mwst: VatSelection::Ohne,
            }
        );
        i.mwst_code = "12".into();
        assert_eq!(i.validate().unwrap().mwst, VatSelection::Code(12));
        i.mwst_code = ALT_VAT.into();
        assert_eq!(i.validate().unwrap().mwst, VatSelection::Keep);

        let bad = PositionInput {
            menge: "0".into(),
            einzelpreis: "-3".into(),
            rabatt_prozent: "101".into(),
            mwst_code: "x".into(),
            einheit: "fass".into(),
            ..Default::default()
        };
        let e = bad.validate().unwrap_err();
        for f in [
            "bezeichnung",
            "menge",
            "einzelpreis",
            "rabatt_prozent",
            "mwst_code",
            "einheit",
        ] {
            assert!(e.contains_key(f), "{f} fehlt in {e:?}");
        }
    }

    #[test]
    fn position_round_trip() {
        // Position vor ANF-012 (ohne Code, Satz 2.6 %) bleibt beim Bearbeiten unverändert
        let old = pos(2500, 1250, 550, 260);
        assert_eq!(old.vat_choice(), ALT_VAT);
        let v = old.to_input().validate().unwrap();
        assert_eq!(
            (v.menge, v.einzelpreis, v.rabatt_prozent),
            (2500, 1250, 550)
        );
        assert_eq!(v.mwst, VatSelection::Keep);
        // Position mit Code und ohne MWST
        let coded = Position {
            mwst_code_id: Some(7),
            ..pos(1000, 100, 0, 810)
        };
        assert_eq!(coded.vat_choice(), "7");
        assert_eq!(
            coded.to_input().validate().unwrap().mwst,
            VatSelection::Code(7)
        );
        assert_eq!(pos(1000, 100, 0, 0).vat_choice(), "");
    }

    #[test]
    fn order_validation() {
        let mut i = OrderInput::new_default();
        assert!(i.validate().contains_key("adresse_id"));
        i.adresse_id = "3".into();
        i.bestelldatum = "2026-11-10".into();
        i.liefertermin = "2026-11-09".into();
        let e = i.normalized().validate();
        assert_eq!(e.len(), 1, "{e:?}");
        assert!(e.contains_key("liefertermin"));
        i.liefertermin = "2026-11-10".into();
        i.waehrung = "eur".into();
        assert!(i.normalized().validate().is_empty());
        i.waehrung = "US".into();
        assert!(i.normalized().validate().contains_key("waehrung"));
    }

    #[test]
    fn ordered_order_keeps_locked_fields() {
        let current = OrderInput {
            adresse_id: "1".into(),
            waehrung: "CHF".into(),
            bestelldatum: "2026-11-01".into(),
            ..Default::default()
        };
        let edit = OrderInput {
            adresse_id: "99".into(),
            waehrung: "USD".into(),
            liefertermin: "2026-12-01".into(),
            bemerkung: "Rampe 2".into(),
            ..Default::default()
        };
        let r = edit.restricted_to_ordered(&current);
        assert_eq!(r.adresse_id, "1");
        assert_eq!(r.waehrung, "CHF");
        assert_eq!(r.liefertermin, "2026-12-01");
        assert_eq!(r.bemerkung, "Rampe 2");
    }

    #[test]
    fn delivered_quantity() {
        assert_eq!(parse_delivered("0"), Ok(0));
        assert_eq!(parse_delivered("2,5"), Ok(2500));
        assert!(parse_delivered("").is_err());
        assert!(parse_delivered("-1").is_err());
    }
}
