//! Server-Funktionen für Geschäftsjahre und Perioden (ANF-013).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in verständliche
//! Meldungen und leiten nach dem Speichern zurück zur Seite.

use leptos::prelude::*;

use super::model::{FiscalYearInput, FiscalYearOverview, SaveOutcome};

#[cfg(feature = "ssr")]
use super::{
    model::{booking_error, label_for, lock_error, suggestion},
    repository,
};
#[cfg(feature = "ssr")]
use crate::backend::finance::date::months_between;

/// Webadresse der Einstellungsseite.
pub const FISCAL_YEARS_URL: &str = "/einstellungen/finanzen/geschaeftsjahre";

/// Alle Geschäftsjahre mit Perioden und der Vorschlag für ein neues (AK-2, AK-4).
#[server]
pub async fn get_fiscal_years() -> Result<FiscalYearOverview, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut jahre = repository::list_years(&pool).await?;
    let periods = repository::list_periods(&pool).await?;
    for year in &mut jahre {
        year.perioden = periods
            .iter()
            .filter(|p| p.geschaeftsjahr_id == year.id)
            .cloned()
            .collect();
    }
    let today = repository::today(&pool).await?;
    let latest_end = jahre.first().map(|j| j.ende.as_str());
    let (vorschlag_beginn, vorschlag_ende) = suggestion(latest_end, &today);
    Ok(FiscalYearOverview {
        jahre,
        vorschlag_beginn,
        vorschlag_ende,
    })
}

/// Neues Geschäftsjahr anlegen (AK-2, AK-3). Die Perioden entstehen automatisch (AK-4).
#[server]
pub async fn create_fiscal_year(
    input: FiscalYearInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock_years(&mut tx).await?;
    let latest = repository::latest_year(&mut tx).await?;
    let errors = input.validate(latest.as_ref().map(|(_, ende)| ende.as_str()));
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let (beginn, ende) = (input.beginn.clone(), input.ende.clone());
    let bezeichnung = label_for(&beginn, &ende);
    let id = match repository::insert_year(&mut tx, &bezeichnung, &beginn, &ende).await {
        Ok(id) => id,
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23P01") => {
            return Err(ServerFnError::new(
                "Das Geschäftsjahr überschneidet sich mit einem bestehenden Geschäftsjahr.",
            ));
        }
        Err(e) => return Err(e.into()),
    };
    for (monat_beginn, monat_ende) in months_between(&beginn, &ende) {
        repository::insert_period(&mut tx, id, &monat_beginn, &monat_ende).await?;
    }
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", "geschaeftsjahr");
    Ok(SaveOutcome::Saved(id))
}

/// Geschäftsjahr löschen (AK-7). Nur das neueste darf gelöscht werden, sonst entsteht eine Lücke (AK-3).
#[server]
pub async fn delete_fiscal_year(id: i64, zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock_years(&mut tx).await?;
    match repository::latest_year(&mut tx).await? {
        Some((latest, _)) if latest == id => {}
        Some(_) => {
            return Err(ServerFnError::new(
                "Nur das neueste Geschäftsjahr kann gelöscht werden, sonst entsteht eine Lücke.",
            ));
        }
        None => return Err(ServerFnError::new("Geschäftsjahr nicht gefunden.")),
    }
    // AK-7: Ein Geschäftsjahr mit Buchungen darf nicht gelöscht werden. Die Tabelle `buchung`
    // kommt mit ANF-017; dann wird hier vorher geprüft.
    repository::delete_year(&mut tx, id).await?;
    tx.commit().await?;
    redirect_back(&zurueck, "geloescht", "geschaeftsjahr");
    Ok(())
}

/// Periode sperren oder entsperren (AK-5). Vermerkt Benutzer und Zeitpunkt.
#[server]
pub async fn set_period_locked(
    periode_id: i64,
    gesperrt: bool,
    zurueck: String,
) -> Result<(), ServerFnError> {
    let user = crate::backend::auth::session::require_user()?;
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let Some((_, year_status)) = repository::year_of_period(&mut tx, periode_id).await? else {
        return Err(ServerFnError::new("Periode nicht gefunden."));
    };
    if let Some(message) = lock_error(&year_status, gesperrt) {
        return Err(ServerFnError::new(message));
    }
    let status = if gesperrt { "gesperrt" } else { "offen" };
    repository::set_period_status(&mut tx, periode_id, status, user.id).await?;
    tx.commit().await?;
    let meldung = if gesperrt {
        "periode-gesperrt"
    } else {
        "periode-offen"
    };
    redirect_back(&zurueck, "gespeichert", meldung);
    Ok(())
}

/// Prüft ein Buchungsdatum (AK-6, AK-8): es muss in einem offenen Geschäftsjahr und einer offenen Periode liegen.
/// Wird von ANF-017 beim Buchen aufgerufen.
#[cfg(feature = "ssr")]
pub async fn check_booking_date(pool: &sqlx::PgPool, datum: &str) -> Result<(), ServerFnError> {
    let found = repository::year_for_date(pool, datum).await?;
    match booking_error(
        found
            .as_ref()
            .map(|(status, periode)| (status.as_str(), periode.as_deref())),
        datum,
    ) {
        Some(message) => Err(ServerFnError::new(message)),
        None => Ok(()),
    }
}

/// Zurück zur Seite (Meldung als Parameter). Die Rücksprung-Adresse wird nur innerhalb der Seite angenommen.
#[cfg(feature = "ssr")]
fn redirect_back(zurueck: &str, param: &str, value: &str) {
    let base = crate::forms::safe_return_url(zurueck)
        .filter(|u| u.starts_with(FISCAL_YEARS_URL))
        .unwrap_or(FISCAL_YEARS_URL);
    leptos_axum::redirect(&crate::forms::with_params(
        base,
        &[(param, value.to_string())],
    ));
}
