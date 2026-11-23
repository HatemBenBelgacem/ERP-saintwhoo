//! Mandanten-Datenbanken anlegen, migrieren und aufräumen (ANF-026 AK-11, AK-13, AK-15, AK-24; nur Server).
//!
//! Ablauf und Fehlerbehandlung; das SQL steht in `repository.rs` (Portal) bzw. `users::repository` (ERP).

use std::time::Instant;

use sqlx::{Connection, PgConnection, PgPool};

use super::model::{STATUS_CREATING, STATUS_READY, username_from_email};
use super::repository;
use crate::db::{ERP_MIGRATIONS, ERP_ROLE, TenantPools};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// Angaben des Administrators für den ersten Benutzer der neuen Datenbank (AK-15).
pub struct Owner {
    pub portal_id: i64,
    pub vorname: String,
    pub nachname: String,
    pub email: String,
}

/// Neue Mandanten-Datenbank vollständig einrichten: anlegen, Zugang beschränken, Migrationen ausführen,
/// Rechte der ERP-Rolle setzen und den Administrator als ersten Benutzer anlegen.
/// Bei einem Fehler wird die halb angelegte Datenbank wieder gelöscht (AK-13).
///
/// Läuft als eigene Aufgabe (`tokio::spawn`): Bricht der Browser die Anfrage ab, wird die Datenbank
/// trotzdem fertig erstellt bzw. aufgeräumt, statt halb angelegt liegen zu bleiben.
pub async fn create(
    portal: PgPool,
    tenants: TenantPools,
    database: String,
    owner: Owner,
) -> Result<(), String> {
    tokio::spawn(async move {
        create_inner(&portal, &tenants, &database, &owner)
            .await
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

async fn create_inner(
    portal: &PgPool,
    tenants: &TenantPools,
    database: &str,
    owner: &Owner,
) -> Result<(), BoxError> {
    let started = Instant::now();
    // Rest eines früheren, abgebrochenen Versuchs entfernen
    tenants.forget(database).await;
    repository::drop_database(portal, database).await?;

    match set_up(portal, tenants, database, owner).await {
        Ok(()) => {
            leptos::logging::log!(
                "Mandanten-Datenbank {database} erstellt ({} ms)",
                started.elapsed().as_millis()
            );
            Ok(())
        }
        Err(e) => {
            if let Err(cleanup) = repository::drop_database(portal, database).await {
                leptos::logging::error!("{database} konnte nicht entfernt werden: {cleanup}");
            }
            Err(e)
        }
    }
}

/// Die einzelnen Schritte von `create`; Fehler räumt `create` auf.
async fn set_up(
    portal: &PgPool,
    tenants: &TenantPools,
    database: &str,
    owner: &Owner,
) -> Result<(), BoxError> {
    repository::create_database(portal, database, ERP_ROLE).await?;
    let mut conn = PgConnection::connect_with(&tenants.admin_options(database)).await?;
    migrate(&mut conn).await?;
    let mut tx = conn.begin().await?;
    crate::backend::users::repository::create_owner(
        &mut tx,
        &owner.vorname,
        &owner.nachname,
        &owner.email,
        &username_from_email(&owner.email),
        owner.portal_id,
    )
    .await?;
    tx.commit().await?;
    conn.close().await?;
    Ok(())
}

/// Migrationen des ERP ausführen und die Rechte der ERP-Rolle nachführen (Verbindung der Eigentümerin).
async fn migrate(conn: &mut PgConnection) -> Result<(), BoxError> {
    // `run_direct` statt `run`: `run` ist über `Acquire<'a>` allgemein und lässt sich in Server-Funktionen
    // nicht als `Send` beweisen („not general enough“). sqlx stellt `run_direct` genau dafür bereit.
    ERP_MIGRATIONS.run_direct(None, &mut *conn, false).await?;
    repository::grant_erp_rights(conn, ERP_ROLE).await?;
    Ok(())
}

/// Beim Serverstart (AK-24): abgebrochene Erstellungen aufräumen und als fehlgeschlagen markieren,
/// dann ausstehende Migrationen in allen bereiten Mandanten-Datenbanken ausführen.
/// Ein Fehler beendet den Start – wie bisher bei der einen ERP-Datenbank.
pub async fn startup(portal: &PgPool, tenants: &TenantPools) -> Result<(), BoxError> {
    for (id, database) in repository::tenants_with_status(portal, STATUS_CREATING).await? {
        leptos::logging::log!("Abgebrochene Erstellung von {database} wird aufgeräumt");
        repository::drop_database(portal, &database).await?;
        repository::fail_interrupted(portal, id).await?;
    }
    for (_, database) in repository::tenants_with_status(portal, STATUS_READY).await? {
        let mut conn = PgConnection::connect_with(&tenants.admin_options(&database))
            .await
            .map_err(|e| format!("{database}: {e}"))?;
        migrate(&mut conn)
            .await
            .map_err(|e| format!("Migrationen in {database}: {e}"))?;
        conn.close().await?;
    }
    leptos::logging::log!(
        "{} Mandanten-Datenbank(en) bereit",
        repository::count_ready(portal).await?
    );
    Ok(())
}
