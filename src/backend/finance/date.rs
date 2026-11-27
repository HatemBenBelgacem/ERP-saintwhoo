//! Kalenderrechnung für die Finanzen (ANF-012, ANF-013). Wird auf Server und im Browser verwendet.
//!
//! Datumswerte sind Texte im Format `JJJJ-MM-TT`. In diesem Format stimmt die Textsortierung mit der
//! Zeitfolge überein, Vergleiche sind daher einfache Textvergleiche. Die Datenbank liefert und nimmt
//! Datumswerte ebenfalls als Text (`::text` bzw. `::date`), damit kein Zeitpaket nötig ist.

use crate::backend::projects::model::is_valid_date;

/// Monatsnamen für Anzeige und Vermerke.
pub const MONTH_NAMES: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];

/// Jahr, Monat und Tag eines gültigen Datums (`None` bei ungültigem Text).
pub fn split(s: &str) -> Option<(i32, u32, u32)> {
    if !is_valid_date(s) {
        return None;
    }
    Some((
        s[0..4].parse().ok()?,
        s[5..7].parse().ok()?,
        s[8..10].parse().ok()?,
    ))
}

/// Anzahl Tage eines Monats (Schaltjahre berücksichtigt).
pub fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        _ => 28,
    }
}

/// Datum im Format `JJJJ-MM-TT`.
pub fn format(year: i32, month: u32, day: u32) -> String {
    format!("{year:04}-{month:02}-{day:02}")
}

/// Letzter Tag eines Monats, z. B. `2026-02-28`.
pub fn last_of_month(year: i32, month: u32) -> String {
    format(year, month, days_in_month(year, month))
}

/// Der Tag nach `s`, z. B. `2026-12-31` → `2027-01-01`.
pub fn next_day(s: &str) -> Option<String> {
    let (y, m, d) = split(s)?;
    if d < days_in_month(y, m) {
        Some(format(y, m, d + 1))
    } else if m < 12 {
        Some(format(y, m + 1, 1))
    } else {
        Some(format(y + 1, 1, 1))
    }
}

/// Monat um `by` Monate verschoben (Jahr und Monat).
pub fn shift_month(year: i32, month: u32, by: i32) -> (i32, u32) {
    let index = year * 12 + (month as i32 - 1) + by;
    (index.div_euclid(12), index.rem_euclid(12) as u32 + 1)
}

/// Anzahl Kalendermonate von `start` bis `end`, beide eingerechnet (Januar bis Dezember = 12).
pub fn months_inclusive(start: &str, end: &str) -> Option<i64> {
    let (sy, sm, _) = split(start)?;
    let (ey, em, _) = split(end)?;
    Some(i64::from(ey) * 12 + i64::from(em) - (i64::from(sy) * 12 + i64::from(sm)) + 1)
}

/// Jahr eines gültigen Datums.
pub fn year_of(s: &str) -> Option<i32> {
    split(s).map(|(y, _, _)| y)
}

/// Erster und letzter Tag jedes Kalendermonats von `start` bis `end` (beide eingerechnet).
pub fn months_between(start: &str, end: &str) -> Vec<(String, String)> {
    let (Some((sy, sm, _)), Some((ey, em, _))) = (split(start), split(end)) else {
        return Vec::new();
    };
    let mut result = Vec::new();
    let (mut y, mut m) = (sy, sm);
    while (y, m) <= (ey, em) {
        result.push((format(y, m, 1), last_of_month(y, m)));
        (y, m) = shift_month(y, m, 1);
    }
    result
}

/// Tage seit dem 1.1.1970 (proleptischer gregorianischer Kalender).
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = i64::from(y) - i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Datum aus Tagen seit dem 1.1.1970.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y as i32, m, d)
}

/// `s` plus `days` Tage, z. B. Fälligkeit = Rechnungsdatum + Zahlungsfrist (ANF-016 AK-6).
pub fn add_days(s: &str, days: i64) -> Option<String> {
    let (y, m, d) = split(s)?;
    let (y, m, d) = civil_from_days(days_from_civil(y, m, d) + days);
    Some(format(y, m, d))
}

/// Anzahl Tage von `start` bis `end` (negativ, wenn `end` vor `start` liegt).
pub fn days_between(start: &str, end: &str) -> Option<i64> {
    let (sy, sm, sd) = split(start)?;
    let (ey, em, ed) = split(end)?;
    Some(days_from_civil(ey, em, ed) - days_from_civil(sy, sm, sd))
}

/// Monatsname mit Jahr, z. B. „März 2026“.
pub fn month_label(s: &str) -> Option<String> {
    let (y, m, _) = split(s)?;
    Some(format!("{} {y}", MONTH_NAMES[m as usize - 1]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leap_years_and_month_lengths() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2100, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2026, 4), 30);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn next_day_crosses_month_and_year() {
        assert_eq!(next_day("2026-12-31").as_deref(), Some("2027-01-01"));
        assert_eq!(next_day("2026-02-28").as_deref(), Some("2026-03-01"));
        assert_eq!(next_day("2028-02-28").as_deref(), Some("2028-02-29"));
        assert_eq!(next_day("2026-06-15").as_deref(), Some("2026-06-16"));
        assert_eq!(next_day("kein Datum"), None);
    }

    #[test]
    fn months_inclusive_counts_calendar_months() {
        assert_eq!(months_inclusive("2026-01-01", "2026-12-31"), Some(12));
        assert_eq!(months_inclusive("2026-01-01", "2026-01-31"), Some(1));
        assert_eq!(months_inclusive("2026-03-01", "2027-02-28"), Some(12));
        assert_eq!(months_inclusive("2026-01-01", "2027-12-31"), Some(24));
        assert_eq!(months_inclusive("2026-01-01", "x"), None);
    }

    #[test]
    fn months_between_yields_first_and_last_day() {
        let months = months_between("2026-11-01", "2027-01-31");
        assert_eq!(
            months,
            vec![
                ("2026-11-01".to_string(), "2026-11-30".to_string()),
                ("2026-12-01".to_string(), "2026-12-31".to_string()),
                ("2027-01-01".to_string(), "2027-01-31".to_string()),
            ]
        );
        assert_eq!(months_between("2026-02-01", "2026-02-28").len(), 1);
        assert!(months_between("2026-02-01", "2026-01-01").is_empty());
    }

    #[test]
    fn shift_and_labels() {
        assert_eq!(shift_month(2026, 1, 11), (2026, 12));
        assert_eq!(shift_month(2026, 12, 1), (2027, 1));
        assert_eq!(shift_month(2026, 1, -1), (2025, 12));
        assert_eq!(month_label("2026-03-01").as_deref(), Some("März 2026"));
        assert_eq!(year_of("2031-07-01"), Some(2031));
    }

    #[test]
    fn add_days_over_month_year_and_leap_day() {
        assert_eq!(add_days("2026-01-31", 30).as_deref(), Some("2026-03-02"));
        assert_eq!(add_days("2026-12-15", 30).as_deref(), Some("2027-01-14"));
        assert_eq!(add_days("2028-02-28", 1).as_deref(), Some("2028-02-29"));
        assert_eq!(add_days("2026-03-01", 0).as_deref(), Some("2026-03-01"));
        assert_eq!(add_days("2026-03-01", -1).as_deref(), Some("2026-02-28"));
        assert_eq!(add_days("kein Datum", 3), None);
        assert_eq!(days_between("2026-11-01", "2026-11-27"), Some(26));
        assert_eq!(days_between("2026-12-31", "2026-01-01"), Some(-364));
    }
}
