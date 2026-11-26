//! Server-Funktionen für eigene Bankkonten (ANF-015).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in verständliche
//! Meldungen und leiten nach dem Speichern zurück zur Seite.

use leptos::prelude::*;

use super::model::{BankAccount, BankAccountInput, SaveOutcome};

#[cfg(feature = "ssr")]
use super::{model::FieldErrors, repository};
#[cfg(feature = "ssr")]
use sqlx::PgConnection;

/// Webadresse der Bankkonten-Seite.
pub const BANK_ACCOUNTS_URL: &str = "/einstellungen/finanzen/bankkonten";

/// Alle Bankkonten, das Standard-Konto zuerst (AK-1).
#[server]
pub async fn list_bank_accounts() -> Result<Vec<BankAccount>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::list(&pool).await?)
}

/// Neues Bankkonto anlegen (AK-2, AK-5). Das erste Konto wird automatisch Standard.
#[server]
pub async fn create_bank_account(
    input: BankAccountInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock(&mut tx).await?;

    let mut errors = input.validate();
    if errors.is_empty() {
        errors.extend(check_references(&mut tx, &input, None).await?);
    }
    let standard = repository::count(&mut tx).await? == 0 || input.is_standard();
    if standard && !input.is_active() {
        errors.insert("aktiv".into(), "Das Standard-Konto muss aktiv sein.".into());
    }
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let Some(konto_id) = input.konto() else {
        return Err(ServerFnError::new("Konto fehlt."));
    };

    if standard {
        repository::clear_standard(&mut tx, None).await?;
    }
    let id = match repository::insert(&mut tx, &input, konto_id, standard, input.is_active()).await
    {
        Ok(id) => id,
        Err(e) => return unique_violation(e),
    };
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", &input.bezeichnung);
    Ok(SaveOutcome::Saved(id))
}

/// Bankkonto ändern (AK-2, AK-5, AK-6). Das Standard-Konto lässt sich nicht deaktivieren oder abwählen.
#[server]
pub async fn update_bank_account(
    id: i64,
    input: BankAccountInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock(&mut tx).await?;
    let Some(current) = repository::find(&mut tx, id).await? else {
        return Err(ServerFnError::new("Bankkonto nicht gefunden."));
    };

    let mut errors = input.validate();
    if errors.is_empty() {
        errors.extend(check_references(&mut tx, &input, Some(id)).await?);
    }
    if current.standard && !input.is_standard() {
        errors.insert(
            "standard".into(),
            "Es braucht genau ein Standard-Konto. Zuerst ein anderes Konto als Standard festlegen."
                .into(),
        );
    }
    if current.standard && !input.is_active() {
        errors.insert(
            "aktiv".into(),
            "Das Standard-Konto bleibt aktiv. Zuerst ein anderes Konto als Standard festlegen."
                .into(),
        );
    }
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let Some(konto_id) = input.konto() else {
        return Err(ServerFnError::new("Konto fehlt."));
    };

    if input.is_standard() && !current.standard {
        repository::clear_standard(&mut tx, Some(id)).await?;
    }
    let changed = match repository::update(
        &mut tx,
        id,
        &input,
        konto_id,
        input.is_standard(),
        input.is_active(),
    )
    .await
    {
        Ok(changed) => changed,
        Err(e) => return unique_violation(e),
    };
    if changed == 0 {
        return Err(ServerFnError::new("Bankkonto nicht gefunden."));
    }
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", &input.bezeichnung);
    Ok(SaveOutcome::Saved(id))
}

/// Bankkonto löschen (AK-6). Das Standard-Konto lässt sich nicht löschen.
/// Zahlungen (ANF-020) gibt es noch nicht; ihre Prüfung folgt mit ANF-020.
#[server]
pub async fn delete_bank_account(id: i64, zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    repository::lock(&mut tx).await?;
    let Some(current) = repository::find(&mut tx, id).await? else {
        return Err(ServerFnError::new("Bankkonto nicht gefunden."));
    };
    if current.standard {
        return Err(ServerFnError::new(
            "Das Standard-Konto kann nicht gelöscht werden. Zuerst ein anderes Konto als Standard festlegen.",
        ));
    }
    repository::delete(&mut tx, id).await?;
    tx.commit().await?;
    redirect_back(&zurueck, "geloescht", "bankkonto");
    Ok(())
}

/// Prüft die Verweise auf den Kontenplan: Konto gibt es, ist ein aktives Aktivkonto und gehört noch zu keinem Bankkonto (AK-2).
#[cfg(feature = "ssr")]
async fn check_references(
    conn: &mut PgConnection,
    input: &BankAccountInput,
    except: Option<i64>,
) -> Result<FieldErrors, ServerFnError> {
    let mut errors = FieldErrors::new();
    let Some(konto_id) = input.konto() else {
        return Ok(errors);
    };
    match repository::konto_info(conn, konto_id).await? {
        None => {
            errors.insert("konto_id".into(), "Dieses Konto gibt es nicht.".into());
        }
        Some((kontoart, aktiv)) => {
            if kontoart != "aktiv" {
                errors.insert(
                    "konto_id".into(),
                    "Ein Bankkonto braucht ein Aktivkonto aus dem Kontenplan.".into(),
                );
            } else if !aktiv {
                errors.insert(
                    "konto_id".into(),
                    "Dieses Konto ist im Kontenplan inaktiv.".into(),
                );
            } else if repository::konto_taken(conn, konto_id, except).await? {
                errors.insert(
                    "konto_id".into(),
                    "Dieses Konto gehört schon zu einem Bankkonto.".into(),
                );
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
        errors.insert(
            "konto_id".into(),
            "Dieses Konto gehört schon zu einem Bankkonto.".into(),
        );
        return Ok(SaveOutcome::Invalid(errors));
    }
    Err(e.into())
}

/// Zurück zur Bankkonten-Seite mit Meldungs-Parameter; Rücksprung-Adresse nur innerhalb der Seite.
#[cfg(feature = "ssr")]
fn redirect_back(zurueck: &str, param: &str, value: &str) {
    let base = crate::forms::safe_return_url(zurueck)
        .filter(|u| u.starts_with(BANK_ACCOUNTS_URL))
        .unwrap_or(BANK_ACCOUNTS_URL);
    leptos_axum::redirect(&crate::forms::with_params(
        base,
        &[(param, value.to_string())],
    ));
}
