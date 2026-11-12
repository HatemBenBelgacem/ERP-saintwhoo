//! Server-Funktionen für Anmelden, Abmelden und Ersteinrichtung (ANF-007).
//!
//! Funktionen mit `prefix = "/oeffentlich"` sind ohne Anmeldung erreichbar (AK-3);
//! alle anderen Server-Funktionen des ERP liegen unter `/api` und verlangen eine Sitzung.

use leptos::prelude::*;

use super::model::{CurrentUser, FirstSetupInput, SaveOutcome};
use crate::backend::users::model::PasswordInput;

#[cfg(feature = "ssr")]
use super::{
    model::{FORM_ERROR, FieldErrors, MAX_FAILED_ATTEMPTS, MSG_EMPTY, MSG_LOCKED, MSG_WRONG},
    repository, session,
};

/// Angemeldeter Benutzer oder `None` (Kopfzeile, Standard in „Aufgaben“).
#[server(prefix = "/oeffentlich")]
pub async fn get_current_user() -> Result<Option<CurrentUser>, ServerFnError> {
    Ok(session::current_user())
}

/// `true`, solange es noch keinen Benutzer gibt – dann zeigt `/anmelden` die Ersteinrichtung (AK-21).
#[server(prefix = "/oeffentlich")]
pub async fn setup_required() -> Result<bool, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::no_users(&pool).await?)
}

/// Anmelden (AK-5 bis AK-12, AK-16). Bei Erfolg neue Sitzung, Cookie und Weiterleitung zu `zurueck`.
/// Fehler erscheinen als `SaveOutcome::Invalid` unter dem Feld „anmeldung“ – immer mit derselben
/// Meldung, egal ob der Name unbekannt, das Passwort falsch oder der Benutzer gesperrt ist.
#[server(prefix = "/oeffentlich")]
pub async fn login(
    benutzername: String,
    passwort: String,
    zurueck: Option<String>,
) -> Result<SaveOutcome, ServerFnError> {
    let username = super::model::normalize_username(&benutzername);
    if username.is_empty() || passwort.is_empty() {
        return Ok(form_error(MSG_EMPTY));
    }
    let pool = crate::db::pool()?;
    if repository::recent_failures(&pool, &username).await? >= MAX_FAILED_ATTEMPTS {
        return Ok(form_error(MSG_LOCKED));
    }

    let found = repository::credentials(&pool, &username).await?;
    // AK-12: Immer einen Hash prüfen, auch bei unbekanntem Namen (gleiche Antwortzeit)
    let stored = found
        .as_ref()
        .map_or_else(dummy_hash, |(_, hash, _)| hash.clone());
    let password_ok = tokio::task::spawn_blocking(move || {
        crate::backend::users::password::verify(&passwort, &stored)
    })
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let user_id = match found {
        Some((id, _, true)) if password_ok => id,
        _ => {
            repository::record_failure(&pool, &username).await?;
            return Ok(form_error(MSG_WRONG));
        }
    };

    // AK-16: immer ein neues Merkmal
    let token = session::new_token();
    let mut tx = pool.begin().await?;
    repository::start_session(&mut tx, user_id, &username, &session::token_hash(&token)).await?;
    tx.commit().await?;

    session::set_cookie(&session::session_cookie(&token));
    leptos_axum::redirect(&super::model::target_after_login(zurueck.as_deref()));
    Ok(SaveOutcome::Saved(user_id))
}

/// Abmelden (AK-18): Sitzung löschen, Cookie entfernen, zur Anmeldeseite.
#[server]
pub async fn logout() -> Result<(), ServerFnError> {
    if let Some(token) = session::current_token() {
        let pool = crate::db::pool()?;
        repository::end_session(&pool, &session::token_hash(&token)).await?;
    }
    session::set_cookie(&session::clear_cookie());
    leptos_axum::redirect("/anmelden?abgemeldet=1");
    Ok(())
}

/// Ersteinrichtung (AK-21, AK-22): Person und ersten Benutzer anlegen und direkt anmelden.
/// Nur möglich, solange es keinen Benutzer gibt.
#[server(prefix = "/oeffentlich")]
pub async fn first_setup(
    input: FirstSetupInput,
    passwort: PasswordInput,
) -> Result<SaveOutcome, ServerFnError> {
    let errors = input.validate(&passwort);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    if !repository::no_users(&pool).await? {
        return Err(ServerFnError::new(
            "Die Ersteinrichtung ist bereits abgeschlossen. Bitte anmelden.",
        ));
    }
    let address = input.address();
    let username = input.username();
    let pw = passwort.passwort;
    let hash = tokio::task::spawn_blocking(move || crate::backend::users::password::hash(&pw))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .map_err(|e| {
            ServerFnError::new(format!("Passwort konnte nicht gespeichert werden: {e}"))
        })?;

    let mut tx = pool.begin().await?;
    let email = (!address.email.is_empty()).then_some(address.email.as_str());
    let Some(user_id) = repository::create_first_user(
        &mut tx,
        &address.vorname,
        &address.nachname,
        email,
        &username,
        &hash,
    )
    .await?
    else {
        return Err(ServerFnError::new(
            "Die Ersteinrichtung ist bereits abgeschlossen. Bitte anmelden.",
        ));
    };
    let token = session::new_token();
    repository::start_session(&mut tx, user_id, &username, &session::token_hash(&token)).await?;
    tx.commit().await?;

    session::set_cookie(&session::session_cookie(&token));
    leptos_axum::redirect("/");
    Ok(SaveOutcome::Saved(user_id))
}

#[cfg(feature = "ssr")]
fn form_error(message: &str) -> SaveOutcome {
    let mut errors = FieldErrors::new();
    errors.insert(FORM_ERROR.into(), message.into());
    SaveOutcome::Invalid(errors)
}

/// Gültiger Hash eines nie vergebenen Passworts – für gleich lange Prüfung bei unbekannten Namen.
#[cfg(feature = "ssr")]
fn dummy_hash() -> String {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        crate::backend::users::password::hash(&session::new_token()).unwrap_or_default()
    })
    .clone()
}
