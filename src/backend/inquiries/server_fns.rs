//! Server-Funktionen für Anfragen (ANF-029).
//!
//! Titel und Beschreibung der Anfrage werden auf dem Server aus der Datenbank gelesen, nicht aus dem
//! Formular übernommen. Das SQL steht in `repository.rs`.

use leptos::prelude::*;

use super::model::InquiryRow;

#[cfg(feature = "ssr")]
use super::repository;

/// Anfragen eines Vorgangs, neueste zuerst (AK-4).
#[server]
pub async fn list_inquiries(vorgang_id: i64) -> Result<Vec<InquiryRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::list_by_vorgang(&pool, vorgang_id).await?)
}

/// Anfrage zum Vorgang anlegen: Titel = Name des Teilprojekts, Beschreibung = Beschreibung des Vorgangs (AK-2, AK-3).
#[server]
pub async fn create_inquiry(vorgang_id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let Some((titel, beschreibung)) = repository::source_of(&mut tx, vorgang_id).await? else {
        return Err(ServerFnError::new("Vorgang nicht gefunden."));
    };
    repository::insert_inquiry(&mut tx, vorgang_id, &titel, beschreibung.as_deref()).await?;
    tx.commit().await?;
    Ok(())
}

/// Anfrage löschen (AK-6). Der Vorgang bleibt unverändert.
#[server]
pub async fn delete_inquiry(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    if repository::delete_inquiry(&pool, id).await? {
        Ok(())
    } else {
        Err(ServerFnError::new("Anfrage nicht gefunden."))
    }
}
