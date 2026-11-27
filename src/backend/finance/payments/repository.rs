//! SQL-Zugriffe für Zahlungen und offene Posten (ANF-020, nur Server).
//!
//! Debitoren- und Kreditorenrechnungen haben dieselben Spalten für Status und offenen Betrag; die Tabelle
//! wählt `beleg_table` aus einer festen Liste.

use sqlx::{AssertSqlSafe, PgConnection, PgExecutor, PgPool};

use super::model::{
    AccountChoice, AgingItem, AllocationValues, AllocationView, BankOption, OpenItem, Payment,
    PaymentFilter, PaymentRow, PaymentValues,
};
use crate::backend::purchase_orders::model::format_fixed;

/// Tabelle der Belege einer Seite (feste Liste, nie Text aus einer Eingabe).
fn beleg_table(seite: &str) -> &'static str {
    if seite == "kreditoren" {
        "kreditorenrechnung"
    } else {
        "debitorenrechnung"
    }
}

/// Spalte der Ausgleichszeile für den Beleg einer Seite.
fn beleg_column(seite: &str) -> &'static str {
    if seite == "kreditoren" {
        "kreditorenrechnung_id"
    } else {
        "debitorenrechnung_id"
    }
}

fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

// ---------- Offene Posten eines Partners ----------

/// Offene Rechnungen und Gutschriften eines Partners; mit `vorauszahlungen` auch offene Vorauszahlungen.
pub async fn open_items<'e>(
    exec: impl PgExecutor<'e>,
    seite: &str,
    adresse_id: i64,
    vorauszahlungen: bool,
) -> Result<Vec<OpenItem>, sqlx::Error> {
    let referenz = if seite == "kreditoren" {
        "r.rechnungsnummer"
    } else {
        "r.referenz_kunde"
    };
    // Geprüft: Tabelle und Spalte aus fester Liste
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT r.art, r.id AS beleg_id, r.nummer, {referenz} AS referenz, r.rechnungsdatum::text AS datum, \
                r.faelligkeit::text AS faelligkeit, r.skonto_bis::text AS skonto_bis, \
                (r.skonto_prozent * 100)::int8 AS skonto_prozent, r.waehrung, \
                ROUND(r.kurs * 1000000)::int8 AS kurs, ROUND(r.gesamtbetrag * 100)::int8 AS gesamtbetrag, \
                ROUND(r.offener_betrag * 100)::int8 AS offen \
         FROM {} r WHERE r.adresse_id = $1 AND r.status IN ('offen', 'teilweise_bezahlt') \
           AND r.offener_betrag > 0 \
         UNION ALL \
         SELECT 'vorauszahlung', z.id, 'Z-' || z.id, z.text, z.datum::text, NULL, NULL, NULL, z.waehrung, \
                ROUND(z.kurs * 1000000)::int8, ROUND(z.vorauszahlung * 100)::int8, \
                ROUND(z.vorauszahlung_offen * 100)::int8 \
         FROM zahlung z WHERE $2 AND z.adresse_id = $1 AND z.seite = $3 AND z.status = 'gebucht' \
           AND z.vorauszahlung_offen > 0 \
         ORDER BY faelligkeit NULLS LAST, beleg_id",
        beleg_table(seite)
    )))
    .bind(adresse_id)
    .bind(vorauszahlungen)
    .bind(seite)
    .fetch_all(exec)
    .await
}

/// Beleg nach Nummer (interne Nummer, bei Kreditoren auch Rechnungsnummer des Lieferanten):
/// (Kennung, Partner-Kennung, Partner-Nummer, Partner-Name, Art, Währung, offen in Rappen).
#[allow(clippy::type_complexity)]
pub async fn find_document(
    pool: &PgPool,
    seite: &str,
    nummer: &str,
    id: Option<i64>,
) -> Result<Option<(i64, i64, String, String, String, String, i64)>, sqlx::Error> {
    let lieferant = if seite == "kreditoren" {
        "OR lower(r.rechnungsnummer) = lower($1)"
    } else {
        ""
    };
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT r.id, a.id, a.nummer, a.anzeigename, r.art, r.waehrung, \
                ROUND(r.offener_betrag * 100)::int8 \
         FROM {} r JOIN adresse a ON a.id = r.adresse_id \
         WHERE r.status IN ('offen', 'teilweise_bezahlt') \
           AND (r.id = $2 OR lower(r.nummer) = lower($1) {lieferant}) \
         ORDER BY r.id LIMIT 1",
        beleg_table(seite)
    )))
    .bind(nummer)
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Belege sperren, bevor ihr offener Betrag geändert wird.
pub async fn lock_documents(
    conn: &mut PgConnection,
    seite: &str,
    ids: &[i64],
) -> Result<(), sqlx::Error> {
    sqlx::query(AssertSqlSafe(format!(
        "SELECT id FROM {} WHERE id = ANY($1) ORDER BY id FOR UPDATE",
        beleg_table(seite)
    )))
    .bind(ids)
    .execute(conn)
    .await?;
    Ok(())
}

/// Nummern von Belegen (für den Buchungstext).
pub async fn document_numbers(
    conn: &mut PgConnection,
    seite: &str,
    ids: &[i64],
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT coalesce(nummer, '') FROM {} WHERE id = ANY($1) ORDER BY id",
        beleg_table(seite)
    )))
    .bind(ids)
    .fetch_all(conn)
    .await
}

/// Anzeigename eines Partners (Buchungstext).
pub async fn open_partner_name(
    conn: &mut PgConnection,
    adresse_id: i64,
) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("SELECT anzeigename FROM adresse WHERE id = $1")
        .bind(adresse_id)
        .fetch_one(conn)
        .await
}

/// MWST-Konto eines Codes.
pub async fn code_accounts<'e>(exec: impl PgExecutor<'e>) -> Result<Vec<(i64, i64)>, sqlx::Error> {
    sqlx::query_as("SELECT id, konto_id FROM mwst_code")
        .fetch_all(exec)
        .await
}

/// Konto eines Bankkontos (nur aktive Bankkonten).
pub async fn bank_account_konto(
    conn: &mut PgConnection,
    id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT konto_id FROM bankkonto WHERE id = $1 AND aktiv")
        .bind(id)
        .fetch_optional(conn)
        .await
}

pub async fn bank_options(pool: &PgPool) -> Result<Vec<BankOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, bezeichnung, standard FROM bankkonto WHERE aktiv ORDER BY standard DESC, lower(bezeichnung)",
    )
    .fetch_all(pool)
    .await
}

/// Aktive Konten für „Differenz ausbuchen“ (AK-4).
pub async fn account_choices<'e>(
    exec: impl PgExecutor<'e>,
) -> Result<Vec<AccountChoice>, sqlx::Error> {
    sqlx::query_as("SELECT id, nummer, bezeichnung FROM konto WHERE aktiv ORDER BY nummer")
        .fetch_all(exec)
        .await
}

pub async fn active_currencies(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT code FROM waehrung WHERE aktiv ORDER BY buchhaltungswaehrung DESC, code",
    )
    .fetch_all(pool)
    .await
}

// ---------- Zahlung schreiben ----------

pub async fn insert_payment(
    conn: &mut PgConnection,
    v: &PaymentValues,
    kurs: i64,
    betrag_chf: i64,
    buchung_id: Option<i64>,
    erfasst_von: Option<i64>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO zahlung (art, seite, datum, adresse_id, bankkonto_id, betrag, waehrung, kurs, betrag_chf, \
                vorauszahlung, vorauszahlung_offen, text, buchung_id, erfasst_von) \
         VALUES ($1, $2, $3::date, $4, $5, $6::numeric, $7, $8::numeric / 1000000, $9::numeric, \
                 $10::numeric, $10::numeric, $11, $12, $13) RETURNING id",
    )
    .bind(&v.art)
    .bind(&v.seite)
    .bind(&v.datum)
    .bind(v.adresse_id)
    .bind(v.bankkonto_id)
    .bind(format_fixed(v.betrag, 2))
    .bind(&v.waehrung)
    .bind(kurs)
    .bind(format_fixed(betrag_chf, 2))
    .bind(format_fixed(v.vorauszahlung, 2))
    .bind(v.text.as_deref())
    .bind(buchung_id)
    .bind(erfasst_von)
    .fetch_one(conn)
    .await
}

/// Ausgleichszeile schreiben und den offenen Betrag des Belegs bzw. der Vorauszahlung verringern (AK-8).
pub async fn apply_allocation(
    conn: &mut PgConnection,
    zahlung_id: i64,
    seite: &str,
    a: &AllocationValues,
) -> Result<(), sqlx::Error> {
    let vorauszahlung = a.art == "vorauszahlung";
    sqlx::query(AssertSqlSafe(format!(
        "INSERT INTO zahlungsausgleich (zahlung_id, {}, betrag, skonto, differenz, differenz_konto_id) \
         VALUES ($1, $2, $3::numeric, $4::numeric, $5::numeric, $6)",
        if vorauszahlung {
            "vorauszahlung_id"
        } else {
            beleg_column(seite)
        }
    )))
    .bind(zahlung_id)
    .bind(a.beleg_id)
    .bind(format_fixed(a.betrag, 2))
    .bind(format_fixed(a.skonto, 2))
    .bind(format_fixed(a.differenz, 2))
    .bind(a.differenz_konto_id)
    .execute(&mut *conn)
    .await?;
    change_open(conn, seite, &a.art, a.beleg_id, -a.cleared()).await
}

/// Offenen Betrag ändern (`delta` negativ = ausgleichen) und den Status neu bestimmen:
/// 0 → „bezahlt“, Gesamtbetrag → „offen“, dazwischen „teilweise bezahlt“.
pub async fn change_open(
    conn: &mut PgConnection,
    seite: &str,
    art: &str,
    beleg_id: i64,
    delta: i64,
) -> Result<(), sqlx::Error> {
    if art == "vorauszahlung" {
        sqlx::query(
            "UPDATE zahlung SET vorauszahlung_offen = vorauszahlung_offen + $2::numeric / 100, \
                    geaendert_am = now() WHERE id = $1",
        )
        .bind(beleg_id)
        .bind(delta)
        .execute(conn)
        .await?;
        return Ok(());
    }
    sqlx::query(AssertSqlSafe(format!(
        "UPDATE {} SET offener_betrag = offener_betrag + $2::numeric / 100, \
                status = CASE WHEN offener_betrag + $2::numeric / 100 = 0 THEN 'bezahlt' \
                              WHEN offener_betrag + $2::numeric / 100 = gesamtbetrag THEN 'offen' \
                              ELSE 'teilweise_bezahlt' END, \
                geaendert_am = now() \
         WHERE id = $1",
        beleg_table(seite)
    )))
    .bind(beleg_id)
    .bind(delta)
    .execute(conn)
    .await?;
    Ok(())
}

/// Ausgleichszeilen einer Zahlung (für Storno und Detailansicht) als Werte.
pub async fn allocations_of(
    conn: &mut PgConnection,
    zahlung_id: i64,
) -> Result<Vec<AllocationValues>, sqlx::Error> {
    let rows: Vec<(String, i64, i64, i64, i64, Option<i64>)> = sqlx::query_as(
        "SELECT CASE WHEN x.vorauszahlung_id IS NOT NULL THEN 'vorauszahlung' \
                     ELSE coalesce(d.art, k.art) END, \
                coalesce(x.debitorenrechnung_id, x.kreditorenrechnung_id, x.vorauszahlung_id), \
                ROUND(x.betrag * 100)::int8, ROUND(x.skonto * 100)::int8, ROUND(x.differenz * 100)::int8, \
                x.differenz_konto_id \
         FROM zahlungsausgleich x \
         LEFT JOIN debitorenrechnung d ON d.id = x.debitorenrechnung_id \
         LEFT JOIN kreditorenrechnung k ON k.id = x.kreditorenrechnung_id \
         WHERE x.zahlung_id = $1 ORDER BY x.id",
    )
    .bind(zahlung_id)
    .fetch_all(conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(art, beleg_id, betrag, skonto, differenz, differenz_konto_id)| AllocationValues {
                art,
                beleg_id,
                betrag,
                skonto,
                differenz,
                differenz_konto_id,
            },
        )
        .collect())
}

/// Zahlung sperren: (Status, Seite).
pub async fn lock_payment(
    conn: &mut PgConnection,
    id: i64,
) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as("SELECT status, seite FROM zahlung WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(conn)
        .await
}

pub async fn mark_cancelled(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE zahlung SET status = 'storniert', geaendert_am = now() WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

// ---------- Lesen ----------

const SELECT_PAYMENT: &str = "\
    SELECT z.id, z.art, z.seite, z.datum::text AS datum, z.adresse_id, a.anzeigename AS partner, \
           b.bezeichnung AS bankkonto, z.waehrung, ROUND(z.kurs * 1000000)::int8 AS kurs, \
           ROUND(z.betrag * 100)::int8 AS betrag, ROUND(z.betrag_chf * 100)::int8 AS betrag_chf, \
           ROUND(z.vorauszahlung * 100)::int8 AS vorauszahlung, \
           ROUND(z.vorauszahlung_offen * 100)::int8 AS vorauszahlung_offen, z.status, z.text, z.buchung_id, \
           (SELECT g.id FROM buchung g WHERE g.storno_von_id = z.buchung_id) AS storno_buchung_id, \
           COALESCE(NULLIF(btrim(concat_ws(' ', ua.vorname, ua.nachname)), ''), bu.benutzername) AS erfasst_von, \
           to_char(z.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am \
    FROM zahlung z JOIN adresse a ON a.id = z.adresse_id \
    LEFT JOIN bankkonto b ON b.id = z.bankkonto_id \
    LEFT JOIN benutzer bu ON bu.id = z.erfasst_von LEFT JOIN adresse ua ON ua.id = bu.adresse_id";

pub async fn find_payment<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<Payment>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!("{SELECT_PAYMENT} WHERE z.id = $1")))
        .bind(id)
        .fetch_optional(exec)
        .await
}

const SELECT_ALLOCATION: &str = "\
    SELECT z.id AS zahlung_id, z.art AS zahlung_art, z.datum::text AS zahlung_datum, z.status AS zahlung_status, \
           CASE WHEN x.vorauszahlung_id IS NOT NULL THEN 'vorauszahlung' ELSE coalesce(d.art, k.art) END AS art, \
           coalesce(x.debitorenrechnung_id, x.kreditorenrechnung_id, x.vorauszahlung_id) AS beleg_id, \
           coalesce(d.nummer, k.nummer, 'Vorauszahlung Z-' || x.vorauszahlung_id, '') AS nummer, z.seite, \
           ROUND(x.betrag * 100)::int8 AS betrag, ROUND(x.skonto * 100)::int8 AS skonto, \
           ROUND(x.differenz * 100)::int8 AS differenz, kd.nummer || ' · ' || kd.bezeichnung AS differenz_konto \
    FROM zahlungsausgleich x JOIN zahlung z ON z.id = x.zahlung_id \
    LEFT JOIN debitorenrechnung d ON d.id = x.debitorenrechnung_id \
    LEFT JOIN kreditorenrechnung k ON k.id = x.kreditorenrechnung_id \
    LEFT JOIN konto kd ON kd.id = x.differenz_konto_id";

pub async fn allocation_views(
    pool: &PgPool,
    zahlung_id: i64,
) -> Result<Vec<AllocationView>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_ALLOCATION} WHERE x.zahlung_id = $1 ORDER BY x.id"
    )))
    .bind(zahlung_id)
    .fetch_all(pool)
    .await
}

/// Zahlungen und Verrechnungen eines Belegs bzw. einer Vorauszahlung (Detailansichten).
pub async fn allocations_of_document(
    pool: &PgPool,
    seite: &str,
    beleg_id: i64,
) -> Result<Vec<AllocationView>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_ALLOCATION} WHERE x.{} = $1 ORDER BY z.datum, x.id",
        beleg_column(seite)
    )))
    .bind(beleg_id)
    .fetch_all(pool)
    .await
}

/// Verrechnungen, die eine Vorauszahlung aufbrauchen.
pub async fn allocations_of_prepayment(
    pool: &PgPool,
    zahlung_id: i64,
) -> Result<Vec<AllocationView>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_ALLOCATION} WHERE x.vorauszahlung_id = $1 ORDER BY z.datum, x.id"
    )))
    .bind(zahlung_id)
    .fetch_all(pool)
    .await
}

/// Filter der Liste (AK-12). $1 Suche, $2 Muster, $3 Art, $4 von, $5 bis, $6 Bankkonto.
const LIST_WHERE: &str = "($1 = '' OR a.anzeigename ILIKE $2 OR a.nummer ILIKE $2 OR z.text ILIKE $2 \
         OR EXISTS (SELECT 1 FROM zahlungsausgleich x \
              LEFT JOIN debitorenrechnung d ON d.id = x.debitorenrechnung_id \
              LEFT JOIN kreditorenrechnung k ON k.id = x.kreditorenrechnung_id \
              WHERE x.zahlung_id = z.id AND (d.nummer ILIKE $2 OR k.nummer ILIKE $2 OR k.rechnungsnummer ILIKE $2))) \
     AND ($3 = '' OR z.art = $3) \
     AND ($4::date IS NULL OR z.datum >= $4::date) \
     AND ($5::date IS NULL OR z.datum <= $5::date) \
     AND ($6 = 0 OR z.bankkonto_id = $6)";

const LIST_FROM: &str = "zahlung z JOIN adresse a ON a.id = z.adresse_id LEFT JOIN bankkonto b ON b.id = z.bankkonto_id";

fn order_by(sort: &str) -> &'static str {
    match sort {
        "art" => "z.art",
        "partner" => "lower(a.anzeigename)",
        "bankkonto" => "lower(b.bezeichnung)",
        "betrag" => "z.betrag_chf",
        "status" => "z.status",
        _ => "z.datum",
    }
}

pub async fn count(pool: &PgPool, f: &PaymentFilter, pattern: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM {LIST_FROM} WHERE {LIST_WHERE}"
    )))
    .bind(&f.suche)
    .bind(pattern)
    .bind(&f.art)
    .bind(opt(&f.von))
    .bind(opt(&f.bis))
    .bind(f.bankkonto_id)
    .fetch_one(pool)
    .await
}

pub async fn page(
    pool: &PgPool,
    f: &PaymentFilter,
    pattern: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<PaymentRow>, sqlx::Error> {
    let direction = if f.richtung == "ab" { "DESC" } else { "ASC" };
    // Geprüft: Sortierung aus fester Liste, Richtung ASC/DESC
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT z.id, z.art, z.seite, z.datum::text AS datum, a.anzeigename AS partner, \
                b.bezeichnung AS bankkonto, z.waehrung, ROUND(z.betrag * 100)::int8 AS betrag, \
                ROUND(z.vorauszahlung_offen * 100)::int8 AS vorauszahlung_offen, \
                (SELECT string_agg(coalesce(d.nummer, k.nummer, 'Z-' || x.vorauszahlung_id), ', ' ORDER BY x.id) \
                 FROM zahlungsausgleich x \
                 LEFT JOIN debitorenrechnung d ON d.id = x.debitorenrechnung_id \
                 LEFT JOIN kreditorenrechnung k ON k.id = x.kreditorenrechnung_id \
                 WHERE x.zahlung_id = z.id) AS belege, z.status \
         FROM {LIST_FROM} WHERE {LIST_WHERE} \
         ORDER BY {} {direction}, z.id DESC LIMIT $7 OFFSET $8",
        order_by(&f.sortierung)
    )))
    .bind(&f.suche)
    .bind(pattern)
    .bind(&f.art)
    .bind(opt(&f.von))
    .bind(opt(&f.bis))
    .bind(f.bankkonto_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Offene Posten zum Stichtag (AK-13), in CHF zum Buchungskurs: Belege bis zum Stichtag, abzüglich der
/// Ausgleiche gültiger Zahlungen bis zum Stichtag; Gutschriften und Vorauszahlungen negativ. Stornierte
/// Belege zählen nicht.
pub async fn aging_items(pool: &PgPool, stichtag: &str) -> Result<Vec<AgingItem>, sqlx::Error> {
    sqlx::query_as(
        "WITH ausgleich AS ( \
             SELECT x.debitorenrechnung_id, x.kreditorenrechnung_id, x.vorauszahlung_id, \
                    x.betrag + x.skonto + x.differenz AS summe \
             FROM zahlungsausgleich x JOIN zahlung z ON z.id = x.zahlung_id \
             WHERE z.status = 'gebucht' AND z.datum <= $1::date), \
         belege AS ( \
             SELECT 'debitoren' AS seite, r.adresse_id, r.art, r.faelligkeit, r.kurs, \
                    r.gesamtbetrag - coalesce((SELECT sum(summe) FROM ausgleich WHERE debitorenrechnung_id = r.id), 0) AS offen \
             FROM debitorenrechnung r \
             WHERE r.status NOT IN ('entwurf', 'storniert') AND r.rechnungsdatum <= $1::date \
             UNION ALL \
             SELECT 'kreditoren', r.adresse_id, r.art, r.faelligkeit, r.kurs, \
                    r.gesamtbetrag - coalesce((SELECT sum(summe) FROM ausgleich WHERE kreditorenrechnung_id = r.id), 0) \
             FROM kreditorenrechnung r \
             WHERE r.status NOT IN ('entwurf', 'storniert') AND r.rechnungsdatum <= $1::date \
             UNION ALL \
             SELECT z.seite, z.adresse_id, 'vorauszahlung', NULL, z.kurs, \
                    z.vorauszahlung - coalesce((SELECT sum(summe) FROM ausgleich WHERE vorauszahlung_id = z.id), 0) \
             FROM zahlung z WHERE z.status = 'gebucht' AND z.vorauszahlung > 0 AND z.datum <= $1::date) \
         SELECT b.seite, b.adresse_id, a.anzeigename AS partner, \
                ($1::date - b.faelligkeit)::int4 AS tage, \
                ROUND(CASE WHEN b.art = 'rechnung' THEN 1 ELSE -1 END * b.offen * b.kurs * 100)::int8 AS offen_chf \
         FROM belege b JOIN adresse a ON a.id = b.adresse_id \
         WHERE b.offen <> 0",
    )
    .bind(stichtag)
    .fetch_all(pool)
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_tables_only() {
        assert_eq!(beleg_table("kreditoren"), "kreditorenrechnung");
        assert_eq!(beleg_table("x; DROP"), "debitorenrechnung");
        assert_eq!(order_by("x; DROP"), "z.datum");
    }
}
