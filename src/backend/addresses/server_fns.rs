//! Server-Funktionen für Adressen (ANF-002).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in
//! verständliche Meldungen und leiten nach dem Speichern weiter.

use leptos::prelude::*;

use super::model::{Address, AddressInput, AddressPage, SaveOutcome};
use crate::forms::PickOption;

#[cfg(feature = "ssr")]
use super::repository;

/// Adressliste mit Suche, Sortierung und Seiten (AK-1 bis AK-6).
#[server]
pub async fn list_addresses(
    suche: String,
    sortierung: String,
    richtung: String,
    seite: i64,
) -> Result<AddressPage, ServerFnError> {
    use super::model::PAGE_SIZE;

    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);

    let total = repository::count(&pool, &suche, &pattern).await?;
    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = seite.clamp(1, page_count);
    let rows = repository::list_page(
        &pool,
        &suche,
        &pattern,
        &sortierung,
        richtung == "ab",
        PAGE_SIZE,
        (page - 1) * PAGE_SIZE,
    )
    .await?;

    Ok(AddressPage { rows, total, page })
}

/// Eine Adresse (AK-7, AK-17). `None`, wenn es sie nicht gibt.
#[server]
pub async fn get_address(id: i64) -> Result<Option<Address>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::find(&pool, id).await?)
}

/// Adress-Auswahl in anderen Modulen (Kunde im Projekt, Lieferant im Einkauf): nach Name, Nummer oder Ort.
#[server]
pub async fn search_addresses(suche: String) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    Ok(repository::search_options(&pool, &suche, &pattern).await?)
}

/// Neue Adresse anlegen (AK-8 bis AK-11). Bei Erfolg Weiterleitung zur Detailansicht.
#[server]
pub async fn create_address(input: AddressInput) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    let pool = crate::db::pool()?;
    let id = repository::insert(&pool, &input).await?;

    leptos_axum::redirect(&format!("/adressen/{id}?gespeichert=1"));
    Ok(SaveOutcome::Saved(id))
}

/// Adresse ändern (AK-12, AK-13). Bei Erfolg Weiterleitung zur Detailansicht.
#[server]
pub async fn update_address(id: i64, input: AddressInput) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    let pool = crate::db::pool()?;
    // ANF-005 AK-20: Die Adresse eines Benutzers muss eine Person bleiben
    if input.typ != "person"
        && let Some(owner) = repository::owner_username(&pool, id).await?
    {
        let mut errors = super::model::FieldErrors::new();
        errors.insert(
            "typ".into(),
            format!("Die Adresse gehört zum Benutzer „{owner}“ und muss eine Person bleiben."),
        );
        return Ok(SaveOutcome::Invalid(errors));
    }
    if repository::update(&pool, id, &input).await? == 0 {
        return Err(ServerFnError::new("Adresse nicht gefunden."));
    }

    leptos_axum::redirect(&format!("/adressen/{id}?gespeichert=1"));
    Ok(SaveOutcome::Saved(id))
}

/// Adresse löschen (AK-15, AK-16). Wird sie noch verwendet, wird das Löschen verhindert.
#[server]
pub async fn delete_address(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    match repository::delete(&pool, id).await {
        Ok(Some(nummer)) => {
            leptos_axum::redirect(&format!("/adressen?geloescht={nummer}"));
            Ok(())
        }
        Ok(None) => Err(ServerFnError::new("Adresse nicht gefunden.")),
        // 23503 = Fremdschlüssel verletzt: ein anderer Datensatz verweist noch auf die Adresse
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            let where_used = match db.table() {
                Some("projekt") => "in Projekten (als Kunde)".to_string(),
                Some("einkaufsauftrag") => "in Einkaufsaufträgen (als Lieferant)".to_string(),
                // ANF-018 AK-16, ANF-010 AK-9
                Some("debitorenrechnung") => "in Ausgangsrechnungen (als Kunde)".to_string(),
                // ANF-025 AK-11
                Some("firma") => "als eigene Firma".to_string(),
                // ANF-005 AK-19
                Some("benutzer") => match repository::owner_username(&pool, id).await? {
                    Some(name) => format!("als Benutzer „{name}“"),
                    None => "als Benutzer".to_string(),
                },
                Some(table) => format!("in der Tabelle „{table}“"),
                None => "in anderen Datensätzen".to_string(),
            };
            Err(ServerFnError::new(format!(
                "Die Adresse wird noch {where_used} verwendet und kann nicht gelöscht werden."
            )))
        }
        Err(e) => Err(e.into()),
    }
}
