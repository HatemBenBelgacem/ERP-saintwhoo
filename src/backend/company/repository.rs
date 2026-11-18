//! SQL-Zugriffe für die eigene Firma (ANF-025, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.

use sqlx::{PgConnection, PgPool};

use super::model::{Company, CompanyInput};
use crate::forms::PickOption;

/// Die Firma mit den Angaben ihrer Adresse; `None`, solange keine erfasst ist.
pub async fn find(pool: &PgPool) -> Result<Option<Company>, sqlx::Error> {
    sqlx::query_as(
        "SELECT f.id, f.uid, f.handelsregister, f.fusszeile, \
                CASE WHEN f.logo IS NULL THEN NULL \
                     ELSE (extract(epoch FROM f.geaendert_am) * 1000)::bigint END AS logo_version, \
                a.id AS adresse_id, a.nummer AS adresse_nummer, a.typ AS adresse_typ, \
                a.anzeigename AS name, a.zusatz, a.strasse, a.postfach, a.plz, a.ort, a.land, \
                a.telefon, a.email, a.webseite \
         FROM firma f JOIN adresse a ON a.id = f.adresse_id",
    )
    .fetch_optional(pool)
    .await
}

/// Typ und Postanschrift einer Adresse (für AK-8); `None`, wenn es sie nicht gibt.
#[allow(clippy::type_complexity)]
pub async fn address_parts(
    pool: &PgPool,
    id: i64,
) -> Result<Option<(String, Option<String>, Option<String>, Option<String>)>, sqlx::Error> {
    sqlx::query_as("SELECT typ, strasse, plz, ort FROM adresse WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Höchstens 20 Adressen vom Typ „Firma“ für die Auswahl (AK-4).
pub async fn search_company_addresses(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
) -> Result<Vec<PickOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, anzeigename AS name FROM adresse \
         WHERE typ = 'firma' AND ($1 = '' OR anzeigename ILIKE $2 OR nummer ILIKE $2 OR ort ILIKE $2) \
         ORDER BY lower(anzeigename), id LIMIT 20",
    )
    .bind(suche)
    .bind(pattern)
    .fetch_all(pool)
    .await
}

/// Firma anlegen (innerhalb einer Transaktion); liefert die Kennung.
/// Eine zweite Firma scheitert an der Eindeutigkeit von `einzig` (23505).
pub async fn insert(
    conn: &mut PgConnection,
    address_id: i64,
    input: &CompanyInput,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO firma (adresse_id, uid, handelsregister, fusszeile) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(address_id)
    .bind(input.uid_value())
    .bind(input.register())
    .bind(input.footer())
    .fetch_one(conn)
    .await
}

/// Firma ändern; Anzahl geänderter Zeilen (0 = keine Firma).
pub async fn update(
    pool: &PgPool,
    address_id: i64,
    input: &CompanyInput,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE firma SET adresse_id = $1, uid = $2, handelsregister = $3, fusszeile = $4, \
                geaendert_am = now()",
    )
    .bind(address_id)
    .bind(input.uid_value())
    .bind(input.register())
    .bind(input.footer())
    .execute(pool)
    .await
    .map(|r| r.rows_affected())
}

/// Logo setzen (`Some`) oder entfernen (`None`); Anzahl geänderter Zeilen.
pub async fn set_logo(pool: &PgPool, logo: Option<(&[u8], &str)>) -> Result<u64, sqlx::Error> {
    sqlx::query("UPDATE firma SET logo = $1, logo_typ = $2, geaendert_am = now()")
        .bind(logo.map(|(bytes, _)| bytes))
        .bind(logo.map(|(_, typ)| typ))
        .execute(pool)
        .await
        .map(|r| r.rows_affected())
}

/// Logo mit Dateityp; `None` ohne Firma oder ohne Logo.
pub async fn logo(pool: &PgPool) -> Result<Option<(Vec<u8>, String)>, sqlx::Error> {
    sqlx::query_as("SELECT logo, logo_typ FROM firma WHERE logo IS NOT NULL")
        .fetch_optional(pool)
        .await
}
