//! Server-Funktionen für den Geländeplan eines Projekts (ANF-027).
//!
//! Prüfen den Plan, steuern die Transaktion und rufen das Repository auf.

use leptos::prelude::*;
use server_fn::codec::Json;

use super::model::{SitePlan, SitePlanPage};
use crate::forms::SaveOutcome;

#[cfg(feature = "ssr")]
use super::repository;

/// Projektkopf und gespeicherter Plan; `None`, wenn es das Projekt nicht gibt (AK-1).
#[server]
pub async fn get_site_plan(projekt_id: i64) -> Result<Option<SitePlanPage>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some((projekt_nummer, projekt_name)) = repository::project_head(&pool, projekt_id).await?
    else {
        return Ok(None);
    };
    let found = repository::find(&pool, projekt_id).await?;
    let kunde_adresse = repository::customer_address(&pool, projekt_id).await?;
    let (plan, geaendert_am) = match found {
        Some((plan, changed)) => (Some(plan), Some(changed)),
        None => (None, None),
    };
    Ok(Some(SitePlanPage {
        projekt_id,
        projekt_nummer,
        projekt_name,
        plan,
        kunde_adresse,
        geaendert_am,
    }))
}

/// Plan als Ganzes speichern (AK-10). Ungültige Pläne werden mit Meldungen abgewiesen (AK-11).
/// Als JSON übertragen, weil der Plan verschachtelte Listen enthält.
#[server(input = Json)]
pub async fn save_site_plan(projekt_id: i64, plan: SitePlan) -> Result<SaveOutcome, ServerFnError> {
    let errors = plan.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    if repository::project_head(&pool, projekt_id).await?.is_none() {
        return Err(ServerFnError::new(
            "Das Projekt gibt es nicht mehr. Der Plan wurde nicht gespeichert.",
        ));
    }
    let mut tx = pool.begin().await?;
    repository::save(&mut tx, projekt_id, &plan).await?;
    tx.commit().await?;
    Ok(SaveOutcome::Saved(projekt_id))
}

/// Plan des Projekts löschen (AK-13); danach zurück zum Projekt.
#[server]
pub async fn delete_site_plan(projekt_id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    repository::delete(&pool, projekt_id).await?;
    leptos_axum::redirect(&format!(
        "/projekte/{projekt_id}?geloescht=Gel%C3%A4ndeplan"
    ));
    Ok(())
}
