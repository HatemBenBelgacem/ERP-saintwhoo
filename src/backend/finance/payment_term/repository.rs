//! SQL-Zugriffe für Zahlungsbedingungen (ANF-016, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Tage als `bigint` übergeben und gelesen, Skonto in Hundertsteln (`* 100`), damit kein Gleitkommawert entsteht.

use sqlx::{AssertSqlSafe, PgConnection, PgPool};

use super::model::{PaymentTerm, PaymentTermInput, Skonto};

/// Eine Zahlungsbedingung; `verwendet` prüft die Adressen (AK-7).
const SELECT_TERM: &str = "\
    SELECT z.id, z.bezeichnung, z.zahlungsfrist_tage::int8 AS zahlungsfrist_tage, \
           z.skontofrist_tage::int8 AS skontofrist_tage, (z.skonto_prozent * 100)::int8 AS skonto_prozent, \
           z.rechnungstext, z.standard, z.aktiv, \
           EXISTS (SELECT 1 FROM adresse a WHERE a.zahlungsbedingung_id = z.id) AS verwendet \
    FROM zahlungsbedingung z";

/// Alle Zahlungsbedingungen; die Standard-Bedingung zuerst.
pub async fn list(pool: &PgPool) -> Result<Vec<PaymentTerm>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_TERM} ORDER BY z.standard DESC, lower(z.bezeichnung), z.id"
    )))
    .fetch_all(pool)
    .await
}

/// Eine Zahlungsbedingung; `None`, wenn es sie nicht gibt.
pub async fn find(conn: &mut PgConnection, id: i64) -> Result<Option<PaymentTerm>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!("{SELECT_TERM} WHERE z.id = $1")))
        .bind(id)
        .fetch_optional(conn)
        .await
}

/// Anzahl der Zahlungsbedingungen (die erste wird automatisch Standard).
pub async fn count(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM zahlungsbedingung")
        .fetch_one(conn)
        .await
}

/// Sperrt die Tabelle für die Dauer der Transaktion, damit nie zwei Standard-Bedingungen entstehen.
pub async fn lock(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("LOCK TABLE zahlungsbedingung IN SHARE ROW EXCLUSIVE MODE")
        .execute(conn)
        .await?;
    Ok(())
}

/// Nimmt allen anderen Bedingungen das Standard-Kennzeichen weg.
pub async fn clear_standard(
    conn: &mut PgConnection,
    except: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE zahlungsbedingung SET standard = false WHERE standard AND ($1::bigint IS NULL OR id <> $1)",
    )
    .bind(except)
    .execute(conn)
    .await?;
    Ok(())
}

/// Legt eine Zahlungsbedingung an; liefert die Kennung.
pub async fn insert(
    conn: &mut PgConnection,
    input: &PaymentTermInput,
    frist: i64,
    skonto: Skonto,
    standard: bool,
    aktiv: bool,
) -> Result<i64, sqlx::Error> {
    let (skontofrist, prozent) = split(skonto);
    sqlx::query_scalar(
        "INSERT INTO zahlungsbedingung (bezeichnung, zahlungsfrist_tage, skontofrist_tage, skonto_prozent, \
                                        rechnungstext, standard, aktiv) \
         VALUES ($1, $2::bigint, $3::bigint, $4::bigint::numeric / 100, $5, $6, $7) RETURNING id",
    )
    .bind(&input.bezeichnung)
    .bind(frist)
    .bind(skontofrist)
    .bind(prozent)
    .bind(&input.rechnungstext)
    .bind(standard)
    .bind(aktiv)
    .fetch_one(conn)
    .await
}

/// Ändert eine Zahlungsbedingung; liefert die Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update(
    conn: &mut PgConnection,
    id: i64,
    input: &PaymentTermInput,
    frist: i64,
    skonto: Skonto,
    standard: bool,
    aktiv: bool,
) -> Result<u64, sqlx::Error> {
    let (skontofrist, prozent) = split(skonto);
    sqlx::query(
        "UPDATE zahlungsbedingung SET bezeichnung = $2, zahlungsfrist_tage = $3::bigint, \
                skontofrist_tage = $4::bigint, skonto_prozent = $5::bigint::numeric / 100, \
                rechnungstext = $6, standard = $7, aktiv = $8, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&input.bezeichnung)
    .bind(frist)
    .bind(skontofrist)
    .bind(prozent)
    .bind(&input.rechnungstext)
    .bind(standard)
    .bind(aktiv)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Löscht eine Zahlungsbedingung. Verweise von Adressen verhindern das mit Fehler 23503 (AK-7).
pub async fn delete(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM zahlungsbedingung WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Skontofrist und Skonto (Hundertstel) für die Spalten; leer = NULL.
fn split(skonto: Skonto) -> (Option<i64>, Option<i64>) {
    match skonto {
        Some((tage, prozent)) => (Some(tage), Some(prozent)),
        None => (None, None),
    }
}
