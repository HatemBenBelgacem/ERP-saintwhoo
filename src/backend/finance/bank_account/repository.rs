//! SQL-Zugriffe für Bankkonten (ANF-015, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.

use sqlx::{AssertSqlSafe, PgConnection, PgPool};

use super::model::{BankAccount, BankAccountInput};

/// Ein Bankkonto mit seinem Konto im Kontenplan.
const SELECT_ACCOUNT: &str = "\
    SELECT b.id, b.bezeichnung, b.art, b.bank, b.iban, b.qr_iban, b.bic, b.konto_id, \
           k.nummer AS konto_nummer, k.bezeichnung AS konto_bezeichnung, b.standard, b.aktiv \
    FROM bankkonto b JOIN konto k ON k.id = b.konto_id";

/// Leerer Text → NULL in der Datenbank.
fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

/// Alle Bankkonten; das Standard-Konto zuerst.
pub async fn list(pool: &PgPool) -> Result<Vec<BankAccount>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_ACCOUNT} ORDER BY b.standard DESC, lower(b.bezeichnung), b.id"
    )))
    .fetch_all(pool)
    .await
}

/// Ein Bankkonto; `None`, wenn es das nicht gibt.
pub async fn find(conn: &mut PgConnection, id: i64) -> Result<Option<BankAccount>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!("{SELECT_ACCOUNT} WHERE b.id = $1")))
        .bind(id)
        .fetch_optional(conn)
        .await
}

/// Anzahl der Bankkonten (das erste Konto wird automatisch Standard, AK-5).
pub async fn count(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM bankkonto")
        .fetch_one(conn)
        .await
}

/// Sperrt die Bankkonten für die Dauer der Transaktion, damit nie zwei Standard-Konten entstehen.
pub async fn lock(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("LOCK TABLE bankkonto IN SHARE ROW EXCLUSIVE MODE")
        .execute(conn)
        .await?;
    Ok(())
}

/// Art und Aktiv-Status eines Kontos im Kontenplan; `None`, wenn es das Konto nicht gibt.
pub async fn konto_info(
    conn: &mut PgConnection,
    konto_id: i64,
) -> Result<Option<(String, bool)>, sqlx::Error> {
    sqlx::query_as("SELECT kontoart, aktiv FROM konto WHERE id = $1")
        .bind(konto_id)
        .fetch_optional(conn)
        .await
}

/// `true`, wenn das Konto schon zu einem anderen Bankkonto gehört.
pub async fn konto_taken(
    conn: &mut PgConnection,
    konto_id: i64,
    except: Option<i64>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM bankkonto WHERE konto_id = $1 AND ($2::bigint IS NULL OR id <> $2))",
    )
    .bind(konto_id)
    .bind(except)
    .fetch_one(conn)
    .await
}

/// Nimmt allen anderen Bankkonten das Standard-Kennzeichen weg.
pub async fn clear_standard(
    conn: &mut PgConnection,
    except: Option<i64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE bankkonto SET standard = false WHERE standard AND ($1::bigint IS NULL OR id <> $1)",
    )
    .bind(except)
    .execute(conn)
    .await?;
    Ok(())
}

/// Legt ein Bankkonto an; liefert die Kennung.
pub async fn insert(
    conn: &mut PgConnection,
    input: &BankAccountInput,
    konto_id: i64,
    standard: bool,
    aktiv: bool,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO bankkonto (bezeichnung, art, bank, iban, qr_iban, bic, konto_id, standard, aktiv) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) RETURNING id",
    )
    .bind(&input.bezeichnung)
    .bind(&input.art)
    .bind(opt(&input.bank))
    .bind(opt(&input.iban))
    .bind(opt(&input.qr_iban))
    .bind(opt(&input.bic))
    .bind(konto_id)
    .bind(standard)
    .bind(aktiv)
    .fetch_one(conn)
    .await
}

/// Ändert ein Bankkonto; liefert die Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update(
    conn: &mut PgConnection,
    id: i64,
    input: &BankAccountInput,
    konto_id: i64,
    standard: bool,
    aktiv: bool,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE bankkonto SET bezeichnung = $2, art = $3, bank = $4, iban = $5, qr_iban = $6, bic = $7, \
                konto_id = $8, standard = $9, aktiv = $10, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&input.bezeichnung)
    .bind(&input.art)
    .bind(opt(&input.bank))
    .bind(opt(&input.iban))
    .bind(opt(&input.qr_iban))
    .bind(opt(&input.bic))
    .bind(konto_id)
    .bind(standard)
    .bind(aktiv)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Löscht ein Bankkonto; liefert die Anzahl gelöschter Zeilen.
pub async fn delete(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM bankkonto WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}
