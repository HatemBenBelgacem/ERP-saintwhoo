//! SQL-Zugriffe für das Buchungsjournal (ANF-017, nur Server).
//!
//! Hier steht nur Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen. Lesefunktionen nehmen
//! einen beliebigen Ausführer (Pool oder Transaktion), Schreibfunktionen eine Verbindung.

use sqlx::{AssertSqlSafe, PgConnection, PgExecutor, PgPool};

use super::model::{
    AccountOption, BookingHead, BookingRef, CodeOption, EffectiveLine, JournalFilter, LineView,
    ProjectOption,
};
use crate::backend::purchase_orders::model::format_fixed;

/// Kopf einer Buchung mit Geschäftsjahr und erfassendem Benutzer (Name aus der Adresse).
const SELECT_HEAD: &str = "\
    SELECT b.id, b.geschaeftsjahr_id, g.bezeichnung AS geschaeftsjahr, b.journalnummer, \
           b.datum::text AS datum, b.belegnummer, b.text, b.status, b.herkunft, b.storno_von_id, \
           COALESCE(NULLIF(btrim(concat_ws(' ', a.vorname, a.nachname)), ''), a.firmenname, bu.benutzername) \
               AS erfasst_von, \
           to_char(b.erstellt_am, 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
           to_char(b.gebucht_am, 'DD.MM.YYYY HH24:MI') AS gebucht_am \
    FROM buchung b \
    JOIN geschaeftsjahr g ON g.id = b.geschaeftsjahr_id \
    LEFT JOIN benutzer bu ON bu.id = b.erfasst_von \
    LEFT JOIN adresse a ON a.id = bu.adresse_id";

/// Filter der Liste (AK-2). Platzhalter: $1 Geschäftsjahr, $2 Suchtext, $3 Suchmuster,
/// $4 Betrag, $5 Herkunft, $6 Status, $7 Von, $8 Bis.
const WHERE_LIST: &str = "\
    WHERE b.geschaeftsjahr_id = $1 \
      AND ($2 = '' OR b.text ILIKE $3 OR b.belegnummer ILIKE $3 \
           OR EXISTS (SELECT 1 FROM buchungszeile z JOIN konto k ON k.id = z.konto_id \
                      WHERE z.buchung_id = b.id \
                        AND (k.nummer LIKE $3 OR k.bezeichnung ILIKE $3 \
                             OR z.soll = $4::numeric OR z.haben = $4::numeric))) \
      AND ($5 = '' OR b.herkunft = $5) \
      AND ($6 = '' OR b.status = $6) \
      AND ($7::date IS NULL OR b.datum >= $7::date) \
      AND ($8::date IS NULL OR b.datum <= $8::date)";

/// Feste Sortierungen (AK-2). Die Auswahl kommt nie als Text in den SQL-Text.
fn order_by(sortierung: &str) -> &'static str {
    match sortierung {
        "datum_auf" => "b.datum ASC, b.id ASC",
        "journal" => "b.journalnummer DESC NULLS LAST, b.id DESC",
        "betrag_ab" => {
            "(SELECT COALESCE(SUM(z.soll), 0) FROM buchungszeile z WHERE z.buchung_id = b.id) DESC, b.id DESC"
        }
        "beleg" => "b.belegnummer ASC, b.id ASC",
        _ => "b.datum DESC, b.id DESC",
    }
}

/// Anzahl und Seite der Buchungen nach dem Filter.
pub async fn count(
    pool: &PgPool,
    filter: &JournalFilter,
    pattern: &str,
    betrag: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT COUNT(*) FROM buchung b {WHERE_LIST}"
    )))
    .bind(filter.geschaeftsjahr_id)
    .bind(&filter.suche)
    .bind(pattern)
    .bind(betrag)
    .bind(&filter.herkunft)
    .bind(&filter.status)
    .bind(non_empty(&filter.von))
    .bind(non_empty(&filter.bis))
    .fetch_one(pool)
    .await
}

/// Eine Seite der Buchungen (ohne Zeilen; die holt `lines_for`).
pub async fn page(
    pool: &PgPool,
    filter: &JournalFilter,
    pattern: &str,
    betrag: Option<&str>,
    offset: i64,
    limit: i64,
) -> Result<Vec<BookingHead>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_HEAD} {WHERE_LIST} ORDER BY {} LIMIT $9 OFFSET $10",
        order_by(&filter.sortierung)
    )))
    .bind(filter.geschaeftsjahr_id)
    .bind(&filter.suche)
    .bind(pattern)
    .bind(betrag)
    .bind(&filter.herkunft)
    .bind(&filter.status)
    .bind(non_empty(&filter.von))
    .bind(non_empty(&filter.bis))
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Leerer Text als fehlender Wert (für `IS NULL`-Prüfungen der Datumsfilter).
fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

/// Kopf einer Buchung; `None`, wenn es sie nicht gibt.
pub async fn find_head<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<BookingHead>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!("{SELECT_HEAD} WHERE b.id = $1")))
        .bind(id)
        .fetch_optional(exec)
        .await
}

/// Zeilen zu mehreren Buchungen, sortiert nach Buchung und Reihenfolge.
pub async fn lines_for<'e>(
    exec: impl PgExecutor<'e>,
    ids: &[i64],
) -> Result<Vec<LineView>, sqlx::Error> {
    sqlx::query_as(
        "SELECT z.id, z.buchung_id, z.reihenfolge, z.konto_id, k.nummer AS konto_nummer, \
                k.bezeichnung AS konto_bezeichnung, \
                ROUND(z.soll * 100)::BIGINT AS soll, ROUND(z.haben * 100)::BIGINT AS haben, \
                z.mwst_code_id, m.code AS mwst_code, ROUND(z.mwst_basis * 100)::BIGINT AS mwst_basis, \
                z.ist_mwst_zeile, z.projekt_id, p.nummer AS projekt_nummer, p.name AS projekt_name \
         FROM buchungszeile z \
         JOIN konto k ON k.id = z.konto_id \
         LEFT JOIN mwst_code m ON m.id = z.mwst_code_id \
         LEFT JOIN projekt p ON p.id = z.projekt_id \
         WHERE z.buchung_id = ANY($1) \
         ORDER BY z.buchung_id, z.reihenfolge",
    )
    .bind(ids)
    .fetch_all(exec)
    .await
}

/// Verweis auf die Buchung, die `id` storniert (Original zur Gegenbuchung).
pub async fn storno_original<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<BookingRef>, sqlx::Error> {
    sqlx::query_as(
        "SELECT o.id, 'Journal ' || COALESCE(o.journalnummer::text, '–') || ' vom ' \
                || to_char(o.datum, 'DD.MM.YYYY') AS label \
         FROM buchung o WHERE o.id = (SELECT b.storno_von_id FROM buchung b WHERE b.id = $1)",
    )
    .bind(id)
    .fetch_optional(exec)
    .await
}

/// Verweis auf die Gegenbuchung einer Buchung (Storno).
pub async fn storniert_durch<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<BookingRef>, sqlx::Error> {
    sqlx::query_as(
        "SELECT g.id, 'Journal ' || COALESCE(g.journalnummer::text, '–') || ' vom ' \
                || to_char(g.datum, 'DD.MM.YYYY') AS label \
         FROM buchung g WHERE g.storno_von_id = $1",
    )
    .bind(id)
    .fetch_optional(exec)
    .await
}

/// Ausgangsrechnung bzw. Gutschrift, aus der die Buchung stammt (AK-12, ANF-018); die Kennung ist die des Belegs.
pub async fn debtor_document<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<BookingRef>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id, CASE r.art WHEN 'gutschrift' THEN 'Gutschrift ' ELSE 'Rechnung ' END \
                || COALESCE(r.nummer, '') AS label \
         FROM debitorenrechnung r WHERE r.buchung_id = $1",
    )
    .bind(id)
    .fetch_optional(exec)
    .await
}

/// Eingangsrechnung bzw. Lieferantengutschrift, aus der die Buchung stammt (AK-12, ANF-019).
pub async fn creditor_document<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<BookingRef>, sqlx::Error> {
    sqlx::query_as(
        "SELECT r.id, CASE r.art WHEN 'gutschrift' THEN 'Lieferantengutschrift ' ELSE 'Eingangsrechnung ' END \
                || COALESCE(r.nummer, '') AS label \
         FROM kreditorenrechnung r WHERE r.buchung_id = $1",
    )
    .bind(id)
    .fetch_optional(exec)
    .await
}

/// Zahlung, aus der die Buchung stammt (AK-12, ANF-020).
pub async fn payment_document<'e>(
    exec: impl PgExecutor<'e>,
    id: i64,
) -> Result<Option<BookingRef>, sqlx::Error> {
    sqlx::query_as(
        "SELECT z.id, CASE z.art WHEN 'ausgang' THEN 'Zahlungsausgang ' ELSE 'Zahlungseingang ' END \
                || to_char(z.datum, 'DD.MM.YYYY') AS label \
         FROM zahlung z WHERE z.buchung_id = $1",
    )
    .bind(id)
    .fetch_optional(exec)
    .await
}

/// Geschäftsjahr zu einem Datum mit Status des Jahrs und der Periode (ANF-013 AK-5, AK-6).
/// `None`, wenn kein Geschäftsjahr das Datum enthält.
pub async fn year_for_date<'e>(
    exec: impl PgExecutor<'e>,
    datum: &str,
) -> Result<Option<(i64, String, Option<String>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT g.id, g.status, p.status \
         FROM geschaeftsjahr g \
         LEFT JOIN periode p ON p.geschaeftsjahr_id = g.id AND $1::date BETWEEN p.beginn AND p.ende \
         WHERE $1::date BETWEEN g.beginn AND g.ende",
    )
    .bind(datum)
    .fetch_optional(exec)
    .await
}

/// Beginn der MWST-Pflicht, `None` wenn sie nicht eingeschaltet ist (ANF-012 AK-8).
pub async fn mwst_pflicht_ab<'e>(exec: impl PgExecutor<'e>) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT CASE WHEN mwst_pflichtig THEN mwst_pflichtig_ab::text END FROM finanzeinstellung",
    )
    .fetch_optional(exec)
    .await?;
    Ok(row.and_then(|(ab,)| ab))
}

/// Aktive Konten für die Auswahl (AK-4), nach Nummer.
pub async fn active_accounts<'e>(
    exec: impl PgExecutor<'e>,
) -> Result<Vec<AccountOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, bezeichnung, mwst_code_id AS vorgabe_mwst FROM konto \
         WHERE aktiv ORDER BY nummer",
    )
    .fetch_all(exec)
    .await
}

/// Aktive MWST-Codes für das Formular.
pub async fn active_codes<'e>(exec: impl PgExecutor<'e>) -> Result<Vec<CodeOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, code, bezeichnung, ROUND(satz * 100)::BIGINT AS satz, \
                gueltig_ab::text AS gueltig_ab, gueltig_bis::text AS gueltig_bis, konto_id \
         FROM mwst_code WHERE aktiv ORDER BY code",
    )
    .fetch_all(exec)
    .await
}

/// Alle Projekte für die Auswahl (AK-8).
pub async fn projects<'e>(exec: impl PgExecutor<'e>) -> Result<Vec<ProjectOption>, sqlx::Error> {
    sqlx::query_as("SELECT id, nummer, name FROM projekt ORDER BY nummer")
        .fetch_all(exec)
        .await
}

/// Belegnummern, die einem Muster wie `M-2026-%` entsprechen (für die nächste freie Nummer).
pub async fn belegnummern_like<'e>(
    exec: impl PgExecutor<'e>,
    pattern: &str,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT belegnummer FROM buchung WHERE belegnummer LIKE $1")
        .bind(pattern)
        .fetch_all(exec)
        .await
}

/// Neue Entwurf-Buchung mit Kopf (AK-3, AK-10). Liefert die Kennung.
pub async fn insert_draft(conn: &mut PgConnection, head: &NewHead) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO buchung (geschaeftsjahr_id, datum, belegnummer, text, herkunft, storno_von_id, erfasst_von) \
         VALUES ($1, $2::date, $3, $4, $5, $6, $7) RETURNING id",
    )
    .bind(head.geschaeftsjahr_id)
    .bind(&head.datum)
    .bind(&head.belegnummer)
    .bind(&head.text)
    .bind(&head.herkunft)
    .bind(head.storno_von_id)
    .bind(head.erfasst_von)
    .fetch_one(conn)
    .await
}

/// Kopf eines Entwurfs ändern (das Geschäftsjahr folgt dem Datum).
pub async fn update_draft_head(
    conn: &mut PgConnection,
    id: i64,
    head: &NewHead,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE buchung SET geschaeftsjahr_id = $2, datum = $3::date, belegnummer = $4, text = $5, \
                geaendert_am = now() WHERE id = $1 AND status = 'entwurf'",
    )
    .bind(id)
    .bind(head.geschaeftsjahr_id)
    .bind(&head.datum)
    .bind(&head.belegnummer)
    .bind(&head.text)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Alle Zeilen eines Entwurfs entfernen, bevor sie neu geschrieben werden.
pub async fn delete_lines(conn: &mut PgConnection, buchung_id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM buchungszeile WHERE buchung_id = $1")
        .bind(buchung_id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Eine Zeile schreiben (Beträge in Rappen, als Festkommatext an die Datenbank).
pub async fn insert_line(
    conn: &mut PgConnection,
    buchung_id: i64,
    reihenfolge: i32,
    line: &EffectiveLine,
) -> Result<(), sqlx::Error> {
    let (soll, haben) = if line.soll {
        (format_fixed(line.betrag, 2), "0.00".to_string())
    } else {
        ("0.00".to_string(), format_fixed(line.betrag, 2))
    };
    sqlx::query(
        "INSERT INTO buchungszeile \
            (buchung_id, reihenfolge, konto_id, soll, haben, mwst_code_id, mwst_basis, ist_mwst_zeile, projekt_id) \
         VALUES ($1, $2, $3, $4::numeric, $5::numeric, $6, $7::numeric, $8, $9)",
    )
    .bind(buchung_id)
    .bind(reihenfolge)
    .bind(line.konto_id)
    .bind(soll)
    .bind(haben)
    .bind(line.mwst_code_id)
    .bind(line.mwst_basis.map(|b| format_fixed(b, 2)))
    .bind(line.ist_mwst_zeile)
    .bind(line.projekt_id)
    .execute(conn)
    .await?;
    Ok(())
}

/// Nächste Journalnummer des Geschäftsjahrs (AK-9). Der Zähler ist in der Transaktion gesperrt,
/// daher gibt es keine Lücken: Bricht die Buchung ab, wird die Nummer nicht verbraucht.
pub async fn next_journal_number(
    conn: &mut PgConnection,
    geschaeftsjahr_id: i64,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO journal_nummer (geschaeftsjahr_id, letzte) VALUES ($1, 1) \
         ON CONFLICT (geschaeftsjahr_id) DO UPDATE SET letzte = journal_nummer.letzte + 1 \
         RETURNING letzte",
    )
    .bind(geschaeftsjahr_id)
    .fetch_one(conn)
    .await
}

/// Entwurf buchen: Status, Journalnummer und Zeitpunkt setzen.
pub async fn book(
    conn: &mut PgConnection,
    id: i64,
    journalnummer: i32,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE buchung SET status = 'gebucht', journalnummer = $2, gebucht_am = now(), geaendert_am = now() \
         WHERE id = $1 AND status = 'entwurf'",
    )
    .bind(id)
    .bind(journalnummer)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Gebuchte Buchung als storniert markieren (der Kopf darf nur diesen Wechsel machen).
pub async fn mark_storniert(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query("UPDATE buchung SET status = 'storniert', geaendert_am = now() WHERE id = $1 AND status = 'gebucht'")
        .bind(id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Entwurf löschen; die Zeilen folgen durch die Verknüpfung.
pub async fn delete_draft(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM buchung WHERE id = $1 AND status = 'entwurf'")
        .bind(id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Werte für eine neue Buchung (Kopf).
pub struct NewHead {
    pub geschaeftsjahr_id: i64,
    pub datum: String,
    pub belegnummer: String,
    pub text: String,
    pub herkunft: String,
    pub storno_von_id: Option<i64>,
    pub erfasst_von: Option<i64>,
}
