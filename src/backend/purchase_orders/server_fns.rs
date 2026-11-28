//! Server-Funktionen für Einkaufsaufträge (ANF-004).
//!
//! Jede Änderung sperrt den Einkaufsauftrag (`FOR UPDATE`) und prüft seinen Status auf dem Server,
//! damit z. B. ein bestellter Auftrag auch über direkte Aufrufe nicht mehr geändert werden kann.
//! Das SQL steht in `repository.rs`; hier stehen Prüfungen, Transaktionen, Meldungen und Weiterleitungen.

use leptos::prelude::*;

use super::model::{
    FromVorgangInput, OrderDetail, OrderInput, OrderPage, OrderRow, PositionInput, SaveOutcome,
};

#[cfg(feature = "ssr")]
use super::{
    model::{FieldErrors, PositionValues, VatSelection, VatWrite, order_from_vorgang},
    repository,
};
#[cfg(feature = "ssr")]
use sqlx::{PgConnection, PgPool};

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

/// Einkaufsaufträge, die aus einem Vorgang erzeugt wurden, neueste zuerst (ANF-029 AK-10).
#[server]
pub async fn list_vorgang_orders(vorgang_id: i64) -> Result<Vec<OrderRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::list_by_vorgang(&pool, vorgang_id).await?)
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

/// Meldung, wenn die Währung nicht aktiv oder unbekannt ist (ANF-014 AK-6).
#[cfg(feature = "ssr")]
const CURRENCY_UNUSABLE: &str =
    "Diese Währung ist nicht (mehr) aktiv. Bitte eine aktive Währung wählen.";

/// `true`, wenn die Währung existiert und aktiv ist.
#[cfg(feature = "ssr")]
async fn currency_usable(conn: &mut PgConnection, code: &str) -> Result<bool, sqlx::Error> {
    crate::backend::finance::currency::repository::is_active(conn, code).await
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
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let mut errors = input.validate();
    if errors.is_empty() && !currency_usable(&mut tx, &input.waehrung).await? {
        errors.insert("waehrung".into(), CURRENCY_UNUSABLE.into());
    }
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let id = match repository::insert_order(&mut tx, &input, None).await {
        Ok(id) => id,
        Err(e) => return reference_outcome(e),
    };
    tx.commit().await?;
    redirect_to_order(id);
    Ok(SaveOutcome::Saved(id))
}

/// Einkaufsauftrag aus einem Vorgang erzeugen (ANF-029 AK-8 bis AK-13): Entwurf mit dem Projekt des Vorgangs,
/// der Beschreibung als Bemerkung und einer Pauschal-Position mit dem Namen des Vorgangs.
#[server]
pub async fn create_order_from_vorgang(
    vorgang_id: i64,
    input: FromVorgangInput,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    // Den MWST-Stand vor der Transaktion lesen, wie bei Positionen (create_position).
    let vat = first_purchase_vat().await?;
    let mut tx = pool.begin().await?;
    // AK-13: Kennung des Vorgangs prüfen; die Angaben kommen aus der Datenbank, nicht aus dem Formular.
    let Some((projekt_id, name, beschreibung)) =
        repository::vorgang_source(&mut tx, vorgang_id).await?
    else {
        return Err(ServerFnError::new("Vorgang nicht gefunden."));
    };

    let order = order_from_vorgang(input.adresse_id.trim(), projekt_id, beschreibung.as_deref());
    let errors = order.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    if !currency_usable(&mut tx, &order.waehrung).await? {
        return Err(ServerFnError::new(CHF_UNUSABLE));
    }

    let order_id = match repository::insert_order(&mut tx, &order, Some(vorgang_id)).await {
        Ok(id) => id,
        Err(e) => return reference_outcome(e),
    };
    // Eine Pauschal-Position mit dem Namen des Vorgangs (AK-9). Menge 1, Preis 0.00 – bitte ergänzen.
    // Der MWST-Stand kommt aus `vat`; das Feld `mwst` wird beim Einfügen nicht gelesen.
    let position = PositionInput {
        bezeichnung: name,
        einheit: "pauschal".into(),
        ..Default::default()
    };
    let values = PositionValues {
        menge: 1000,
        einzelpreis: 0,
        rabatt_prozent: 0,
        mwst: VatSelection::Keep,
    };
    repository::insert_position(&mut tx, order_id, &position, &values, vat).await?;
    tx.commit().await?;
    redirect_to_order(order_id);
    Ok(SaveOutcome::Saved(order_id))
}

/// Meldung, wenn die Währung des Einkaufsauftrags (CHF) nicht aktiv ist (ANF-014 AK-6).
#[cfg(feature = "ssr")]
const CHF_UNUSABLE: &str = "Die Währung CHF ist nicht aktiv. Sie muss unter Einstellungen › Finanzen › Währungen aktiviert werden.";

/// MWST-Stand der Pauschal-Position: erster gültiger Vorsteuer-Code, wie ihn das Auswahlfeld beim Anlegen
/// einer Position vorschlägt. Ist die MWST-Pflicht nicht in Kraft, gilt „ohne MWST“ (Satz 0).
#[cfg(feature = "ssr")]
async fn first_purchase_vat() -> Result<VatWrite, ServerFnError> {
    let codes = crate::backend::finance::vat::server_fns::list_purchase_codes().await?;
    Ok(match codes.first() {
        Some(code) => VatWrite {
            code_id: Some(code.id),
            satz: code.satz,
        },
        None => VatWrite {
            code_id: None,
            satz: 0,
        },
    })
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
    let mut errors = input.validate();
    // Eine unveränderte Währung darf auch dann bleiben, wenn sie inzwischen deaktiviert ist.
    if errors.is_empty()
        && input.waehrung != current.waehrung
        && !currency_usable(&mut tx, &input.waehrung).await?
    {
        errors.insert("waehrung".into(), CURRENCY_UNUSABLE.into());
    }
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
    let vat = match resolve_vat(&pool, values.mwst, None).await? {
        Ok(vat) => vat,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let mut tx = pool.begin().await?;
    require_draft(&lock_order(&mut tx, auftrag_id).await?)?;
    let id = repository::insert_position(&mut tx, auftrag_id, &input, &values, vat).await?;
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
    let current = repository::position_vat(&mut tx, id).await?;
    let vat = match resolve_vat(&pool, values.mwst, current).await? {
        Ok(vat) => vat,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    repository::update_position(&mut tx, id, &input, &values, vat).await?;
    repository::touch(&mut tx, order_id).await?;
    tx.commit().await?;
    redirect_to_order(order_id);
    Ok(SaveOutcome::Saved(id))
}

/// MWST-Auswahl in den zu speichernden Stand umsetzen (ANF-012 AK-9). `current`: Stand der bestehenden Position.
/// Die Prüfung des Codes (gültig, Vorsteuer, aktiv) übernimmt der Modul `vat`.
#[cfg(feature = "ssr")]
async fn resolve_vat(
    pool: &PgPool,
    selection: VatSelection,
    current: Option<VatWrite>,
) -> Result<Result<VatWrite, FieldErrors>, ServerFnError> {
    Ok(match selection {
        VatSelection::Ohne => Ok(VatWrite {
            code_id: None,
            satz: 0,
        }),
        VatSelection::Keep => current
            .ok_or_else(|| field_error("Bitte einen MWST-Code wählen oder „ohne MWST“.".into())),
        VatSelection::Code(id) => {
            match crate::backend::finance::vat::server_fns::purchase_code(pool, id).await {
                Ok(code) => Ok(VatWrite {
                    code_id: Some(code.id),
                    satz: code.satz,
                }),
                Err(e) => Err(field_error(crate::forms::error_message(&e))),
            }
        }
    })
}

/// Fehler an einem Feld der Auswahl „MWST-Code“.
#[cfg(feature = "ssr")]
fn field_error(message: String) -> FieldErrors {
    let mut errors = FieldErrors::new();
    errors.insert("mwst_code".into(), message);
    errors
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
