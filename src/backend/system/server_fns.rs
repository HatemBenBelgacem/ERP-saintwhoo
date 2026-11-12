//! Server-Funktionen für den Systemzustand.

use leptos::prelude::*;

/// Liefert die PostgreSQL-Version – prüft, ob die Datenbank erreichbar ist.
#[server]
pub async fn db_status() -> Result<String, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(super::repository::database_version(&pool).await?)
}
