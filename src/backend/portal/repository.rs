//! SQL-Zugriffe für Mandanten und Mandanten-Datenbanken (ANF-026, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! `CREATE DATABASE`, `GRANT` und `ALTER ROLE` erlauben keine Parameter (`$1`). Datenbanknamen gelangen
//! deshalb nur nach `is_valid_database_name` in den SQL-Text, Rollen nur als feste Texte und Passwörter
//! nur als Hexadezimal-Zufallswert (`AssertSqlSafe`, siehe CLAUDE.md).

use sqlx::{AssertSqlSafe, ConnectOptions, Connection, PgConnection, PgPool};

use super::model::{STATUS_CREATING, STATUS_FAILED, STATUS_READY, is_valid_database_name};

/// Fehler für einen Namen, der nicht dem erzeugten Muster entspricht – darf nie in SQL gelangen.
fn invalid_name(name: &str) -> sqlx::Error {
    sqlx::Error::Protocol(format!("Ungültiger Datenbankname: {name:?}"))
}

fn checked(name: &str) -> Result<&str, sqlx::Error> {
    if is_valid_database_name(name) {
        Ok(name)
    } else {
        Err(invalid_name(name))
    }
}

/// Rollen- und Passwort-Text prüfen: nur Kleinbuchstaben, Ziffern und `_` bzw. Hexadezimalzeichen.
fn checked_role(role: &str) -> Result<&str, sqlx::Error> {
    if !role.is_empty()
        && role
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        Ok(role)
    } else {
        Err(sqlx::Error::Protocol("Ungültiger Rollenname".into()))
    }
}

// ------------------------------------------------------------------------------- Rolle (Serverstart)

/// Rolle für den ERP-Betrieb anlegen bzw. ihr ein neues Passwort geben (ANF-026 AK-21): Anmeldung erlaubt,
/// aber keine Datenbanken, keine Rollen, kein Superuser. Die Portal-Datenbank bleibt für sie gesperrt.
/// Läuft auf einer eigenen Verbindung ohne Protokollierung der Befehle, damit das Passwort in keinem
/// Protokoll des Servers erscheint.
pub async fn ensure_erp_role(
    portal: &PgPool,
    role: &str,
    password: &str,
) -> Result<(), sqlx::Error> {
    let role = checked_role(role)?;
    if password.len() < 32 || !password.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(sqlx::Error::Protocol("Ungültiges Rollenpasswort".into()));
    }
    let options = portal
        .connect_options()
        .as_ref()
        .clone()
        .disable_statement_logging();
    let mut conn = PgConnection::connect_with(&options).await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = $1)")
            .bind(role)
            .fetch_one(&mut conn)
            .await?;
    if !exists {
        // Gleichzeitig startender zweiter Server: „existiert bereits“ ist kein Fehler
        if let Err(e) = sqlx::query(AssertSqlSafe(format!("CREATE ROLE {role}")))
            .execute(&mut conn)
            .await
            && e.as_database_error()
                .is_none_or(|d| d.code().as_deref() != Some("42710"))
        {
            return Err(e);
        }
    }
    sqlx::query(AssertSqlSafe(format!(
        "ALTER ROLE {role} WITH LOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION PASSWORD '{password}'"
    )))
    .execute(&mut conn)
    .await?;
    let portal_db: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut conn)
        .await?;
    // Portal-Datenbank: nur die Portal-Rolle (Eigentümerin) darf sich verbinden
    let portal_db = portal_db.replace('"', "\"\"");
    sqlx::query(AssertSqlSafe(format!(
        "REVOKE CONNECT, TEMPORARY ON DATABASE \"{portal_db}\" FROM PUBLIC"
    )))
    .execute(&mut conn)
    .await?;
    conn.close().await
}

// ------------------------------------------------------------------------------- Verzeichnis (Portal)

/// Zeile des Administrators sperren, damit zwei gleichzeitige Anfragen nacheinander entscheiden (AK-14).
pub async fn lock_owner(conn: &mut PgConnection, owner_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT id FROM portal_benutzer WHERE id = $1 FOR UPDATE")
        .bind(owner_id)
        .execute(&mut *conn)
        .await?;
    Ok(())
}

/// Mandanten des Administrators: (Kennung, Status).
pub async fn tenants_of(
    conn: &mut PgConnection,
    owner_id: i64,
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    sqlx::query_as("SELECT id, status FROM mandant WHERE besitzer_id = $1 ORDER BY id")
        .bind(owner_id)
        .fetch_all(&mut *conn)
        .await
}

/// Neuen Mandanten mit Status „wird erstellt“ anlegen; liefert Kennung und erzeugten Datenbanknamen.
pub async fn insert_tenant(
    conn: &mut PgConnection,
    owner_id: i64,
) -> Result<(i64, String), sqlx::Error> {
    sqlx::query_as(
        "INSERT INTO mandant (besitzer_id, status) VALUES ($1, $2) RETURNING id, datenbankname",
    )
    .bind(owner_id)
    .bind(STATUS_CREATING)
    .fetch_one(&mut *conn)
    .await
}

/// Fehlgeschlagenen Mandanten erneut auf „wird erstellt“ setzen; liefert den Datenbanknamen (AK-13).
pub async fn restart_tenant(conn: &mut PgConnection, id: i64) -> Result<String, sqlx::Error> {
    sqlx::query_scalar(
        "UPDATE mandant SET status = $2, geaendert_am = now() WHERE id = $1 RETURNING datenbankname",
    )
    .bind(id)
    .bind(STATUS_CREATING)
    .fetch_one(&mut *conn)
    .await
}

/// Status eines Mandanten setzen.
pub async fn set_status(pool: &PgPool, id: i64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE mandant SET status = $2, geaendert_am = now() WHERE id = $1")
        .bind(id)
        .bind(status)
        .execute(pool)
        .await?;
    Ok(())
}

/// Den Mandanten in allen Sitzungen seines Besitzers wählen (nach dem Erstellen).
pub async fn select_in_sessions(pool: &PgPool, id: i64, owner_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE portal_sitzung SET mandant_id = $1 WHERE portal_benutzer_id = $2")
        .bind(id)
        .bind(owner_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Vorname, Nachname und E-Mail eines Zugangs (für den ersten Benutzer der Mandanten-Datenbank, AK-15).
pub async fn owner_details(
    pool: &PgPool,
    owner_id: i64,
) -> Result<(String, String, String), sqlx::Error> {
    sqlx::query_as("SELECT vorname, nachname, email FROM portal_benutzer WHERE id = $1")
        .bind(owner_id)
        .fetch_one(pool)
        .await
}

/// Firmennamen setzen: der Mandant ist damit definiert (ANF-026 AK-18). Nur für den Besitzer, nur bei bereiter
/// Datenbank und nur einmal; liefert die Anzahl geänderter Zeilen (0 = nicht erlaubt bzw. schon definiert).
pub async fn define_tenant(
    pool: &PgPool,
    id: i64,
    owner_id: i64,
    company_name: &str,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE mandant SET firmenname = $3, geaendert_am = now() \
         WHERE id = $1 AND besitzer_id = $2 AND status = $4 AND firmenname IS NULL",
    )
    .bind(id)
    .bind(owner_id)
    .bind(company_name)
    .bind(STATUS_READY)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Mandanten mit dem Status `status`: (Kennung, Datenbankname).
pub async fn tenants_with_status(
    pool: &PgPool,
    status: &str,
) -> Result<Vec<(i64, String)>, sqlx::Error> {
    sqlx::query_as("SELECT id, datenbankname FROM mandant WHERE status = $1 ORDER BY id")
        .bind(status)
        .fetch_all(pool)
        .await
}

/// Beim Start abgebrochene Erstellungen als fehlgeschlagen markieren (Server während des Erstellens beendet).
pub async fn fail_interrupted(pool: &PgPool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE mandant SET status = $2, geaendert_am = now() WHERE id = $1 AND status = $3",
    )
    .bind(id)
    .bind(STATUS_FAILED)
    .bind(STATUS_CREATING)
    .execute(pool)
    .await?;
    Ok(())
}

/// Bereite Mandanten zählen (Protokoll beim Start).
pub async fn count_ready(pool: &PgPool) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM mandant WHERE status = $1")
        .bind(STATUS_READY)
        .fetch_one(pool)
        .await
}

// ------------------------------------------------------------------------------- Datenbanken anlegen

/// Leere Datenbank anlegen und den Zugang beschränken: nur die Portal-Rolle (Eigentümerin) und die
/// ERP-Rolle dürfen sich verbinden (AK-11, AK-21). Eigentümerin ist die Rolle der Portal-Verbindung.
pub async fn create_database(
    portal: &PgPool,
    name: &str,
    erp_role: &str,
) -> Result<(), sqlx::Error> {
    let name = checked(name)?;
    let role = checked_role(erp_role)?;
    sqlx::query(AssertSqlSafe(format!("CREATE DATABASE \"{name}\"")))
        .execute(portal)
        .await?;
    sqlx::query(AssertSqlSafe(format!(
        "REVOKE CONNECT, TEMPORARY ON DATABASE \"{name}\" FROM PUBLIC"
    )))
    .execute(portal)
    .await?;
    sqlx::query(AssertSqlSafe(format!(
        "GRANT CONNECT ON DATABASE \"{name}\" TO {role}"
    )))
    .execute(portal)
    .await?;
    Ok(())
}

/// Datenbank löschen, auch wenn noch Verbindungen offen sind (Aufräumen nach einem Fehler, AK-13).
pub async fn drop_database(portal: &PgPool, name: &str) -> Result<(), sqlx::Error> {
    let name = checked(name)?;
    sqlx::query(AssertSqlSafe(format!(
        "DROP DATABASE IF EXISTS \"{name}\" WITH (FORCE)"
    )))
    .execute(portal)
    .await?;
    Ok(())
}

/// Rechte der ERP-Rolle in einer Mandanten-Datenbank (auf einer Verbindung der Eigentümerin): Daten lesen
/// und schreiben, Folgen (Kennungen) verwenden – aber keine Tabellen anlegen oder ändern und keinen Zugriff
/// auf die Liste der Migrationen. Nach jeder Migration erneut ausführen, damit neue Tabellen dazukommen.
pub async fn grant_erp_rights(conn: &mut PgConnection, erp_role: &str) -> Result<(), sqlx::Error> {
    let role = checked_role(erp_role)?;
    for sql in [
        "REVOKE CREATE ON SCHEMA public FROM PUBLIC".to_string(),
        format!("GRANT USAGE ON SCHEMA public TO {role}"),
        format!("GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO {role}"),
        format!("GRANT USAGE, SELECT, UPDATE ON ALL SEQUENCES IN SCHEMA public TO {role}"),
        format!("REVOKE ALL ON TABLE _sqlx_migrations FROM {role}"),
    ] {
        sqlx::query(AssertSqlSafe(sql)).execute(&mut *conn).await?;
    }
    Ok(())
}
