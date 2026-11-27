//! Server-Funktionen für Projekte (ANF-003).
//!
//! Prüfen die Eingaben, steuern Transaktionen, rufen das Repository auf, übersetzen
//! Datenbankfehler in verständliche Meldungen und leiten nach dem Speichern weiter.

use leptos::prelude::*;

use super::model::{ElementInput, Level, PickOption, ProjectPage, ProjectTree, SaveOutcome};

#[cfg(feature = "ssr")]
use super::repository;

/// Projektliste mit Suche, Statusfilter, Sortierung und Seiten (AK-1 bis AK-3).
/// `status`: leer = alle Status.
#[server]
pub async fn list_projects(
    suche: String,
    status: String,
    sortierung: String,
    richtung: String,
    seite: i64,
) -> Result<ProjectPage, ServerFnError> {
    use super::model::PAGE_SIZE;

    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);

    let total = repository::count_projects(&pool, &suche, &pattern, &status).await?;
    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = seite.clamp(1, page_count);
    let mut rows = repository::list_projects_page(
        &pool,
        &suche,
        &pattern,
        &status,
        &sortierung,
        richtung == "auf",
        PAGE_SIZE,
        (page - 1) * PAGE_SIZE,
    )
    .await?;

    // Struktur aller Projekte der Seite für die Übersicht
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut children = repository::load_children(&pool, &ids).await?;
    for row in &mut rows {
        row.children = children.remove(&row.id).unwrap_or_default();
    }

    Ok(ProjectPage { rows, total, page })
}

/// Vollständiges Projekt, zu dem das Element gehört (AK-4, AK-6, AK-17). `None`, wenn es das Element nicht gibt.
#[server]
pub async fn get_tree(level: Level, id: i64) -> Result<Option<ProjectTree>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(project_id) = repository::project_id_of(&pool, level, id).await? else {
        return Ok(None);
    };
    Ok(repository::load_tree(&pool, project_id).await?)
}

/// Projekt-Auswahl (z. B. im Einkaufsauftrag, ANF-004): nach Nummer oder Name, neueste zuerst.
#[server]
pub async fn search_projects(suche: String) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    Ok(repository::search_options(&pool, &suche, &pattern).await?)
}

/// Element anlegen (AK-7 bis AK-10). Projekte erhalten eine Nummer P-JJJJ-NNN,
/// andere Elemente werden ans Ende ihres übergeordneten Elements gestellt.
#[server]
pub async fn create_element(
    level: Level,
    parent_id: Option<i64>,
    input: ElementInput,
    zurueck: Option<String>,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized(level);
    let errors = input.validate(level);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    if let Some(errors) = check_responsible(&pool, level, None, &input).await? {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let mut tx = pool.begin().await?;

    let id = match repository::insert_element(&mut tx, level, parent_id, &input).await {
        Ok(id) => id,
        Err(e) => return foreign_key_outcome(level, e),
    };
    tx.commit().await?;
    redirect_after_save(level, id, zurueck.as_deref());
    Ok(SaveOutcome::Saved(id))
}

/// Nach dem Speichern zurück zur Seite unter dem Modal-Fenster (`zurueck`), sonst zur Detailansicht.
#[cfg(feature = "ssr")]
fn redirect_after_save(level: Level, id: i64, zurueck: Option<&str>) {
    let target = zurueck
        .and_then(crate::forms::safe_return_url)
        .map_or_else(|| level.url(id), str::to_string);
    leptos_axum::redirect(&crate::forms::with_params(
        &target,
        &[("gespeichert", "1".into())],
    ));
}

/// Verantwortlichen Benutzer prüfen (ANF-005 AK-24, AK-33): neu hinterlegen lassen sich nur aktive
/// Benutzer; ein bereits hinterlegter, inzwischen gesperrter Benutzer darf beim Bearbeiten bleiben.
/// `id`: Kennung beim Bearbeiten, `None` beim Anlegen.
#[cfg(feature = "ssr")]
async fn check_responsible(
    pool: &sqlx::PgPool,
    level: Level,
    id: Option<i64>,
    input: &ElementInput,
) -> Result<Option<super::model::FieldErrors>, ServerFnError> {
    let Some(user_id) = input.responsible_id() else {
        return Ok(None);
    };
    let message = match repository::user_active(pool, user_id).await? {
        Some(true) => return Ok(None),
        Some(false) => {
            let current = match id {
                Some(id) => repository::responsible_of(pool, level, id).await?,
                None => None,
            };
            if current == Some(user_id) {
                return Ok(None);
            }
            "Dieser Benutzer ist gesperrt. Bitte einen aktiven Benutzer wählen."
        }
        None => "Diesen Benutzer gibt es nicht mehr.",
    };
    let mut errors = super::model::FieldErrors::new();
    errors.insert("benutzer_id".into(), message.into());
    Ok(Some(errors))
}

/// Fremdschlüssel-Fehler beim Speichern verständlich melden.
#[cfg(feature = "ssr")]
fn foreign_key_outcome(level: Level, e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    match &e {
        // Benutzer wurde zwischen Prüfung und Speichern gelöscht (ANF-005)
        sqlx::Error::Database(db)
            if db.code().as_deref() == Some("23503")
                && db.constraint().is_some_and(|c| c.contains("benutzer_id")) =>
        {
            let mut errors = super::model::FieldErrors::new();
            errors.insert(
                "benutzer_id".into(),
                "Diesen Benutzer gibt es nicht mehr.".into(),
            );
            Ok(SaveOutcome::Invalid(errors))
        }
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => {
            if level == Level::Project {
                let mut errors = super::model::FieldErrors::new();
                errors.insert(
                    "adresse_id".into(),
                    "Diese Adresse gibt es nicht mehr.".into(),
                );
                Ok(SaveOutcome::Invalid(errors))
            } else {
                Err(ServerFnError::new(
                    "Das übergeordnete Element gibt es nicht mehr.",
                ))
            }
        }
        _ => Err(e.into()),
    }
}

/// Element ändern (AK-11).
#[server]
pub async fn update_element(
    level: Level,
    id: i64,
    input: ElementInput,
    zurueck: Option<String>,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized(level);
    let errors = input.validate(level);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    if let Some(errors) = check_responsible(&pool, level, Some(id), &input).await? {
        return Ok(SaveOutcome::Invalid(errors));
    }

    match repository::update_element(&pool, level, id, &input).await {
        Ok(0) => Err(ServerFnError::new(format!(
            "{} nicht gefunden.",
            level.label()
        ))),
        Ok(_) => {
            redirect_after_save(level, id, zurueck.as_deref());
            Ok(SaveOutcome::Saved(id))
        }
        Err(e) => foreign_key_outcome(level, e),
    }
}

/// Element mit allen untergeordneten Elementen löschen (AK-14, AK-15).
/// Ein einziger DELETE-Befehl; die Datenbank löscht über ON DELETE CASCADE alles darunter – ganz oder gar nicht.
#[server]
pub async fn delete_element(level: Level, id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let target = if level == Level::Project {
        let nummer = match repository::delete_project(&pool, id).await {
            Ok(n) => n,
            // 23503: noch Einkaufsaufträge zugeordnet (ANF-004 AK-23)
            Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
                let orders = repository::order_numbers_of_project(&pool, id).await?;
                // ANF-010 AK-9: Buchungszeilen mit diesem Projekt nennen
                let mut verwendet = Vec::new();
                if !orders.is_empty() {
                    verwendet.push(format!("Einkaufsaufträgen ({})", orders.join(", ")));
                }
                if db.table() == Some("buchungszeile") {
                    verwendet.push("Buchungen".to_string());
                }
                // ANF-018 AK-16: Ausgangsrechnungen mit Nummer nennen
                let invoices = repository::invoice_numbers_of_project(&pool, id).await?;
                if !invoices.is_empty() {
                    verwendet.push(format!("Ausgangsrechnungen ({})", invoices.join(", ")));
                }
                let list = if verwendet.is_empty() {
                    "anderen Datensätzen".to_string()
                } else {
                    verwendet.join(" und ")
                };
                return Err(ServerFnError::new(format!(
                    "Das Projekt wird noch in {list} verwendet und kann nicht gelöscht werden."
                )));
            }
            Err(e) => return Err(e.into()),
        };
        nummer.map(|n| format!("/projekte?geloescht={}", crate::forms::encode(&n)))
    } else {
        let row = repository::delete_child(&pool, level, id).await?;
        row.map(|(parent, name)| {
            let parent_level = level.parent().unwrap_or(Level::Project);
            format!(
                "{}?geloescht={}",
                parent_level.url(parent),
                crate::forms::encode(&name)
            )
        })
    };
    match target {
        Some(url) => {
            leptos_axum::redirect(&url);
            Ok(())
        }
        None => Err(ServerFnError::new(format!(
            "{} nicht gefunden.",
            level.label()
        ))),
    }
}

/// Aufgabe direkt im Baum erledigen bzw. wieder öffnen (AK-12).
#[server]
pub async fn toggle_task(id: i64, erledigt: bool) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    repository::set_task_done(&pool, id, erledigt).await?;
    Ok(())
}

/// Element innerhalb seines übergeordneten Elements eine Stelle nach oben/unten schieben (AK-13).
#[server]
pub async fn move_element(level: Level, id: i64, nach_oben: bool) -> Result<(), ServerFnError> {
    if level == Level::Project {
        return Err(ServerFnError::new("Projekte haben keine Reihenfolge."));
    }
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    if !repository::move_element(&mut tx, level, id, nach_oben).await? {
        return Err(ServerFnError::new(format!(
            "{} nicht gefunden.",
            level.label()
        )));
    }
    tx.commit().await?;
    Ok(())
}
