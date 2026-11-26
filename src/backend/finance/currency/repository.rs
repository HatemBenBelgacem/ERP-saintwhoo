//! SQL-Zugriffe für Währungen und Kurse (ANF-014, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Rundung in Hundertstel (`* 100`), Kurse in Millionstel (`* 1000000`), damit kein Gleitkommawert entsteht.

use sqlx::{AssertSqlSafe, PgConnection, PgPool};

use super::model::{Currency, CurrencyInput, ExchangeRate, RateInput};

/// Eine Währung mit ihrem letzten Kurs; `verwendet` prüft die Einkaufsaufträge (AK-5).
const SELECT_CURRENCY: &str = "\
    SELECT w.code, w.bezeichnung, (w.rundung * 100)::int8 AS rundung, w.buchhaltungswaehrung, w.aktiv, \
           (SELECT (k.kurs * 1000000)::int8 FROM waehrungskurs k WHERE k.waehrung_code = w.code \
            ORDER BY k.datum DESC LIMIT 1) AS letzter_kurs, \
           (SELECT to_char(k.datum, 'YYYY-MM-DD') FROM waehrungskurs k WHERE k.waehrung_code = w.code \
            ORDER BY k.datum DESC LIMIT 1) AS letzter_kurs_datum, \
           EXISTS (SELECT 1 FROM einkaufsauftrag e WHERE e.waehrung = w.code) AS verwendet \
    FROM waehrung w";

/// Alle Währungen; die Buchhaltungswährung zuerst.
pub async fn list(pool: &PgPool) -> Result<Vec<Currency>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_CURRENCY} ORDER BY w.buchhaltungswaehrung DESC, w.code"
    )))
    .fetch_all(pool)
    .await
}

/// Eine Währung; `None`, wenn es sie nicht gibt.
pub async fn find(conn: &mut PgConnection, code: &str) -> Result<Option<Currency>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_CURRENCY} WHERE w.code = $1"
    )))
    .bind(code)
    .fetch_optional(conn)
    .await
}

/// `true`, wenn die Währung existiert und aktiv ist (AK-6, Einkauf).
pub async fn is_active(conn: &mut PgConnection, code: &str) -> Result<bool, sqlx::Error> {
    let aktiv: Option<bool> = sqlx::query_scalar("SELECT aktiv FROM waehrung WHERE code = $1")
        .bind(code)
        .fetch_optional(conn)
        .await?;
    Ok(aktiv.unwrap_or(false))
}

/// Legt eine Währung an (nie als Buchhaltungswährung).
pub async fn insert(
    conn: &mut PgConnection,
    input: &CurrencyInput,
    rundung: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO waehrung (code, bezeichnung, rundung, buchhaltungswaehrung, aktiv) \
         VALUES ($1, $2, $3::bigint::numeric / 100, false, $4)",
    )
    .bind(&input.code)
    .bind(&input.bezeichnung)
    .bind(rundung)
    .bind(input.is_active())
    .execute(conn)
    .await?;
    Ok(())
}

/// Ändert Bezeichnung, Rundung und Aktiv-Status; liefert die Anzahl geänderter Zeilen.
pub async fn update(
    conn: &mut PgConnection,
    code: &str,
    input: &CurrencyInput,
    rundung: i64,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE waehrung SET bezeichnung = $2, rundung = $3::bigint::numeric / 100, aktiv = $4, \
                geaendert_am = now() \
         WHERE code = $1",
    )
    .bind(code)
    .bind(&input.bezeichnung)
    .bind(rundung)
    .bind(input.is_active())
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Löscht eine Währung; ihre Kurse gehen mit. Einkaufsaufträge verhindern das mit Fehler 23503 (AK-5).
pub async fn delete(conn: &mut PgConnection, code: &str) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM waehrung WHERE code = $1")
        .bind(code)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Speichert einen Kurs; ein zweiter Kurs am selben Datum verletzt die Eindeutigkeit (AK-3).
pub async fn insert_rate(
    conn: &mut PgConnection,
    code: &str,
    input: &RateInput,
    kurs: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO waehrungskurs (waehrung_code, datum, kurs) \
         VALUES ($1, $2::date, $3::bigint::numeric / 1000000)",
    )
    .bind(code)
    .bind(&input.datum)
    .bind(kurs)
    .execute(conn)
    .await?;
    Ok(())
}

/// Neuester Kurs einer Währung bis einschliesslich `datum` (AK-4).
pub async fn rate_on(
    pool: &PgPool,
    code: &str,
    datum: &str,
) -> Result<Option<ExchangeRate>, sqlx::Error> {
    let row: Option<(String, i64)> = sqlx::query_as(
        "SELECT to_char(datum, 'YYYY-MM-DD'), (kurs * 1000000)::int8 FROM waehrungskurs \
         WHERE waehrung_code = $1 AND datum <= $2::date ORDER BY datum DESC LIMIT 1",
    )
    .bind(code)
    .bind(datum)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(datum, kurs)| ExchangeRate { datum, kurs }))
}

/// Heutiges Datum der Datenbank als `JJJJ-MM-TT` (Vorgabe im Kurs-Formular).
pub async fn today(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("SELECT CURRENT_DATE::text")
        .fetch_one(pool)
        .await
}
