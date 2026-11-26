//! Server-Funktionen für Währungen und Umrechnungskurse (ANF-014).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in verständliche
//! Meldungen und leiten nach dem Speichern zurück zur Seite.

use leptos::prelude::*;

use super::model::{Currency, CurrencyInput, ExchangeRate, RateInput, SaveOutcome};

#[cfg(feature = "ssr")]
use super::{
    model::{BOOKING_CURRENCY, FieldErrors},
    repository,
};

/// Webadresse der Währungen-Seite.
pub const CURRENCIES_URL: &str = "/einstellungen/finanzen/waehrungen";

/// Alle Währungen mit letztem Kurs; die Buchhaltungswährung zuerst (AK-1).
#[server]
pub async fn list_currencies() -> Result<Vec<Currency>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::list(&pool).await?)
}

/// Heutiges Datum der Datenbank, als Vorgabe für das Kurs-Formular.
#[server]
pub async fn today_date() -> Result<String, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::today(&pool).await?)
}

/// Neue Währung anlegen (AK-2). Die Buchhaltungswährung wird nicht hier angelegt.
#[server]
pub async fn create_currency(
    input: CurrencyInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let mut errors = input.validate(true);
    let Some(rundung) = input.rundung_value().filter(|_| errors.is_empty()) else {
        return Ok(SaveOutcome::Invalid(errors));
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    if repository::find(&mut tx, &input.code).await?.is_some() {
        errors.insert("code".into(), "Diese Währung ist bereits erfasst.".into());
        return Ok(SaveOutcome::Invalid(errors));
    }
    if let Err(e) = repository::insert(&mut tx, &input, rundung).await {
        return unique_violation(e);
    }
    tx.commit().await?;
    redirect_back(&zurueck, &format!("waehrung {}", input.code));
    Ok(SaveOutcome::Saved(0))
}

/// Währung ändern: Bezeichnung, Rundung, Aktiv-Status (AK-2, AK-5). Die Buchhaltungswährung bleibt aktiv.
#[server]
pub async fn update_currency(
    code: String,
    input: CurrencyInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let Some(current) = repository::find(&mut tx, &code).await? else {
        return Err(ServerFnError::new("Währung nicht gefunden."));
    };
    let mut errors = input.validate(false);
    if current.buchhaltungswaehrung && !input.is_active() {
        errors.insert(
            "aktiv".into(),
            "Die Buchhaltungswährung bleibt aktiv.".into(),
        );
    }
    let Some(rundung) = input.rundung_value().filter(|_| errors.is_empty()) else {
        return Ok(SaveOutcome::Invalid(errors));
    };
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    if repository::update(&mut tx, &code, &input, rundung).await? == 0 {
        return Err(ServerFnError::new("Währung nicht gefunden."));
    }
    tx.commit().await?;
    redirect_back(&zurueck, &format!("waehrung {code}"));
    Ok(SaveOutcome::Saved(0))
}

/// Währung löschen (AK-5). Die Buchhaltungswährung und verwendete Währungen lassen sich nicht löschen.
#[server]
pub async fn delete_currency(code: String, zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let Some(current) = repository::find(&mut tx, &code).await? else {
        return Err(ServerFnError::new("Währung nicht gefunden."));
    };
    if current.buchhaltungswaehrung {
        return Err(ServerFnError::new(
            "Die Buchhaltungswährung kann nicht gelöscht werden.",
        ));
    }
    match repository::delete(&mut tx, &code).await {
        Ok(_) => {
            tx.commit().await?;
            redirect_back(&zurueck, "geloescht waehrung");
            Ok(())
        }
        // 23503 = Fremdschlüssel verletzt: ein Einkaufsauftrag verwendet die Währung
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            Err(ServerFnError::new(
                "Die Währung wird in Belegen verwendet und kann nicht gelöscht werden. Stattdessen kann sie deaktiviert werden.",
            ))
        }
        Err(e) => Err(e.into()),
    }
}

/// Kurs einer Fremdwährung erfassen (AK-3): CHF für eine Einheit an einem Datum.
#[server]
pub async fn save_rate(
    code: String,
    input: RateInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let Some(currency) = repository::find(&mut tx, &code).await? else {
        return Err(ServerFnError::new("Währung nicht gefunden."));
    };
    if currency.code == BOOKING_CURRENCY {
        return Err(ServerFnError::new(
            "Für die Buchhaltungswährung braucht es keinen Kurs.",
        ));
    }
    let mut errors = input.validate();
    let Some(kurs) = input.kurs_value().filter(|_| errors.is_empty()) else {
        return Ok(SaveOutcome::Invalid(errors));
    };
    if let Err(e) = repository::insert_rate(&mut tx, &code, &input, kurs).await {
        if let sqlx::Error::Database(db) = &e
            && db.code().as_deref() == Some("23505")
        {
            errors.insert(
                "datum".into(),
                "Für dieses Datum gibt es schon einen Kurs.".into(),
            );
            return Ok(SaveOutcome::Invalid(errors));
        }
        return Err(e.into());
    }
    tx.commit().await?;
    redirect_back(&zurueck, &format!("kurs {code}"));
    Ok(SaveOutcome::Saved(0))
}

/// Kurs für den Beleg (AK-4): der neueste Kurs mit Datum bis zum Belegdatum. Wird von ANF-018 und ANF-019 verwendet.
#[server]
pub async fn get_exchange_rate(
    code: String,
    datum: String,
) -> Result<Option<ExchangeRate>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::rate_on(&pool, &code, &datum).await?)
}

/// Eindeutigkeit, die zwischen Prüfung und Speichern verletzt wurde (Code schon vergeben).
#[cfg(feature = "ssr")]
fn unique_violation(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    if let sqlx::Error::Database(db) = &e
        && db.code().as_deref() == Some("23505")
    {
        let mut errors = FieldErrors::new();
        errors.insert("code".into(), "Diese Währung ist bereits erfasst.".into());
        return Ok(SaveOutcome::Invalid(errors));
    }
    Err(e.into())
}

/// Zurück zur Währungen-Seite; `meldung` steht als Text im Parameter `gespeichert` oder `geloescht`.
/// Die Meldung ist „<art> <code>“, z. B. „kurs EUR“ oder „geloescht waehrung“.
#[cfg(feature = "ssr")]
fn redirect_back(zurueck: &str, meldung: &str) {
    let base = crate::forms::safe_return_url(zurueck)
        .filter(|u| u.starts_with(CURRENCIES_URL))
        .unwrap_or(CURRENCIES_URL);
    let (param, value) = match meldung.strip_prefix("geloescht ") {
        Some(rest) => ("geloescht", rest.to_string()),
        None => ("gespeichert", meldung.to_string()),
    };
    leptos_axum::redirect(&crate::forms::with_params(base, &[(param, value)]));
}
