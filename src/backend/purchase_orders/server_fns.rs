//! Server-Funktionen für Einkaufsaufträge (ANF-004).
//!
//! Jede Änderung sperrt den Einkaufsauftrag (`FOR UPDATE`) und prüft seinen Status auf dem Server,
//! damit z. B. ein bestellter Auftrag auch über direkte Aufrufe nicht mehr geändert werden kann.
//! Das SQL steht in `repository.rs`; hier stehen Prüfungen, Transaktionen, Meldungen und Weiterleitungen.

use leptos::prelude::*;

use super::model::{OrderDetail, OrderInput, OrderPage, OrderRow, PositionInput, SaveOutcome};

#[cfg(feature = "ssr")]
use super::repository;
#[cfg(feature = "ssr")]
use sqlx::PgConnection;

/// Liste mit Suche, Statusfilter, Sortierung und Seiten (AK-1 bis AK-5).
/// `status`: „offen“, „alle“ oder ein einzelner Status.
#[server]
pub async fn list_orders(
    suche: String,
    status: String,
    sortierung: String,
    richtung: String,
    seite: i64,
) -> Result<OrderPage, ServerFnError> {
    use super::model::PAGE_SIZE;

    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);

    let total = repository::count_orders(&pool, &suche, &pattern, &status).await?;
    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = seite.clamp(1, page_count);
    let rows = repository::list_orders_page(
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

    Ok(OrderPage { rows, total, page })
}

/// Einkaufsaufträge eines Projekts (AK-22), neueste zuerst.
#[server]
pub async fn list_project_orders(projekt_id: i64) -> Result<Vec<OrderRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::list_by_project(&pool, projekt_id).await?)
}

/// Ein Einkaufsauftrag mit Positionen (AK-6, AK-7, AK-24). `None`, wenn es ihn nicht gibt.
#[server]
pub async fn get_order(id: i64) -> Result<Option<OrderDetail>, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut conn = pool.acquire().await?;
    Ok(repository::load_detail(&mut conn, id).await?)
}

// ---------- Hilfen: Sperren, Statusregeln, Fehler ----------

#[cfg(feature = "ssr")]
fn not_found() -> ServerFnError {
    ServerFnError::new("Einkaufsauftrag nicht gefunden.")
}

/// Sperrt den Einkaufsauftrag bis zum Ende der Transaktion und liefert seinen Status.
#[cfg(feature = "ssr")]
async fn lock_order(conn: &mut PgConnection, id: i64) -> Result<String, ServerFnError> {
    repository::lock_order(conn, id)
        .await?
        .ok_or_else(not_found)
}

/// Einkaufsauftrag einer Position sperren: (Kennung des Auftrags, Status).
#[cfg(feature = "ssr")]
async fn lock_order_of_position(
    conn: &mut PgConnection,
    position_id: i64,
) -> Result<(i64, String), ServerFnError> {
    repository::lock_order_of_position(conn, position_id)
        .await?
        .ok_or_else(|| ServerFnError::new("Position nicht gefunden."))
}

/// Positionen und Lieferant sind nur im Entwurf änderbar (AK-13, AK-15).
#[cfg(feature = "ssr")]
fn require_draft(status: &str) -> Result<(), ServerFnError> {
    if status == "entwurf" {
        Ok(())
    } else {
        Err(ServerFnError::new(
            "Der Einkaufsauftrag ist nicht mehr im Entwurf; Positionen können nicht mehr geändert werden.",
        ))
    }
}

/// Wareneingang nur bei bestellten, noch nicht vollständig gelieferten Aufträgen (AK-16).
#[cfg(feature = "ssr")]
fn require_ordered(status: &str) -> Result<(), ServerFnError> {
    if matches!(status, "bestellt" | "teilweise_geliefert") {
        Ok(())
    } else {
        Err(ServerFnError::new(
            "Liefermengen können nur bei bestellten Aufträgen erfasst werden.",
        ))
    }
}

/// Fehlercode der Datenbank und Name der verletzten Regel.
#[cfg(feature = "ssr")]
fn db_error(e: &sqlx::Error) -> Option<(String, String)> {
    match e {
        sqlx::Error::Database(db) => Some((
            db.code().map(|c| c.to_string()).unwrap_or_default(),
            db.constraint().unwrap_or_default().to_string(),
        )),
        _ => None,
    }
}

/// Fehler bei Lieferant oder Projekt (gelöscht, während das Formular offen war) als Feldfehler melden.
#[cfg(feature = "ssr")]
fn reference_outcome(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    match db_error(&e) {
        Some((code, constraint)) if code == "23503" => {
            let mut errors = super::model::FieldErrors::new();
            if constraint.contains("projekt") {
                errors.insert(
                    "projekt_id".into(),
                    "Dieses Projekt gibt es nicht mehr.".into(),
                );
            } else {
                errors.insert(
                    "adresse_id".into(),
                    "Diese Adresse gibt es nicht mehr.".into(),
                );
            }
            Ok(SaveOutcome::Invalid(errors))
        }
        _ => Err(e.into()),
    }
}

#[cfg(feature = "ssr")]
fn redirect_to_order(id: i64) {
    leptos_axum::redirect(&format!("/einkauf/{id}?gespeichert=1"));
}

// ---------- Auftrag ----------

/// Neuen Einkaufsauftrag im Status „Entwurf“ anlegen, Nummer E-JJJJ-NNNN (AK-8 bis AK-10, AK-12).
#[server]
pub async fn create_order(input: OrderInput) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let id = match repository::insert_order(&mut tx, &input).await {
        Ok(id) => id,
        Err(e) => return reference_outcome(e),
    };
    tx.commit().await?;
    redirect_to_order(id);
    Ok(SaveOutcome::Saved(id))
}

/// Kopfdaten ändern (AK-13, AK-15). Nach dem Bestellen werden nur Liefertermin, Referenz und Bemerkung übernommen.
#[server]
pub async fn update_order(id: i64, input: OrderInput) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    lock_order(&mut tx, id).await?;
    let current = repository::load_detail(&mut tx, id)
        .await?
        .ok_or_else(not_found)?
        .order;

    let input = input.normalized();
    let input = if current.is_draft() {
        input
    } else if current.is_ordered() {
        input.restricted_to_ordered(&current.to_input())
    } else {
        return Err(ServerFnError::new(
            "Der Einkaufsauftrag ist abgeschlossen und kann nicht mehr geändert werden.",
        ));
    };
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    if let Err(e) = repository::update_order(&mut tx, id, &input).await {
        return reference_outcome(e);
    }
    tx.commit().await?;
    redirect_to_order(id);
    Ok(SaveOutcome::Saved(id))
}

/// Bestellen: Entwurf → Bestellt, Bestelldatum = heute, falls leer (AK-14).
#[server]
pub async fn place_order(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    if lock_order(&mut tx, id).await? != "entwurf" {
        return Err(ServerFnError::new(
            "Der Einkaufsauftrag wurde bereits bestellt.",
        ));
    }
    if repository::count_positions(&mut tx, id).await? == 0 {
        return Err(ServerFnError::new(
            "Ohne Positionen kann nicht bestellt werden. Bitte zuerst eine Position erfassen.",
        ));
    }
    match repository::mark_ordered(&mut tx, id).await {
        Ok(()) => {}
        Err(e) if db_error(&e).is_some_and(|(code, _)| code == "23514") => {
            return Err(ServerFnError::new(
                "Der Liefertermin liegt vor dem Bestelldatum (heute). Bitte zuerst den Liefertermin anpassen.",
            ));
        }
        Err(e) => return Err(e.into()),
    }
    tx.commit().await?;
    Ok(())
}

/// Stornieren (AK-19): nur im Entwurf oder bestellt, solange nichts geliefert ist.
#[server]
pub async fn cancel_order(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    lock_order(&mut tx, id).await?;
    let detail = repository::load_detail(&mut tx, id)
        .await?
        .ok_or_else(not_found)?;
    if !detail.order.can_cancel(&detail.positions) {
        return Err(ServerFnError::new(
            "Nur Einkaufsaufträge im Entwurf oder bestellte Aufträge ohne Lieferung können storniert werden.",
        ));
    }
    repository::set_status(&mut tx, id, "storniert").await?;
    tx.commit().await?;
    Ok(())
}

/// Löschen (AK-20): nur im Entwurf; Positionen werden mitgelöscht.
#[server]
pub async fn delete_order(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    if lock_order(&mut tx, id).await? != "entwurf" {
        return Err(ServerFnError::new(
            "Nur Einkaufsaufträge im Entwurf können gelöscht werden. Bitte stattdessen stornieren.",
        ));
    }
    let nummer = repository::delete_order(&mut tx, id).await?;
    tx.commit().await?;
    leptos_axum::redirect(&format!(
        "/einkauf?geloescht={}",
        crate::forms::encode(&nummer)
    ));
    Ok(())
}

// ---------- Positionen ----------

/// Position am Ende anfügen (AK-11, AK-12). Nur im Entwurf.
#[server]
pub async fn create_position(
    auftrag_id: i64,
    input: PositionInput,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let values = match input.validate() {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    require_draft(&lock_order(&mut tx, auftrag_id).await?)?;
    let id = repository::insert_position(&mut tx, auftrag_id, &input, &values).await?;
    repository::touch(&mut tx, auftrag_id).await?;
    tx.commit().await?;
    redirect_to_order(auftrag_id);
    Ok(SaveOutcome::Saved(id))
}

/// Position ändern (AK-13). Nur im Entwurf.
#[server]
pub async fn update_position(id: i64, input: PositionInput) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let values = match input.validate() {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = lock_order_of_position(&mut tx, id).await?;
    require_draft(&status)?;
    repository::update_position(&mut tx, id, &input, &values).await?;
    repository::touch(&mut tx, order_id).await?;
    tx.commit().await?;
    redirect_to_order(order_id);
    Ok(SaveOutcome::Saved(id))
}

/// Position löschen (AK-13). Nur im Entwurf.
#[server]
pub async fn delete_position(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = lock_order_of_position(&mut tx, id).await?;
    require_draft(&status)?;
    repository::delete_position(&mut tx, id).await?;
    repository::touch(&mut tx, order_id).await?;
    tx.commit().await?;
    Ok(())
}

/// Position eine Stelle nach oben/unten schieben (AK-13). Nur im Entwurf.
#[server]
pub async fn move_position(id: i64, nach_oben: bool) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = lock_order_of_position(&mut tx, id).await?;
    require_draft(&status)?;
    repository::move_position(&mut tx, order_id, id, nach_oben).await?;
    tx.commit().await?;
    Ok(())
}

// ---------- Wareneingang ----------

/// Gelieferte Menge einer Position erfassen und Status neu bestimmen (AK-16 bis AK-18).
#[server]
pub async fn record_delivery(id: i64, geliefert: String) -> Result<SaveOutcome, ServerFnError> {
    use super::model::{delivery_status, parse_delivered};

    let value = match parse_delivered(&geliefert) {
        Ok(v) => v,
        Err(message) => {
            let mut errors = super::model::FieldErrors::new();
            errors.insert("geliefert".into(), message);
            return Ok(SaveOutcome::Invalid(errors));
        }
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = lock_order_of_position(&mut tx, id).await?;
    require_ordered(&status)?;
    repository::set_delivered(&mut tx, id, value).await?;
    let positions = repository::load_positions(&mut tx, order_id).await?;
    repository::set_status(&mut tx, order_id, delivery_status(&positions)).await?;
    tx.commit().await?;
    redirect_to_order(order_id);
    Ok(SaveOutcome::Saved(id))
}

/// Alle Positionen vollständig geliefert: Liefermenge = bestellte Menge (Überlieferungen bleiben), Status „Geliefert“.
#[server]
pub async fn deliver_all(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    require_ordered(&lock_order(&mut tx, id).await?)?;
    repository::deliver_all_positions(&mut tx, id).await?;
    repository::set_status(&mut tx, id, "geliefert").await?;
    tx.commit().await?;
    Ok(())
}
