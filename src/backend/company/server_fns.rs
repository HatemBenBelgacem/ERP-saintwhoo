//! Server-Funktionen der eigenen Firma (ANF-025).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in
//! verständliche Meldungen und leiten nach dem Speichern zur Firmenseite.

use leptos::prelude::*;
use server_fn::codec::{MultipartData, MultipartFormData};

use super::model::{Company, CompanyInput, PickOption, SaveOutcome};
use crate::backend::addresses::model::AddressInput;

#[cfg(feature = "ssr")]
use super::{
    model::{FieldErrors, LOGO_MAX_BYTES, company_address_error, detect_logo_type},
    repository,
};

/// Webadresse der Einstellungsseite.
pub const COMPANY_URL: &str = "/einstellungen/firma";

/// Die eigene Firma; `None`, solange keine erfasst ist (AK-2, AK-9).
#[server]
pub async fn get_company() -> Result<Option<Company>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::find(&pool).await?)
}

/// Adressauswahl im Firmenformular (AK-4): nur Adressen vom Typ „Firma“.
#[server]
pub async fn search_company_addresses(suche: String) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    Ok(repository::search_company_addresses(&pool, &suche, &pattern).await?)
}

/// Firma erfassen (AK-4, AK-6, AK-8): bestehende Adresse wählen oder neue Adresse im selben
/// Schritt anlegen (`input.modus = "neu"`, Felder in `adresse`).
#[server]
pub async fn create_company(
    input: CompanyInput,
    adresse: AddressInput,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let mut errors = input.validate();
    let pool = crate::db::pool()?;

    let new_address = if input.creates_address() {
        let address = AddressInput {
            typ: "firma".into(),
            ..adresse
        }
        .normalized();
        errors.extend(address.validate());
        for (field, value) in [
            ("strasse", &address.strasse),
            ("plz", &address.plz),
            ("ort", &address.ort),
        ] {
            if value.is_empty() {
                errors
                    .entry(field.into())
                    .or_insert_with(|| "Für den Absender der Belege nötig.".into());
            }
        }
        Some(address)
    } else {
        check_address(&pool, &input, &mut errors).await?;
        None
    };
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    let mut tx = pool.begin().await?;
    let address_id = match &new_address {
        Some(address) => crate::backend::addresses::repository::insert(&mut *tx, address).await?,
        None => input.address_id().unwrap_or_default(),
    };
    let id = match repository::insert(&mut tx, address_id, &input).await {
        Ok(id) => id,
        Err(e) => return already_exists(e),
    };
    tx.commit().await?;
    leptos_axum::redirect(&format!("{COMPANY_URL}?gespeichert=1"));
    Ok(SaveOutcome::Saved(id))
}

/// UID, Handelsregister, Fusszeile und verknüpfte Adresse ändern (AK-5, AK-6, AK-8).
#[server]
pub async fn update_company(input: CompanyInput) -> Result<SaveOutcome, ServerFnError> {
    let input = CompanyInput {
        modus: "bestehend".into(),
        ..input
    }
    .normalized();
    let mut errors = input.validate();
    let pool = crate::db::pool()?;
    check_address(&pool, &input, &mut errors).await?;
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let address_id = input.address_id().unwrap_or_default();
    if repository::update(&pool, address_id, &input).await? == 0 {
        return Err(ServerFnError::new("Es ist noch keine Firma erfasst."));
    }
    leptos_axum::redirect(&format!("{COMPANY_URL}?gespeichert=1"));
    Ok(SaveOutcome::Saved(address_id))
}

/// Logo hochladen oder ersetzen (AK-7): PNG, JPG oder SVG bis 1 MB, Feld `logo`.
/// Der Dateityp wird am Inhalt erkannt, nicht an der Dateiendung.
#[server(input = MultipartFormData)]
pub async fn upload_logo(data: MultipartData) -> Result<SaveOutcome, ServerFnError> {
    let invalid = |m: &str| {
        let mut errors = FieldErrors::new();
        errors.insert("logo".into(), m.into());
        Ok(SaveOutcome::Invalid(errors))
    };
    let mut multipart = data
        .into_inner()
        .ok_or_else(|| ServerFnError::new("Keine Datei erhalten."))?;
    let mut logo: Option<Vec<u8>> = None;
    while let Some(mut field) = multipart
        .next_field()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    {
        if field.name() != Some("logo") {
            continue;
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = field
            .chunk()
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
        {
            if bytes.len() + chunk.len() > LOGO_MAX_BYTES {
                return invalid("Die Datei ist grösser als 1 MB.");
            }
            bytes.extend_from_slice(&chunk);
        }
        logo = Some(bytes);
    }
    let Some(bytes) = logo.filter(|b| !b.is_empty()) else {
        return invalid("Bitte eine Datei wählen.");
    };
    let Some(typ) = detect_logo_type(&bytes) else {
        return invalid("Erlaubt sind nur PNG-, JPG- oder SVG-Dateien.");
    };
    let pool = crate::db::pool()?;
    if repository::set_logo(&pool, Some((&bytes, typ))).await? == 0 {
        return Err(ServerFnError::new("Bitte zuerst die Firma erfassen."));
    }
    leptos_axum::redirect(&format!("{COMPANY_URL}?gespeichert=logo"));
    Ok(SaveOutcome::Saved(0))
}

/// Logo entfernen (AK-7).
#[server]
pub async fn remove_logo() -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    repository::set_logo(&pool, None).await?;
    leptos_axum::redirect(&format!("{COMPANY_URL}?gespeichert=logo-entfernt"));
    Ok(())
}

/// Gewählte Adresse vorhanden, vom Typ „Firma“ und mit vollständiger Postanschrift? (AK-8)
#[cfg(feature = "ssr")]
async fn check_address(
    pool: &sqlx::PgPool,
    input: &CompanyInput,
    errors: &mut FieldErrors,
) -> Result<(), ServerFnError> {
    let Some(id) = input.address_id() else {
        return Ok(());
    };
    let message = match repository::address_parts(pool, id).await? {
        None => Some("Diese Adresse gibt es nicht mehr.".to_string()),
        Some((typ, strasse, plz, ort)) => {
            company_address_error(&typ, strasse.as_deref(), plz.as_deref(), ort.as_deref())
        }
    };
    if let Some(m) = message {
        errors.entry("adresse_id".into()).or_insert(m);
    }
    Ok(())
}

/// Zweite Firma (Eindeutigkeit von `einzig`) als verständliche Meldung.
#[cfg(feature = "ssr")]
fn already_exists(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    if let sqlx::Error::Database(db) = &e
        && db.code().as_deref() == Some("23505")
    {
        return Err(ServerFnError::new(
            "Es ist bereits eine Firma erfasst. Bitte diese bearbeiten.",
        ));
    }
    Err(e.into())
}
