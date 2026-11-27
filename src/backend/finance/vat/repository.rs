//! SQL-Zugriffe für MWST-Codes und MWST-Einstellungen (ANF-012, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Datumswerte werden als Text gelesen (`::text`) und als Text übergeben (`::date`); Beträge in
//! Hundertsteln (`::int8`), damit kein Gleitkommawert entsteht.

use sqlx::{AssertSqlSafe, PgConnection, PgPool};

use super::model::{CodeValues, VatCode, VatSettings};

/// Ein MWST-Code mit Konto. `verwendet` prüft Einkaufspositionen, Buchungszeilen (ANF-017) und Positionen von
/// Ausgangsrechnungen (ANF-018); Eingangsrechnungen folgen (AK-5, AK-7).
const SELECT_CODE: &str = "\
    SELECT m.id, m.code, m.bezeichnung, m.art, (m.satz * 100)::int8 AS satz, \
           m.gueltig_ab::text AS gueltig_ab, m.gueltig_bis::text AS gueltig_bis, m.konto_id, \
           k.nummer AS konto_nummer, k.bezeichnung AS konto_bezeichnung, m.formularziffer, m.aktiv, \
           (EXISTS (SELECT 1 FROM einkaufsposition p WHERE p.mwst_code_id = m.id) \
            OR EXISTS (SELECT 1 FROM buchungszeile z WHERE z.mwst_code_id = m.id) \
            OR EXISTS (SELECT 1 FROM debitorenposition d WHERE d.mwst_code_id = m.id)) AS verwendet \
    FROM mwst_code m JOIN konto k ON k.id = m.konto_id";

/// Die einzige Zeile der MWST-Einstellungen.
pub async fn settings(pool: &PgPool) -> Result<Option<VatSettings>, sqlx::Error> {
    sqlx::query_as(
        "SELECT mwst_pflichtig, mwst_pflichtig_ab::text AS mwst_pflichtig_ab, \
                (umsatzgrenze * 100)::int8 AS umsatzgrenze, methode, abrechnungsart, periodizitaet \
         FROM finanzeinstellung",
    )
    .fetch_optional(pool)
    .await
}

/// Speichert die MWST-Einstellungen.
pub async fn update_settings(pool: &PgPool, s: &VatSettings) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE finanzeinstellung SET mwst_pflichtig = $1, mwst_pflichtig_ab = $2::date, \
                umsatzgrenze = $3::numeric / 100, methode = $4, abrechnungsart = $5, periodizitaet = $6",
    )
    .bind(s.mwst_pflichtig)
    .bind(s.mwst_pflichtig_ab.as_deref())
    .bind(s.umsatzgrenze)
    .bind(&s.methode)
    .bind(&s.abrechnungsart)
    .bind(&s.periodizitaet)
    .execute(pool)
    .await
    .map(|r| r.rows_affected())
}

/// UID der Firma (ANF-025); `None`, wenn keine Firma oder keine UID erfasst ist.
pub async fn uid(pool: &PgPool) -> Result<Option<String>, sqlx::Error> {
    let uid: Option<Option<String>> = sqlx::query_scalar("SELECT uid FROM firma")
        .fetch_optional(pool)
        .await?;
    Ok(uid.flatten())
}

/// Alle MWST-Codes, nach Art und Satz geordnet (höchster Satz zuerst).
pub async fn list_codes(pool: &PgPool) -> Result<Vec<VatCode>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!(
        "{SELECT_CODE} ORDER BY m.art, m.satz DESC, m.code"
    )))
    .fetch_all(pool)
    .await
}

/// Ein MWST-Code; `None`, wenn es ihn nicht gibt.
pub async fn find_code(pool: &PgPool, id: i64) -> Result<Option<VatCode>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!("{SELECT_CODE} WHERE m.id = $1")))
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// `true`, wenn der Code schon von einem anderen MWST-Code als `except` belegt ist.
pub async fn code_taken(
    pool: &PgPool,
    code: &str,
    except: Option<i64>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM mwst_code WHERE code = $1 AND ($2::bigint IS NULL OR id <> $2))",
    )
    .bind(code)
    .bind(except)
    .fetch_one(pool)
    .await
}

/// Kontoart und Aktiv-Status eines Kontos; `None`, wenn es das Konto nicht gibt.
pub async fn konto_info(pool: &PgPool, id: i64) -> Result<Option<(String, bool)>, sqlx::Error> {
    sqlx::query_as("SELECT kontoart, aktiv FROM konto WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Kennung eines Kontos nach Nummer (für die Standard-Codes).
pub async fn konto_id_by_number(
    conn: &mut PgConnection,
    nummer: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT id FROM konto WHERE nummer = $1")
        .bind(nummer)
        .fetch_optional(conn)
        .await
}

/// Konto eines Systemkontos nach Rolle (ANF-011), z. B. „vorsteuer“.
pub async fn system_konto_id(
    conn: &mut PgConnection,
    rolle: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT konto_id FROM systemkonto WHERE rolle = $1")
        .bind(rolle)
        .fetch_optional(conn)
        .await
}

/// Legt einen MWST-Code an und gibt seine Kennung zurück.
pub async fn insert_code(conn: &mut PgConnection, v: &CodeValues) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO mwst_code (code, bezeichnung, art, satz, gueltig_ab, gueltig_bis, konto_id, \
                                formularziffer, aktiv) \
         VALUES ($1, $2, $3, $4::numeric / 100, $5::date, $6::date, $7, $8, $9) RETURNING id",
    )
    .bind(&v.code)
    .bind(&v.bezeichnung)
    .bind(&v.art)
    .bind(v.satz)
    .bind(&v.gueltig_ab)
    .bind(v.gueltig_bis.as_deref())
    .bind(v.konto_id)
    .bind(v.formularziffer.as_deref())
    .bind(v.aktiv)
    .fetch_one(conn)
    .await
}

/// Ändert einen MWST-Code. Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update_code(
    conn: &mut PgConnection,
    id: i64,
    v: &CodeValues,
) -> Result<u64, sqlx::Error> {
    sqlx::query(
        "UPDATE mwst_code SET code = $2, bezeichnung = $3, art = $4, satz = $5::numeric / 100, \
                gueltig_ab = $6::date, gueltig_bis = $7::date, konto_id = $8, formularziffer = $9, \
                aktiv = $10, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(&v.code)
    .bind(&v.bezeichnung)
    .bind(&v.art)
    .bind(v.satz)
    .bind(&v.gueltig_ab)
    .bind(v.gueltig_bis.as_deref())
    .bind(v.konto_id)
    .bind(v.formularziffer.as_deref())
    .bind(v.aktiv)
    .execute(conn)
    .await
    .map(|r| r.rows_affected())
}

/// Löscht einen MWST-Code. Verweise (Einkauf, später Rechnungen) verhindern das mit Fehler 23503 (AK-7).
pub async fn delete_code(conn: &mut PgConnection, id: i64) -> Result<u64, sqlx::Error> {
    sqlx::query("DELETE FROM mwst_code WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await
        .map(|r| r.rows_affected())
}

/// Heutiges Datum der Datenbank als `JJJJ-MM-TT` (Gültigkeit der Codes bei der Erfassung).
pub async fn today(pool: &PgPool) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("SELECT CURRENT_DATE::text")
        .fetch_one(pool)
        .await
}

/// Anzahl der MWST-Codes (Standard-Codes nur bei leerer Liste, AK-2).
pub async fn count_codes(conn: &mut PgConnection) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM mwst_code")
        .fetch_one(conn)
        .await
}

/// Sperrt die MWST-Codes für die Dauer der Transaktion (gleichzeitiges Übernehmen der Standard-Codes).
pub async fn lock_codes(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query("LOCK TABLE mwst_code IN SHARE ROW EXCLUSIVE MODE")
        .execute(conn)
        .await?;
    Ok(())
}
