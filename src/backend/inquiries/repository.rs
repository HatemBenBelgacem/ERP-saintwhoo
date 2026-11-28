//! SQL-Zugriffe für Anfragen (ANF-029, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff. Fehler werden als `sqlx::Error` zurückgegeben;
//! die Server-Funktionen übersetzen sie. Funktionen mit `&mut PgConnection` laufen in der Transaktion des Aufrufers.

use sqlx::{PgConnection, PgPool};

use super::model::{InquiryRow, format_nummer};

/// Name des Teilprojekts (Titel der Anfrage) und Beschreibung eines Vorgangs. `None`, wenn es den Vorgang nicht gibt.
/// Sperrt den Vorgang bis zum Ende der Transaktion, damit er nicht gleichzeitig gelöscht wird.
pub async fn source_of(
    conn: &mut PgConnection,
    vorgang_id: i64,
) -> Result<Option<(String, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT t.name, v.beschreibung FROM vorgang v JOIN teilprojekt t ON t.id = v.teilprojekt_id \
         WHERE v.id = $1 FOR SHARE OF v",
    )
    .bind(vorgang_id)
    .fetch_optional(conn)
    .await
}

/// Anfragen eines Vorgangs, neueste zuerst.
pub async fn list_by_vorgang(
    pool: &PgPool,
    vorgang_id: i64,
) -> Result<Vec<InquiryRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, titel, beschreibung, \
                to_char(erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am \
         FROM anfrage WHERE vorgang_id = $1 ORDER BY id DESC",
    )
    .bind(vorgang_id)
    .fetch_all(pool)
    .await
}

/// Neue Anfrage speichern; Nummer `A-JJJJ-NNNN` aus dem Zähler des Jahres. Liefert die Kennung.
pub async fn insert_inquiry(
    conn: &mut PgConnection,
    vorgang_id: i64,
    titel: &str,
    beschreibung: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let (year, n): (i32, i32) = sqlx::query_as(
        "INSERT INTO anfrage_nummer (jahr, letzte) \
         VALUES (extract(year FROM now() AT TIME ZONE 'Europe/Zurich')::int, 1) \
         ON CONFLICT (jahr) DO UPDATE SET letzte = anfrage_nummer.letzte + 1 \
         RETURNING jahr, letzte",
    )
    .fetch_one(&mut *conn)
    .await?;
    sqlx::query_scalar(
        "INSERT INTO anfrage (nummer, vorgang_id, titel, beschreibung) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(format_nummer(year, n))
    .bind(vorgang_id)
    .bind(titel)
    .bind(beschreibung)
    .fetch_one(conn)
    .await
}

/// Anfrage löschen. `false`, wenn es sie nicht gibt.
pub async fn delete_inquiry(pool: &PgPool, id: i64) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM anfrage WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
