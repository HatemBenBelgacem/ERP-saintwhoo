//! Server-Funktionen für Kontenplan und Systemkonten (ANF-011).
//!
//! Prüfen die Eingaben, rufen das Repository auf, übersetzen Datenbankfehler in
//! verständliche Meldungen und leiten nach dem Speichern zurück zur Liste.

use std::collections::BTreeMap;

use leptos::prelude::*;

use super::model::{Account, AccountInput, ChartSummary, SaveOutcome};

#[cfg(feature = "ssr")]
use super::{
    model::{
        FieldErrors, KMU_SYSTEM_ACCOUNTS, KMU_TEMPLATE, SYSTEM_ROLES, SystemAccount, role_label,
        system_assignment_error, system_role,
    },
    repository,
};

/// Webadresse der Einstellungsseite.
pub const ACCOUNTS_URL: &str = "/einstellungen/finanzen/kontenplan";

/// Kontenplan mit Suche, Kontoart-Filter und optional inaktiven Konten (AK-1, AK-2).
#[server]
pub async fn list_accounts(
    suche: String,
    kontoart: String,
    inaktive: bool,
) -> Result<Vec<Account>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    let kontoart = if super::model::KINDS.iter().any(|(k, _)| *k == kontoart) {
        kontoart
    } else {
        String::new()
    };
    Ok(repository::list(&pool, &suche, &pattern, &kontoart, inaktive).await?)
}

/// Ein Konto; `None`, wenn es es nicht gibt.
#[server]
pub async fn get_account(id: i64) -> Result<Option<Account>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::find(&pool, id).await?)
}

/// Anzahl Konten und Systemkonten (AK-3, AK-11, AK-12).
#[server]
pub async fn get_chart_summary() -> Result<ChartSummary, ServerFnError> {
    let pool = crate::db::pool()?;
    let anzahl = repository::count(&pool).await?;
    let assigned = repository::system_accounts(&pool).await?;
    let ids: Vec<i64> = assigned.iter().map(|(_, id)| *id).collect();
    let accounts = repository::find_many(&pool, &ids).await?;
    let systemkonten = SYSTEM_ROLES
        .iter()
        .map(|role| SystemAccount {
            rolle: role.key.to_string(),
            konto: assigned
                .iter()
                .find(|(r, _)| r == role.key)
                .and_then(|(_, id)| accounts.iter().find(|a| a.id == *id))
                .cloned(),
        })
        .collect();
    Ok(ChartSummary {
        anzahl,
        systemkonten,
    })
}

/// Neues Konto anlegen (AK-4, AK-5, AK-8). Danach zurück zur Liste (`zurueck`).
#[server]
pub async fn create_account(
    input: AccountInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let mut errors = input.validate();
    let pool = crate::db::pool()?;
    check_number(&pool, &input, None, &mut errors).await?;
    check_vat(&pool, &input, &mut errors).await?;
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let id = match repository::insert(&pool, &input).await {
        Ok(id) => id,
        Err(e) => return unique_violation(e),
    };
    redirect_back(&zurueck, "gespeichert", &input.nummer);
    Ok(SaveOutcome::Saved(id))
}

/// Konto bearbeiten und deaktivieren (AK-6, AK-10). Ein Systemkonto bleibt aktiv und behält
/// eine zur Rolle passende Kontoart (AK-11).
#[server]
pub async fn update_account(
    id: i64,
    input: AccountInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let mut errors = input.validate();
    let pool = crate::db::pool()?;
    let Some(current) = repository::find(&pool, id).await? else {
        return Err(ServerFnError::new("Konto nicht gefunden."));
    };
    check_number(&pool, &input, Some(id), &mut errors).await?;
    check_vat(&pool, &input, &mut errors).await?;
    // AK-6: Nach der ersten Buchung lassen sich Nummer und Kontoart nicht mehr ändern
    if current.gebucht {
        if current.nummer != input.nummer {
            errors.entry("nummer".into()).or_insert_with(|| {
                "Auf das Konto ist gebucht. Die Nummer lässt sich nicht mehr ändern.".into()
            });
        }
        if current.kontoart != input.kontoart {
            errors.entry("kontoart".into()).or_insert_with(|| {
                "Auf das Konto ist gebucht. Die Kontoart lässt sich nicht mehr ändern.".into()
            });
        }
    }
    for role in current.rollen.iter().filter_map(|r| system_role(r)) {
        if !input.is_active() {
            errors.entry("aktiv".into()).or_insert_with(|| {
                format!(
                    "Das Konto ist Systemkonto „{}“ und muss aktiv bleiben. Zuerst ein anderes Konto zuordnen.",
                    role.label
                )
            });
        } else if let Some(m) = system_assignment_error(role, &input.kontoart, true) {
            errors.entry("kontoart".into()).or_insert(m);
        }
    }
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    match repository::update(&pool, id, &input).await {
        Ok(0) => Err(ServerFnError::new("Konto nicht gefunden.")),
        Ok(_) => {
            redirect_back(&zurueck, "gespeichert", &input.nummer);
            Ok(SaveOutcome::Saved(id))
        }
        Err(e) => unique_violation(e),
    }
}

/// Konto löschen (AK-9). Wird es noch verwendet, wird das Löschen verhindert (AK-10).
#[server]
pub async fn delete_account(id: i64, zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    match repository::delete(&pool, id).await {
        Ok(Some(nummer)) => {
            redirect_back(&zurueck, "geloescht", &nummer);
            Ok(())
        }
        Ok(None) => Err(ServerFnError::new("Konto nicht gefunden.")),
        // 23503 = Fremdschlüssel verletzt: Systemkonto (später auch Buchungen, Bankkonten …)
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            let usage = match db.table() {
                Some("buchungszeile") => "in Buchungen".to_string(),
                Some("systemkonto") => {
                    let roles = repository::find(&pool, id)
                        .await?
                        .map(|a| a.rollen)
                        .unwrap_or_default();
                    let labels: Vec<String> = roles
                        .iter()
                        .map(|r| format!("„{}“", role_label(r)))
                        .collect();
                    format!("als Systemkonto {}", labels.join(", "))
                }
                Some(table) => format!("in der Tabelle „{table}“"),
                None => "in anderen Datensätzen".to_string(),
            };
            Err(ServerFnError::new(format!(
                "Das Konto kann nicht gelöscht werden, weil es {usage} verwendet wird. \
                 Stattdessen kann es deaktiviert werden."
            )))
        }
        Err(e) => Err(e.into()),
    }
}

/// Vorlage „KMU-Kontenrahmen“ übernehmen und Systemkonten vorbelegen (AK-3).
/// Nur bei leerem Kontenplan.
#[server]
pub async fn apply_template(zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    if repository::lock_and_count(&mut tx).await? > 0 {
        return Err(ServerFnError::new(
            "Die Vorlage kann nur bei einem leeren Kontenplan übernommen werden.",
        ));
    }
    repository::insert_template(&mut tx, KMU_TEMPLATE).await?;
    let mut assignments = Vec::new();
    for (role, nummer) in KMU_SYSTEM_ACCOUNTS {
        if let Some(id) = repository::id_by_number(&mut tx, nummer).await? {
            assignments.push((*role, id));
        }
    }
    repository::replace_system_accounts(&mut tx, &assignments).await?;
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", "vorlage");
    Ok(())
}

/// Systemkonten speichern (AK-11). `konten`: Rolle → Kennung des Kontos (leer = keines).
#[server]
pub async fn save_system_accounts(
    konten: BTreeMap<String, String>,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut errors = FieldErrors::new();
    let mut wanted: Vec<(&'static str, i64)> = Vec::new();
    for role in SYSTEM_ROLES {
        let value = konten.get(role.key).map_or("", |v| v.trim());
        if value.is_empty() {
            continue;
        }
        match value.parse::<i64>() {
            Ok(id) => wanted.push((role.key, id)),
            Err(_) => {
                errors.insert(role.key.into(), "Ungültiges Konto.".into());
            }
        }
    }
    let ids: Vec<i64> = wanted.iter().map(|(_, id)| *id).collect();
    let accounts = repository::find_many(&pool, &ids).await?;
    for (key, id) in &wanted {
        let role = system_role(key).expect("Rolle aus SYSTEM_ROLES");
        let message = match accounts.iter().find(|a| a.id == *id) {
            None => Some("Dieses Konto gibt es nicht mehr.".to_string()),
            Some(a) => system_assignment_error(role, &a.kontoart, a.aktiv),
        };
        if let Some(m) = message {
            errors.insert((*key).into(), m);
        }
    }
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let mut tx = pool.begin().await?;
    repository::replace_system_accounts(&mut tx, &wanted).await?;
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", "systemkonten");
    Ok(SaveOutcome::Saved(0))
}

/// Nummer frei? (AK-4)
#[cfg(feature = "ssr")]
async fn check_number(
    pool: &sqlx::PgPool,
    input: &AccountInput,
    id: Option<i64>,
    errors: &mut FieldErrors,
) -> Result<(), ServerFnError> {
    if !errors.contains_key("nummer") && repository::number_taken(pool, &input.nummer, id).await? {
        errors.insert(
            "nummer".into(),
            "Diese Kontonummer ist bereits vergeben.".into(),
        );
    }
    Ok(())
}

/// Eindeutigkeit, die zwischen Prüfung und Speichern verletzt wurde, als Feldfehler melden.
#[cfg(feature = "ssr")]
fn unique_violation(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    if let sqlx::Error::Database(db) = &e
        && db.code().as_deref() == Some("23505")
    {
        let mut errors = FieldErrors::new();
        errors.insert(
            "nummer".into(),
            "Diese Kontonummer ist bereits vergeben.".into(),
        );
        return Ok(SaveOutcome::Invalid(errors));
    }
    Err(e.into())
}

/// Zurück zur Liste (Suche und Filter bleiben erhalten) mit Meldungs-Parameter.
#[cfg(feature = "ssr")]
fn redirect_back(zurueck: &str, param: &str, value: &str) {
    let base = crate::forms::safe_return_url(zurueck)
        .filter(|u| u.starts_with(ACCOUNTS_URL))
        .unwrap_or(ACCOUNTS_URL);
    leptos_axum::redirect(&crate::forms::with_params(
        base,
        &[(param, value.to_string())],
    ));
}

/// Vorgabe-MWST prüfen: der Code muss existieren und aktiv sein (ANF-011 AK-7).
#[cfg(feature = "ssr")]
async fn check_vat(
    pool: &sqlx::PgPool,
    input: &AccountInput,
    errors: &mut FieldErrors,
) -> Result<(), ServerFnError> {
    if input.mwst_code_id.is_empty() || errors.contains_key("mwst_code_id") {
        return Ok(());
    }
    let aktiv = match input.mwst_id() {
        Some(id) => repository::mwst_code_active(pool, id).await?,
        None => false,
    };
    if !aktiv {
        errors.insert(
            "mwst_code_id".into(),
            "Dieser MWST-Code gibt es nicht oder er ist inaktiv.".into(),
        );
    }
    Ok(())
}
