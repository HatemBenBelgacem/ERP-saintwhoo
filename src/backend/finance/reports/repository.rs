//! SQL-Zugriffe für die Auswertungen (ANF-021, nur Server). Entwürfe bleiben immer außen vor (AK-9).
//! Zeiträume sind optional: `None` = ohne Grenze. Beträge kommen in Rappen zurück.

use sqlx::{PgExecutor, PgPool};

use super::model::{AccountBalance, CounterLine, ProjectRow, StatementLine};
use crate::backend::finance::journal::model::AccountOption;

/// Salden aller Konten im Zeitraum (auch Konten ohne Bewegung, sortiert nach Nummer).
/// Zeitraum `von` = `None` ab Anfang, `bis` = `None` ohne Ende.
pub async fn balances<'e>(
    exec: impl PgExecutor<'e>,
    von: Option<&str>,
    bis: Option<&str>,
    projekt_id: Option<i64>,
) -> Result<Vec<AccountBalance>, sqlx::Error> {
    sqlx::query_as(
        "SELECT k.id AS konto_id, k.nummer, k.bezeichnung, k.kontoart, k.kontengruppe, \
                ROUND(COALESCE(SUM(s.soll), 0) * 100)::BIGINT AS soll, \
                ROUND(COALESCE(SUM(s.haben), 0) * 100)::BIGINT AS haben \
         FROM konto k \
         LEFT JOIN (SELECT z.konto_id, z.soll, z.haben \
                    FROM buchungszeile z JOIN buchung b ON b.id = z.buchung_id \
                    WHERE b.status <> 'entwurf' \
                      AND ($1::date IS NULL OR b.datum >= $1::date) \
                      AND ($2::date IS NULL OR b.datum <= $2::date) \
                      AND ($3::bigint IS NULL OR z.projekt_id = $3)) s ON s.konto_id = k.id \
         GROUP BY k.id \
         ORDER BY k.nummer",
    )
    .bind(von)
    .bind(bis)
    .bind(projekt_id)
    .fetch_all(exec)
    .await
}

/// Zeilen eines Kontos im Zeitraum, nach Datum und Journalnummer (Kontoauszug, AK-2).
pub async fn statement_lines(
    pool: &PgPool,
    konto_id: i64,
    von: Option<&str>,
    bis: Option<&str>,
) -> Result<Vec<StatementLine>, sqlx::Error> {
    sqlx::query_as(
        "SELECT b.id AS buchung_id, b.datum::text AS datum, b.journalnummer, b.belegnummer, b.text, \
                ROUND(z.soll * 100)::BIGINT AS soll, ROUND(z.haben * 100)::BIGINT AS haben, \
                z.ist_mwst_zeile \
         FROM buchungszeile z JOIN buchung b ON b.id = z.buchung_id \
         WHERE z.konto_id = $1 AND b.status <> 'entwurf' \
           AND ($2::date IS NULL OR b.datum >= $2::date) \
           AND ($3::date IS NULL OR b.datum <= $3::date) \
         ORDER BY b.datum, b.journalnummer, z.reihenfolge",
    )
    .bind(konto_id)
    .bind(von)
    .bind(bis)
    .fetch_all(pool)
    .await
}

/// Alle Zeilen der genannten Buchungen mit Kontonummer (für das Gegenkonto).
pub async fn counter_lines(
    pool: &PgPool,
    buchung_ids: &[i64],
) -> Result<Vec<CounterLine>, sqlx::Error> {
    sqlx::query_as(
        "SELECT z.buchung_id, z.konto_id, k.nummer, z.ist_mwst_zeile \
         FROM buchungszeile z JOIN konto k ON k.id = z.konto_id \
         WHERE z.buchung_id = ANY($1)",
    )
    .bind(buchung_ids)
    .fetch_all(pool)
    .await
}

/// Projekte mit Erlösen, Aufwand und offenen Einkaufsaufträgen im Zeitraum (AK-7).
/// Offene Aufträge: Restmenge × Preis nach Rabatt aus bestellten oder teilweise gelieferten Aufträgen (ANF-004).
pub async fn project_rows(
    pool: &PgPool,
    projekt_id: Option<i64>,
    von: Option<&str>,
    bis: Option<&str>,
) -> Result<Vec<ProjectRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT p.id AS projekt_id, p.nummer, p.name, \
                ROUND(COALESCE(SUM(CASE WHEN k.kontoart = 'ertrag' THEN s.haben - s.soll END), 0) * 100)::BIGINT \
                    AS erloes, \
                ROUND(COALESCE(SUM(CASE WHEN k.kontoart = 'aufwand' THEN s.soll - s.haben END), 0) * 100)::BIGINT \
                    AS aufwand, \
                ROUND(COALESCE((SELECT SUM(GREATEST(e.menge - e.gelieferte_menge, 0) \
                                           * e.einzelpreis * (1 - e.rabatt_prozent / 100)) \
                                FROM einkaufsposition e JOIN einkaufsauftrag a ON a.id = e.einkaufsauftrag_id \
                                WHERE a.projekt_id = p.id AND a.status IN ('bestellt', 'teilweise_geliefert')), 0) \
                      * 100)::BIGINT AS offen \
         FROM projekt p \
         LEFT JOIN (SELECT z.projekt_id, z.konto_id, z.soll, z.haben \
                    FROM buchungszeile z JOIN buchung b ON b.id = z.buchung_id \
                    WHERE b.status <> 'entwurf' AND z.projekt_id IS NOT NULL \
                      AND ($2::date IS NULL OR b.datum >= $2::date) \
                      AND ($3::date IS NULL OR b.datum <= $3::date)) s ON s.projekt_id = p.id \
         LEFT JOIN konto k ON k.id = s.konto_id \
         WHERE ($1::bigint IS NULL OR p.id = $1) \
         GROUP BY p.id \
         ORDER BY p.nummer",
    )
    .bind(projekt_id)
    .bind(von)
    .bind(bis)
    .fetch_all(pool)
    .await
}

/// Alle Konten für die Auswahl im Kontoauszug, auch inaktive (Auswertungen sehen sie weiterhin, ANF-011 AK-10).
pub async fn all_accounts<'e>(
    exec: impl PgExecutor<'e>,
) -> Result<Vec<AccountOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, bezeichnung, NULL::BIGINT AS vorgabe_mwst FROM konto ORDER BY nummer",
    )
    .fetch_all(exec)
    .await
}

/// Gebuchte Erlöse (Kontenklasse 3, Haben minus Soll) im Zeitraum in Rappen (ANF-012 AK-11).
pub async fn revenue<'e>(
    exec: impl PgExecutor<'e>,
    von: &str,
    bis: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT ROUND(COALESCE(SUM(z.haben - z.soll), 0) * 100)::BIGINT \
         FROM buchungszeile z \
         JOIN buchung b ON b.id = z.buchung_id \
         JOIN konto k ON k.id = z.konto_id \
         WHERE b.status <> 'entwurf' AND k.nummer LIKE '3%' AND b.datum BETWEEN $1::date AND $2::date",
    )
    .bind(von)
    .bind(bis)
    .fetch_one(exec)
    .await
}
