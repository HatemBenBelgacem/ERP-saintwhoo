//! Datenmodell der Anfragen (ANF-029). Wird auf Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

/// Zeile der Anfrageliste im Vorgang (AK-4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct InquiryRow {
    pub id: i64,
    pub nummer: String,
    pub titel: String,
    pub beschreibung: Option<String>,
    pub erstellt_am: String,
}

/// Fachliche Nummer einer Anfrage: `A-JJJJ-NNNN` (AK-3).
pub fn format_nummer(jahr: i32, laufnummer: i32) -> String {
    format!("A-{jahr}-{laufnummer:04}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nummer_format() {
        assert_eq!(format_nummer(2026, 1), "A-2026-0001");
        assert_eq!(format_nummer(2027, 12345), "A-2027-12345");
    }
}
