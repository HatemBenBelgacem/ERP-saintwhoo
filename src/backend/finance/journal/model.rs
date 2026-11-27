//! Buchungsjournal (ANF-017): Datentypen, Prüfregeln und MWST-Berechnung. Wird auf dem Server und im
//! Browser verwendet (Browser: Vorschau der MWST-Zeile und Differenz). Beträge sind in Rappen (ganze Zahlen).

use serde::{Deserialize, Serialize};

pub use crate::forms::{FieldErrors, SaveOutcome};

use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::projects::model::is_valid_date;
use crate::backend::purchase_orders::model::{format_amount, parse_fixed};

/// Längen der Textfelder (Datenbank: `text` 200, `belegnummer` 30).
pub const TEXT_MAX: usize = 200;
pub const BELEG_MAX: usize = 30;
/// Suchbegriff in der Liste (AK-2).
pub const SEARCH_MAX: usize = 100;
/// Einträge je Seite (AK-2: ab 50 Einträgen seitenweise).
pub const PAGE_SIZE: i64 = 50;

pub const STATUSES: &[(&str, &str)] = &[
    ("entwurf", "Entwurf"),
    ("gebucht", "Gebucht"),
    ("storniert", "Storniert"),
];

pub const HERKUNFT: &[(&str, &str)] = &[
    ("manuell", "Manuell"),
    ("debitor", "Debitoren"),
    ("kreditor", "Kreditoren"),
    ("zahlung", "Zahlung"),
    ("mwst", "MWST"),
    ("abschluss", "Abschluss"),
    ("eroeffnung", "Eröffnung"),
];

pub const SORTS: &[(&str, &str)] = &[
    ("datum_ab", "Datum absteigend"),
    ("datum_auf", "Datum aufsteigend"),
    ("journal", "Journalnummer"),
    ("betrag_ab", "Betrag absteigend"),
    ("beleg", "Belegnummer"),
];

pub const SIDES: &[(&str, &str)] = &[("soll", "Soll"), ("haben", "Haben")];

pub const AMOUNT_KINDS: &[(&str, &str)] = &[
    ("brutto", "Brutto (mit MWST)"),
    ("netto", "Netto (ohne MWST)"),
];

fn label_in(list: &'static [(&'static str, &'static str)], key: &str) -> &'static str {
    list.iter()
        .find(|(k, _)| *k == key)
        .map_or("–", |(_, label)| label)
}

pub fn status_label(key: &str) -> &'static str {
    label_in(STATUSES, key)
}

pub fn herkunft_label(key: &str) -> &'static str {
    label_in(HERKUNFT, key)
}

pub fn status_badge(status: &str) -> &'static str {
    match status {
        "gebucht" => "badge badge-success badge-soft",
        "storniert" => "badge badge-error badge-soft",
        _ => "badge badge-ghost",
    }
}

/// Betrag für eine Tabellenzelle: leer bei null (Soll- oder Habenseite ohne Betrag).
pub fn amount_cell(cents: i64) -> String {
    if cents == 0 {
        String::new()
    } else {
        format_amount(cents)
    }
}

/// Schlüssel aus einem Auswahltext („1020 · Bankguthaben“ → „1020“). Die Auswahl sendet den Text.
pub fn key_of(text: &str) -> &str {
    text.split(" · ").next().unwrap_or("").trim()
}

/// Auswahltext für Konto und Projekt: „1020 · Bankguthaben“.
pub fn option_label(nummer: &str, bezeichnung: &str) -> String {
    format!("{nummer} · {bezeichnung}")
}

// ---------------------------------------------------------------------------------------------
// Daten aus der Datenbank
// ---------------------------------------------------------------------------------------------

/// Kopf einer Buchung (Datum als `JJJJ-MM-TT`, Zeitpunkte als Text).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BookingHead {
    pub id: i64,
    pub geschaeftsjahr_id: i64,
    pub geschaeftsjahr: String,
    pub journalnummer: Option<i32>,
    pub datum: String,
    pub belegnummer: String,
    pub text: String,
    pub status: String,
    pub herkunft: String,
    pub storno_von_id: Option<i64>,
    pub erfasst_von: Option<String>,
    pub erstellt_am: String,
    pub gebucht_am: Option<String>,
}

impl BookingHead {
    pub fn is_draft(&self) -> bool {
        self.status == "entwurf"
    }

    /// Eine gebuchte manuelle Buchung lässt sich stornieren (AK-11, AK-12). Stornobuchungen nicht.
    pub fn can_storno(&self) -> bool {
        self.status == "gebucht" && self.herkunft == "manuell" && self.storno_von_id.is_none()
    }

    /// Entwürfe lassen sich bearbeiten, buchen und löschen (AK-10); gebuchte Belege nicht (ANF-010 AK-7).
    pub fn is_editable(&self) -> bool {
        self.is_draft() && self.herkunft == "manuell"
    }
}

/// Zeile einer Buchung (Beträge in Rappen). Steuerzeilen tragen `ist_mwst_zeile`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct LineView {
    pub id: i64,
    pub buchung_id: i64,
    pub reihenfolge: i32,
    pub konto_id: i64,
    pub konto_nummer: String,
    pub konto_bezeichnung: String,
    pub soll: i64,
    pub haben: i64,
    pub mwst_code_id: Option<i64>,
    pub mwst_code: Option<String>,
    pub mwst_basis: Option<i64>,
    pub ist_mwst_zeile: bool,
    pub projekt_id: Option<i64>,
    pub projekt_nummer: Option<String>,
    pub projekt_name: Option<String>,
}

impl LineView {
    /// Betrag der Zeile in Rappen, auf der Seite, auf der sie steht.
    pub fn betrag(&self) -> i64 {
        self.soll.max(self.haben)
    }

    pub fn seite(&self) -> &'static str {
        if self.soll > 0 { "soll" } else { "haben" }
    }
}

/// Verweis auf eine andere Buchung (Stornoverweis).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BookingRef {
    pub id: i64,
    pub label: String,
}

/// Buchung mit Zeilen für die Liste (Ausklappen bei Sammelbuchungen, AK-1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookingRow {
    pub head: BookingHead,
    pub zeilen: Vec<LineView>,
    /// Summe Soll in Rappen (bei ausgeglichenen Buchungen gleich der Summe Haben)
    pub summe: i64,
}

impl BookingRow {
    /// Konto der Sollseite und der Habenseite für die Spalten der Liste. Mehr als eine Konto
    /// einer Seite ergibt „Diverse“ (AK-1).
    pub fn konto_labels(&self) -> (String, String) {
        let seite = |soll: bool| -> String {
            let mut kontos: Vec<&LineView> = Vec::new();
            for z in self
                .zeilen
                .iter()
                .filter(|z| !z.ist_mwst_zeile && (z.soll > 0) == soll)
            {
                if !kontos.iter().any(|k| k.konto_id == z.konto_id) {
                    kontos.push(z);
                }
            }
            match kontos.as_slice() {
                [] => "–".into(),
                [one] => one.konto_nummer.clone(),
                _ => "Diverse".into(),
            }
        };
        (seite(true), seite(false))
    }
}

/// Journalseite mit Filter (AK-2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JournalPage {
    /// Geschäftsjahr der Liste (laufendes oder neuestes, wenn keines gewählt ist)
    pub geschaeftsjahr_id: i64,
    pub rows: Vec<BookingRow>,
    pub anzahl: i64,
    pub seite: i64,
    pub seiten: i64,
}

/// Detailansicht einer Buchung (AK-13).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookingDetail {
    pub head: BookingHead,
    pub zeilen: Vec<LineView>,
    pub storno_von: Option<BookingRef>,
    pub storniert_durch: Option<BookingRef>,
    /// Herkunftsbeleg einer automatischen Buchung, z. B. Ausgangsrechnung (AK-12, ANF-018)
    pub herkunftsbeleg: Option<BookingRef>,
    pub summe: i64,
}

/// Filter der Liste; kommt vollständig aus der Webadresse.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct JournalFilter {
    pub geschaeftsjahr_id: i64,
    pub suche: String,
    pub herkunft: String,
    pub status: String,
    pub von: String,
    pub bis: String,
    pub sortierung: String,
    pub seite: i64,
}

impl JournalFilter {
    /// Unbekannte Werte fallen auf die Vorgabe zurück; Seiten beginnen bei 1.
    pub fn normalized(&self) -> Self {
        let pick = |value: &str, list: &[(&str, &str)]| {
            if list.iter().any(|(k, _)| *k == value) {
                value.to_string()
            } else {
                String::new()
            }
        };
        let date = |value: &str| {
            if is_valid_date(value) {
                value.to_string()
            } else {
                String::new()
            }
        };
        Self {
            geschaeftsjahr_id: self.geschaeftsjahr_id,
            suche: self.suche.trim().chars().take(SEARCH_MAX).collect(),
            herkunft: pick(&self.herkunft, HERKUNFT),
            status: pick(&self.status, STATUSES),
            von: date(&self.von),
            bis: date(&self.bis),
            sortierung: if SORTS.iter().any(|(k, _)| *k == self.sortierung) {
                self.sortierung.clone()
            } else {
                "datum_ab".into()
            },
            seite: self.seite.max(1),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Eingaben und Vorschlagsdaten für das Formular
// ---------------------------------------------------------------------------------------------

/// Eine Zeile im Formular. `konto` und `projekt` sind Auswahltexte („1020 · Bankguthaben“).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LineInput {
    pub seite: String,
    pub konto: String,
    pub betrag: String,
    pub mwst: String,
    pub projekt: String,
}

impl LineInput {
    /// Leere Zeile (ohne Eingaben) wird beim Speichern übergangen.
    pub fn is_empty(&self) -> bool {
        self.konto.trim().is_empty()
            && self.betrag.trim().is_empty()
            && self.mwst.trim().is_empty()
            && self.projekt.trim().is_empty()
    }
}

/// Eingabe des Formulars „Neue Buchung“ (AK-3).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BookingInput {
    pub datum: String,
    pub belegnummer: String,
    pub text: String,
    /// `brutto` oder `netto` (AK-5, Vorgabe brutto)
    pub betragsart: String,
    pub zeilen: Vec<LineInput>,
    /// `true` = buchen (AK-6), `false` = als Entwurf speichern (AK-10)
    pub buchen: bool,
}

/// Konto für die Auswahl im Formular (nur aktive Konten, AK-4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AccountOption {
    pub id: i64,
    pub nummer: String,
    pub bezeichnung: String,
    pub vorgabe_mwst: Option<i64>,
}

/// MWST-Code für das Formular (nur aktive Codes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct CodeOption {
    pub id: i64,
    pub code: String,
    pub bezeichnung: String,
    pub satz: i64,
    pub gueltig_ab: String,
    pub gueltig_bis: Option<String>,
    pub konto_id: i64,
}

/// Projekt für die Auswahl im Formular.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProjectOption {
    pub id: i64,
    pub nummer: String,
    pub name: String,
}

/// Alles, was das Formular braucht: Vorgaben, Auswahllisten und die MWST-Pflicht.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BookingForm {
    pub heute: String,
    pub belegnummer: String,
    pub konten: Vec<AccountOption>,
    pub codes: Vec<CodeOption>,
    pub projekte: Vec<ProjectOption>,
    /// Beginn der MWST-Pflicht (`None` = nicht eingeschaltet, AK-10 ANF-012)
    pub mwst_pflicht_ab: Option<String>,
}

// ---------------------------------------------------------------------------------------------
// Prüfung und Umwandlung
// ---------------------------------------------------------------------------------------------

/// Geprüfte Zeile mit Kennungen.
#[derive(Clone, Debug, PartialEq)]
pub struct LineValues {
    /// `true` = Sollseite
    pub soll: bool,
    pub konto_id: i64,
    /// Betrag in Rappen, wie eingegeben (netto oder brutto je nach `brutto`)
    pub betrag: i64,
    pub mwst_code_id: Option<i64>,
    pub projekt_id: Option<i64>,
}

/// Geprüfte Buchung.
#[derive(Clone, Debug, PartialEq)]
pub struct BookingValues {
    pub datum: String,
    pub belegnummer: String,
    pub text: String,
    pub brutto: bool,
    pub zeilen: Vec<LineValues>,
    pub buchen: bool,
}

/// Zeile nach der MWST-Aufteilung, so wie sie in der Datenbank steht.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectiveLine {
    pub soll: bool,
    pub konto_id: i64,
    pub betrag: i64,
    pub mwst_code_id: Option<i64>,
    pub mwst_basis: Option<i64>,
    pub ist_mwst_zeile: bool,
    pub projekt_id: Option<i64>,
}

/// Ganzzahlige Division mit kaufmännischer Rundung (ab 0.5 aufrunden).
pub fn div_round(num: i128, den: i128) -> i128 {
    (num * 2 + den) / (den * 2)
}

/// Teilt einen Betrag in Nettobasis und MWST (Rappen). `satz` in Hundertstel Prozent (810 = 8.10 %).
/// Brutto: Steuer aus dem Bruttobetrag herausrechnen. Netto: Steuer auf die Basis aufschlagen.
pub fn tax_split(betrag: i64, satz: i64, brutto: bool) -> (i64, i64) {
    let (b, r) = (i128::from(betrag), i128::from(satz));
    if brutto {
        let tax = div_round(b * r, 10_000 + r);
        ((b - tax) as i64, tax as i64)
    } else {
        let tax = div_round(b * r, 10_000);
        (betrag, tax as i64)
    }
}

/// Zerlegt die Eingaben in die Zeilen der Datenbank: Eine Zeile mit MWST-Code bekommt eine
/// zusätzliche Steuerzeile auf dem MWST-Konto des Codes, auf derselben Seite (AK-5).
pub fn expand(values: &BookingValues, codes: &[CodeOption]) -> Vec<EffectiveLine> {
    let mut lines = Vec::new();
    for z in &values.zeilen {
        let code = z
            .mwst_code_id
            .and_then(|id| codes.iter().find(|c| c.id == id));
        match code {
            Some(code) => {
                let (basis, tax) = tax_split(z.betrag, code.satz, values.brutto);
                lines.push(EffectiveLine {
                    soll: z.soll,
                    konto_id: z.konto_id,
                    betrag: basis,
                    mwst_code_id: Some(code.id),
                    mwst_basis: Some(basis),
                    ist_mwst_zeile: false,
                    projekt_id: z.projekt_id,
                });
                if tax > 0 {
                    lines.push(EffectiveLine {
                        soll: z.soll,
                        konto_id: code.konto_id,
                        betrag: tax,
                        mwst_code_id: None,
                        mwst_basis: None,
                        ist_mwst_zeile: true,
                        projekt_id: None,
                    });
                }
            }
            None => lines.push(EffectiveLine {
                soll: z.soll,
                konto_id: z.konto_id,
                betrag: z.betrag,
                mwst_code_id: None,
                mwst_basis: None,
                ist_mwst_zeile: false,
                projekt_id: z.projekt_id,
            }),
        }
    }
    lines
}

/// Summen Soll und Haben der aufgeteilten Zeilen in Rappen (AK-6).
pub fn totals(lines: &[EffectiveLine]) -> (i64, i64) {
    lines.iter().fold((0, 0), |(s, h), l| {
        if l.soll {
            (s + l.betrag, h)
        } else {
            (s, h + l.betrag)
        }
    })
}

/// Fehlerschlüssel einer Zeile, z. B. `zeile2_konto` (Zählung ab 1, nach dem Entfernen leerer Zeilen).
fn line_key(n: usize, field: &str) -> String {
    format!("zeile{n}_{field}")
}

/// Prüft die Eingaben gegen die Stammdaten (Konten, Codes, Projekte, MWST-Pflicht). Die Stammdaten
/// liefert der Server; so lässt sich die Prüfung ohne Datenbank testen.
/// Gebuchte Buchungen brauchen mindestens zwei Zeilen und müssen ausgeglichen sein (AK-6).
pub fn check(input: &BookingInput, form: &BookingForm) -> Result<BookingValues, FieldErrors> {
    let mut errors = FieldErrors::new();
    let datum = input.datum.trim().to_string();
    if !is_valid_date(&datum) {
        errors.insert("datum".into(), "Bitte ein gültiges Datum eingeben.".into());
    }
    let belegnummer = input.belegnummer.trim().to_string();
    if belegnummer.is_empty() {
        errors.insert(
            "belegnummer".into(),
            "Bitte eine Belegnummer eingeben.".into(),
        );
    } else if belegnummer.chars().count() > BELEG_MAX {
        errors.insert(
            "belegnummer".into(),
            format!("Höchstens {BELEG_MAX} Zeichen."),
        );
    }
    let text = input.text.trim().to_string();
    if text.is_empty() {
        errors.insert("text".into(), "Bitte einen Text eingeben.".into());
    } else if text.chars().count() > TEXT_MAX {
        errors.insert("text".into(), format!("Höchstens {TEXT_MAX} Zeichen."));
    }
    let brutto = input.betragsart != "netto";

    let mut zeilen = Vec::new();
    let mut n = 0;
    for line in input.zeilen.iter().filter(|z| !z.is_empty()) {
        n += 1;
        let (konto_id, konto_error) = account_for(line, form);
        if let Some(message) = konto_error {
            errors.insert(line_key(n, "konto"), message);
        }
        let betrag = match parse_fixed(line.betrag.trim(), 2) {
            Some(v) if v > 0 => Some(v),
            _ => {
                errors.insert(
                    line_key(n, "betrag"),
                    "Bitte einen Betrag über 0 eingeben (z. B. 108.10).".into(),
                );
                None
            }
        };
        let seite_soll = match line.seite.as_str() {
            "soll" => Some(true),
            "haben" => Some(false),
            _ => {
                errors.insert(line_key(n, "seite"), "Bitte Soll oder Haben wählen.".into());
                None
            }
        };
        let mwst_code_id = mwst_for(line, &datum, form, n, &mut errors);
        let projekt_id = project_for(line, form, n, &mut errors);
        if let (Some(soll), Some(betrag), Some(konto_id)) = (seite_soll, betrag, konto_id) {
            zeilen.push(LineValues {
                soll,
                konto_id,
                betrag,
                mwst_code_id,
                projekt_id,
            });
        }
    }

    if input.buchen && zeilen.len() < 2 && errors.is_empty() {
        errors.insert(
            "zeilen".into(),
            "Eine Buchung braucht mindestens zwei Zeilen.".into(),
        );
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    let values = BookingValues {
        datum,
        belegnummer,
        text,
        brutto,
        zeilen,
        buchen: input.buchen,
    };
    if values.buchen {
        let (soll, haben) = totals(&expand(&values, &form.codes));
        if soll != haben {
            errors.insert(
                "ausgleich".into(),
                format!(
                    "Soll und Haben sind nicht ausgeglichen: Differenz {}.",
                    format_amount((soll - haben).abs())
                ),
            );
            return Err(errors);
        }
    }
    Ok(values)
}

/// Konto der Zeile: Nummer aus dem Auswahltext, nur aktive Konten (AK-4).
fn account_for(line: &LineInput, form: &BookingForm) -> (Option<i64>, Option<String>) {
    let key = key_of(&line.konto);
    if key.is_empty() {
        return (None, Some("Bitte ein Konto wählen.".into()));
    }
    match form.konten.iter().find(|k| k.nummer == key) {
        Some(konto) => (Some(konto.id), None),
        None => (
            None,
            Some("Dieses Konto gibt es nicht oder es ist inaktiv.".into()),
        ),
    }
}

/// MWST-Code der Zeile. Prüft Pflicht, Beginn und Gültigkeit am Belegdatum (ANF-012 AK-6, AK-10).
fn mwst_for(
    line: &LineInput,
    datum: &str,
    form: &BookingForm,
    n: usize,
    errors: &mut FieldErrors,
) -> Option<i64> {
    let raw = line.mwst.trim();
    if raw.is_empty() {
        return None;
    }
    let message = |text: String, errors: &mut FieldErrors| {
        errors.insert(line_key(n, "mwst"), text);
    };
    let Some(pflicht_ab) = form.mwst_pflicht_ab.as_deref() else {
        message(
            "Die MWST-Pflicht ist nicht eingeschaltet. Der Beleg wird ohne MWST erfasst.".into(),
            errors,
        );
        return None;
    };
    if is_valid_date(datum) && datum < pflicht_ab {
        message(
            format!(
                "Die MWST-Pflicht gilt erst ab dem {}. Für diesen Beleg wird kein MWST-Code verwendet.",
                german_date(pflicht_ab)
            ),
            errors,
        );
        return None;
    }
    let code = raw
        .parse::<i64>()
        .ok()
        .and_then(|id| form.codes.iter().find(|c| c.id == id));
    match code {
        None => {
            message(
                "Der MWST-Code ist nicht gefunden oder inaktiv.".into(),
                errors,
            );
            None
        }
        Some(code) if !code_valid_on(code, datum) => {
            message(
                format!(
                    "Der Code {} ist am {} nicht gültig.",
                    code.code,
                    german_date(datum)
                ),
                errors,
            );
            None
        }
        Some(code) => Some(code.id),
    }
}

/// Gültigkeit eines Codes am Belegdatum (AK-6 ANF-012).
pub fn code_valid_on(code: &CodeOption, datum: &str) -> bool {
    code.gueltig_ab.as_str() <= datum && code.gueltig_bis.as_deref().is_none_or(|bis| datum <= bis)
}

/// Projekt der Zeile (optional, AK-8).
fn project_for(
    line: &LineInput,
    form: &BookingForm,
    n: usize,
    errors: &mut FieldErrors,
) -> Option<i64> {
    let key = key_of(&line.projekt);
    if key.is_empty() {
        return None;
    }
    match form.projekte.iter().find(|p| p.nummer == key) {
        Some(projekt) => Some(projekt.id),
        None => {
            errors.insert(
                line_key(n, "projekt"),
                "Dieses Projekt gibt es nicht.".into(),
            );
            None
        }
    }
}

/// Hinweis, warum eine Buchung nicht storniert werden kann (AK-11, AK-12); `None` = stornierbar.
pub fn storno_hint(head: &BookingHead) -> Option<&'static str> {
    if head.can_storno() {
        None
    } else if head.status == "storniert" {
        Some("Diese Buchung ist bereits storniert.")
    } else if head.storno_von_id.is_some() {
        Some("Eine Stornobuchung lässt sich nicht erneut stornieren.")
    } else if head.herkunft != "manuell" {
        Some("Automatische Buchungen werden über ihren Beleg storniert.")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn account(id: i64, nummer: &str, vorgabe: Option<i64>) -> AccountOption {
        AccountOption {
            id,
            nummer: nummer.into(),
            bezeichnung: format!("Konto {nummer}"),
            vorgabe_mwst: vorgabe,
        }
    }

    fn code(id: i64, satz: i64, konto_id: i64) -> CodeOption {
        CodeOption {
            id,
            code: format!("C{id}"),
            bezeichnung: "Test".into(),
            satz,
            gueltig_ab: "2024-01-01".into(),
            gueltig_bis: None,
            konto_id,
        }
    }

    fn form() -> BookingForm {
        BookingForm {
            heute: "2026-11-26".into(),
            belegnummer: "M-2026-0001".into(),
            konten: vec![
                account(1, "1020", None),
                account(2, "4000", Some(10)),
                account(3, "1170", None),
            ],
            codes: vec![code(10, 810, 3)],
            projekte: vec![ProjectOption {
                id: 7,
                nummer: "P-2026-001".into(),
                name: "Hallenbau".into(),
            }],
            mwst_pflicht_ab: Some("2024-01-01".into()),
        }
    }

    fn line(seite: &str, konto: &str, betrag: &str, mwst: &str) -> LineInput {
        LineInput {
            seite: seite.into(),
            konto: konto.into(),
            betrag: betrag.into(),
            mwst: mwst.into(),
            projekt: String::new(),
        }
    }

    fn input(zeilen: Vec<LineInput>, brutto: bool, buchen: bool) -> BookingInput {
        BookingInput {
            datum: "2026-11-20".into(),
            belegnummer: " M-2026-0001 ".into(),
            text: " Büromaterial ".into(),
            betragsart: if brutto { "brutto" } else { "netto" }.into(),
            zeilen,
            buchen,
        }
    }

    #[test]
    fn tax_brutto_and_netto() {
        // 108.10 brutto mit 8.1 % → 100.00 netto und 8.10 MWST
        assert_eq!(tax_split(10_810, 810, true), (10_000, 810));
        // 100.00 netto mit 8.1 % → 8.10 MWST
        assert_eq!(tax_split(10_000, 810, false), (10_000, 810));
        // Rappen werden kaufmännisch gerundet: 0.05 netto × 8.1 % = 0.00405 → 0.00
        assert_eq!(tax_split(5, 810, false), (5, 0));
        // Satz 0 erzeugt keine Steuer
        assert_eq!(tax_split(10_000, 0, false), (10_000, 0));
    }

    #[test]
    fn expand_adds_tax_line_on_same_side() {
        let values = check(
            &input(
                vec![
                    line("soll", "4000 · Konto 4000", "100.00", "10"),
                    line("haben", "1020", "108.10", ""),
                ],
                false,
                true,
            ),
            &form(),
        )
        .unwrap();
        let lines = expand(&values, &form().codes);
        // Aufwand 100.00 (Basis), Vorsteuer 8.10 auf derselben Seite, Bank 108.10
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].betrag, 10_000);
        assert_eq!(lines[0].mwst_basis, Some(10_000));
        assert!(lines[1].ist_mwst_zeile && lines[1].soll);
        assert_eq!((lines[1].konto_id, lines[1].betrag), (3, 810));
        assert_eq!(totals(&lines), (10_810, 10_810));
    }

    #[test]
    fn brutto_is_default_and_balances() {
        let values = check(
            &input(
                vec![
                    line("soll", "4000", "108.10", "10"),
                    line("haben", "1020", "108.10", ""),
                ],
                true,
                true,
            ),
            &form(),
        )
        .unwrap();
        assert!(values.brutto);
        let (soll, haben) = totals(&expand(&values, &form().codes));
        assert_eq!((soll, haben), (10_810, 10_810));
    }

    #[test]
    fn unbalanced_booking_is_rejected_but_draft_is_kept() {
        let zeilen = vec![
            line("soll", "4000", "100.00", ""),
            line("haben", "1020", "99.00", ""),
        ];
        let err = check(&input(zeilen.clone(), true, true), &form()).unwrap_err();
        assert!(err["ausgleich"].contains("Differenz 1.00"));
        assert!(check(&input(zeilen, true, false), &form()).is_ok());
    }

    #[test]
    fn booking_needs_two_lines() {
        let err = check(
            &input(vec![line("soll", "4000", "10.00", "")], true, true),
            &form(),
        )
        .unwrap_err();
        assert!(err.contains_key("zeilen"));
        // Entwurf mit einer Zeile ist erlaubt
        assert!(
            check(
                &input(vec![line("soll", "4000", "10.00", "")], true, false),
                &form()
            )
            .is_ok()
        );
    }

    #[test]
    fn empty_rows_are_skipped_and_errors_name_the_row() {
        let zeilen = vec![
            line("soll", "4000", "10.00", ""),
            LineInput::default(),
            line("haben", "9999", "10.00", ""),
            line("", "1020", "0", ""),
        ];
        let err = check(&input(zeilen, true, false), &form()).unwrap_err();
        assert!(err.contains_key("zeile2_konto"));
        assert!(err.contains_key("zeile3_seite"));
        assert!(err.contains_key("zeile3_betrag"));
        assert!(!err.contains_key("zeile3_konto"));
        assert!(!err.contains_key("zeile1_konto"));
    }

    #[test]
    fn mwst_rules_by_date_and_pflicht() {
        let mut f = form();
        // Beleg vor dem Beginn der MWST-Pflicht
        f.mwst_pflicht_ab = Some("2026-12-01".into());
        let zeilen = vec![
            line("soll", "4000", "10.00", "10"),
            line("haben", "1020", "10.00", ""),
        ];
        let err = check(&input(zeilen.clone(), true, false), &f).unwrap_err();
        assert!(err["zeile1_mwst"].contains("gilt erst ab dem 01.12.2026"));
        // Pflicht nicht eingeschaltet
        f.mwst_pflicht_ab = None;
        assert!(check(&input(zeilen, true, false), &f).is_err());
    }

    #[test]
    fn code_must_be_valid_on_date() {
        let mut f = form();
        f.codes[0].gueltig_bis = Some("2026-11-01".into());
        let zeilen = vec![line("soll", "4000", "10.00", "10")];
        let err = check(&input(zeilen, true, false), &f).unwrap_err();
        assert!(err["zeile1_mwst"].contains("nicht gültig"));
    }

    #[test]
    fn project_and_account_keys() {
        assert_eq!(key_of("1020 · Bankguthaben UBS"), "1020");
        assert_eq!(key_of("  1020 "), "1020");
        let mut zeile = line("soll", "4000", "10.00", "");
        zeile.projekt = "P-2026-001 · Hallenbau".into();
        let values = check(
            &input(vec![zeile, line("haben", "1020", "10.00", "")], true, false),
            &form(),
        )
        .unwrap();
        assert_eq!(values.zeilen[0].projekt_id, Some(7));
    }

    #[test]
    fn list_labels_show_diverse_for_collective_bookings() {
        let head = |status: &str, herkunft: &str, storno: Option<i64>| BookingHead {
            id: 1,
            geschaeftsjahr_id: 1,
            geschaeftsjahr: "2026".into(),
            journalnummer: Some(1),
            datum: "2026-01-02".into(),
            belegnummer: "M-1".into(),
            text: "Test".into(),
            status: status.into(),
            herkunft: herkunft.into(),
            storno_von_id: storno,
            erfasst_von: None,
            erstellt_am: String::new(),
            gebucht_am: None,
        };
        assert!(head("gebucht", "manuell", None).can_storno());
        assert!(!head("gebucht", "debitor", None).can_storno());
        assert!(!head("gebucht", "manuell", Some(4)).can_storno());
        assert!(!head("storniert", "manuell", None).can_storno());
        assert!(head("entwurf", "manuell", None).is_editable());
        assert!(storno_hint(&head("gebucht", "debitor", None)).is_some());
        assert!(storno_hint(&head("gebucht", "manuell", None)).is_none());
    }

    #[test]
    fn filter_normalizes_unknown_values() {
        let f = JournalFilter {
            geschaeftsjahr_id: 3,
            suche: "  Bank  ".into(),
            herkunft: "foo".into(),
            status: "gebucht".into(),
            von: "2026-02-30".into(),
            bis: "2026-12-31".into(),
            sortierung: "irgendwas".into(),
            seite: 0,
        }
        .normalized();
        assert_eq!(f.suche, "Bank");
        assert_eq!(f.herkunft, "");
        assert_eq!(f.status, "gebucht");
        assert_eq!(f.von, "");
        assert_eq!(f.bis, "2026-12-31");
        assert_eq!(f.sortierung, "datum_ab");
        assert_eq!(f.seite, 1);
    }
}
