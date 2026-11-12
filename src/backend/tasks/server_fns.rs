//! Server-Funktionen für die Navigation „Aufgaben“ (ANF-006).
//!
//! Erledigt/offen setzen läuft für beide Arten über `projects::server_fns::toggle_task` (AK-8).

use leptos::prelude::*;

use super::model::{SaveOutcome, StandaloneTaskInput, TaskItem, TaskLookup};

#[cfg(feature = "ssr")]
use super::{model::FieldErrors, repository};

/// Aufgaben des Benutzers mit Filtern (AK-3 bis AK-7).
/// `status`: „offen“, „erledigt“ oder „alle“; `art`: „mit“, „ohne“ oder „alle“.
#[server]
pub async fn list_tasks(
    benutzer_id: i64,
    status: String,
    art: String,
    suche: String,
) -> Result<Vec<TaskItem>, ServerFnError> {
    let pool = crate::db::pool()?;
    let status = match status.as_str() {
        "offen" | "erledigt" => status,
        _ => String::new(),
    };
    let art = match art.as_str() {
        "mit" | "ohne" => art,
        _ => String::new(),
    };
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    Ok(repository::list_for_user(&pool, benutzer_id, &status, &art, &suche, &pattern).await?)
}

/// Aufgabe unter `/aufgaben/<kennung>` (AK-14, AK-17).
#[server]
pub async fn get_task(id: i64) -> Result<TaskLookup, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(match repository::is_standalone(&pool, id).await? {
        Some(true) => match repository::find_standalone(&pool, id).await? {
            Some(task) => TaskLookup::Found(task),
            None => TaskLookup::Missing,
        },
        Some(false) => TaskLookup::InProject(id),
        None => TaskLookup::Missing,
    })
}

/// Neue Aufgabe ohne Projekt (AK-10 bis AK-12). Nur hier lassen sich solche Aufgaben anlegen.
/// Danach zurück zur Liste (`zurueck`), sonst zur Detailansicht.
#[server]
pub async fn create_task(
    input: StandaloneTaskInput,
    zurueck: Option<String>,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    if let Some(errors) = check_assignee(&pool, &input, None).await? {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let id = match repository::insert_standalone(&pool, &input).await {
        Ok(id) => id,
        Err(e) => return foreign_key_outcome(e),
    };
    let target = zurueck
        .as_deref()
        .and_then(crate::forms::safe_return_url)
        .map_or_else(|| super::model::task_url(id), str::to_string);
    leptos_axum::redirect(&crate::forms::with_params(
        &target,
        &[("gespeichert", "1".into())],
    ));
    Ok(SaveOutcome::Saved(id))
}

/// Aufgabe ohne Projekt ändern (AK-15).
#[server]
pub async fn update_task(
    id: i64,
    input: StandaloneTaskInput,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    if let Some(errors) = check_assignee(&pool, &input, Some(id)).await? {
        return Ok(SaveOutcome::Invalid(errors));
    }
    match repository::update_standalone(&pool, id, &input).await {
        Ok(0) => Err(ServerFnError::new("Aufgabe nicht gefunden.")),
        Ok(_) => {
            leptos_axum::redirect(&format!("{}?gespeichert=1", super::model::task_url(id)));
            Ok(SaveOutcome::Saved(id))
        }
        Err(e) => foreign_key_outcome(e),
    }
}

/// Aufgabe ohne Projekt löschen (AK-16); danach zur Aufgabenliste des zuständigen Benutzers.
#[server]
pub async fn delete_task(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    match repository::delete_standalone(&pool, id).await? {
        Some((name, user_id)) => {
            leptos_axum::redirect(&format!(
                "{}&geloescht={}",
                super::model::user_tasks_url(user_id),
                crate::forms::encode(&name)
            ));
            Ok(())
        }
        None => Err(ServerFnError::new("Aufgabe nicht gefunden.")),
    }
}

/// Zuständigen Benutzer prüfen (AK-12, AK-15): neu zuweisen nur an aktive Benutzer;
/// ein bereits zuständiger, inzwischen gesperrter Benutzer darf beim Bearbeiten bleiben.
#[cfg(feature = "ssr")]
async fn check_assignee(
    pool: &sqlx::PgPool,
    input: &StandaloneTaskInput,
    id: Option<i64>,
) -> Result<Option<FieldErrors>, ServerFnError> {
    let Some(user_id) = input.user_id() else {
        return Ok(None);
    };
    let message = match repository::user_active(pool, user_id).await? {
        Some(true) => return Ok(None),
        Some(false) => {
            let current = match id {
                Some(id) => repository::assignee_of(pool, id).await?,
                None => None,
            };
            if current == Some(user_id) {
                return Ok(None);
            }
            "Dieser Benutzer ist gesperrt. Bitte einen aktiven Benutzer wählen."
        }
        None => "Diesen Benutzer gibt es nicht mehr.",
    };
    let mut errors = FieldErrors::new();
    errors.insert("benutzer_id".into(), message.into());
    Ok(Some(errors))
}

/// Benutzer wurde zwischen Prüfung und Speichern gelöscht.
#[cfg(feature = "ssr")]
fn foreign_key_outcome(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    match &e {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => {
            let mut errors = FieldErrors::new();
            errors.insert(
                "benutzer_id".into(),
                "Diesen Benutzer gibt es nicht mehr.".into(),
            );
            Ok(SaveOutcome::Invalid(errors))
        }
        _ => Err(e.into()),
    }
}
