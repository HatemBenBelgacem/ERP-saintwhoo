//! SQL-Zugriffe für Kontenplan und Systemkonten (ANF-011, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.

use sqlx::{AssertSqlSafe, PgConnection, PgPool};

use super::model::{Account, AccountInput, TemplateAccount};

/// Konto mit seinen Systemkonto-Rollen; muss mit `GROUP BY k.id` abgeschlossen werden.
const SELECT_ACCOUNT: &str = "\
    SELECT k.id, k.nummer, k.bezeichnung, k.kontoart, k.kontengruppe, k.aktiv, k.bemerkung, \
           COALESCE(array_agg(s.rolle::text ORDER BY s.rolle) FILTER (WHERE s.rolle IS NOT NULL), \
                    '{}'::text[]) AS rollen \
    FROM konto k LEFT JOIN systemkonto s ON s.konto_id = k.id";

/// Kontenplan mit Suche (Nummer, Bezeichnung, Kontengruppe), Kontoart und inaktiven Konten (AK-1, AK-2).
/// `kontoart` leer = alle. Sortiert nach Nummer (als Text: Unterkonten folgen ihrem Hauptkonto).
pub async fn list(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    kontoart: &str,
    include_inactive: bool,
) -> Result<Vec<Account>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_ACCOUNT} \
         WHERE ($1 = '' OR k.nummer LIKE $2 OR k.bezeichnung ILIKE $2 OR k.kontengruppe ILIKE $2) \
           AND ($3 = '' OR k.kontoart = $3) \
           AND ($4 OR k.aktiv) \
         GROUP BY k.id ORDER BY k.nummer"
    )))
    .bind(suche)
    .bind(pattern)
    .bind(kontoart)
    .bind(include_inactive)
    .fetch_all(pool)
    .await
}

/// Ein Konto; `None`, wenn es es nicht gibt.
pub async fn find(pool: &PgPool, id: i64) -> Result<Option<Account>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_ACCOUNT} WHERE k.id = $1 GROUP BY k.id"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Mehrere Konten nach Kennung (für die Prüfung der Systemkonten).
pub async fn find_many(pool: &PgPool, ids: &[i64]) -> Result<Vec<Account>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_ACCOUNT} WHERE k.id = ANY($1) GROUP BY k.id"
    )))
    .bind(ids)
    .fetch_all(pool)
    .await
}

pub async fn count(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM konto")
        .fetch_one(pool)
        .await
}

/// `true`, wenn die Nummer schon von einem anderen Konto als `except` belegt ist.
pub async fn number_taken(
    pool: &PgPool,
    nummer: &str,
    except: Option<i64>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM konto WHERE nummer = $1 AND ($2::bigint IS NULL OR id <> $2))",
    )
    .bind(nummer)
    .bind(except)
    .fetch_one(pool)
    .await
}

pub async fn insert(pool: &PgPool, input: &AccountInput) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO konto (nummer, bezeichnung, kontoart, kontengruppe, bemerkung, aktiv) \
         VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
    )
    .bind(&input.nummer)
    .bind(&input.bezeichnung)
    .bind(&input.kontoart)
    .bind(input.group())
    .bind(input.remark())
    .bind(input.is_active())
    .fetch_one(pool)
    .await
}

/// Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update(pool: &PgPool, id: i64, input: &AccountInput) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE konto SET nummer = $2, bezeichnung = $3, kontoart = $4, kontengruppe = $5, \
                bemerkung = $6, aktiv = $7, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&input.nummer)
    .bind(&input.bezeichnung)
    .bind(&input.kontoart)
    .bind(input.group())
    .bind(input.remark())
    .bind(input.is_active())
    .execute(pool)
    .await
    .map(|r| r.rows_affected())
}

/// Löscht ein Konto und gibt seine Nummer zurück (`None` = nicht gefunden).
/// Verweise (Systemkonto, später Buchungen) verhindern das Löschen mit Fehler 23503.
pub async fn delete(pool: &PgPool, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("DELETE FROM konto WHERE id = $1 RETURNING nummer")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Zuordnung Rolle → Konto aller Systemkonten.
pub async fn system_accounts(pool: &PgPool) -> Result<Vec<(String, i64)>, sqlx::Error> {
    sqlx::query_as("SELECT rolle::text, konto_id FROM systemkonto")
        .fetch_all(pool)
        .await
}

/// Ersetzt alle Systemkonten durch `assignments` (Rolle, Konto) – innerhalb einer Transaktion.
pub async fn replace_system_accounts(
    conn: &mut PgConnection,
    assignments: &[(&str, i64)],
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM systemkonto")
        .execute(&mut *conn)
        .await?;
    for (role, id) in assignments {
        sqlx::query("INSERT INTO systemkonto (rolle, konto_id) VALUES ($1, $2)")
            .bind(role)
            .bind(id)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Sperrt den Kontenplan für die Dauer der Transaktion gegen gleichzeitiges Anlegen und
/// gibt die Anzahl Konten zurück (Vorlage nur bei leerem Kontenplan, AK-3).
pub async fn lock_and_count(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    sqlx::query("LOCK TABLE konto IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut *conn)
        .await?;
    sqlx::query_scalar("SELECT count(*) FROM konto")
        .fetch_one(&mut *conn)
        .await
}

/// Legt die Konten der Vorlage an.
pub async fn insert_template(
    conn: &mut PgConnection,
    accounts: &[TemplateAccount],
) -> Result<(), sqlx::Error> {
    for a in accounts {
        sqlx::query(
            "INSERT INTO konto (nummer, bezeichnung, kontoart, kontengruppe) VALUES ($1, $2, $3, $4)",
        )
        .bind(a.nummer)
        .bind(a.bezeichnung)
        .bind(a.kontoart)
        .bind(a.gruppe)
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

/// Kennung eines Kontos nach Nummer (innerhalb einer Transaktion).
pub async fn id_by_number(
    conn: &mut PgConnection,
    nummer: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM konto WHERE nummer = $1")
        .bind(nummer)
        .fetch_optional(&mut *conn)
        .await
}
