//! SQL-Zugriffe für Geschäftsjahre und Perioden (ANF-013, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Datumswerte werden als Text gelesen (`::text`) und als Text übergeben (`::date`).

use sqlx::{PgConnection, PgPool};

use super::model::{FiscalYear, Period};

/// Alle Geschäftsjahre, neueste zuoberst (Perioden werden getrennt geladen).
pub async fn list_years(pool: &PgPool) -> Result<Vec<FiscalYear>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, bezeichnung, beginn::text AS beginn, ende::text AS ende, status \
         FROM geschaeftsjahr ORDER BY beginn DESC",
    )
    .fetch_all(pool)
    .await
}

/// Alle Perioden aller Geschäftsjahre in zeitlicher Reihenfolge.
pub async fn list_periods(pool: &PgPool) -> Result<Vec<Period>, sqlx::Error> {
    sqlx::query_as(
        "SELECT p.id, p.geschaeftsjahr_id, p.beginn::text AS beginn, p.ende::text AS ende, p.status, \
                CASE WHEN p.geaendert_von IS NULL THEN NULL \
                     ELSE b.benutzername || ' · ' || to_char(p.geaendert_am, 'DD.MM.YYYY HH24:MI') \
                END AS vermerk \
         FROM periode p LEFT JOIN benutzer b ON b.id = p.geaendert_von \
         ORDER BY p.beginn",
    )
    .fetch_all(pool)
    .await
}

/// Neuestes Geschäftsjahr: Kennung und Ende (`None`, wenn keines existiert).
pub async fn latest_year(conn: &mut PgConnection) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as("SELECT id, ende::text FROM geschaeftsjahr ORDER BY ende DESC LIMIT 1")
        .fetch_optional(&mut *conn)
        .await
}

/// Legt ein Geschäftsjahr an und gibt seine Kennung zurück.
pub async fn insert_year(
    conn: &mut PgConnection,
    bezeichnung: &str,
    beginn: &str,
    ende: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO geschaeftsjahr (bezeichnung, beginn, ende) VALUES ($1, $2::date, $3::date) RETURNING id",
    )
    .bind(bezeichnung)
    .bind(beginn)
    .bind(ende)
    .fetch_one(conn)
    .await
}

/// Legt eine Periode (Kalendermonat) zu einem Geschäftsjahr an.
pub async fn insert_period(
    conn: &mut PgConnection,
    year_id: i64,
    beginn: &str,
    ende: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO periode (geschaeftsjahr_id, beginn, ende) VALUES ($1, $2::date, $3::date)",
    )
    .bind(year_id)
    .bind(beginn)
    .bind(ende)
    .execute(conn)
    .await?;
    Ok(())
}

/// Geschäftsjahr mit seinen Perioden löschen; gibt die Anzahl gelöschter Geschäftsjahre zurück.
pub async fn delete_year(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM geschaeftsjahr WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Geschäftsjahr einer Periode: Kennung und Status des Geschäftsjahrs (`None`, wenn es die Periode nicht gibt).
pub async fn year_of_period(
    conn: &mut PgConnection,
    period_id: i64,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT g.id, g.status FROM periode p JOIN geschaeftsjahr g ON g.id = p.geschaeftsjahr_id \
         WHERE p.id = $1",
    )
    .bind(period_id)
    .fetch_optional(conn)
    .await
}

/// Setzt den Status einer Periode und vermerkt Benutzer und Zeitpunkt (AK-5).
pub async fn set_period_status(
    conn: &mut PgConnection,
    period_id: i64,
    status: &str,
    user_id: i64,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE periode SET status = $2, geaendert_von = $3, geaendert_am = now() WHERE id = $1",
    )
    .bind(period_id)
    .bind(status)
    .bind(user_id)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Heutiges Datum der Datenbank (Zeitzone des Servers) als `JJJJ-MM-TT`.
pub async fn today(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("SELECT CURRENT_DATE::text")
        .fetch_one(pool)
        .await
}

/// Geschäftsjahr zu einem Datum mit dem Status der dazugehörigen Periode (ANF-017 prüft damit das Buchungsdatum).
/// `None`, wenn kein Geschäftsjahr das Datum enthält. Die Periode ist `None`, wenn keine passt.
pub async fn year_for_date(
    pool: &PgPool,
    datum: &str,
) -> Result<Option<(String, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT g.status, p.status \
         FROM geschaeftsjahr g \
         LEFT JOIN periode p ON p.geschaeftsjahr_id = g.id AND $1::date BETWEEN p.beginn AND p.ende \
         WHERE $1::date BETWEEN g.beginn AND g.ende",
    )
    .bind(datum)
    .fetch_optional(pool)
    .await
}

/// Sperrt die Geschäftsjahre für die Dauer der Transaktion, damit zwei gleichzeitige Anlagen
/// nicht dieselbe Lücke füllen (AK-3).
pub async fn lock_years(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("LOCK TABLE geschaeftsjahr IN SHARE ROW EXCLUSIVE MODE")
        .execute(conn)
        .await?;
    Ok(())
}
