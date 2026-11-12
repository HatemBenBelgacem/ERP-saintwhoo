//! SQL-Zugriffe für Anmeldung und Sitzungen (ANF-007, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.

use sqlx::{PgConnection, PgPool};

use super::model::{CurrentUser, LOCK_MINUTES, SESSION_IDLE_HOURS, SESSION_MAX_DAYS};

/// Gültige Sitzung zum Merkmal-Hash: Benutzer aktiv, nicht länger als 12 Stunden untätig und nicht älter
/// als 7 Tage (AK-14, AK-15). „Zuletzt aktiv“ wird höchstens einmal pro Minute nachgeführt.
pub async fn user_for_session(
    pool: &PgPool,
    token_hash: &str,
) -> Result<Option<CurrentUser>, sqlx::Error> {
    sqlx::query_as(
        "WITH s AS ( \
             SELECT s.id, s.zuletzt_aktiv, b.id AS benutzer_id, b.benutzername, a.anzeigename AS name \
             FROM sitzung s JOIN benutzer b ON b.id = s.benutzer_id JOIN adresse a ON a.id = b.adresse_id \
             WHERE s.merkmal_hash = $1 AND b.aktiv \
               AND s.zuletzt_aktiv > now() - make_interval(hours => $2) \
               AND s.erstellt_am > now() - make_interval(days => $3) \
         ), \
         u AS ( \
             UPDATE sitzung SET zuletzt_aktiv = now() \
             WHERE id IN (SELECT id FROM s WHERE zuletzt_aktiv < now() - interval '1 minute') \
         ) \
         SELECT benutzer_id AS id, benutzername, name FROM s",
    )
    .bind(token_hash)
    .bind(SESSION_IDLE_HOURS)
    .bind(SESSION_MAX_DAYS)
    .fetch_optional(pool)
    .await
}

/// Anzahl Fehlversuche für den Benutzernamen in den letzten 15 Minuten (AK-11).
pub async fn recent_failures(pool: &PgPool, username: &str) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT count(*) FROM anmeldeversuch \
         WHERE benutzername = $1 AND zeitpunkt > now() - make_interval(mins => $2)",
    )
    .bind(username)
    .bind(LOCK_MINUTES)
    .fetch_one(pool)
    .await
}

/// Fehlversuch festhalten (auch für unbekannte Benutzernamen).
pub async fn record_failure(pool: &PgPool, username: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO anmeldeversuch (benutzername) VALUES ($1)")
        .bind(username)
        .execute(pool)
        .await?;
    Ok(())
}

/// (Kennung, Passwort-Hash, aktiv) zum Benutzernamen, ohne Beachtung der Gross-/Kleinschreibung.
pub async fn credentials(
    pool: &PgPool,
    username: &str,
) -> Result<Option<(i64, String, bool)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, passwort_hash, aktiv FROM benutzer WHERE lower(benutzername) = lower($1)",
    )
    .bind(username)
    .fetch_optional(pool)
    .await
}

/// Nach erfolgreicher Anmeldung: Fehlversuche des Namens löschen, abgelaufene Sitzungen und alte
/// Fehlversuche aufräumen (AK-23), neue Sitzung anlegen und „Letzte Anmeldung“ setzen (AK-10).
pub async fn start_session(
    conn: &mut PgConnection,
    user_id: i64,
    username: &str,
    token_hash: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM anmeldeversuch WHERE benutzername = $1 OR zeitpunkt < now() - interval '1 day'")
        .bind(username)
        .execute(&mut *conn)
        .await?;
    sqlx::query(
        "DELETE FROM sitzung WHERE zuletzt_aktiv < now() - make_interval(hours => $1) \
                                OR erstellt_am < now() - make_interval(days => $2)",
    )
    .bind(SESSION_IDLE_HOURS)
    .bind(SESSION_MAX_DAYS)
    .execute(&mut *conn)
    .await?;
    sqlx::query("INSERT INTO sitzung (merkmal_hash, benutzer_id) VALUES ($1, $2)")
        .bind(token_hash)
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    sqlx::query("UPDATE benutzer SET letzte_anmeldung = now() WHERE id = $1")
        .bind(user_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Eine Sitzung beenden (Abmelden, AK-18).
pub async fn end_session(pool: &PgPool, token_hash: &str) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sitzung WHERE merkmal_hash = $1")
        .bind(token_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// Alle Sitzungen eines Benutzers beenden (gesperrt oder Passwort neu gesetzt, AK-15).
pub async fn end_all_sessions(pool: &PgPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sitzung WHERE benutzer_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Alle Sitzungen eines Benutzers beenden ausser der mit `keep_hash` (Passwort neu gesetzt, AK-15).
pub async fn end_sessions_except(
    pool: &PgPool,
    user_id: i64,
    keep_hash: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sitzung WHERE benutzer_id = $1 AND merkmal_hash IS DISTINCT FROM $2")
        .bind(user_id)
        .bind(keep_hash)
        .execute(pool)
        .await?;
    Ok(())
}

/// `true`, solange es noch keinen Benutzer gibt (Ersteinrichtung, AK-21).
pub async fn no_users(pool: &PgPool) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT NOT EXISTS (SELECT 1 FROM benutzer)")
        .fetch_one(pool)
        .await
}

/// Ersteinrichtung in der Transaktion des Aufrufers: Tabelle sperren und prüfen, dass es noch keinen
/// Benutzer gibt (AK-22); dann Person und Benutzer anlegen. Liefert die Kennung oder `None`, wenn es
/// inzwischen schon einen Benutzer gibt.
pub async fn create_first_user(
    conn: &mut PgConnection,
    vorname: &str,
    nachname: &str,
    email: Option<&str>,
    username: &str,
    password_hash: &str,
) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query("LOCK TABLE benutzer IN SHARE ROW EXCLUSIVE MODE")
        .execute(&mut *conn)
        .await?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM benutzer)")
        .fetch_one(&mut *conn)
        .await?;
    if exists {
        return Ok(None);
    }
    let address_id: i64 = sqlx::query_scalar(
        "INSERT INTO adresse (typ, vorname, nachname, email) VALUES ('person', $1, $2, $3) RETURNING id",
    )
    .bind(vorname)
    .bind(nachname)
    .bind(email)
    .fetch_one(&mut *conn)
    .await?;
    let id = sqlx::query_scalar(
        "INSERT INTO benutzer (benutzername, adresse_id, passwort_hash, aktiv) \
         VALUES ($1, $2, $3, true) RETURNING id",
    )
    .bind(username)
    .bind(address_id)
    .bind(password_hash)
    .fetch_one(&mut *conn)
    .await?;
    Ok(Some(id))
}
