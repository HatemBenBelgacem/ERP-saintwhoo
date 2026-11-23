//! Server-Funktionen für Registrierung, Anmelden und Abmelden über das Portal (ANF-007, ANF-026).
//!
//! Funktionen mit `prefix = "/oeffentlich"` sind ohne Anmeldung erreichbar (ANF-007 AK-3).
//! Funktionen unter `/api/portal` verlangen eine Sitzung, aber keinen Mandanten (ANF-026).
//! Alle anderen Server-Funktionen des ERP liegen unter `/api` und verlangen Sitzung und Mandant.

use leptos::prelude::*;

use super::model::{CurrentUser, RegistrationInput, SaveOutcome};
use crate::backend::users::model::PasswordInput;

#[cfg(feature = "ssr")]
use super::{
    model::{
        FORM_ERROR, FieldErrors, MAX_FAILED_ATTEMPTS, MAX_REGISTRATIONS, MSG_EMAIL_TAKEN,
        MSG_EMPTY, MSG_LOCKED, MSG_TOO_MANY_REGISTRATIONS, MSG_WRONG, PORTAL_PATH,
    },
    repository, session,
};

/// Angemeldeter Zugang oder `None` (Kopfzeile, Portal, Standard in „Aufgaben“).
#[server(prefix = "/oeffentlich")]
pub async fn get_current_user() -> Result<Option<CurrentUser>, ServerFnError> {
    Ok(session::current_user())
}

/// Registrieren (ANF-026 AK-1 bis AK-5): Zugang als Administrator anlegen und direkt anmelden.
/// Danach geht es zur Portal-Übersicht.
#[server(prefix = "/oeffentlich")]
pub async fn register(
    input: RegistrationInput,
    passwort: PasswordInput,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let errors = input.validate(&passwort);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::portal_pool()?;
    if repository::recent_registrations(&pool).await? >= MAX_REGISTRATIONS {
        return Ok(form_error(MSG_TOO_MANY_REGISTRATIONS));
    }
    let pw = passwort.passwort;
    let hash = tokio::task::spawn_blocking(move || crate::backend::users::password::hash(&pw))
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .map_err(|e| {
            ServerFnError::new(format!("Passwort konnte nicht gespeichert werden: {e}"))
        })?;

    let mut tx = pool.begin().await?;
    let Some(user_id) = repository::create_user(
        &mut tx,
        &input.vorname,
        &input.nachname,
        &input.email,
        &hash,
    )
    .await?
    else {
        let mut errors = FieldErrors::new();
        errors.insert("email".into(), MSG_EMAIL_TAKEN.into());
        return Ok(SaveOutcome::Invalid(errors));
    };
    let token = session::new_token();
    repository::start_session(&mut tx, user_id, &input.email, &session::token_hash(&token)).await?;
    tx.commit().await?;

    session::set_cookie(&session::session_cookie(&token));
    leptos_axum::redirect(PORTAL_PATH);
    Ok(SaveOutcome::Saved(user_id))
}

/// Anmelden mit E-Mail und Passwort (ANF-007 AK-5 bis AK-12, AK-16). Bei Erfolg neue Sitzung, Cookie
/// und Weiterleitung zu `zurueck` – ohne Zugang zum ERP zur Portal-Übersicht (ANF-026 AK-19).
/// Fehler erscheinen als `SaveOutcome::Invalid` unter dem Feld „anmeldung“ – immer mit derselben
/// Meldung, egal ob die E-Mail unbekannt, das Passwort falsch oder der Zugang gesperrt ist.
#[server(prefix = "/oeffentlich")]
pub async fn login(
    email: String,
    passwort: String,
    zurueck: Option<String>,
) -> Result<SaveOutcome, ServerFnError> {
    let email = super::model::normalize_email(&email);
    if email.is_empty() || passwort.is_empty() {
        return Ok(form_error(MSG_EMPTY));
    }
    let pool = crate::db::portal_pool()?;
    if repository::recent_failures(&pool, &email).await? >= MAX_FAILED_ATTEMPTS {
        return Ok(form_error(MSG_LOCKED));
    }

    let found = repository::credentials(&pool, &email).await?;
    // ANF-007 AK-12: Immer einen Hash prüfen, auch bei unbekannter E-Mail (gleiche Antwortzeit)
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
            repository::record_failure(&pool, &email).await?;
            return Ok(form_error(MSG_WRONG));
        }
    };

    // ANF-007 AK-16: immer ein neues Merkmal
    let token = session::new_token();
    let mut tx = pool.begin().await?;
    let tenant =
        repository::start_session(&mut tx, user_id, &email, &session::token_hash(&token)).await?;
    tx.commit().await?;

    session::set_cookie(&session::session_cookie(&token));
    let target = match &tenant {
        Some(t) if t.is_usable() => super::model::target_after_login(zurueck.as_deref()),
        // ANF-026 AK-19: Mandant noch nicht definiert → Modal-Fenster im Portal
        _ => super::model::portal_target(tenant.as_ref()),
    };
    leptos_axum::redirect(&target);
    Ok(SaveOutcome::Saved(user_id))
}

/// Abmelden (ANF-007 AK-18): Sitzung löschen, Cookie entfernen, zur Anmeldeseite.
/// Liegt unter `/api/portal`, damit es auch ohne Mandant funktioniert.
#[server(prefix = "/api/portal")]
pub async fn logout() -> Result<(), ServerFnError> {
    if let Some(token) = session::current_token() {
        let pool = crate::db::portal_pool()?;
        repository::end_session(&pool, &session::token_hash(&token)).await?;
    }
    session::set_cookie(&session::clear_cookie());
    leptos_axum::redirect("/anmelden?abgemeldet=1");
    Ok(())
}

#[cfg(feature = "ssr")]
fn form_error(message: &str) -> SaveOutcome {
    let mut errors = FieldErrors::new();
    errors.insert(FORM_ERROR.into(), message.into());
    SaveOutcome::Invalid(errors)
}

/// Gültiger Hash eines nie vergebenen Passworts – für gleich lange Prüfung bei unbekannter E-Mail.
#[cfg(feature = "ssr")]
fn dummy_hash() -> String {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| {
        crate::backend::users::password::hash(&session::new_token()).unwrap_or_default()
    })
    .clone()
}

#[cfg(test)]
mod tests {
    use leptos::server_fn::ServerFn;

    use super::super::model::{PORTAL_API_PREFIX, PUBLIC_API_PREFIX};
    use super::*;

    #[test]
    fn prefixes_match_middleware_rules() {
        // Abmelden muss ohne Mandant gehen (ANF-026), Registrieren und Anmelden ohne Sitzung.
        assert!(<Logout as ServerFn>::PATH.starts_with(&format!("{PORTAL_API_PREFIX}/")));
        for path in [
            <Register as ServerFn>::PATH,
            <Login as ServerFn>::PATH,
            <GetCurrentUser as ServerFn>::PATH,
        ] {
            assert!(path.starts_with(&format!("{PUBLIC_API_PREFIX}/")), "{path}");
        }
    }
}
