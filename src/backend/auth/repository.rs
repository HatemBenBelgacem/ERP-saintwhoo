//! SQL-Zugriffe für Registrierung, Anmeldung und Sitzungen in der Portal-Datenbank
//! (ANF-007, ANF-026; nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Alle Funktionen erwarten den Pool bzw. die Verbindung der **Portal-Datenbank**.

use sqlx::{PgConnection, PgPool};

use super::model::{
    CurrentUser, LOCK_MINUTES, REGISTRATION_WINDOW_MINUTES, SESSION_IDLE_HOURS, SESSION_MAX_DAYS,
    SessionTenant, TENANT_READY,
};

/// Zeile einer gültigen Sitzung mit Zugang und (falls gewählt) Mandant.
#[derive(sqlx::FromRow)]
struct SessionRow {
    portal_id: i64,
    email: String,
    name: String,
    ist_administrator: bool,
    mandant_id: Option<i64>,
    firmenname: Option<String>,
    status: Option<String>,
    datenbankname: Option<String>,
}

impl From<SessionRow> for CurrentUser {
    fn from(r: SessionRow) -> Self {
        let mandant = match (r.mandant_id, r.status, r.datenbankname) {
            (Some(id), Some(status), Some(datenbankname)) => Some(SessionTenant {
                id,
                firmenname: r.firmenname,
                status,
                datenbankname,
            }),
            _ => None,
        };
        CurrentUser {
            portal_id: r.portal_id,
            email: r.email,
            name: r.name,
            ist_administrator: r.ist_administrator,
            mandant,
            benutzer_id: None,
        }
    }
}

/// Gültige Sitzung zum Merkmal-Hash: Zugang aktiv, nicht länger als 12 Stunden untätig und nicht älter
/// als 7 Tage (ANF-007 AK-14, AK-15). Der Mandant zählt nur, wenn er dem Zugang gehört (ANF-026 AK-21).
/// „Zuletzt aktiv“ wird höchstens einmal pro Minute nachgeführt.
pub async fn user_for_session(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    let row: Option<SessionRow> = sqlx::query_as(
        "WITH s AS ( \
             SELECT s.id, s.zuletzt_aktiv, b.id AS portal_id, b.email, \
                    b.vorname || ' ' || b.nachname AS name, b.ist_administrator, \
                    m.id AS mandant_id, m.firmenname, m.status, m.datenbankname \
             FROM portal_sitzung s \
             JOIN portal_benutzer b ON b.id = s.portal_benutzer_id \
             LEFT JOIN mandant m ON m.id = s.mandant_id AND m.besitzer_id = b.id \
             WHERE s.merkmal_hash = $1 AND b.aktiv \
               AND s.zuletzt_aktiv > now() - make_interval(hours => $2) \
               AND s.erstellt_am > now() - make_interval(days => $3) \
         ), \
         u AS ( \
             UPDATE portal_sitzung SET zuletzt_aktiv = now() \
             WHERE id IN (SELECT id FROM s WHERE zuletzt_aktiv < now() - interval '1 minute') \
         ) \
         SELECT portal_id, email, name, ist_administrator, mandant_id, firmenname, status, \
                datenbankname FROM s",
    )
    .bind(token_hash)
    .bind(SESSION_IDLE_HOURS)
    .bind(SESSION_MAX_DAYS)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(CurrentUser::from))
}

/// Anzahl Fehlversuche für die E-Mail in den letzten 15 Minuten (ANF-007 AK-11).
pub async fn recent_failures(pool: &PgPool, email: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM portal_anmeldeversuch \
         WHERE lower(email) = lower($1) AND zeitpunkt > now() - make_interval(mins => $2)",
    )
    .bind(email)
    .bind(LOCK_MINUTES)
    .fetch_one(pool)
    .await
}

/// Fehlversuch festhalten (auch für unbekannte E-Mail-Adressen).
pub async fn record_failure(pool: &PgPool, email: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO portal_anmeldeversuch (email) VALUES ($1)")
        .bind(email)
        .execute(pool)
        .await?;
    Ok(())
}

/// (Kennung, Passwort-Hash, aktiv) zur E-Mail, ohne Beachtung der Gross-/Kleinschreibung.
pub async fn credentials(
    pool: &PgPool,
    email: &str,
) -> Result<Option<(i64, String, bool)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, passwort_hash, aktiv FROM portal_benutzer WHERE lower(email) = lower($1)",
    )
    .bind(email)
    .fetch_optional(pool)
    .await
}

/// Anzahl Registrierungen insgesamt in den letzten `REGISTRATION_WINDOW_MINUTES` Minuten.
pub async fn recent_registrations(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM portal_benutzer \
         WHERE erstellt_am > now() - make_interval(mins => $1)",
    )
    .bind(REGISTRATION_WINDOW_MINUTES)
    .fetch_one(pool)
    .await
}

/// Neuen Zugang als Administrator anlegen (ANF-026 AK-4). `None`, wenn es die E-Mail schon gibt
/// (ANF-026 AK-3) – auch bei zwei gleichzeitigen Registrierungen entscheidet der eindeutige Index.
pub async fn create_user(
    conn: &mut PgConnection,
    vorname: &str,
    nachname: &str,
    email: &str,
    password_hash: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO portal_benutzer (vorname, nachname, email, passwort_hash, ist_administrator) \
         VALUES ($1, $2, $3, $4, true) \
         ON CONFLICT ((lower(email))) DO NOTHING RETURNING id",
    )
    .bind(vorname)
    .bind(nachname)
    .bind(email)
    .bind(password_hash)
    .fetch_optional(&mut *conn)
    .await
}

/// Nach erfolgreicher Anmeldung bzw. Registrierung: Fehlversuche der E-Mail löschen, abgelaufene
/// Sitzungen und alte Fehlversuche aufräumen (ANF-007 AK-23), neue Sitzung mit dem Mandanten des
/// Zugangs anlegen und „Letzte Anmeldung“ setzen (ANF-007 AK-10).
/// Liefert `true`, wenn der Mandant bereit und definiert ist (Zugang zum ERP, ANF-026 AK-19).
pub async fn start_session(
    conn: &mut PgConnection,
    user_id: i64,
    email: &str,
    token_hash: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query(
        "DELETE FROM portal_anmeldeversuch \
         WHERE lower(email) = lower($1) OR zeitpunkt < now() - interval '1 day'",
    )
    .bind(email)
    .execute(&mut *conn)
    .await?;
    sqlx::query(
        "DELETE FROM portal_sitzung WHERE zuletzt_aktiv < now() - make_interval(hours => $1) \
                                       OR erstellt_am < now() - make_interval(days => $2)",
    )
    .bind(SESSION_IDLE_HOURS)
    .bind(SESSION_MAX_DAYS)
    .execute(&mut *conn)
    .await?;
    // Eine Datenbank pro Administrator (ANF-026 AK-14): bevorzugt die bereite, sonst die zuletzt
    // begonnene (z. B. fehlgeschlagen – dann zeigt das Portal „Erneut versuchen“, AK-13)
    let erp_access: Option<bool> = sqlx::query_scalar(
        "WITH m AS ( \
             SELECT id, firmenname, status FROM mandant \
             WHERE besitzer_id = $2 ORDER BY (status = $3) DESC, id DESC LIMIT 1 \
         ), \
         s AS ( \
             INSERT INTO portal_sitzung (merkmal_hash, portal_benutzer_id, mandant_id) \
             VALUES ($1, $2, (SELECT id FROM m)) \
         ) \
         SELECT status = $3 AND firmenname IS NOT NULL FROM m",
    )
    .bind(token_hash)
    .bind(user_id)
    .bind(TENANT_READY)
    .fetch_optional(&mut *conn)
    .await?;
    sqlx::query("UPDATE portal_benutzer SET letzte_anmeldung = now() WHERE id = $1")
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    Ok(erp_access.unwrap_or(false))
}

/// Eine Sitzung beenden (Abmelden, ANF-007 AK-18).
pub async fn end_session(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM portal_sitzung WHERE merkmal_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await?;
    Ok(())
}
