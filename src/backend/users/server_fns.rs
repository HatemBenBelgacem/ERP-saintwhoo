//! Server-Funktionen für Benutzer (ANF-005).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in
//! verständliche Meldungen und leiten nach dem Speichern weiter.

use leptos::prelude::*;

use super::model::{PickOption, SaveOutcome, UserDetail, UserInput, UserRow};

#[cfg(feature = "ssr")]
use super::{
    model::{FieldErrors, Responsibility},
    repository,
};

/// Webadresse der Detailansicht.
pub fn user_url(id: i64) -> String {
    format!("/einstellungen/benutzer/{id}")
}

/// Benutzerliste mit Suche, Statusfilter und Sortierung (AK-1 bis AK-5).
/// `status`: „alle“ bzw. leer, „aktiv“ oder „gesperrt“.
#[server]
pub async fn list_users(
    suche: String,
    status: String,
    sortierung: String,
    richtung: String,
) -> Result<Vec<UserRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    let status = match status.as_str() {
        "aktiv" | "gesperrt" => status,
        _ => String::new(),
    };
    Ok(repository::list(
        &pool,
        &suche,
        &pattern,
        &status,
        &sortierung,
        richtung == "ab",
    )
    .await?)
}

/// Ein Benutzer mit seiner Verantwortung in Projekten (AK-6, AK-22, AK-31). `None`, wenn es ihn nicht gibt.
#[server]
pub async fn get_user(id: i64) -> Result<Option<UserDetail>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(user) = repository::find(&pool, id).await? else {
        return Ok(None);
    };
    let responsibility = Responsibility {
        assigned: repository::assigned_elements(&pool, id).await?,
        open_tasks: crate::backend::tasks::repository::list_for_user(
            &pool, id, "offen", "", "", "%%",
        )
        .await?,
    };
    Ok(Some(UserDetail {
        user,
        responsibility,
    }))
}

/// Ein Benutzer als Auswahl-Eintrag (auch gesperrte), z. B. für die Benutzer-Auswahl in „Aufgaben“ (ANF-006 AK-2).
#[server]
pub async fn get_user_option(id: i64) -> Result<Option<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::find(&pool, id).await?.map(|u| PickOption {
        id: u.id,
        nummer: u.benutzername,
        name: u.name,
    }))
}

/// Auswahl „Verantwortlich“ in Projekten (AK-24): nur aktive Benutzer, nach Name oder Benutzername.
#[server]
pub async fn search_active_users(suche: String) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    Ok(repository::search_active(&pool, &suche, &pattern).await?)
}

/// Adress-Auswahl im Benutzerformular (AK-10): nur Personen ohne Benutzer.
/// `benutzer_id`: beim Bearbeiten gilt die eigene Adresse als frei.
#[server]
pub async fn search_person_addresses(
    suche: String,
    benutzer_id: Option<i64>,
) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    Ok(repository::search_free_persons(&pool, &suche, &pattern, benutzer_id).await?)
}

/// Neuen Benutzer anlegen (AK-9 bis AK-13). Bei Erfolg Weiterleitung zur Detailansicht.
/// Ohne Passwort: angemeldet wird nur über das Portal (ANF-026).
#[server]
pub async fn create_user(input: UserInput) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let mut errors = input.validate();
    let pool = crate::db::pool()?;
    check_unique(&pool, &input, None, &mut errors).await?;
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    let id = match repository::insert(&pool, &input).await {
        Ok(id) => id,
        Err(e) => return unique_violation(e),
    };
    leptos_axum::redirect(&format!("{}?gespeichert=1", user_url(id)));
    Ok(SaveOutcome::Saved(id))
}

/// Benutzername, Adresse und Status ändern (AK-14, AK-16, AK-17).
#[server]
pub async fn update_user(id: i64, input: UserInput) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let mut errors = input.validate();
    // ANF-007 AK-20: sich nicht selbst aussperren
    if !input.is_active() && is_self(id) {
        errors.insert(
            "status".into(),
            "Sie können sich nicht selbst sperren.".into(),
        );
    }
    let pool = crate::db::pool()?;
    check_unique(&pool, &input, Some(id), &mut errors).await?;
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    match repository::update(&pool, id, &input).await {
        Ok(0) => Err(ServerFnError::new("Benutzer nicht gefunden.")),
        Ok(_) => {
            leptos_axum::redirect(&format!("{}?gespeichert=1", user_url(id)));
            Ok(SaveOutcome::Saved(id))
        }
        Err(e) => unique_violation(e),
    }
}

/// Benutzer löschen (AK-18, AK-32). Ist er noch in Projekten hinterlegt, wird das Löschen verhindert.
/// Die Adresse bleibt erhalten.
#[server]
pub async fn delete_user(id: i64) -> Result<(), ServerFnError> {
    // ANF-007 AK-20: sich nicht selbst löschen (Sitzungen anderer verschwinden mit dem Benutzer, AK-15)
    if is_self(id) {
        return Err(ServerFnError::new("Sie können sich nicht selbst löschen."));
    }
    let pool = crate::db::pool()?;
    match repository::delete(&pool, id).await {
        Ok(Some(name)) => {
            leptos_axum::redirect(&format!(
                "/einstellungen/benutzer?geloescht={}",
                crate::forms::encode(&name)
            ));
            Ok(())
        }
        Ok(None) => Err(ServerFnError::new("Benutzer nicht gefunden.")),
        // 23503 = Fremdschlüssel verletzt: noch bei Projekt, Teilprojekt oder Vorgang hinterlegt
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            let counts = repository::responsibility_counts(&pool, id).await?;
            let standalone = crate::backend::tasks::repository::count_for_user(&pool, id).await?;
            Err(ServerFnError::new(super::model::still_responsible_message(
                &counts,
                usize::try_from(standalone).unwrap_or_default(),
            )))
        }
        Err(e) => Err(e.into()),
    }
}

/// Prüft auf dem Server, was das Modell nicht wissen kann (AK-13): Benutzername frei,
/// Adresse vorhanden, vom Typ „Person“ und keinem anderen Benutzer zugeordnet.
#[cfg(feature = "ssr")]
async fn check_unique(
    pool: &sqlx::PgPool,
    input: &UserInput,
    id: Option<i64>,
    errors: &mut FieldErrors,
) -> Result<(), ServerFnError> {
    if !errors.contains_key("benutzername")
        && repository::username_taken(pool, &input.benutzername, id).await?
    {
        errors.insert(
            "benutzername".into(),
            "Dieser Benutzername ist bereits vergeben.".into(),
        );
    }
    if let Some(address_id) = input.address_id() {
        let message = match repository::address_type(pool, address_id).await?.as_deref() {
            None => Some("Diese Adresse gibt es nicht mehr.".to_string()),
            Some("person") => repository::address_owner(pool, address_id, id)
                .await?
                .map(|owner| format!("Diese Adresse gehört bereits zum Benutzer „{owner}“.")),
            Some(_) => Some("Bitte eine Adresse vom Typ „Person“ wählen.".to_string()),
        };
        if let Some(m) = message {
            errors.entry("adresse_id".into()).or_insert(m);
        }
    }
    Ok(())
}

/// Eindeutigkeit, die zwischen Prüfung und Speichern verletzt wurde, als Feldfehler melden.
#[cfg(feature = "ssr")]
fn unique_violation(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    if let sqlx::Error::Database(db) = &e {
        let field = match (db.code().as_deref(), db.constraint()) {
            (Some("23505"), Some(c)) if c.contains("benutzername") => {
                Some(("benutzername", "Dieser Benutzername ist bereits vergeben."))
            }
            (Some("23505"), Some(c)) if c.contains("adresse_id") => Some((
                "adresse_id",
                "Diese Adresse gehört bereits zu einem anderen Benutzer.",
            )),
            (Some("23503"), _) => Some(("adresse_id", "Diese Adresse gibt es nicht mehr.")),
            _ => None,
        };
        if let Some((f, m)) = field {
            let mut errors = FieldErrors::new();
            errors.insert(f.into(), m.into());
            return Ok(SaveOutcome::Invalid(errors));
        }
    }
    Err(e.into())
}

/// `true`, wenn `id` der angemeldete Benutzer ist (ANF-007 AK-20).
#[cfg(feature = "ssr")]
fn is_self(id: i64) -> bool {
    crate::backend::auth::session::current_user().is_some_and(|u| u.benutzer_id == Some(id))
}
