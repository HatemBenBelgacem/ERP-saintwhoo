//! SQL-Zugriffe für Debitoren (ANF-018, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Statusprüfungen, keine Weiterleitungen.
//! Funktionen mit `&mut PgConnection` laufen in der Transaktion des Aufrufers.

use sqlx::{AssertSqlSafe, PgConnection, PgExecutor, PgPool};

use super::model::{
    CustomerAddress, Invoice, InvoiceFilter, InvoiceInput, InvoicePosition, InvoiceRow,
    PositionInput, PositionValues, PrintBank, TermOption,
};
use crate::backend::finance::journal::model::{AccountOption, CodeOption};
use crate::backend::purchase_orders::model::format_fixed;
use crate::forms::PickOption;

/// Heutiges Datum in der Schweiz.
const TODAY: &str = "(now() AT TIME ZONE 'Europe/Zurich')::date";

/// Tage seit Fälligkeit bei offenen, überfälligen Belegen (AK-3), sonst NULL.
const OVERDUE_DAYS: &str = "CASE WHEN r.status IN ('offen', 'teilweise_bezahlt') \
         AND r.faelligkeit < (now() AT TIME ZONE 'Europe/Zurich')::date \
     THEN ((now() AT TIME ZONE 'Europe/Zurich')::date - r.faelligkeit)::int4 END";

const FROM: &str = "debitorenrechnung r JOIN adresse a ON a.id = r.adresse_id \
     LEFT JOIN projekt p ON p.id = r.projekt_id";

/// Gesamtbetrag einer Zeile: festgeschrieben oder (im Entwurf) aus den Positionen, ohne Rundung.
/// Die genaue Rechnung macht `model::totals`; hier dient er nur der Sortierung.
const AMOUNT_SORT: &str = "coalesce(r.gesamtbetrag, (SELECT coalesce(sum(round(x.menge * x.einzelpreis \
         * (1 - x.rabatt_prozent / 100), 2) * (1 + coalesce(m.satz, 0) / 100)), 0) \
       FROM debitorenposition x LEFT JOIN mwst_code m ON m.id = x.mwst_code_id \
      WHERE x.debitorenrechnung_id = r.id))";

/// Spalten einer Listenzeile.
const ROW_COLUMNS: &str = "r.id, r.nummer, r.art, a.anzeigename AS kunde, r.projekt_id, \
     p.nummer AS projekt_nummer, r.status, r.rechnungsdatum::text AS rechnungsdatum, \
     r.faelligkeit::text AS faelligkeit, r.waehrung, \
     ROUND(r.gesamtbetrag * 100)::int8 AS gesamtbetrag, ROUND(r.offener_betrag * 100)::int8 AS offener_betrag";

/// Filter der Liste (AK-2). $1 Suchtext, $2 Muster, $3 Status, $4 Von, $5 Bis.
const LIST_WHERE: &str = "($1 = '' OR r.nummer ILIKE $2 OR a.anzeigename ILIKE $2 OR a.nummer ILIKE $2 \
         OR p.nummer ILIKE $2 OR p.name ILIKE $2 OR r.referenz_kunde ILIKE $2) \
     AND ($3 = 'alle' OR ($3 = 'offen' AND r.status IN ('offen', 'teilweise_bezahlt')) OR r.status = $3) \
     AND ($4::date IS NULL OR r.rechnungsdatum >= $4::date) \
     AND ($5::date IS NULL OR r.rechnungsdatum <= $5::date)";

/// Sortierausdruck aus einer festen Liste (kein Text der Webadresse im SQL).
fn order_by(sort: &str) -> String {
    match sort {
        "nummer" => "r.nummer".into(),
        "kunde" => "lower(a.anzeigename)".into(),
        "projekt" => "p.nummer".into(),
        "rechnungsdatum" => "r.rechnungsdatum".into(),
        "betrag" => AMOUNT_SORT.into(),
        "offen" => "r.offener_betrag".into(),
        "status" => {
            "array_position(ARRAY['entwurf','offen','teilweise_bezahlt','bezahlt','storniert'], \
                     r.status::text)"
                .into()
        }
        _ => "r.faelligkeit".into(),
    }
}

/// Leerer Text → NULL in der Datenbank.
fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

// ---------- Liste ----------

pub async fn count(pool: &PgPool, f: &InvoiceFilter, pattern: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM {FROM} WHERE {LIST_WHERE}"
    )))
    .bind(&f.suche)
    .bind(pattern)
    .bind(&f.status)
    .bind(opt(&f.von))
    .bind(opt(&f.bis))
    .fetch_one(pool)
    .await
}

pub async fn page(
    pool: &PgPool,
    f: &InvoiceFilter,
    pattern: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<InvoiceRow>, sqlx::Error> {
    let direction = if f.richtung == "ab" { "DESC" } else { "ASC" };
    // Geprüft: Sortierung aus fester Liste (order_by), Richtung ist ASC/DESC.
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {ROW_COLUMNS}, {OVERDUE_DAYS} AS tage_ueberfaellig FROM {FROM} WHERE {LIST_WHERE} \
         ORDER BY {} {direction} NULLS LAST, r.id DESC LIMIT $6 OFFSET $7",
        order_by(&f.sortierung)
    )))
    .bind(&f.suche)
    .bind(pattern)
    .bind(&f.status)
    .bind(opt(&f.von))
    .bind(opt(&f.bis))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Belege eines Projekts (AK-15), neueste zuerst.
pub async fn by_project(pool: &PgPool, projekt_id: i64) -> Result<Vec<InvoiceRow>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {ROW_COLUMNS}, {OVERDUE_DAYS} AS tage_ueberfaellig FROM {FROM} \
         WHERE r.projekt_id = $1 ORDER BY r.rechnungsdatum DESC, r.id DESC"
    )))
    .bind(projekt_id)
    .fetch_all(pool)
    .await
}

/// Offene Belege eines Kunden (AK-16), nach Fälligkeit.
pub async fn open_of_customer(
    pool: &PgPool,
    adresse_id: i64,
) -> Result<Vec<InvoiceRow>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {ROW_COLUMNS}, {OVERDUE_DAYS} AS tage_ueberfaellig FROM {FROM} \
         WHERE r.adresse_id = $1 AND r.status IN ('offen', 'teilweise_bezahlt') \
         ORDER BY r.faelligkeit, r.id"
    )))
    .bind(adresse_id)
    .fetch_all(pool)
    .await
}

/// Positionen mehrerer Belege (für die Beträge von Entwürfen in Listen).
pub async fn positions_of<'e>(
    exec: impl PgExecutor<'e>,
    ids: &[i64],
) -> Result<Vec<(i64, InvoicePosition)>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct Row {
        debitorenrechnung_id: i64,
        #[sqlx(flatten)]
        position: InvoicePosition,
    }
    let rows: Vec<Row> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT x.debitorenrechnung_id, {POSITION_COLUMNS} {POSITION_FROM} \
         WHERE x.debitorenrechnung_id = ANY($1) ORDER BY x.debitorenrechnung_id, x.reihenfolge, x.id"
    )))
    .bind(ids)
    .fetch_all(exec)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.debitorenrechnung_id, r.position))
        .collect())
}

// ---------- Ein Beleg ----------

const POSITION_COLUMNS: &str = "x.id, x.bezeichnung, (x.menge * 1000)::int8 AS menge, x.einheit, \
     (x.einzelpreis * 100)::int8 AS einzelpreis, (x.rabatt_prozent * 100)::int8 AS rabatt_prozent, \
     x.konto_id, k.nummer AS konto_nummer, k.bezeichnung AS konto_bezeichnung, \
     x.mwst_code_id, m.code AS mwst_code, (m.satz * 100)::int8 AS mwst_satz";

const POSITION_FROM: &str = "FROM debitorenposition x JOIN konto k ON k.id = x.konto_id \
     LEFT JOIN mwst_code m ON m.id = x.mwst_code_id";

/// Positionen eines Belegs in ihrer Reihenfolge.
pub async fn positions<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Vec<InvoicePosition>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {POSITION_COLUMNS} {POSITION_FROM} \
         WHERE x.debitorenrechnung_id = $1 ORDER BY x.reihenfolge, x.id"
    )))
    .bind(id)
    .fetch_all(exec)
    .await
}

/// Kopf eines Belegs; `None`, wenn es ihn nicht gibt.
pub async fn find<'e>(exec: impl PgExecutor<'e>, id: i64) -> Result<Option<Invoice>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT r.id, r.nummer, r.art, r.adresse_id, a.nummer AS kunde_nummer, a.anzeigename AS kunde, \
                r.projekt_id, p.nummer AS projekt_nummer, p.name AS projekt_name, \
                r.bezug_rechnung_id, b.nummer AS bezug_nummer, r.status, \
                r.rechnungsdatum::text AS rechnungsdatum, r.zahlungsbedingung_id, \
                z.bezeichnung AS zahlungsbedingung, z.rechnungstext AS zahlungstext, \
                r.faelligkeit::text AS faelligkeit, r.skonto_bis::text AS skonto_bis, \
                (r.skonto_prozent * 100)::int8 AS skonto_prozent, r.waehrung, \
                ROUND(r.kurs * 1000000)::int8 AS kurs, r.referenz_kunde, r.einleitungstext, r.schlusstext, \
                r.bemerkung, ROUND(r.gesamtbetrag * 100)::int8 AS gesamtbetrag, \
                ROUND(r.gesamtbetrag_chf * 100)::int8 AS gesamtbetrag_chf, \
                ROUND(r.offener_betrag * 100)::int8 AS offener_betrag, r.buchung_id, \
                (SELECT g.id FROM buchung g WHERE g.storno_von_id = r.buchung_id) AS storno_buchung_id, \
                {OVERDUE_DAYS} AS tage_ueberfaellig, \
                to_char(r.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                to_char(r.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
         FROM {FROM} \
         JOIN zahlungsbedingung z ON z.id = r.zahlungsbedingung_id \
         LEFT JOIN debitorenrechnung b ON b.id = r.bezug_rechnung_id \
         WHERE r.id = $1"
    )))
    .bind(id)
    .fetch_optional(exec)
    .await
}

/// Sperrt den Beleg bis zum Ende der Transaktion und liefert seinen Status (`None` = nicht gefunden).
pub async fn lock(conn: &mut PgConnection, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT status FROM debitorenrechnung WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(conn)
        .await
}

/// Beleg einer Position sperren: (Kennung des Belegs, Status), `None` = Position nicht gefunden.
pub async fn lock_of_position(
    conn: &mut PgConnection,
    position_id: i64,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id, r.status FROM debitorenposition x \
         JOIN debitorenrechnung r ON r.id = x.debitorenrechnung_id \
         WHERE x.id = $1 FOR UPDATE OF r",
    )
    .bind(position_id)
    .fetch_optional(conn)
    .await
}

/// Anschrift des Kunden (Druck, AK-11).
pub async fn customer_address(
    pool: &PgPool,
    adresse_id: i64,
) -> Result<Option<CustomerAddress>, sqlx::Error> {
    sqlx::query_as(
        "SELECT anzeigename, zusatz, strasse, postfach, plz, ort, land FROM adresse WHERE id = $1",
    )
    .bind(adresse_id)
    .fetch_optional(pool)
    .await
}

/// Standard-Bankkonto (ANF-015 AK-5) für die Bankverbindung im Druck.
pub async fn standard_bank(pool: &PgPool) -> Result<Option<PrintBank>, sqlx::Error> {
    sqlx::query_as(
        "SELECT bezeichnung, bank, iban, bic FROM bankkonto WHERE standard AND aktiv LIMIT 1",
    )
    .fetch_optional(pool)
    .await
}

// ---------- Stammdaten ----------

/// Aktive Zahlungsbedingungen, Standard zuerst (AK-4).
pub async fn active_terms(pool: &PgPool) -> Result<Vec<TermOption>, sqlx::Error> {
    let rows: Vec<(i64, String, bool)> = sqlx::query_as(
        "SELECT id, bezeichnung, standard FROM zahlungsbedingung WHERE aktiv \
         ORDER BY standard DESC, lower(bezeichnung), id",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(id, bezeichnung, standard)| TermOption {
            id,
            bezeichnung,
            standard,
        })
        .collect())
}

/// Zahlungsbedingung, die gewählt werden darf: aktiv oder die bisherige des Belegs.
pub async fn term_usable(
    conn: &mut PgConnection,
    id: i64,
    bisher: Option<i64>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM zahlungsbedingung WHERE id = $1 AND (aktiv OR id = $2))",
    )
    .bind(id)
    .bind(bisher)
    .fetch_one(conn)
    .await
}

/// Vorgabe-Zahlungsbedingung: die der Adresse (wenn aktiv), sonst die Standard-Bedingung (AK-4).
pub async fn default_term(
    conn: &mut PgConnection,
    adresse_id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT coalesce( \
             (SELECT z.id FROM adresse a JOIN zahlungsbedingung z ON z.id = a.zahlungsbedingung_id \
              WHERE a.id = $1 AND z.aktiv), \
             (SELECT id FROM zahlungsbedingung WHERE standard))",
    )
    .bind(adresse_id)
    .fetch_one(conn)
    .await
}

/// Frist und Skonto einer Zahlungsbedingung: (Zahlungsfrist, Skontofrist, Skonto in Hundertstel Prozent).
pub async fn term_days(
    conn: &mut PgConnection,
    id: i64,
) -> Result<Option<(i64, Option<i64>, Option<i64>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT zahlungsfrist_tage::int8, skontofrist_tage::int8, (skonto_prozent * 100)::int8 \
         FROM zahlungsbedingung WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(conn)
    .await
}

/// Aktive Ertragskonten (AK-5).
pub async fn income_accounts<'e>(
    exec: impl PgExecutor<'e>,
) -> Result<Vec<AccountOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, bezeichnung, mwst_code_id AS vorgabe_mwst FROM konto \
         WHERE aktiv AND kontoart = 'ertrag' ORDER BY nummer",
    )
    .fetch_all(exec)
    .await
}

/// Aktive Umsatzsteuer-Codes (die Gültigkeit am Datum prüft `model`).
pub async fn sales_codes<'e>(exec: impl PgExecutor<'e>) -> Result<Vec<CodeOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, code, bezeichnung, ROUND(satz * 100)::BIGINT AS satz, \
                gueltig_ab::text AS gueltig_ab, gueltig_bis::text AS gueltig_bis, konto_id \
         FROM mwst_code WHERE aktiv AND art = 'umsatzsteuer' ORDER BY satz DESC, code",
    )
    .fetch_all(exec)
    .await
}

/// Konto einer Systemkonto-Rolle (ANF-011), `None` ohne Zuordnung.
pub async fn system_account<'e>(
    exec: impl PgExecutor<'e>,
    rolle: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT konto_id FROM systemkonto WHERE rolle = $1")
        .bind(rolle)
        .fetch_optional(exec)
        .await
}

/// Ist die eigene Firma erfasst (ANF-025 AK-10)?
pub async fn company_exists<'e>(exec: impl PgExecutor<'e>) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM firma)")
        .fetch_one(exec)
        .await
}

/// Kunde eines Projekts (AK-7), `None` ohne Projekt oder Kunde.
pub async fn project_customer(
    pool: &PgPool,
    projekt_id: i64,
) -> Result<Option<PickOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT a.id, a.nummer, a.anzeigename AS name FROM projekt p \
         JOIN adresse a ON a.id = p.adresse_id WHERE p.id = $1",
    )
    .bind(projekt_id)
    .fetch_optional(pool)
    .await
}

// ---------- Kopf schreiben ----------

/// Neuen Entwurf speichern; liefert die Kennung.
pub async fn insert(
    conn: &mut PgConnection,
    input: &InvoiceInput,
    zahlungsbedingung_id: i64,
    bezug_rechnung_id: Option<i64>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO debitorenrechnung (art, adresse_id, projekt_id, bezug_rechnung_id, rechnungsdatum, \
                zahlungsbedingung_id, waehrung, referenz_kunde, einleitungstext, schlusstext, bemerkung) \
         VALUES ($1, $2, $3, $4, $5::date, $6, $7, $8, $9, $10, $11) RETURNING id",
    )
    .bind(&input.art)
    .bind(input.customer_id())
    .bind(input.project_id())
    .bind(bezug_rechnung_id)
    .bind(&input.rechnungsdatum)
    .bind(zahlungsbedingung_id)
    .bind(&input.waehrung)
    .bind(opt(&input.referenz_kunde))
    .bind(opt(&input.einleitungstext))
    .bind(opt(&input.schlusstext))
    .bind(opt(&input.bemerkung))
    .fetch_one(conn)
    .await
}

/// Kopfdaten ändern (im Entwurf alle Felder; nach dem Buchen gleicht `restricted_to_booked` sie an).
pub async fn update(
    conn: &mut PgConnection,
    id: i64,
    input: &InvoiceInput,
    zahlungsbedingung_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE debitorenrechnung SET adresse_id = $2, projekt_id = $3, rechnungsdatum = $4::date, \
                zahlungsbedingung_id = $5, waehrung = $6, referenz_kunde = $7, einleitungstext = $8, \
                schlusstext = $9, bemerkung = $10, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(input.customer_id())
    .bind(input.project_id())
    .bind(&input.rechnungsdatum)
    .bind(zahlungsbedingung_id)
    .bind(&input.waehrung)
    .bind(opt(&input.referenz_kunde))
    .bind(opt(&input.einleitungstext))
    .bind(opt(&input.schlusstext))
    .bind(opt(&input.bemerkung))
    .execute(conn)
    .await?;
    Ok(())
}

/// Beleg als geändert markieren (auch bei Änderungen an Positionen).
pub async fn touch(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE debitorenrechnung SET geaendert_am = now() WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Nächste Laufnummer je Art und Jahr (AK-8). Die Zeile bleibt bis zum Ende der Transaktion gesperrt.
pub async fn next_number(
    conn: &mut PgConnection,
    art: &str,
    jahr: i32,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO debitorenrechnung_nummer (art, jahr, letzte) VALUES ($1, $2, 1) \
         ON CONFLICT (art, jahr) DO UPDATE SET letzte = debitorenrechnung_nummer.letzte + 1 \
         RETURNING letzte",
    )
    .bind(art)
    .bind(jahr)
    .fetch_one(conn)
    .await
}

/// Werte, die beim Buchen festgeschrieben werden (AK-8).
pub struct BookedValues {
    pub nummer: String,
    pub faelligkeit: String,
    pub skonto: Option<(String, i64)>,
    /// Millionstel
    pub kurs: i64,
    pub gesamtbetrag: i64,
    pub gesamtbetrag_chf: i64,
    pub buchung_id: i64,
}

/// Entwurf buchen: Status „offen“, Nummer, Fälligkeit, Kurs, Beträge und Buchung setzen.
pub async fn mark_booked(
    conn: &mut PgConnection,
    id: i64,
    v: &BookedValues,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE debitorenrechnung SET status = 'offen', nummer = $2, faelligkeit = $3::date, \
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

/// Storniert (AK-13): Status „storniert“, nichts mehr offen.
pub async fn mark_cancelled(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE debitorenrechnung SET status = 'storniert', offener_betrag = 0, geaendert_am = now() \
         WHERE id = $1 AND status = 'offen'",
    )
    .bind(id)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Entwurf löschen (AK-12); die Positionen folgen. Liefert die Art, `None` wenn kein Entwurf.
pub async fn delete_draft(conn: &mut PgConnection, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "DELETE FROM debitorenrechnung WHERE id = $1 AND status = 'entwurf' RETURNING art",
    )
    .bind(id)
    .fetch_optional(conn)
    .await
}

/// Gutschrift-Entwurf aus einer gebuchten Rechnung (AK-14): gleicher Kunde, Projekt, Währung,
/// Zahlungsbedingung und Referenz, Datum heute, Verweis auf die Rechnung. Liefert die Kennung.
pub async fn insert_credit_note_from(
    conn: &mut PgConnection,
    rechnung_id: i64,
) -> Result<i64, sqlx::Error> {
    let id: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
        "INSERT INTO debitorenrechnung (art, adresse_id, projekt_id, bezug_rechnung_id, rechnungsdatum, \
                zahlungsbedingung_id, waehrung, referenz_kunde) \
         SELECT 'gutschrift', adresse_id, projekt_id, id, {TODAY}, zahlungsbedingung_id, waehrung, referenz_kunde \
         FROM debitorenrechnung WHERE id = $1 RETURNING id"
    )))
    .bind(rechnung_id)
    .fetch_one(&mut *conn)
    .await?;
    sqlx::query(
        "INSERT INTO debitorenposition (debitorenrechnung_id, reihenfolge, bezeichnung, menge, einheit, \
                einzelpreis, rabatt_prozent, konto_id, mwst_code_id) \
         SELECT $2, reihenfolge, bezeichnung, menge, einheit, einzelpreis, rabatt_prozent, konto_id, mwst_code_id \
         FROM debitorenposition WHERE debitorenrechnung_id = $1",
    )
    .bind(rechnung_id)
    .bind(id)
    .execute(conn)
    .await?;
    Ok(id)
}

// ---------- Positionen schreiben ----------

/// Position am Ende anfügen; liefert die Kennung.
pub async fn insert_position(
    conn: &mut PgConnection,
    rechnung_id: i64,
    input: &PositionInput,
    v: &PositionValues,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO debitorenposition (debitorenrechnung_id, reihenfolge, bezeichnung, menge, einheit, \
                einzelpreis, rabatt_prozent, konto_id, mwst_code_id) \
         VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM debitorenposition WHERE debitorenrechnung_id = $1), \
                 $2, $3::numeric / 1000, $4, $5::numeric / 100, $6::numeric / 100, $7, $8) \
         RETURNING id",
    )
    .bind(rechnung_id)
    .bind(&input.bezeichnung)
    .bind(v.menge)
    .bind(&input.einheit)
    .bind(v.einzelpreis)
    .bind(v.rabatt_prozent)
    .bind(v.konto_id)
    .bind(v.mwst_code_id)
    .fetch_one(conn)
    .await
}

/// Position ändern.
pub async fn update_position(
    conn: &mut PgConnection,
    id: i64,
    input: &PositionInput,
    v: &PositionValues,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE debitorenposition SET bezeichnung = $2, menge = $3::numeric / 1000, einheit = $4, \
                einzelpreis = $5::numeric / 100, rabatt_prozent = $6::numeric / 100, konto_id = $7, \
                mwst_code_id = $8, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&input.bezeichnung)
    .bind(v.menge)
    .bind(&input.einheit)
    .bind(v.einzelpreis)
    .bind(v.rabatt_prozent)
    .bind(v.konto_id)
    .bind(v.mwst_code_id)
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn delete_position(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM debitorenposition WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Position eine Stelle nach oben/unten schieben (AK-5).
pub async fn move_position(
    conn: &mut PgConnection,
    rechnung_id: i64,
    id: i64,
    nach_oben: bool,
) -> Result<(), sqlx::Error> {
    // Lückenlos durchnummerieren, dann mit dem Nachbarn tauschen
    sqlx::query(
        "UPDATE debitorenposition t SET reihenfolge = r.rn \
         FROM (SELECT id, row_number() OVER (ORDER BY reihenfolge, id)::int AS rn \
               FROM debitorenposition WHERE debitorenrechnung_id = $1) r \
         WHERE t.id = r.id",
    )
    .bind(rechnung_id)
    .execute(&mut *conn)
    .await?;
    let pos: i32 = sqlx::query_scalar("SELECT reihenfolge FROM debitorenposition WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *conn)
        .await?;
    let other = if nach_oben { pos - 1 } else { pos + 1 };
    let swapped = sqlx::query(
        "UPDATE debitorenposition SET reihenfolge = $3 WHERE debitorenrechnung_id = $1 AND reihenfolge = $2",
    )
    .bind(rechnung_id)
    .bind(other)
    .bind(pos)
    .execute(&mut *conn)
    .await?;
    if swapped.rows_affected() == 1 {
        sqlx::query("UPDATE debitorenposition SET reihenfolge = $2 WHERE id = $1")
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
        assert_eq!(order_by("; DROP TABLE debitorenrechnung"), "r.faelligkeit");
        assert_eq!(order_by("kunde"), "lower(a.anzeigename)");
    }
}
