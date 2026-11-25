//! Server-Funktionen für MWST-Codes, MWST-Einstellungen und die Standard-Codes (ANF-012).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in verständliche
//! Meldungen und leiten nach dem Speichern zurück zur Seite.

use leptos::prelude::*;

use super::model::{SaveOutcome, VatCode, VatCodeInput, VatOverview, VatSettingsInput};

#[cfg(feature = "ssr")]
use super::{
    model::{
        CodeValues, FieldErrors, STANDARD_CODES, STANDARD_VALID_FROM, konto_error,
        locked_change_errors, pflicht_gilt_am, system_role_for,
    },
    repository,
};

/// Webadresse der MWST-Seite.
pub const VAT_URL: &str = "/einstellungen/finanzen/mwst";

/// Codes, Einstellungen und UID der Firma für die MWST-Seite (AK-1, AK-8).
#[server]
pub async fn get_vat_overview() -> Result<VatOverview, ServerFnError> {
    let pool = crate::db::pool()?;
    let settings = repository::settings(&pool).await?.unwrap_or_default();
    let codes = repository::list_codes(&pool).await?;
    let uid = repository::uid(&pool).await?;
    Ok(VatOverview {
        settings,
        codes,
        uid,
    })
}

/// Vorsteuer-Codes, die der Einkauf heute wählen darf (AK-9). Leer, solange die MWST-Pflicht aus ist.
#[server]
pub async fn list_purchase_codes() -> Result<Vec<VatCode>, ServerFnError> {
    let pool = crate::db::pool()?;
    let settings = repository::settings(&pool).await?.unwrap_or_default();
    let today = repository::today(&pool).await?;
    if !pflicht_gilt_am(&settings, &today) {
        return Ok(Vec::new());
    }
    Ok(repository::list_codes(&pool)
        .await?
        .into_iter()
        .filter(|c| c.art == "vorsteuer" && c.aktiv && c.is_valid_on(&today))
        .collect())
}

/// MWST-Einstellungen speichern (AK-8). Die Pflicht braucht eine UID der Firma.
#[server]
pub async fn save_vat_settings(
    input: VatSettingsInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let pool = crate::db::pool()?;
    let uid = repository::uid(&pool).await?;
    let settings = match input.check(uid.is_some()) {
        Ok(settings) => settings,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    repository::update_settings(&pool, &settings).await?;
    redirect_back(&zurueck, "gespeichert", "mwst-einstellungen");
    Ok(SaveOutcome::Saved(0))
}

/// Neuen MWST-Code anlegen (AK-3, AK-4).
#[server]
pub async fn create_vat_code(
    input: VatCodeInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let values = match input.normalized().check() {
        Ok(values) => values,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let errors = check_references(&pool, &values, None).await?;
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let mut tx = pool.begin().await?;
    let id = match repository::insert_code(&mut tx, &values).await {
        Ok(id) => id,
        Err(e) => return unique_violation(e),
    };
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", &values.code);
    Ok(SaveOutcome::Saved(id))
}

/// MWST-Code ändern (AK-5). Ein verwendeter Code behält Satz und Art.
#[server]
pub async fn update_vat_code(
    id: i64,
    input: VatCodeInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(existing) = repository::find_code(&pool, id).await? else {
        return Err(ServerFnError::new("MWST-Code nicht gefunden."));
    };
    let values = match input.normalized().check() {
        Ok(values) => values,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let mut errors = locked_change_errors(&existing, &values);
    errors.extend(check_references(&pool, &values, Some(id)).await?);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let mut tx = pool.begin().await?;
    let changed = match repository::update_code(&mut tx, id, &values).await {
        Ok(changed) => changed,
        Err(e) => return unique_violation(e),
    };
    if changed == 0 {
        return Err(ServerFnError::new("MWST-Code nicht gefunden."));
    }
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", &values.code);
    Ok(SaveOutcome::Saved(id))
}

/// MWST-Code löschen (AK-7). Ein verwendeter Code kann nur deaktiviert werden.
#[server]
pub async fn delete_vat_code(id: i64, zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    match repository::delete_code(&mut tx, id).await {
        Ok(0) => Err(ServerFnError::new("MWST-Code nicht gefunden.")),
        Ok(_) => {
            tx.commit().await?;
            redirect_back(&zurueck, "geloescht", "mwst-code");
            Ok(())
        }
        // 23503 = Fremdschlüssel verletzt: der Code wird verwendet (Einkauf, später Rechnungen)
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            Err(ServerFnError::new(
                "Der Code wird verwendet und kann nicht gelöscht werden. Stattdessen kann er deaktiviert werden.",
            ))
        }
        Err(e) => Err(e.into()),
    }
}

/// Schweizer Standard-Codes anlegen (AK-2). Nur bei leerer Code-Liste; die Konten kommen aus dem Kontenplan.
#[server]
pub async fn adopt_standard_codes(zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock_codes(&mut tx).await?;
    if repository::count_codes(&mut tx).await? > 0 {
        return Err(ServerFnError::new(
            "Die Standard-Codes können nur übernommen werden, solange noch keine MWST-Codes vorhanden sind.",
        ));
    }
    for standard in STANDARD_CODES {
        let konto = match standard.konto_nummer {
            Some(nummer) => repository::konto_id_by_number(&mut tx, nummer).await?,
            None => repository::system_konto_id(&mut tx, system_role_for(standard.art)).await?,
        };
        let Some(konto_id) = konto else {
            return Err(ServerFnError::new(format!(
                "Das Konto für {} fehlt. Zuerst den Kontenplan übernehmen und die Systemkonten zuordnen.",
                standard.code
            )));
        };
        let values = CodeValues {
            code: standard.code.into(),
            bezeichnung: standard.bezeichnung.into(),
            art: standard.art.into(),
            satz: standard.satz,
            gueltig_ab: STANDARD_VALID_FROM.into(),
            gueltig_bis: None,
            konto_id,
            formularziffer: None,
            aktiv: true,
        };
        repository::insert_code(&mut tx, &values).await?;
    }
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", "standard");
    Ok(())
}

/// Prüft einen Code für den Einkauf (AK-9): Pflicht eingeschaltet, aktiv, Vorsteuer und heute gültig.
/// Wird von ANF-004 beim Speichern einer Position verwendet.
#[cfg(feature = "ssr")]
pub async fn purchase_code(pool: &sqlx::PgPool, id: i64) -> Result<VatCode, ServerFnError> {
    let settings = repository::settings(pool).await?.unwrap_or_default();
    let today = repository::today(pool).await?;
    if !pflicht_gilt_am(&settings, &today) {
        return Err(ServerFnError::new(
            "Die MWST-Pflicht gilt heute noch nicht. Die Position wird ohne MWST erfasst.",
        ));
    }
    let Some(code) = repository::find_code(pool, id).await? else {
        return Err(ServerFnError::new("Der MWST-Code gibt es nicht mehr."));
    };
    if !code.aktiv || code.art != "vorsteuer" {
        return Err(ServerFnError::new(
            "Im Einkauf ist nur ein aktiver Vorsteuer-Code wählbar.",
        ));
    }
    if !code.is_valid_on(&today) {
        return Err(ServerFnError::new(format!(
            "Der Code {} ist heute nicht gültig.",
            code.code
        )));
    }
    Ok(code)
}

/// Feldfehler, die nur mit der Datenbank feststellbar sind: Code schon vergeben, Konto passt nicht.
#[cfg(feature = "ssr")]
async fn check_references(
    pool: &sqlx::PgPool,
    values: &CodeValues,
    except: Option<i64>,
) -> Result<FieldErrors, ServerFnError> {
    let mut errors = FieldErrors::new();
    if repository::code_taken(pool, &values.code, except).await? {
        errors.insert("code".into(), "Dieser Code ist bereits vergeben.".into());
    }
    match repository::konto_info(pool, values.konto_id).await? {
        None => {
            errors.insert("konto_id".into(), "Dieses Konto gibt es nicht.".into());
        }
        Some((kontoart, aktiv)) => {
            if let Some(message) = konto_error(&values.art, &kontoart, aktiv) {
                errors.insert("konto_id".into(), message);
            }
        }
    }
    Ok(errors)
}

/// Eindeutigkeit, die zwischen Prüfung und Speichern verletzt wurde, als Feldfehler melden.
#[cfg(feature = "ssr")]
fn unique_violation(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    if let sqlx::Error::Database(db) = &e
        && db.code().as_deref() == Some("23505")
    {
        let mut errors = FieldErrors::new();
        errors.insert("code".into(), "Dieser Code ist bereits vergeben.".into());
        return Ok(SaveOutcome::Invalid(errors));
    }
    Err(e.into())
}

/// Zurück zur MWST-Seite mit Meldungs-Parameter; Rücksprung-Adresse nur innerhalb der Seite.
#[cfg(feature = "ssr")]
fn redirect_back(zurueck: &str, param: &str, value: &str) {
    let base = crate::forms::safe_return_url(zurueck)
        .filter(|u| u.starts_with(VAT_URL))
        .unwrap_or(VAT_URL);
    leptos_axum::redirect(&crate::forms::with_params(
        base,
        &[(param, value.to_string())],
    ));
}
