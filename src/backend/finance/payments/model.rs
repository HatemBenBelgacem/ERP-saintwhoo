//! Zahlungen und offene Posten (ANF-020): Datentypen, Vorschlag zur Verteilung, Prüfregeln, Zeilen der
//! Buchung mit Skonto (MWST-Korrektur) und Kursdifferenz, Altersgliederung. Server und Browser.
//!
//! Beträge in Rappen der Belegwährung, Kurse in Millionsteln. Die Buchung zeigt der Eingang (Debitoren);
//! beim Ausgang (Kreditoren) sind alle Seiten vertauscht.

use serde::{Deserialize, Serialize};

use crate::backend::finance::debtors::model::to_chf;
use crate::backend::finance::journal::model::{EffectiveLine, div_round, tax_split};
use crate::backend::projects::model::is_valid_date;
use crate::backend::purchase_orders::model::{format_amount, parse_fixed};
pub use crate::forms::{FieldErrors, PickOption, SaveOutcome};

pub const PAYMENTS_URL: &str = "/finanzen/zahlungen";
pub const PAGE_SIZE: i64 = 50;
pub const TEXT_MAX: usize = 200;
pub const SEARCH_MAX: usize = 100;

pub const KINDS: &[(&str, &str)] = &[
    ("eingang", "Eingang"),
    ("ausgang", "Ausgang"),
    ("verrechnung", "Verrechnung"),
];

pub const KIND_FILTERS: &[(&str, &str)] = &[
    ("", "Alle Arten"),
    ("eingang", "Eingang"),
    ("ausgang", "Ausgang"),
    ("verrechnung", "Verrechnung"),
];

pub const SORT_COLUMNS: &[(&str, &str)] = &[
    ("datum", "Datum"),
    ("art", "Art"),
    ("partner", "Partner"),
    ("bankkonto", "Bankkonto"),
    ("betrag", "Betrag"),
    ("status", "Status"),
];

/// Altersgliederung (AK-13): Beschriftungen der fünf Spalten.
pub const AGING_LABELS: [&str; 5] = ["nicht fällig", "1–30", "31–60", "61–90", "über 90"];

fn label_in(list: &'static [(&'static str, &'static str)], key: &str) -> &'static str {
    list.iter().find(|(k, _)| *k == key).map_or("–", |(_, l)| l)
}

pub fn kind_label(key: &str) -> &'static str {
    label_in(KINDS, key)
}

/// Seite zur Art: Eingang → Debitoren, Ausgang → Kreditoren.
pub fn side_of(art: &str) -> &'static str {
    if art == "ausgang" {
        "kreditoren"
    } else {
        "debitoren"
    }
}

pub fn status_badge(status: &str) -> &'static str {
    if status == "storniert" {
        "badge badge-sm badge-error badge-soft"
    } else {
        "badge badge-sm badge-success badge-soft"
    }
}

// ---------------------------------------------------------------------------------------------
// Offene Posten eines Partners
// ---------------------------------------------------------------------------------------------

/// Offener Posten eines Partners: Rechnung, Gutschrift oder Vorauszahlung.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct OpenItem {
    /// `rechnung`, `gutschrift` oder `vorauszahlung`
    pub art: String,
    /// Kennung des Belegs bzw. der Zahlung
    pub beleg_id: i64,
    pub nummer: String,
    /// Rechnungsnummer des Lieferanten (Kreditoren)
    pub referenz: Option<String>,
    pub datum: String,
    pub faelligkeit: Option<String>,
    pub skonto_bis: Option<String>,
    /// Hundertstel Prozent
    pub skonto_prozent: Option<i64>,
    pub waehrung: String,
    /// Buchungskurs des Belegs (Millionstel)
    pub kurs: i64,
    pub gesamtbetrag: i64,
    pub offen: i64,
}

impl OpenItem {
    /// Schlüssel in Formular und Webadresse: `r:12`, `g:5`, `v:7`.
    pub fn key(&self) -> String {
        format!("{}:{}", key_prefix(&self.art), self.beleg_id)
    }

    pub fn is_invoice(&self) -> bool {
        self.art == "rechnung"
    }

    /// Möglicher Skonto am Zahlungsdatum (AK-3): innerhalb der Frist Prozent vom Gesamtbetrag, höchstens offen.
    pub fn discount_on(&self, datum: &str) -> Option<i64> {
        let (bis, prozent) = (self.skonto_bis.as_deref()?, self.skonto_prozent?);
        if !self.is_invoice() || datum > bis {
            return None;
        }
        let skonto = div_round(i128::from(self.gesamtbetrag) * i128::from(prozent), 10_000) as i64;
        Some(skonto.min(self.offen))
    }
}

pub fn key_prefix(art: &str) -> &'static str {
    match art {
        "gutschrift" => "g",
        "vorauszahlung" => "v",
        _ => "r",
    }
}

/// Zerlegt einen Schlüssel `r:12` in Art und Kennung.
pub fn parse_key(key: &str) -> Option<(&'static str, i64)> {
    let (prefix, id) = key.split_once(':')?;
    let art = match prefix {
        "r" => "rechnung",
        "g" => "gutschrift",
        "v" => "vorauszahlung",
        _ => return None,
    };
    Some((art, id.parse().ok()?))
}

/// Belegart für die Anzeige.
pub fn item_label(art: &str) -> &'static str {
    match art {
        "gutschrift" => "Gutschrift",
        "vorauszahlung" => "Vorauszahlung",
        _ => "Rechnung",
    }
}

// ---------------------------------------------------------------------------------------------
// Eingaben
// ---------------------------------------------------------------------------------------------

/// Zuordnung zu einem offenen Posten (Texte aus dem Formular).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AllocationInput {
    pub key: String,
    pub betrag: String,
    pub skonto: String,
    pub differenz: String,
    pub differenz_konto_id: String,
}

/// Eingabe einer Zahlung bzw. Verrechnung (AK-1, AK-6, AK-11).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaymentInput {
    /// `eingang`, `ausgang` oder `verrechnung`
    pub art: String,
    /// Bei Verrechnung: `debitoren` oder `kreditoren`
    pub seite: String,
    pub datum: String,
    pub bankkonto_id: String,
    pub waehrung: String,
    pub betrag: String,
    pub adresse_id: String,
    pub text: String,
    pub zeilen: Vec<AllocationInput>,
}

/// Geprüfte Zuordnung (Rappen).
#[derive(Clone, Debug, PartialEq)]
pub struct AllocationValues {
    pub art: String,
    pub beleg_id: i64,
    pub betrag: i64,
    pub skonto: i64,
    pub differenz: i64,
    pub differenz_konto_id: Option<i64>,
}

impl AllocationValues {
    /// Betrag, um den der offene Posten sinkt.
    pub fn cleared(&self) -> i64 {
        self.betrag + self.skonto + self.differenz
    }
}

/// Geprüfte Zahlung.
#[derive(Clone, Debug, PartialEq)]
pub struct PaymentValues {
    pub art: String,
    pub seite: String,
    pub datum: String,
    pub bankkonto_id: Option<i64>,
    pub waehrung: String,
    pub betrag: i64,
    pub adresse_id: i64,
    pub text: Option<String>,
    pub zeilen: Vec<AllocationValues>,
    /// Nicht zugeordneter Mehrbetrag (AK-5)
    pub vorauszahlung: i64,
}

/// Prüft die Eingabe gegen die offenen Posten des Partners. `konten`: Kennungen wählbarer Konten für Differenzen.
pub fn check(
    input: &PaymentInput,
    items: &[OpenItem],
    konten: &[i64],
) -> Result<PaymentValues, FieldErrors> {
    let mut errors = FieldErrors::new();
    let mut err = |f: &str, m: String| {
        errors.entry(f.into()).or_insert(m);
    };
    let art = input.art.trim();
    if !KINDS.iter().any(|(k, _)| *k == art) {
        err("art", "Unbekannte Art der Zahlung.".into());
    }
    let verrechnung = art == "verrechnung";
    let seite = if verrechnung {
        input.seite.trim().to_string()
    } else {
        side_of(art).to_string()
    };
    if seite != "debitoren" && seite != "kreditoren" {
        err("seite", "Bitte Debitoren oder Kreditoren wählen.".into());
    }
    let datum = input.datum.trim().to_string();
    if !is_valid_date(&datum) {
        err("datum", "Bitte ein gültiges Datum eingeben.".into());
    }
    let adresse_id = input.adresse_id.trim().parse::<i64>().ok();
    if adresse_id.is_none() {
        err(
            "adresse_id",
            if seite == "kreditoren" {
                "Bitte einen Lieferanten wählen.".into()
            } else {
                "Bitte einen Kunden wählen.".into()
            },
        );
    }
    let bankkonto_id = input.bankkonto_id.trim().parse::<i64>().ok();
    if !verrechnung && bankkonto_id.is_none() {
        err("bankkonto_id", "Bitte ein Bankkonto wählen.".into());
    }
    let waehrung = input.waehrung.trim().to_uppercase();
    if !crate::backend::finance::currency::model::is_code(&waehrung) {
        err("waehrung", "Bitte eine Währung wählen.".into());
    }
    let betrag = if verrechnung {
        Some(0)
    } else {
        match parse_fixed(input.betrag.trim(), 2) {
            Some(b) if b > 0 => Some(b),
            _ => {
                err(
                    "betrag",
                    "Bitte einen Betrag über 0 eingeben (z. B. 108.10).".into(),
                );
                None
            }
        }
    };
    let text = input.text.trim();
    if text.chars().count() > TEXT_MAX {
        err("text", format!("Höchstens {TEXT_MAX} Zeichen."));
    }

    let amount = |s: &str| -> Option<i64> {
        let s = s.trim();
        if s.is_empty() {
            Some(0)
        } else {
            parse_fixed(s, 2)
        }
    };
    let mut zeilen = Vec::new();
    for z in &input.zeilen {
        let Some((zart, beleg_id)) = parse_key(&z.key) else {
            continue;
        };
        let key = z.key.clone();
        let (Some(b), Some(s), Some(d)) =
            (amount(&z.betrag), amount(&z.skonto), amount(&z.differenz))
        else {
            err(
                &format!("{key}_betrag"),
                "Bitte Beträge mit höchstens 2 Nachkommastellen eingeben.".into(),
            );
            continue;
        };
        if b + s + d == 0 {
            continue;
        }
        let Some(item) = items
            .iter()
            .find(|i| i.art == zart && i.beleg_id == beleg_id)
        else {
            err(
                &format!("{key}_betrag"),
                "Dieser Posten ist nicht mehr offen.".into(),
            );
            continue;
        };
        if item.waehrung != waehrung {
            err(
                &format!("{key}_betrag"),
                format!(
                    "Der Posten ist in {}; die Zahlung in {waehrung}.",
                    item.waehrung
                ),
            );
        }
        if (s > 0 || d > 0) && !item.is_invoice() {
            err(
                &format!("{key}_skonto"),
                "Skonto und Differenz gibt es nur bei Rechnungen.".into(),
            );
        }
        if verrechnung && (s > 0 || d > 0) {
            err(
                &format!("{key}_skonto"),
                "Bei einer Verrechnung gibt es weder Skonto noch Differenz.".into(),
            );
        }
        if zart == "vorauszahlung" && !verrechnung {
            err(
                &format!("{key}_betrag"),
                "Vorauszahlungen werden über „Verrechnen“ ausgeglichen.".into(),
            );
        }
        if b + s + d > item.offen {
            err(
                &format!("{key}_betrag"),
                format!("Höchstens der offene Betrag {}.", format_amount(item.offen)),
            );
        }
        let konto = z.differenz_konto_id.trim().parse::<i64>().ok();
        if d > 0 && !konto.is_some_and(|k| konten.contains(&k)) {
            err(
                &format!("{key}_differenz"),
                "Bitte ein Konto für die Differenz wählen.".into(),
            );
        }
        zeilen.push(AllocationValues {
            art: zart.to_string(),
            beleg_id,
            betrag: b,
            skonto: s,
            differenz: d,
            differenz_konto_id: konto.filter(|_| d > 0),
        });
    }

    // Rechnungen erhöhen, Gutschriften und Vorauszahlungen verringern den Betrag der Zahlung
    let zugeordnet: i64 = zeilen
        .iter()
        .map(|z| {
            if z.art == "rechnung" {
                z.betrag
            } else {
                -z.betrag
            }
        })
        .sum();
    let mut vorauszahlung = 0;
    if let Some(betrag) = betrag {
        if verrechnung {
            if zeilen.is_empty() {
                err(
                    "zeilen",
                    "Bitte eine Rechnung und eine Gutschrift bzw. Vorauszahlung wählen.".into(),
                );
            } else if zugeordnet != 0 {
                err(
                    "zeilen",
                    format!(
                        "Rechnungen und Gutschriften müssen sich ausgleichen: Differenz {}.",
                        format_amount(zugeordnet)
                    ),
                );
            }
        } else if zugeordnet > betrag {
            err(
                "zeilen",
                format!(
                    "Zugeordnet ist {} mehr als der Betrag der Zahlung.",
                    format_amount(zugeordnet - betrag)
                ),
            );
        } else if zugeordnet < 0 {
            err(
                "zeilen",
                "Die Gutschriften übersteigen die Rechnungen. Bitte verrechnen.".into(),
            );
        } else {
            vorauszahlung = betrag - zugeordnet;
        }
    }
    match (adresse_id, betrag) {
        (Some(adresse_id), Some(betrag)) if errors.is_empty() => Ok(PaymentValues {
            art: art.to_string(),
            seite,
            datum,
            bankkonto_id: bankkonto_id.filter(|_| !verrechnung),
            waehrung,
            betrag: if verrechnung {
                zeilen
                    .iter()
                    .filter(|z| z.art == "rechnung")
                    .map(|z| z.betrag)
                    .sum()
            } else {
                betrag
            },
            adresse_id,
            text: (!text.is_empty()).then(|| text.to_string()),
            zeilen,
            vorauszahlung,
        }),
        _ => Err(errors),
    }
}

/// Vorschlag zur Verteilung (AK-2, AK-3): Rechnungen nach ältester Fälligkeit; reicht der Rest innerhalb der
/// Skontofrist bis auf den Skonto, wird der Skonto vorgeschlagen. Gutschriften und Vorauszahlungen bleiben leer.
pub fn propose(items: &[OpenItem], betrag: i64, datum: &str) -> Vec<AllocationInput> {
    let mut order: Vec<&OpenItem> = items.iter().filter(|i| i.is_invoice()).collect();
    order.sort_by(|a, b| {
        a.faelligkeit
            .cmp(&b.faelligkeit)
            .then_with(|| a.beleg_id.cmp(&b.beleg_id))
    });
    let mut rest = betrag;
    let mut result = Vec::new();
    for item in order {
        if rest <= 0 {
            break;
        }
        let (paid, skonto) = if rest >= item.offen {
            (item.offen, 0)
        } else {
            match item.discount_on(datum) {
                Some(s) if item.offen - rest <= s => (rest, item.offen - rest),
                _ => (rest, 0),
            }
        };
        rest -= paid;
        result.push(AllocationInput {
            key: item.key(),
            betrag: fixed(paid),
            skonto: if skonto > 0 {
                fixed(skonto)
            } else {
                String::new()
            },
            ..Default::default()
        });
    }
    result
}

fn fixed(cents: i64) -> String {
    crate::backend::purchase_orders::model::format_fixed(cents, 2)
}

// ---------------------------------------------------------------------------------------------
// Buchung
// ---------------------------------------------------------------------------------------------

/// MWST-Anteil einer Rechnung für die Skonto-Korrektur: Brutto je Code, Satz und Steuerkonto.
#[derive(Clone, Debug, PartialEq)]
pub struct VatShare {
    pub code_id: i64,
    pub rate: i64,
    pub tax_konto_id: i64,
    pub gross: i64,
}

/// Zuordnung mit allem, was die Buchung braucht.
#[derive(Clone, Debug, PartialEq)]
pub struct BookingItem {
    pub values: AllocationValues,
    /// Buchungskurs des Belegs
    pub kurs: i64,
    pub gesamtbetrag: i64,
    pub vat: Vec<VatShare>,
}

/// Konten der Buchung.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaymentAccounts {
    pub bank: i64,
    /// Debitoren bzw. Kreditoren
    pub partner: i64,
    pub skonto: Option<i64>,
    pub kursdifferenz: Option<i64>,
}

/// Skonto anteilig nach MWST-Codes der Rechnung (Offene Frage, geklärt): je Code der Bruttoanteil, daraus
/// Netto und Steuer; der Rest (Positionen ohne Code, Rundung) ohne Steuer. Liefert (Code, Steuerkonto, Netto, Steuer).
pub fn split_discount(
    skonto: i64,
    gesamtbetrag: i64,
    vat: &[VatShare],
) -> Vec<(Option<i64>, Option<i64>, i64, i64)> {
    let mut parts = Vec::new();
    let mut verteilt = 0;
    if gesamtbetrag > 0 {
        for share in vat {
            let anteil = div_round(
                i128::from(skonto) * i128::from(share.gross),
                i128::from(gesamtbetrag),
            ) as i64;
            let anteil = anteil.min(skonto - verteilt);
            if anteil <= 0 {
                continue;
            }
            verteilt += anteil;
            let (netto, steuer) = tax_split(anteil, share.rate, true);
            parts.push((Some(share.code_id), Some(share.tax_konto_id), netto, steuer));
        }
    }
    if skonto - verteilt > 0 {
        parts.push((None, None, skonto - verteilt, 0));
    }
    parts
}

/// Zeilen der Buchung einer Zahlung (AK-8, AK-9). Eingang: Soll Bank, Haben Debitoren je Rechnung, Soll Skonto
/// mit Steuerkorrektur, Soll Differenzkonto, Soll Debitoren je Gutschrift, Haben Debitoren für die Vorauszahlung,
/// Kursdifferenz zum Ausgleich. Ausgang: alle Seiten vertauscht.
pub fn payment_lines(
    seite: &str,
    betrag: i64,
    kurs: i64,
    vorauszahlung: i64,
    items: &[BookingItem],
    accounts: PaymentAccounts,
) -> Result<Vec<EffectiveLine>, String> {
    // Für den Eingang: `true` = Soll. Beim Ausgang wird am Schluss umgedreht.
    let mut lines: Vec<EffectiveLine> = Vec::new();
    let mut push = |soll: bool, konto_id: i64, betrag: i64, code: Option<i64>, ist_mwst: bool| {
        if betrag > 0 {
            lines.push(EffectiveLine {
                soll,
                konto_id,
                betrag,
                mwst_code_id: code,
                mwst_basis: code.map(|_| betrag),
                ist_mwst_zeile: ist_mwst,
                projekt_id: None,
            });
        }
    };
    push(true, accounts.bank, to_chf(betrag, kurs), None, false);
    for item in items {
        let v = &item.values;
        if v.art == "rechnung" {
            push(
                false,
                accounts.partner,
                to_chf(v.cleared(), item.kurs),
                None,
                false,
            );
            if v.skonto > 0 {
                let Some(skonto_konto) = accounts.skonto else {
                    return Err(if seite == "kreditoren" {
                        "Das Systemkonto „Skonto erhalten“ fehlt (Einstellungen → Finanzen → Kontenplan → Systemkonten).".into()
                    } else {
                        "Das Systemkonto „Skonto gewährt“ fehlt (Einstellungen → Finanzen → Kontenplan → Systemkonten).".into()
                    });
                };
                for (code, tax_konto, netto, steuer) in
                    split_discount(v.skonto, item.gesamtbetrag, &item.vat)
                {
                    push(true, skonto_konto, to_chf(netto, item.kurs), code, false);
                    if let Some(tax_konto) = tax_konto {
                        push(true, tax_konto, to_chf(steuer, item.kurs), None, true);
                    }
                }
            }
            if v.differenz > 0 {
                let Some(konto) = v.differenz_konto_id else {
                    return Err("Bitte ein Konto für die Differenz wählen.".into());
                };
                push(true, konto, to_chf(v.differenz, item.kurs), None, false);
            }
        } else {
            // Gutschrift: ihr offener Betrag wird mit der Zahlung aufgebraucht
            push(
                true,
                accounts.partner,
                to_chf(v.betrag, item.kurs),
                None,
                false,
            );
        }
    }
    push(
        false,
        accounts.partner,
        to_chf(vorauszahlung, kurs),
        None,
        false,
    );
    // AK-9: Differenz aus unterschiedlichen Kursen auf „Kursdifferenzen“
    let (soll, haben) = crate::backend::finance::journal::model::totals(&lines);
    if soll != haben {
        let Some(konto) = accounts.kursdifferenz else {
            return Err(
                "Das Systemkonto „Kursdifferenzen“ fehlt (Einstellungen → Finanzen → Kontenplan → Systemkonten)."
                    .into(),
            );
        };
        lines.push(EffectiveLine {
            soll: haben > soll,
            konto_id: konto,
            betrag: (soll - haben).abs(),
            mwst_code_id: None,
            mwst_basis: None,
            ist_mwst_zeile: false,
            projekt_id: None,
        });
    }
    if seite == "kreditoren" {
        for l in &mut lines {
            l.soll = !l.soll;
        }
    }
    Ok(lines)
}

/// Text der Buchung, z. B. „Zahlungseingang Muster AG: R-2026-0001, R-2026-0002“ (höchstens 200 Zeichen).
pub fn booking_text(art: &str, partner: &str, nummern: &[String]) -> String {
    let kind = if art == "ausgang" {
        "Zahlungsausgang"
    } else {
        "Zahlungseingang"
    };
    let belege = if nummern.is_empty() {
        "Vorauszahlung".to_string()
    } else {
        nummern.join(", ")
    };
    format!("{kind} {partner}: {belege}")
        .chars()
        .take(crate::backend::finance::journal::model::TEXT_MAX)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Listen und Anzeige
// ---------------------------------------------------------------------------------------------

/// Zeile der Zahlungsliste (AK-12).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PaymentRow {
    pub id: i64,
    pub art: String,
    pub seite: String,
    pub datum: String,
    pub partner: String,
    pub bankkonto: Option<String>,
    pub waehrung: String,
    pub betrag: i64,
    pub vorauszahlung_offen: i64,
    pub belege: Option<String>,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentPage {
    pub rows: Vec<PaymentRow>,
    pub total: i64,
    pub page: i64,
}

impl PaymentPage {
    pub fn page_count(&self) -> i64 {
        ((self.total + PAGE_SIZE - 1) / PAGE_SIZE).max(1)
    }
}

/// Filter der Liste (AK-12), vollständig aus der Webadresse.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PaymentFilter {
    pub suche: String,
    pub art: String,
    pub von: String,
    pub bis: String,
    pub bankkonto_id: i64,
    pub sortierung: String,
    pub richtung: String,
    pub seite: i64,
}

impl PaymentFilter {
    pub fn normalized(&self) -> Self {
        let date = |v: &str| {
            if is_valid_date(v) {
                v.to_string()
            } else {
                String::new()
            }
        };
        let sortierung = if SORT_COLUMNS.iter().any(|(k, _)| *k == self.sortierung) {
            self.sortierung.clone()
        } else {
            "datum".into()
        };
        let richtung = match self.richtung.as_str() {
            "auf" | "ab" => self.richtung.clone(),
            _ => default_dir(&sortierung).into(),
        };
        Self {
            suche: self.suche.trim().chars().take(SEARCH_MAX).collect(),
            art: if KINDS.iter().any(|(k, _)| *k == self.art) {
                self.art.clone()
            } else {
                String::new()
            },
            von: date(&self.von),
            bis: date(&self.bis),
            bankkonto_id: self.bankkonto_id.max(0),
            sortierung,
            richtung,
            seite: self.seite.max(1),
        }
    }
}

pub fn default_dir(sort: &str) -> &'static str {
    if sort == "datum" { "ab" } else { "auf" }
}

/// Ausgleichszeile in der Detailansicht einer Zahlung bzw. eines Belegs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AllocationView {
    pub zahlung_id: i64,
    pub zahlung_art: String,
    pub zahlung_datum: String,
    pub zahlung_status: String,
    /// `rechnung`, `gutschrift` oder `vorauszahlung`
    pub art: String,
    pub beleg_id: i64,
    pub nummer: String,
    pub seite: String,
    pub betrag: i64,
    pub skonto: i64,
    pub differenz: i64,
    pub differenz_konto: Option<String>,
}

impl AllocationView {
    /// Webadresse des Belegs (Rechnung bzw. Gutschrift) oder der Vorauszahlung.
    pub fn href(&self) -> String {
        match (self.art.as_str(), self.seite.as_str()) {
            ("vorauszahlung", _) => format!("{PAYMENTS_URL}/{}", self.beleg_id),
            (_, "kreditoren") => format!("/finanzen/kreditoren/{}", self.beleg_id),
            _ => format!("/finanzen/debitoren/{}", self.beleg_id),
        }
    }
}

/// Kopf einer Zahlung (Detailansicht).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Payment {
    pub id: i64,
    pub art: String,
    pub seite: String,
    pub datum: String,
    pub adresse_id: i64,
    pub partner: String,
    pub bankkonto: Option<String>,
    pub waehrung: String,
    pub kurs: i64,
    pub betrag: i64,
    pub betrag_chf: i64,
    pub vorauszahlung: i64,
    pub vorauszahlung_offen: i64,
    pub status: String,
    pub text: Option<String>,
    pub buchung_id: Option<i64>,
    pub storno_buchung_id: Option<i64>,
    pub erfasst_von: Option<String>,
    pub erstellt_am: String,
}

impl Payment {
    /// Stornieren, solange die Vorauszahlung nicht verrechnet ist (AK-10).
    pub fn can_storno(&self) -> bool {
        self.status == "gebucht" && self.vorauszahlung_offen == self.vorauszahlung
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentDetail {
    pub payment: Payment,
    pub zeilen: Vec<AllocationView>,
    /// Verrechnungen, die die Vorauszahlung dieser Zahlung aufbrauchen (AK-5)
    pub verwendungen: Vec<AllocationView>,
}

/// Bankkonto für die Auswahl.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct BankOption {
    pub id: i64,
    pub bezeichnung: String,
    pub standard: bool,
}

/// Konto für die Auswahl „Differenz ausbuchen“ (AK-4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AccountChoice {
    pub id: i64,
    pub nummer: String,
    pub bezeichnung: String,
}

/// Vorgaben für das Zahlungsfenster.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PaymentFormData {
    pub heute: String,
    pub bankkonten: Vec<BankOption>,
    pub konten: Vec<AccountChoice>,
    pub waehrungen: Vec<String>,
}

/// Vorbelegung aus einem Beleg (Rechnungsnummer oder `?beleg=`, AK-2): Partner, Währung, Schlüssel.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentHint {
    pub seite: String,
    pub partner: PickOption,
    pub waehrung: String,
    pub key: String,
    pub offen: i64,
}

// ---------------------------------------------------------------------------------------------
// Offene Posten mit Altersgliederung (AK-13)
// ---------------------------------------------------------------------------------------------

/// Offener Posten zum Stichtag (aus der Datenbank, Betrag in CHF mit Vorzeichen).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AgingItem {
    pub seite: String,
    pub adresse_id: i64,
    pub partner: String,
    /// Tage überfällig am Stichtag; `None` = ohne Fälligkeit (Vorauszahlung)
    pub tage: Option<i32>,
    pub offen_chf: i64,
}

/// Summen eines Partners je Altersklasse.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PartnerAging {
    pub adresse_id: i64,
    pub partner: String,
    pub klassen: [i64; 5],
    pub total: i64,
}

/// Debitoren und Kreditoren mit Summen.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgingReport {
    pub stichtag: String,
    pub debitoren: Vec<PartnerAging>,
    pub kreditoren: Vec<PartnerAging>,
}

/// Altersklasse zu den Tagen überfällig: 0 = nicht fällig, 1 = 1–30, 2 = 31–60, 3 = 61–90, 4 = über 90.
pub fn aging_class(tage: Option<i32>) -> usize {
    match tage {
        None => 0,
        Some(t) if t <= 0 => 0,
        Some(t) if t <= 30 => 1,
        Some(t) if t <= 60 => 2,
        Some(t) if t <= 90 => 3,
        Some(_) => 4,
    }
}

/// Gruppiert die Posten je Partner, nach Name sortiert; Partner mit Summe 0 entfallen.
pub fn aging_by_partner(items: &[AgingItem], seite: &str) -> Vec<PartnerAging> {
    let mut result: Vec<PartnerAging> = Vec::new();
    for item in items.iter().filter(|i| i.seite == seite) {
        let entry = match result
            .iter_mut()
            .position(|p| p.adresse_id == item.adresse_id)
        {
            Some(i) => &mut result[i],
            None => {
                result.push(PartnerAging {
                    adresse_id: item.adresse_id,
                    partner: item.partner.clone(),
                    klassen: [0; 5],
                    total: 0,
                });
                result.last_mut().expect("eben angefügt")
            }
        };
        entry.klassen[aging_class(item.tage)] += item.offen_chf;
        entry.total += item.offen_chf;
    }
    result.retain(|p| p.total != 0 || p.klassen.iter().any(|k| *k != 0));
    result.sort_by_key(|a| a.partner.to_lowercase());
    result
}

/// Summe je Altersklasse und gesamt.
pub fn aging_totals(rows: &[PartnerAging]) -> ([i64; 5], i64) {
    let mut klassen = [0; 5];
    let mut total = 0;
    for r in rows {
        for (k, v) in klassen.iter_mut().zip(r.klassen) {
            *k += v;
        }
        total += r.total;
    }
    (klassen, total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::finance::debtors::model::RATE_FACTOR;

    fn item(art: &str, id: i64, offen: i64, faellig: &str) -> OpenItem {
        OpenItem {
            art: art.into(),
            beleg_id: id,
            nummer: format!("N{id}"),
            referenz: None,
            datum: "2026-10-01".into(),
            faelligkeit: Some(faellig.into()),
            skonto_bis: None,
            skonto_prozent: None,
            waehrung: "CHF".into(),
            kurs: RATE_FACTOR,
            gesamtbetrag: offen,
            offen,
        }
    }

    fn line(key: &str, betrag: &str) -> AllocationInput {
        AllocationInput {
            key: key.into(),
            betrag: betrag.into(),
            ..Default::default()
        }
    }

    fn input(art: &str, betrag: &str, zeilen: Vec<AllocationInput>) -> PaymentInput {
        PaymentInput {
            art: art.into(),
            seite: String::new(),
            datum: "2026-11-27".into(),
            bankkonto_id: "1".into(),
            waehrung: "CHF".into(),
            betrag: betrag.into(),
            adresse_id: "3".into(),
            text: String::new(),
            zeilen,
        }
    }

    #[test]
    fn keys_round_trip() {
        assert_eq!(parse_key("r:12"), Some(("rechnung", 12)));
        assert_eq!(parse_key("v:7"), Some(("vorauszahlung", 7)));
        assert_eq!(parse_key("x:1"), None);
        assert_eq!(item("gutschrift", 5, 1, "2026-01-01").key(), "g:5");
    }

    #[test]
    fn propose_pays_oldest_first_and_suggests_discount() {
        let mut a = item("rechnung", 1, 10_000, "2026-12-01");
        let b = item("rechnung", 2, 5_000, "2026-11-01");
        // Skonto 2 % bis 10.12.: 200.00
        a.skonto_bis = Some("2026-12-10".into());
        a.skonto_prozent = Some(200);
        let items = vec![a, b, item("gutschrift", 3, 1_000, "2026-11-01")];
        let p = propose(&items, 14_800, "2026-11-27");
        assert_eq!(p[0].key, "r:2");
        assert_eq!(p[0].betrag, "50.00");
        assert_eq!(p[1].key, "r:1");
        assert_eq!(
            (p[1].betrag.as_str(), p[1].skonto.as_str()),
            ("98.00", "2.00")
        );
        assert_eq!(p.len(), 2, "Gutschriften bleiben leer");
        // Ausserhalb der Frist kein Skonto, Teilzahlung
        let p = propose(&items, 14_800, "2026-12-11");
        assert_eq!((p[1].betrag.as_str(), p[1].skonto.as_str()), ("98.00", ""));
        // Fehlbetrag grösser als Skonto: Teilzahlung
        let p = propose(&items, 10_000, "2026-11-27");
        assert_eq!((p[1].betrag.as_str(), p[1].skonto.as_str()), ("50.00", ""));
    }

    #[test]
    fn check_payment_with_overpayment_and_errors() {
        let items = vec![
            item("rechnung", 1, 10_000, "2026-12-01"),
            item("gutschrift", 2, 2_000, "2026-11-01"),
        ];
        let v = check(
            &input(
                "eingang",
                "100",
                vec![line("r:1", "100"), line("g:2", "20")],
            ),
            &items,
            &[],
        )
        .unwrap();
        assert_eq!(v.vorauszahlung, 2_000);
        assert_eq!(v.seite, "debitoren");
        // Mehr zugeordnet als bezahlt
        let e = check(
            &input("eingang", "50", vec![line("r:1", "100")]),
            &items,
            &[],
        )
        .unwrap_err();
        assert!(e.contains_key("zeilen"));
        // Mehr als offen
        let e = check(
            &input("eingang", "200", vec![line("r:1", "150")]),
            &items,
            &[],
        )
        .unwrap_err();
        assert!(e.contains_key("r:1_betrag"));
        // Differenz ohne Konto, Skonto auf Gutschrift
        let mut l = line("r:1", "90");
        l.differenz = "10".into();
        let mut g = line("g:2", "0");
        g.skonto = "1".into();
        let e = check(&input("eingang", "90", vec![l.clone(), g]), &items, &[]).unwrap_err();
        assert!(e.contains_key("r:1_differenz") && e.contains_key("g:2_skonto"));
        l.differenz_konto_id = "9".into();
        let v = check(&input("eingang", "90", vec![l]), &items, &[9]).unwrap();
        assert_eq!(v.zeilen[0].cleared(), 10_000);
    }

    #[test]
    fn check_offsetting_must_balance() {
        let items = vec![
            item("rechnung", 1, 10_000, "2026-12-01"),
            item("gutschrift", 2, 3_000, "2026-11-01"),
            item("vorauszahlung", 4, 2_000, "2026-11-01"),
        ];
        let mut i = input(
            "verrechnung",
            "",
            vec![line("r:1", "50"), line("g:2", "30"), line("v:4", "20")],
        );
        i.seite = "debitoren".into();
        i.bankkonto_id = String::new();
        let v = check(&i, &items, &[]).unwrap();
        assert_eq!(
            (v.betrag, v.vorauszahlung, v.bankkonto_id),
            (5_000, 0, None)
        );
        i.zeilen[0].betrag = "60".into();
        assert!(check(&i, &items, &[]).unwrap_err().contains_key("zeilen"));
        // Vorauszahlung nicht in einer Zahlung
        let e = check(
            &input("eingang", "20", vec![line("v:4", "20")]),
            &items,
            &[],
        )
        .unwrap_err();
        assert!(e.contains_key("v:4_betrag"));
    }

    #[test]
    fn discount_is_split_by_vat_codes() {
        // Rechnung 108.10 brutto mit 8.1 %, Skonto 2.16
        let vat = vec![VatShare {
            code_id: 1,
            rate: 810,
            tax_konto_id: 2200,
            gross: 10_810,
        }];
        assert_eq!(
            split_discount(216, 10_810, &vat),
            vec![(Some(1), Some(2200), 200, 16)]
        );
        // Teil ohne Code bleibt ohne Steuer
        let parts = split_discount(300, 20_810, &vat);
        assert_eq!(parts[0].0, Some(1));
        assert_eq!(parts[1], (None, None, 300 - (parts[0].2 + parts[0].3), 0));
    }

    fn accounts() -> PaymentAccounts {
        PaymentAccounts {
            bank: 1020,
            partner: 1100,
            skonto: Some(3800),
            kursdifferenz: Some(6960),
        }
    }

    fn booking_item(art: &str, betrag: i64, skonto: i64, kurs: i64, gesamt: i64) -> BookingItem {
        BookingItem {
            values: AllocationValues {
                art: art.into(),
                beleg_id: 1,
                betrag,
                skonto,
                differenz: 0,
                differenz_konto_id: None,
            },
            kurs,
            gesamtbetrag: gesamt,
            vat: vec![VatShare {
                code_id: 1,
                rate: 810,
                tax_konto_id: 2200,
                gross: gesamt,
            }],
        }
    }

    #[test]
    fn incoming_payment_with_discount_and_prepayment() {
        let items = vec![booking_item("rechnung", 10_594, 216, RATE_FACTOR, 10_810)];
        let lines =
            payment_lines("debitoren", 11_000, RATE_FACTOR, 406, &items, accounts()).unwrap();
        let (soll, haben) = crate::backend::finance::journal::model::totals(&lines);
        assert_eq!(soll, haben);
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 1020 && l.soll && l.betrag == 11_000)
        );
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 1100 && !l.soll && l.betrag == 10_810)
        );
        assert!(
            lines.iter().any(|l| l.konto_id == 3800
                && l.soll
                && l.betrag == 200
                && l.mwst_code_id == Some(1))
        );
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 2200 && l.soll && l.betrag == 16 && l.ist_mwst_zeile)
        );
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 1100 && !l.soll && l.betrag == 406)
        );
        assert!(!lines.iter().any(|l| l.konto_id == 6960));
    }

    #[test]
    fn outgoing_payment_swaps_sides_and_books_exchange_difference() {
        // EUR 100 gebucht zu 0.95 (95.00 CHF), bezahlt zu 0.97 (97.00 CHF): Kursverlust 2.00
        let items = vec![booking_item("rechnung", 10_000, 0, 950_000, 10_000)];
        let lines = payment_lines("kreditoren", 10_000, 970_000, 0, &items, accounts()).unwrap();
        let (soll, haben) = crate::backend::finance::journal::model::totals(&lines);
        assert_eq!(soll, haben);
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 1020 && !l.soll && l.betrag == 9_700)
        );
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 1100 && l.soll && l.betrag == 9_500)
        );
        assert!(
            lines
                .iter()
                .any(|l| l.konto_id == 6960 && l.soll && l.betrag == 200)
        );
        // Ohne Kursdifferenzkonto: Meldung
        let no_fx = PaymentAccounts {
            kursdifferenz: None,
            ..accounts()
        };
        assert!(payment_lines("kreditoren", 10_000, 970_000, 0, &items, no_fx).is_err());
    }

    #[test]
    fn credit_note_used_in_payment() {
        let items = vec![
            booking_item("rechnung", 10_000, 0, RATE_FACTOR, 10_000),
            booking_item("gutschrift", 3_000, 0, RATE_FACTOR, 3_000),
        ];
        let lines = payment_lines("debitoren", 7_000, RATE_FACTOR, 0, &items, accounts()).unwrap();
        let (soll, haben) = crate::backend::finance::journal::model::totals(&lines);
        assert_eq!((soll, haben), (10_000, 10_000));
    }

    #[test]
    fn aging_classes_and_grouping() {
        assert_eq!(
            [
                None,
                Some(-5),
                Some(0),
                Some(1),
                Some(30),
                Some(31),
                Some(61),
                Some(91)
            ]
            .map(aging_class),
            [0, 0, 0, 1, 1, 2, 3, 4]
        );
        let it = |adresse_id: i64, partner: &str, tage: Option<i32>, offen: i64| AgingItem {
            seite: "debitoren".into(),
            adresse_id,
            partner: partner.into(),
            tage,
            offen_chf: offen,
        };
        let rows = aging_by_partner(
            &[
                it(2, "Zeta AG", Some(45), 1_000),
                it(1, "Alpha AG", Some(-3), 500),
                it(1, "Alpha AG", Some(100), 200),
                it(1, "Alpha AG", None, -100),
                it(3, "Null AG", Some(5), 0),
            ],
            "debitoren",
        );
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].partner, "Alpha AG");
        assert_eq!(rows[0].klassen, [400, 0, 0, 0, 200]);
        assert_eq!(rows[0].total, 600);
        assert_eq!(aging_totals(&rows), ([400, 0, 1_000, 0, 200], 1_600));
    }

    #[test]
    fn texts_and_filter() {
        assert_eq!(
            booking_text("eingang", "Muster AG", &["R-1".into(), "R-2".into()]),
            "Zahlungseingang Muster AG: R-1, R-2"
        );
        assert!(booking_text("ausgang", "L", &[]).ends_with("Vorauszahlung"));
        let f = PaymentFilter::default().normalized();
        assert_eq!(
            (f.sortierung.as_str(), f.richtung.as_str()),
            ("datum", "ab")
        );
        assert_eq!(side_of("ausgang"), "kreditoren");
    }
}
