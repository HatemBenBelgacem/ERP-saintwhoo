//! Server-Funktionen für Zahlungsbedingungen (ANF-016).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in verständliche
//! Meldungen und leiten nach dem Speichern zurück zur Seite.

use leptos::prelude::*;

use super::model::{PaymentTerm, PaymentTermInput, SaveOutcome};

#[cfg(feature = "ssr")]
use super::{model::FieldErrors, repository};

/// Webadresse der Zahlungsbedingungen-Seite.
pub const PAYMENT_TERMS_URL: &str = "/einstellungen/finanzen/zahlungsbedingungen";

/// Alle Zahlungsbedingungen, die Standard-Bedingung zuerst (AK-1).
#[server]
pub async fn list_payment_terms() -> Result<Vec<PaymentTerm>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::list(&pool).await?)
}

/// Neue Zahlungsbedingung anlegen (AK-2, AK-4). Die erste wird automatisch Standard.
#[server]
pub async fn create_payment_term(
    input: PaymentTermInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock(&mut tx).await?;

    let mut errors = input.validate();
    let standard = repository::count(&mut tx).await? == 0 || input.is_standard();
    if standard && !input.is_active() {
        errors.insert(
            "aktiv".into(),
            "Die Standard-Bedingung muss aktiv sein.".into(),
        );
    }
    let Some((frist, skonto)) = input.terms().filter(|_| errors.is_empty()) else {
        return Ok(SaveOutcome::Invalid(errors));
    };

    if standard {
        repository::clear_standard(&mut tx, None).await?;
    }
    let id = match repository::insert(&mut tx, &input, frist, skonto, standard, input.is_active())
        .await
    {
        Ok(id) => id,
        Err(e) => return unique_violation(e),
    };
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", &input.bezeichnung);
    Ok(SaveOutcome::Saved(id))
}

/// Zahlungsbedingung ändern (AK-2, AK-4). Die Standard-Bedingung bleibt aktiv und lässt sich nicht abwählen.
#[server]
pub async fn update_payment_term(
    id: i64,
    input: PaymentTermInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock(&mut tx).await?;
    let Some(current) = repository::find(&mut tx, id).await? else {
        return Err(ServerFnError::new("Zahlungsbedingung nicht gefunden."));
    };

    let mut errors = input.validate();
    if current.standard && !input.is_standard() {
        errors.insert(
            "standard".into(),
            "Es braucht genau eine Standard-Bedingung. Zuerst eine andere als Standard festlegen."
                .into(),
        );
    }
    if current.standard && !input.is_active() {
        errors.insert(
            "aktiv".into(),
            "Die Standard-Bedingung bleibt aktiv. Zuerst eine andere als Standard festlegen."
                .into(),
        );
    }
    let Some((frist, skonto)) = input.terms().filter(|_| errors.is_empty()) else {
        return Ok(SaveOutcome::Invalid(errors));
    };

    if input.is_standard() && !current.standard {
        repository::clear_standard(&mut tx, Some(id)).await?;
    }
    let changed = match repository::update(
        &mut tx,
        id,
        &input,
        frist,
        skonto,
        input.is_standard(),
        input.is_active(),
    )
    .await
    {
        Ok(changed) => changed,
        Err(e) => return unique_violation(e),
    };
    if changed == 0 {
        return Err(ServerFnError::new("Zahlungsbedingung nicht gefunden."));
    }
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", &input.bezeichnung);
    Ok(SaveOutcome::Saved(id))
}

/// Zahlungsbedingung löschen (AK-7). Die Standard-Bedingung lässt sich nicht löschen;
/// eine von Adressen verwendete Bedingung nur deaktivieren.
#[server]
pub async fn delete_payment_term(id: i64, zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock(&mut tx).await?;
    let Some(current) = repository::find(&mut tx, id).await? else {
        return Err(ServerFnError::new("Zahlungsbedingung nicht gefunden."));
    };
    if current.standard {
        return Err(ServerFnError::new(
            "Die Standard-Bedingung kann nicht gelöscht werden. Zuerst eine andere als Standard festlegen.",
        ));
    }
    match repository::delete(&mut tx, id).await {
        Ok(_) => {
            tx.commit().await?;
            redirect_back(&zurueck, "geloescht", "zahlungsbedingung");
            Ok(())
        }
        // 23503 = Fremdschlüssel verletzt: eine Adresse verwendet die Bedingung
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            Err(ServerFnError::new(
                "Die Zahlungsbedingung wird von Adressen verwendet und kann nicht gelöscht werden. Stattdessen kann sie deaktiviert werden.",
            ))
        }
        Err(e) => Err(e.into()),
    }
}

/// Eindeutigkeit der Standard-Bedingung, die zwischen Prüfung und Speichern verletzt wurde.
#[cfg(feature = "ssr")]
fn unique_violation(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    if let sqlx::Error::Database(db) = &e
        && db.code().as_deref() == Some("23505")
    {
        let mut errors = FieldErrors::new();
        errors.insert(
            "standard".into(),
            "Es darf nur eine Standard-Bedingung geben.".into(),
        );
        return Ok(SaveOutcome::Invalid(errors));
    }
    Err(e.into())
}

/// Zurück zur Seite mit Meldungs-Parameter; Rücksprung-Adresse nur innerhalb der Seite.
#[cfg(feature = "ssr")]
fn redirect_back(zurueck: &str, param: &str, value: &str) {
    let base = crate::forms::safe_return_url(zurueck)
        .filter(|u| u.starts_with(PAYMENT_TERMS_URL))
        .unwrap_or(PAYMENT_TERMS_URL);
    leptos_axum::redirect(&crate::forms::with_params(
        base,
        &[(param, value.to_string())],
    ));
}
