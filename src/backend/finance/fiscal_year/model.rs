//! Datenmodell und Prüfregeln für Geschäftsjahre und Perioden (ANF-013).
//! Wird auf dem Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

use crate::backend::finance::date::{
    days_in_month, last_of_month, month_label, months_inclusive, next_day, shift_month, split,
    year_of,
};
use crate::backend::projects::model::is_valid_date;
pub use crate::forms::{FieldErrors, SaveOutcome};

/// Kürzeste und längste erlaubte Dauer eines Geschäftsjahrs in Monaten (AK-2).
pub const MIN_MONTHS: i64 = 1;
pub const MAX_MONTHS: i64 = 23;

/// Status eines Geschäftsjahrs (Abschluss folgt mit ANF-023).
pub const STATUSES: &[(&str, &str)] = &[("offen", "Offen"), ("abgeschlossen", "Abgeschlossen")];

/// Status einer Periode (AK-5).
pub const PERIOD_STATUSES: &[(&str, &str)] = &[("offen", "Offen"), ("gesperrt", "Gesperrt")];

pub fn status_label(status: &str) -> &'static str {
    STATUSES
        .iter()
        .find(|(k, _)| *k == status)
        .map_or("–", |(_, l)| l)
}

pub fn period_status_label(status: &str) -> &'static str {
    PERIOD_STATUSES
        .iter()
        .find(|(k, _)| *k == status)
        .map_or("–", |(_, l)| l)
}

/// Plakette des Status: offen neutral bzw. gesperrt/abgeschlossen als Hinweis.
pub fn status_badge(status: &str) -> &'static str {
    match status {
        "gesperrt" => "badge badge-sm badge-warning badge-soft",
        "abgeschlossen" => "badge badge-sm badge-neutral",
        _ => "badge badge-sm badge-success badge-soft",
    }
}

/// Eine Periode (Kalendermonat) eines Geschäftsjahrs (AK-4, AK-5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Period {
    pub id: i64,
    pub geschaeftsjahr_id: i64,
    /// `JJJJ-MM-TT`, immer der 1. des Monats
    pub beginn: String,
    /// Letzter Tag des Monats
    pub ende: String,
    /// `offen` oder `gesperrt`
    pub status: String,
    /// Benutzername und Zeitpunkt der letzten Sperrung bzw. Entsperrung (AK-5)
    pub vermerk: Option<String>,
}

impl Period {
    /// „März 2026“
    pub fn label(&self) -> String {
        month_label(&self.beginn).unwrap_or_default()
    }
}

/// Ein Geschäftsjahr mit seinen Perioden.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct FiscalYear {
    pub id: i64,
    /// Jahr des Endes, bei abweichendem Jahr „2026/2027“
    pub bezeichnung: String,
    pub beginn: String,
    pub ende: String,
    /// `offen` oder `abgeschlossen`
    pub status: String,
    #[cfg_attr(feature = "ssr", sqlx(skip))]
    pub perioden: Vec<Period>,
}

impl FiscalYear {
    /// Lesbarer Zeitraum, z. B. „1. Januar 2026 bis 31. Dezember 2026“ (kurz: „01.01.2026 – 31.12.2026“).
    pub fn range_text(&self) -> String {
        format!(
            "{} – {}",
            german_date(&self.beginn),
            german_date(&self.ende)
        )
    }
}

/// `2026-01-31` → `31.01.2026`
pub fn german_date(s: &str) -> String {
    match split(s) {
        Some((y, m, d)) => format!("{d:02}.{m:02}.{y}"),
        None => s.to_string(),
    }
}

/// Alle Geschäftsjahre (neueste zuoberst) und der Vorschlag für ein neues (AK-2).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FiscalYearOverview {
    pub jahre: Vec<FiscalYear>,
    pub vorschlag_beginn: String,
    pub vorschlag_ende: String,
}

/// Bezeichnung aus Beginn und Ende: „2026“, bei abweichendem Jahr „2026/2027“.
pub fn label_for(beginn: &str, ende: &str) -> String {
    match (year_of(beginn), year_of(ende)) {
        (Some(a), Some(b)) if a != b => format!("{a}/{b}"),
        (_, Some(b)) => b.to_string(),
        _ => String::new(),
    }
}

/// Vorschlag für ein neues Geschäftsjahr (AK-2): beginnt am Tag nach dem letzten Ende, sonst am 1. Januar
/// des laufenden Jahres; endet ein Jahr später (letzter Tag des Monats).
pub fn suggestion(prev_ende: Option<&str>, today: &str) -> (String, String) {
    let beginn = prev_ende
        .and_then(next_day)
        .unwrap_or_else(|| format!("{}-01-01", year_of(today).unwrap_or(2000)));
    let (y, m) = split(&beginn).map_or((2000, 1), |(y, m, _)| (y, m));
    let (ey, em) = shift_month(y, m, 11);
    let ende = last_of_month(ey, em);
    (beginn, ende)
}

/// Eingabe eines neuen Geschäftsjahrs (AK-2). Die Bezeichnung ergibt sich aus den Daten.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FiscalYearInput {
    pub beginn: String,
    pub ende: String,
}

impl FiscalYearInput {
    pub fn normalized(&self) -> Self {
        Self {
            beginn: self.beginn.trim().into(),
            ende: self.ende.trim().into(),
        }
    }

    /// Prüft Beginn, Ende, Dauer (AK-2) und den lückenlosen Anschluss an das letzte Geschäftsjahr (AK-3).
    /// `prev_ende` ist das Ende des neuesten bestehenden Geschäftsjahrs (`None`, wenn keines existiert).
    pub fn validate(&self, prev_ende: Option<&str>) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let beginn = date_field(&self.beginn, "beginn", "Beginn", &mut errors);
        let ende = date_field(&self.ende, "ende", "Ende", &mut errors);

        if let Some(b) = &beginn
            && split(b).is_some_and(|(_, _, d)| d != 1)
        {
            set_first(
                &mut errors,
                "beginn".into(),
                "Das Geschäftsjahr beginnt am 1. eines Monats.".into(),
            );
        }
        if let Some(e) = &ende
            && split(e).is_some_and(|(y, m, d)| d != days_in_month(y, m))
        {
            set_first(
                &mut errors,
                "ende".into(),
                "Das Geschäftsjahr endet am letzten Tag eines Monats.".into(),
            );
        }
        if let (Some(b), Some(e)) = (&beginn, &ende) {
            match months_inclusive(b, e) {
                Some(n) if n < MIN_MONTHS => {
                    set_first(
                        &mut errors,
                        "ende".into(),
                        "Das Ende liegt vor dem Beginn.".into(),
                    );
                }
                Some(n) if n > MAX_MONTHS => {
                    set_first(
                        &mut errors,
                        "ende".into(),
                        format!(
                            "Ein Geschäftsjahr dauert höchstens {MAX_MONTHS} Monate (hier: {n})."
                        ),
                    );
                }
                _ => {}
            }
        }
        if let (Some(prev), Some(b)) = (prev_ende, &beginn)
            && let Some(expected) = next_day(prev)
            && &expected != b
        {
            set_first(
                &mut errors,
                "beginn".into(),
                format!(
                    "Das Geschäftsjahr muss lückenlos anschliessen und am {} beginnen.",
                    german_date(&expected)
                ),
            );
        }
        errors
    }

    /// Geprüfte Werte `(beginn, ende)`; `None`, wenn die Prüfung Fehler findet.
    pub fn values(&self, prev_ende: Option<&str>) -> Option<(String, String)> {
        self.validate(prev_ende)
            .is_empty()
            .then(|| (self.beginn.clone(), self.ende.clone()))
    }
}

/// Meldung je Feld: die erste Prüfung gewinnt, damit die genauere Meldung (z. B. „am 1. eines Monats“) sichtbar bleibt.
fn set_first(errors: &mut FieldErrors, field: String, message: String) {
    errors.entry(field).or_insert(message);
}

/// Liest ein Datumsfeld; meldet fehlende oder ungültige Werte. `Some` nur bei gültigem Datum.
fn date_field(value: &str, field: &str, label: &str, errors: &mut FieldErrors) -> Option<String> {
    if value.is_empty() {
        errors.insert(field.into(), format!("Bitte den {label} eingeben."));
        None
    } else if !is_valid_date(value) {
        errors.insert(field.into(), "Bitte ein gültiges Datum eingeben.".into());
        None
    } else {
        Some(value.to_string())
    }
}

/// Meldung, wenn eine Periode nicht geändert werden darf (AK-5): entsperren (`gesperrt_nach` = false)
/// ist nur möglich, solange das Geschäftsjahr nicht abgeschlossen ist.
pub fn lock_error(year_status: &str, gesperrt_nach: bool) -> Option<String> {
    (!gesperrt_nach && year_status == "abgeschlossen").then(|| {
        "Das Geschäftsjahr ist abgeschlossen. Perioden können nicht mehr entsperrt werden.".into()
    })
}

/// Prüft, ob ein Buchungsdatum in einem offenen Geschäftsjahr und einer offenen Periode liegt (AK-6, AK-8).
/// `found`: Status des Geschäftsjahrs und der Periode zum Datum (`None` = kein Geschäftsjahr).
/// Wird von ANF-017 beim Buchen verwendet.
pub fn booking_error(found: Option<(&str, Option<&str>)>, datum: &str) -> Option<String> {
    match found {
        None => Some(format!(
            "Kein offenes Geschäftsjahr für das Datum {}.",
            german_date(datum)
        )),
        Some((year_status, _)) if year_status != "offen" => {
            Some("Das Geschäftsjahr für dieses Datum ist abgeschlossen.".into())
        }
        Some((_, Some("gesperrt"))) => Some(format!(
            "Die Periode für das Datum {} ist gesperrt.",
            german_date(datum)
        )),
        Some(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(beginn: &str, ende: &str) -> FiscalYearInput {
        FiscalYearInput {
            beginn: beginn.into(),
            ende: ende.into(),
        }
    }

    #[test]
    fn suggestion_starts_after_last_year() {
        assert_eq!(
            suggestion(None, "2026-11-19"),
            ("2026-01-01".into(), "2026-12-31".into())
        );
        assert_eq!(
            suggestion(Some("2026-12-31"), "2026-11-19"),
            ("2027-01-01".into(), "2027-12-31".into())
        );
    }

    #[test]
    fn valid_twelve_month_year() {
        assert!(input("2026-01-01", "2026-12-31").validate(None).is_empty());
        // verlängertes erstes Geschäftsjahr mit 18 Monaten
        assert!(input("2026-07-01", "2027-12-31").validate(None).is_empty());
        assert_eq!(
            input("2026-01-01", "2026-12-31").values(None),
            Some(("2026-01-01".into(), "2026-12-31".into()))
        );
    }

    #[test]
    fn duration_between_one_and_twenty_three_months() {
        let too_long = input("2026-01-01", "2027-12-31").validate(None);
        assert!(too_long.contains_key("ende"), "{too_long:?}");
        assert!(too_long["ende"].contains("24"));
        let backwards = input("2026-06-01", "2026-05-31").validate(None);
        assert!(backwards.contains_key("ende"));
        assert!(input("2026-06-01", "2026-06-30").validate(None).is_empty());
    }

    #[test]
    fn boundaries_of_month() {
        let e = input("2026-01-02", "2026-12-31").validate(None);
        assert!(e.contains_key("beginn"));
        let e = input("2026-01-01", "2026-12-30").validate(None);
        assert!(e.contains_key("ende"));
        let e = input("", "x").validate(None);
        assert!(e.contains_key("beginn") && e.contains_key("ende"));
        assert_eq!(input("", "").values(None), None);
    }

    #[test]
    fn must_follow_previous_year_without_gap() {
        let e = input("2027-02-01", "2028-01-31").validate(Some("2026-12-31"));
        assert!(e.contains_key("beginn"), "{e:?}");
        assert!(e["beginn"].contains("01.01.2027"));
        assert!(
            input("2027-01-01", "2027-12-31")
                .validate(Some("2026-12-31"))
                .is_empty()
        );
    }

    #[test]
    fn labels_and_booking_rule() {
        assert_eq!(label_for("2026-01-01", "2026-12-31"), "2026");
        assert_eq!(label_for("2026-07-01", "2027-06-30"), "2026/2027");
        assert_eq!(
            booking_error(Some(("offen", Some("offen"))), "2026-03-01"),
            None
        );
        assert_eq!(booking_error(Some(("offen", None)), "2026-03-01"), None);
        assert!(
            booking_error(Some(("offen", Some("gesperrt"))), "2026-03-01")
                .unwrap()
                .contains("gesperrt")
        );
        assert!(
            booking_error(None, "2030-01-01")
                .unwrap()
                .contains("Kein offenes")
        );
        assert!(
            booking_error(Some(("abgeschlossen", None)), "2026-03-01")
                .unwrap()
                .contains("abgeschlossen")
        );
        assert!(lock_error("abgeschlossen", false).is_some());
        assert!(lock_error("abgeschlossen", true).is_none());
        assert!(lock_error("offen", false).is_none());
    }
}
