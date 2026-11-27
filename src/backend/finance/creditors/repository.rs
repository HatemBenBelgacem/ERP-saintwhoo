//! SQL-Zugriffe für Kreditoren (ANF-019, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Statusprüfungen, keine Weiterleitungen.
//! Zahlungsbedingungen und Systemkonten liest `debtors::repository` (gleiche Abfragen).

use sqlx::{AssertSqlSafe, PgConnection, PgExecutor, PgPool};

use super::model::{CreditorFilter, CreditorInput, CreditorInvoice, CreditorPosition, CreditorRow};
use crate::backend::finance::journal::model::{AccountOption, CodeOption};
use crate::backend::purchase_orders::model::format_fixed;
use crate::forms::PickOption;

const TODAY: &str = "(now() AT TIME ZONE 'Europe/Zurich')::date";

/// Tage seit Fälligkeit bei offenen, überfälligen Belegen (AK-3).
const OVERDUE_DAYS: &str = "CASE WHEN r.status IN ('offen', 'teilweise_bezahlt') \
         AND r.faelligkeit < (now() AT TIME ZONE 'Europe/Zurich')::date \
     THEN ((now() AT TIME ZONE 'Europe/Zurich')::date - r.faelligkeit)::int4 END";

/// Skontofrist läuft noch (AK-3).
const DISCOUNT_RUNNING: &str = "coalesce(r.status IN ('offen', 'teilweise_bezahlt') \
     AND r.skonto_bis >= (now() AT TIME ZONE 'Europe/Zurich')::date, false)";

/// Gesamtbetrag: festgeschrieben oder (im Entwurf) Summe der Positionen plus Rundung, in Rappen.
const AMOUNT: &str = "ROUND(coalesce(r.gesamtbetrag, \
     (SELECT coalesce(sum(x.netto + x.mwst_betrag), 0) FROM kreditorenposition x \
      WHERE x.kreditorenrechnung_id = r.id) + r.rundung) * 100)::int8";

const FROM: &str = "kreditorenrechnung r JOIN adresse a ON a.id = r.adresse_id \
     LEFT JOIN projekt p ON p.id = r.projekt_id \
     LEFT JOIN einkaufsauftrag e ON e.id = r.einkaufsauftrag_id";

const ROW_COLUMNS: &str = "r.id, r.nummer, r.art, r.rechnungsnummer, a.anzeigename AS lieferant, \
     r.einkaufsauftrag_id, e.nummer AS einkaufsauftrag_nummer, r.projekt_id, p.nummer AS projekt_nummer, \
     r.status, r.rechnungsdatum::text AS rechnungsdatum, r.faelligkeit::text AS faelligkeit, \
     r.skonto_bis::text AS skonto_bis, r.waehrung, \
     ROUND(r.offener_betrag * 100)::int8 AS offener_betrag";

/// Filter (AK-2). $1 Suchtext, $2 Muster, $3 Status, $4 Von, $5 Bis, $6 fällig bis.
const LIST_WHERE: &str = "($1 = '' OR r.nummer ILIKE $2 OR r.rechnungsnummer ILIKE $2 \
         OR a.anzeigename ILIKE $2 OR a.nummer ILIKE $2 OR e.nummer ILIKE $2 OR p.nummer ILIKE $2 \
         OR p.name ILIKE $2 OR r.zahlungsreferenz ILIKE $2) \
     AND ($3 = 'alle' OR ($3 = 'offen' AND r.status IN ('offen', 'teilweise_bezahlt')) OR r.status = $3) \
     AND ($4::date IS NULL OR r.rechnungsdatum >= $4::date) \
     AND ($5::date IS NULL OR r.rechnungsdatum <= $5::date) \
     AND ($6::date IS NULL OR r.faelligkeit <= $6::date)";

fn order_by(sort: &str) -> String {
    match sort {
        "nummer" => "r.nummer".into(),
        "rechnungsnummer" => "lower(r.rechnungsnummer)".into(),
        "lieferant" => "lower(a.anzeigename)".into(),
        "auftrag" => "e.nummer".into(),
        "projekt" => "p.nummer".into(),
        "rechnungsdatum" => "r.rechnungsdatum".into(),
        "skonto" => "r.skonto_bis".into(),
        "betrag" => AMOUNT.into(),
        "offen" => "r.offener_betrag".into(),
        "status" => {
            "array_position(ARRAY['entwurf','offen','teilweise_bezahlt','bezahlt','storniert'], \
                     r.status::text)"
                .into()
        }
        _ => "r.faelligkeit".into(),
    }
}

fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

fn select_rows() -> String {
    format!(
        "SELECT {ROW_COLUMNS}, {AMOUNT} AS gesamtbetrag, {OVERDUE_DAYS} AS tage_ueberfaellig, \
         {DISCOUNT_RUNNING} AS skonto_laeuft FROM {FROM}"
    )
}

// ---------- Liste ----------

pub async fn count(pool: &PgPool, f: &CreditorFilter, pattern: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM {FROM} WHERE {LIST_WHERE}"
    )))
    .bind(&f.suche)
    .bind(pattern)
    .bind(&f.status)
    .bind(opt(&f.von))
    .bind(opt(&f.bis))
    .bind(opt(&f.faellig_bis))
    .fetch_one(pool)
    .await
}

pub async fn page(
    pool: &PgPool,
    f: &CreditorFilter,
    pattern: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<CreditorRow>, sqlx::Error> {
    let direction = if f.richtung == "ab" { "DESC" } else { "ASC" };
    // Geprüft: Sortierung aus fester Liste (order_by), Richtung ist ASC/DESC.
    sqlx::query_as(AssertSqlSafe(format!(
        "{} WHERE {LIST_WHERE} ORDER BY {} {direction} NULLS LAST, r.id DESC LIMIT $7 OFFSET $8",
        select_rows(),
        order_by(&f.sortierung)
    )))
    .bind(&f.suche)
    .bind(pattern)
    .bind(&f.status)
    .bind(opt(&f.von))
    .bind(opt(&f.bis))
    .bind(opt(&f.faellig_bis))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Eingangsrechnungen eines Projekts (Kopf oder Position, AK-15).
pub async fn by_project(pool: &PgPool, projekt_id: i64) -> Result<Vec<CreditorRow>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{} WHERE r.projekt_id = $1 OR EXISTS (SELECT 1 FROM kreditorenposition x \
             WHERE x.kreditorenrechnung_id = r.id AND x.projekt_id = $1) \
         ORDER BY r.rechnungsdatum DESC, r.id DESC",
        select_rows()
    )))
    .bind(projekt_id)
    .fetch_all(pool)
    .await
}

/// Offene Kreditoren eines Lieferanten (AK-15).
pub async fn open_of_supplier(
    pool: &PgPool,
    adresse_id: i64,
) -> Result<Vec<CreditorRow>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{} WHERE r.adresse_id = $1 AND r.status IN ('offen', 'teilweise_bezahlt') \
         ORDER BY r.faelligkeit, r.id",
        select_rows()
    )))
    .bind(adresse_id)
    .fetch_all(pool)
    .await
}

/// Eingangsrechnungen eines Einkaufsauftrags (AK-9).
pub async fn by_order(pool: &PgPool, auftrag_id: i64) -> Result<Vec<CreditorRow>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{} WHERE r.einkaufsauftrag_id = $1 ORDER BY r.rechnungsdatum, r.id",
        select_rows()
    )))
    .bind(auftrag_id)
    .fetch_all(pool)
    .await
}

/// Nummern der Eingangsrechnungen eines Projekts (Löschsperre, AK-15).
pub async fn numbers_of_project(
    pool: &PgPool,
    projekt_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT coalesce(r.nummer, 'Entwurf') FROM kreditorenrechnung r \
         WHERE r.projekt_id = $1 OR EXISTS (SELECT 1 FROM kreditorenposition x \
             WHERE x.kreditorenrechnung_id = r.id AND x.projekt_id = $1) ORDER BY r.id",
    )
    .bind(projekt_id)
    .fetch_all(pool)
    .await
}

// ---------- Ein Beleg ----------

const POSITION_COLUMNS: &str = "x.id, x.bezeichnung, x.konto_id, k.nummer AS konto_nummer, \
     k.bezeichnung AS konto_bezeichnung, x.mwst_code_id, m.code AS mwst_code, \
     (m.satz * 100)::int8 AS mwst_satz, ROUND(x.netto * 100)::int8 AS netto, \
     ROUND(x.mwst_betrag * 100)::int8 AS mwst_betrag, x.projekt_id, pp.nummer AS projekt_nummer, \
     x.einkaufsposition_id";

const POSITION_FROM: &str = "FROM kreditorenposition x JOIN konto k ON k.id = x.konto_id \
     LEFT JOIN mwst_code m ON m.id = x.mwst_code_id LEFT JOIN projekt pp ON pp.id = x.projekt_id";

pub async fn positions<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Vec<CreditorPosition>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {POSITION_COLUMNS} {POSITION_FROM} \
         WHERE x.kreditorenrechnung_id = $1 ORDER BY x.reihenfolge, x.id"
    )))
    .bind(id)
    .fetch_all(exec)
    .await
}

pub async fn find<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<CreditorInvoice>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT r.id, r.nummer, r.art, r.adresse_id, a.nummer AS lieferant_nummer, a.anzeigename AS lieferant, \
                r.rechnungsnummer, r.einkaufsauftrag_id, e.nummer AS einkaufsauftrag_nummer, \
                r.projekt_id, p.nummer AS projekt_nummer, p.name AS projekt_name, r.status, \
                r.rechnungsdatum::text AS rechnungsdatum, r.zahlungsbedingung_id, \
                z.bezeichnung AS zahlungsbedingung, r.faelligkeit::text AS faelligkeit, \
                r.skonto_bis::text AS skonto_bis, (r.skonto_prozent * 100)::int8 AS skonto_prozent, \
                r.waehrung, ROUND(r.kurs * 1000000)::int8 AS kurs, \
                ROUND(r.betrag_gemaess_rechnung * 100)::int8 AS betrag_gemaess_rechnung, \
                ROUND(r.rundung * 100)::int8 AS rundung, ROUND(r.gesamtbetrag * 100)::int8 AS gesamtbetrag, \
                ROUND(r.gesamtbetrag_chf * 100)::int8 AS gesamtbetrag_chf, \
                ROUND(r.offener_betrag * 100)::int8 AS offener_betrag, r.zahlungsreferenz, r.bemerkung, \
                r.buchung_id, (SELECT g.id FROM buchung g WHERE g.storno_von_id = r.buchung_id) AS storno_buchung_id, \
                {OVERDUE_DAYS} AS tage_ueberfaellig, {DISCOUNT_RUNNING} AS skonto_laeuft, \
                to_char(r.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                to_char(r.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
         FROM {FROM} JOIN zahlungsbedingung z ON z.id = r.zahlungsbedingung_id \
         WHERE r.id = $1"
    )))
    .bind(id)
    .fetch_optional(exec)
    .await
}

pub async fn lock(conn: &mut PgConnection, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT status FROM kreditorenrechnung WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(conn)
        .await
}

pub async fn lock_of_position(
    conn: &mut PgConnection,
    position_id: i64,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id, r.status FROM kreditorenposition x \
         JOIN kreditorenrechnung r ON r.id = x.kreditorenrechnung_id \
         WHERE x.id = $1 FOR UPDATE OF r",
    )
    .bind(position_id)
    .fetch_optional(conn)
    .await
}

/// Gibt es eine andere, nicht stornierte Rechnung desselben Lieferanten mit dieser Nummer (AK-5)?
pub async fn duplicate_exists(
    conn: &mut PgConnection,
    adresse_id: i64,
    rechnungsnummer: &str,
    ausser: Option<i64>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM kreditorenrechnung WHERE adresse_id = $1 \
             AND lower(rechnungsnummer) = lower($2) AND status <> 'storniert' \
             AND ($3::bigint IS NULL OR id <> $3))",
    )
    .bind(adresse_id)
    .bind(rechnungsnummer)
    .bind(ausser)
    .fetch_one(conn)
    .await
}

// ---------- Stammdaten ----------

/// Aktive Aufwandskonten (AK-6).
pub async fn expense_accounts<'e>(
    exec: impl PgExecutor<'e>,
) -> Result<Vec<AccountOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, bezeichnung, mwst_code_id AS vorgabe_mwst FROM konto \
         WHERE aktiv AND kontoart = 'aufwand' ORDER BY nummer",
    )
    .fetch_all(exec)
    .await
}

/// Aktive Vorsteuer-Codes (die Gültigkeit am Datum prüft `model`).
pub async fn input_tax_codes<'e>(
    exec: impl PgExecutor<'e>,
) -> Result<Vec<CodeOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, code, bezeichnung, ROUND(satz * 100)::BIGINT AS satz, \
                gueltig_ab::text AS gueltig_ab, gueltig_bis::text AS gueltig_bis, konto_id \
         FROM mwst_code WHERE aktiv AND art = 'vorsteuer' ORDER BY satz DESC, code",
    )
    .fetch_all(exec)
    .await
}

/// Einkaufsaufträge für die Auswahl: nicht im Entwurf und nicht storniert, neueste zuerst.
pub async fn search_orders(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
) -> Result<Vec<PickOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT e.id, e.nummer, a.anzeigename AS name FROM einkaufsauftrag e \
         JOIN adresse a ON a.id = e.adresse_id \
         WHERE e.status IN ('bestellt', 'teilweise_geliefert', 'geliefert') \
           AND ($1 = '' OR e.nummer ILIKE $2 OR a.anzeigename ILIKE $2) \
         ORDER BY e.id DESC LIMIT 20",
    )
    .bind(suche)
    .bind(pattern)
    .fetch_all(pool)
    .await
}

/// Lieferant, Projekt, Währung und Status eines Einkaufsauftrags.
pub async fn order_head(
    conn: &mut PgConnection,
    auftrag_id: i64,
) -> Result<Option<(i64, Option<i64>, String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT adresse_id, projekt_id, waehrung, status FROM einkaufsauftrag WHERE id = $1",
    )
    .bind(auftrag_id)
    .fetch_optional(conn)
    .await
}

/// Positionen eines Einkaufsauftrags für die Übernahme (AK-8): Kennung, Bezeichnung, verrechenbarer Betrag
/// (gelieferte Menge × Einzelpreis abzüglich Rabatt, in Rappen), MWST-Code und -Satz.
#[allow(clippy::type_complexity)]
pub async fn order_positions(
    conn: &mut PgConnection,
    auftrag_id: i64,
) -> Result<Vec<(i64, String, i64, Option<i64>, i64)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, bezeichnung, \
                ROUND(round(gelieferte_menge * einzelpreis * (1 - rabatt_prozent / 100), 2) * 100)::int8, \
                mwst_code_id, (mwst_satz * 100)::int8 \
         FROM einkaufsposition WHERE einkaufsauftrag_id = $1 ORDER BY reihenfolge, id",
    )
    .bind(auftrag_id)
    .fetch_all(conn)
    .await
}

// ---------- Kopf schreiben ----------

pub async fn insert(
    conn: &mut PgConnection,
    input: &CreditorInput,
    zahlungsbedingung_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO kreditorenrechnung (art, adresse_id, rechnungsnummer, einkaufsauftrag_id, projekt_id, \
                rechnungsdatum, zahlungsbedingung_id, faelligkeit, waehrung, betrag_gemaess_rechnung, \
                zahlungsreferenz, bemerkung) \
         VALUES ($1, $2, $3, $4, $5, $6::date, $7, $8::date, $9, $10::numeric / 100, $11, $12) RETURNING id",
    )
    .bind(&input.art)
    .bind(input.supplier_id())
    .bind(&input.rechnungsnummer)
    .bind(input.order_id())
    .bind(input.project_id())
    .bind(&input.rechnungsdatum)
    .bind(zahlungsbedingung_id)
    .bind(opt(&input.faelligkeit))
    .bind(&input.waehrung)
    .bind(input.control_amount())
    .bind(opt(&input.zahlungsreferenz))
    .bind(opt(&input.bemerkung))
    .fetch_one(conn)
    .await
}

pub async fn update(
    conn: &mut PgConnection,
    id: i64,
    input: &CreditorInput,
    zahlungsbedingung_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE kreditorenrechnung SET adresse_id = $2, rechnungsnummer = $3, einkaufsauftrag_id = $4, \
                projekt_id = $5, rechnungsdatum = $6::date, zahlungsbedingung_id = $7, faelligkeit = $8::date, \
                waehrung = $9, betrag_gemaess_rechnung = $10::numeric / 100, zahlungsreferenz = $11, \
                bemerkung = $12, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(input.supplier_id())
    .bind(&input.rechnungsnummer)
    .bind(input.order_id())
    .bind(input.project_id())
    .bind(&input.rechnungsdatum)
    .bind(zahlungsbedingung_id)
    .bind(opt(&input.faelligkeit))
    .bind(&input.waehrung)
    .bind(input.control_amount())
    .bind(opt(&input.zahlungsreferenz))
    .bind(opt(&input.bemerkung))
    .execute(conn)
    .await?;
    Ok(())
}

/// Entwurf aus einem Einkaufsauftrag (AK-8); Rechnungsnummer und Kontrollbetrag bleiben leer. Liefert die Kennung.
pub async fn insert_from_order(
    conn: &mut PgConnection,
    auftrag_id: i64,
    adresse_id: i64,
    projekt_id: Option<i64>,
    waehrung: &str,
    zahlungsbedingung_id: i64,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(AssertSqlSafe(format!(
        "INSERT INTO kreditorenrechnung (art, adresse_id, einkaufsauftrag_id, projekt_id, \
                rechnungsdatum, zahlungsbedingung_id, waehrung) \
         VALUES ('rechnung', $1, $2, $3, {TODAY}, $4, $5) RETURNING id"
    )))
    .bind(adresse_id)
    .bind(auftrag_id)
    .bind(projekt_id)
    .bind(zahlungsbedingung_id)
    .bind(waehrung)
    .fetch_one(conn)
    .await
}

pub async fn touch(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE kreditorenrechnung SET geaendert_am = now() WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Ausgeglichene Rundung setzen (AK-7), in Rappen.
pub async fn set_rounding(
    conn: &mut PgConnection,
    id: i64,
    rundung: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE kreditorenrechnung SET rundung = $2::numeric / 100, geaendert_am = now() \
         WHERE id = $1 AND status = 'entwurf'",
    )
    .bind(id)
    .bind(rundung)
    .execute(conn)
    .await?;
    Ok(())
}

/// Nächste interne Laufnummer des Jahres (AK-10).
pub async fn next_number(conn: &mut PgConnection, jahr: i32) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO kreditorenrechnung_nummer (jahr, letzte) VALUES ($1, 1) \
         ON CONFLICT (jahr) DO UPDATE SET letzte = kreditorenrechnung_nummer.letzte + 1 \
         RETURNING letzte",
    )
    .bind(jahr)
    .fetch_one(conn)
    .await
}

/// Werte, die beim Buchen festgeschrieben werden (AK-10).
pub struct BookedValues {
    pub nummer: String,
    pub faelligkeit: String,
    pub skonto: Option<(String, i64)>,
    pub kurs: i64,
    pub gesamtbetrag: i64,
    pub gesamtbetrag_chf: i64,
    pub buchung_id: i64,
}

pub async fn mark_booked(
    conn: &mut PgConnection,
    id: i64,
    v: &BookedValues,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE kreditorenrechnung SET status = 'offen', nummer = $2, faelligkeit = $3::date, \
                skonto_bis = $4::date, skonto_prozent = $5::numeric / 100, kurs = $6::numeric / 1000000, \
                gesamtbetrag = $7::numeric, gesamtbetrag_chf = $8::numeric, offener_betrag = $7::numeric, \
                buchung_id = $9, geaendert_am = now() \
         WHERE id = $1 AND status = 'entwurf'",
    )
    .bind(id)
    .bind(&v.nummer)
    .bind(&v.faelligkeit)
    .bind(v.skonto.as_ref().map(|(d, _)| d.as_str()))
    .bind(v.skonto.as_ref().map(|(_, p)| *p))
    .bind(v.kurs)
    .bind(format_fixed(v.gesamtbetrag, 2))
    .bind(format_fixed(v.gesamtbetrag_chf, 2))
    .bind(v.buchung_id)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

pub async fn mark_cancelled(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE kreditorenrechnung SET status = 'storniert', offener_betrag = 0, geaendert_am = now() \
         WHERE id = $1 AND status = 'offen'",
    )
    .bind(id)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

pub async fn delete_draft(conn: &mut PgConnection, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "DELETE FROM kreditorenrechnung WHERE id = $1 AND status = 'entwurf' RETURNING art",
    )
    .bind(id)
    .fetch_optional(conn)
    .await
}

// ---------- Positionen schreiben ----------

/// Werte einer Position in Rappen.
pub struct PositionWrite<'a> {
    pub bezeichnung: &'a str,
    pub konto_id: i64,
    pub mwst_code_id: Option<i64>,
    pub netto: i64,
    pub mwst_betrag: i64,
    pub projekt_id: Option<i64>,
    pub einkaufsposition_id: Option<i64>,
}

pub async fn insert_position(
    conn: &mut PgConnection,
    rechnung_id: i64,
    p: &PositionWrite<'_>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO kreditorenposition (kreditorenrechnung_id, reihenfolge, bezeichnung, konto_id, \
                mwst_code_id, netto, mwst_betrag, projekt_id, einkaufsposition_id) \
         VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM kreditorenposition \
                      WHERE kreditorenrechnung_id = $1), \
                 $2, $3, $4, $5::numeric / 100, $6::numeric / 100, $7, $8) RETURNING id",
    )
    .bind(rechnung_id)
    .bind(p.bezeichnung)
    .bind(p.konto_id)
    .bind(p.mwst_code_id)
    .bind(p.netto)
    .bind(p.mwst_betrag)
    .bind(p.projekt_id)
    .bind(p.einkaufsposition_id)
    .fetch_one(conn)
    .await
}

pub async fn update_position(
    conn: &mut PgConnection,
    id: i64,
    p: &PositionWrite<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE kreditorenposition SET bezeichnung = $2, konto_id = $3, mwst_code_id = $4, \
                netto = $5::numeric / 100, mwst_betrag = $6::numeric / 100, projekt_id = $7, \
                geaendert_am = now() WHERE id = $1",
    )
    .bind(id)
    .bind(p.bezeichnung)
    .bind(p.konto_id)
    .bind(p.mwst_code_id)
    .bind(p.netto)
    .bind(p.mwst_betrag)
    .bind(p.projekt_id)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete_position(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM kreditorenposition WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

pub async fn move_position(
    conn: &mut PgConnection,
    rechnung_id: i64,
    id: i64,
    nach_oben: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE kreditorenposition t SET reihenfolge = r.rn \
         FROM (SELECT id, row_number() OVER (ORDER BY reihenfolge, id)::int AS rn \
               FROM kreditorenposition WHERE kreditorenrechnung_id = $1) r \
         WHERE t.id = r.id",
    )
    .bind(rechnung_id)
    .execute(&mut *conn)
    .await?;
    let pos: i32 = sqlx::query_scalar("SELECT reihenfolge FROM kreditorenposition WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *conn)
        .await?;
    let other = if nach_oben { pos - 1 } else { pos + 1 };
    let swapped = sqlx::query(
        "UPDATE kreditorenposition SET reihenfolge = $3 WHERE kreditorenrechnung_id = $1 AND reihenfolge = $2",
    )
    .bind(rechnung_id)
    .bind(other)
    .bind(pos)
    .execute(&mut *conn)
    .await?;
    if swapped.rows_affected() == 1 {
        sqlx::query("UPDATE kreditorenposition SET reihenfolge = $2 WHERE id = $1")
            .bind(id)
            .bind(other)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_sort_falls_back_to_due_date() {
        assert_eq!(
            order_by("1; DROP TABLE kreditorenrechnung"),
            "r.faelligkeit"
        );
        assert_eq!(order_by("skonto"), "r.skonto_bis");
    }
}
