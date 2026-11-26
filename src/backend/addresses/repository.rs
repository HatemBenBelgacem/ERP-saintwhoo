//! SQL-Zugriffe für Adressen (ANF-002, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Fehler werden als `sqlx::Error` zurückgegeben; die Server-Funktionen übersetzen sie.

use sqlx::{AssertSqlSafe, PgPool, postgres::PgArguments, query::Query};

use super::model::{Address, AddressInput, AddressRow};
use crate::forms::PickOption;

const SELECT_ADDRESS: &str = "\
    SELECT id, nummer, anzeigename, typ, anrede, firmenname, vorname, nachname, zusatz, \
           strasse, postfach, plz, ort, land, email, telefon, mobiltelefon, webseite, notiz, zahlungsbedingung_id, \
           (SELECT b.id FROM benutzer b WHERE b.adresse_id = adresse.id) AS benutzer_id, \
           (SELECT b.benutzername FROM benutzer b WHERE b.adresse_id = adresse.id) AS benutzername, \
           EXISTS (SELECT 1 FROM firma f WHERE f.adresse_id = adresse.id) AS eigene_firma, \
           to_char(erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
           to_char(geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
    FROM adresse";

/// Suchbedingung (AK-2); $1 = Suchbegriff, $2 = LIKE-Muster.
const SEARCH_WHERE: &str = "\
    ($1 = '' OR nummer ILIKE $2 OR firmenname ILIKE $2 OR vorname ILIKE $2 OR nachname ILIKE $2 \
     OR ort ILIKE $2 OR plz ILIKE $2 OR email ILIKE $2)";

const COLUMNS: &str = "typ, anrede, firmenname, vorname, nachname, zusatz, strasse, postfach, \
                       plz, ort, land, email, telefon, mobiltelefon, webseite, notiz, zahlungsbedingung_id";

/// Sortierausdruck für einen Schlüssel aus der Webadresse (nur feste Werte → kein SQL-Einschleusen).
fn order_by(sort: &str) -> &'static str {
    match sort {
        "nummer" => "nummer",
        "plz" => "plz",
        "ort" => "lower(ort)",
        "land" => "land",
        "email" => "lower(email)",
        "telefon" => "telefon",
        _ => "lower(anzeigename)",
    }
}

/// Leerer Text → NULL in der Datenbank.
fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

/// Bindet alle Eingabefelder in fester Reihenfolge ($1 … $17).
fn bind_input<'q>(
    q: Query<'q, sqlx::Postgres, PgArguments>,
    i: &'q AddressInput,
) -> Query<'q, sqlx::Postgres, PgArguments> {
    q.bind(&i.typ)
        .bind(opt(&i.anrede))
        .bind(opt(&i.firmenname))
        .bind(opt(&i.vorname))
        .bind(opt(&i.nachname))
        .bind(opt(&i.zusatz))
        .bind(opt(&i.strasse))
        .bind(opt(&i.postfach))
        .bind(opt(&i.plz))
        .bind(opt(&i.ort))
        .bind(&i.land)
        .bind(opt(&i.email))
        .bind(opt(&i.telefon))
        .bind(opt(&i.mobiltelefon))
        .bind(opt(&i.webseite))
        .bind(opt(&i.notiz))
        .bind(i.zahlungsbedingung_id.parse::<i64>().ok())
}

/// Anzahl Adressen, die zur Suche passen.
pub async fn count(pool: &PgPool, suche: &str, pattern: &str) -> Result<i64, sqlx::Error> {
    // Geprüft: SQL besteht nur aus festen Texten, Eingaben laufen als Parameter.
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM adresse WHERE {SEARCH_WHERE}"
    )))
    .bind(suche)
    .bind(pattern)
    .fetch_one(pool)
    .await
}

/// Eine Seite der Adressliste.
pub async fn list_page(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    sortierung: &str,
    descending: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<AddressRow>, sqlx::Error> {
    let direction = if descending { "DESC" } else { "ASC" };
    // Geprüft: Sortierung kommt aus einer festen Liste (order_by), Richtung ist ASC/DESC.
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT id, nummer, anzeigename, plz, ort, land, email, telefon FROM adresse \
         WHERE {SEARCH_WHERE} ORDER BY {} {direction} NULLS LAST, id LIMIT $3 OFFSET $4",
        order_by(sortierung),
    )))
    .bind(suche)
    .bind(pattern)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Eine Adresse; `None`, wenn es sie nicht gibt.
pub async fn find(pool: &PgPool, id: i64) -> Result<Option<Address>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!("{SELECT_ADDRESS} WHERE id = $1")))
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Höchstens 20 Treffer für die Adress-Auswahl (nach Name, Nummer oder Ort).
/// Die eigene Firma ist nicht wählbar (ANF-025 AK-12) – die Auswahl dient Kunden und Lieferanten.
pub async fn search_options(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
) -> Result<Vec<PickOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, anzeigename AS name FROM adresse \
         WHERE ($1 = '' OR anzeigename ILIKE $2 OR nummer ILIKE $2 OR ort ILIKE $2) \
           AND NOT EXISTS (SELECT 1 FROM firma f WHERE f.adresse_id = adresse.id) \
         ORDER BY lower(anzeigename), id LIMIT 20",
    )
    .bind(suche)
    .bind(pattern)
    .fetch_all(pool)
    .await
}

/// Neue Adresse speichern; liefert die Kennung.
/// Auch innerhalb einer Transaktion nutzbar (z. B. Firma mit neuer Adresse, ANF-025).
pub async fn insert<'e>(
    executor: impl sqlx::PgExecutor<'e>,
    input: &AddressInput,
) -> Result<i64, sqlx::Error> {
    let sql = format!(
        "INSERT INTO adresse ({COLUMNS}) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17) RETURNING id"
    );
    let row = bind_input(sqlx::query(AssertSqlSafe(sql)), input)
        .fetch_one(executor)
        .await?;
    sqlx::Row::try_get(&row, "id")
}

/// Adresse ändern; liefert die Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update(pool: &PgPool, id: i64, input: &AddressInput) -> Result<u64, sqlx::Error> {
    let sets = COLUMNS
        .split(',')
        .enumerate()
        .map(|(i, c)| format!("{} = ${}", c.trim(), i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let query = format!("UPDATE adresse SET {sets}, geaendert_am = now() WHERE id = $18");
    let result = bind_input(sqlx::query(AssertSqlSafe(query)), input)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Benutzername des Benutzers, dem die Adresse gehört (ANF-005 AK-19, AK-20).
pub async fn owner_username(pool: &PgPool, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT benutzername FROM benutzer WHERE adresse_id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Adresse löschen; liefert ihre Nummer oder `None`, wenn es sie nicht gab.
pub async fn delete(pool: &PgPool, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("DELETE FROM adresse WHERE id = $1 RETURNING nummer")
        .bind(id)
        .fetch_optional(pool)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_sort_falls_back_to_name() {
        assert_eq!(order_by("; DROP TABLE adresse"), "lower(anzeigename)");
    }
}
