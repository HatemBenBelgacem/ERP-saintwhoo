//! Server-Funktionen des Portals (ANF-026). Alle unter `/api/portal`: angemeldet, aber ohne Mandant erreichbar.

use leptos::prelude::*;

use crate::backend::addresses::model::AddressInput;
use crate::backend::company::model::{CompanyInput, SaveOutcome};

#[cfg(feature = "ssr")]
use super::{
    model::{
        CreationDecision, MSG_ALREADY_DEFINED, MSG_FAILED, MSG_LIMIT, MSG_NOT_ADMIN, MSG_NOT_READY,
        MSG_RUNNING, STATUS_FAILED, STATUS_READY, creation_decision,
    },
    provision, repository,
};

/// Eigene Datenbank erstellen (ANF-026 AK-10 bis AK-15). Der Mandant ist danach in allen Sitzungen
/// des Administrators gewählt; als Nächstes wird der Mandant definiert (AK-16).
#[server(prefix = "/api/portal")]
pub async fn create_database() -> Result<(), ServerFnError> {
    let user = crate::backend::auth::session::require_user()?;
    if !user.ist_administrator {
        return Err(ServerFnError::new(MSG_NOT_ADMIN));
    }
    let portal = crate::db::portal_pool()?;
    let tenants = crate::db::tenant_pools()?;

    // AK-14: Zeile des Administrators sperren, dann entscheiden – zwei gleichzeitige Klicks laufen nacheinander
    let mut tx = portal.begin().await?;
    repository::lock_owner(&mut tx, user.portal_id).await?;
    let decision = {
        let existing = repository::tenants_of(&mut tx, user.portal_id).await?;
        let existing: Vec<(i64, &str)> = existing.iter().map(|(id, s)| (*id, s.as_str())).collect();
        creation_decision(&existing)
    };
    let (id, database) = match decision {
        CreationDecision::New => repository::insert_tenant(&mut tx, user.portal_id).await?,
        CreationDecision::Retry(id) => (id, repository::restart_tenant(&mut tx, id).await?),
        CreationDecision::AlreadyRunning => return Err(ServerFnError::new(MSG_RUNNING)),
        CreationDecision::LimitReached => return Err(ServerFnError::new(MSG_LIMIT)),
    };
    tx.commit().await?;
    // Mandant sofort in den Sitzungen wählen: das Portal zeigt so auch „Fehler – Erneut versuchen“ (AK-13)
    repository::select_in_sessions(&portal, id, user.portal_id).await?;

    let (vorname, nachname, email) = repository::owner_details(&portal, user.portal_id).await?;
    let owner = provision::Owner {
        portal_id: user.portal_id,
        vorname,
        nachname,
        email,
    };
    if let Err(e) = provision::create(portal.clone(), tenants, database.clone(), owner).await {
        leptos::logging::error!("Datenbank {database} konnte nicht erstellt werden: {e}");
        repository::set_status(&portal, id, STATUS_FAILED).await?;
        return Err(ServerFnError::new(MSG_FAILED));
    }
    repository::set_status(&portal, id, STATUS_READY).await?;
    // AK-16: direkt danach das Modal-Fenster „Mandant definieren“
    leptos_axum::redirect(&format!(
        "{}?{}={id}",
        crate::backend::auth::model::PORTAL_PATH,
        crate::backend::auth::model::TENANT_PARAM
    ));
    Ok(())
}

/// Mandant definieren (ANF-026 AK-16 bis AK-18): In der Mandanten-Datenbank eine Adresse vom Typ „Firma“ und
/// die eigene Firma (ANF-025) anlegen, im Portal den Firmennamen setzen. Danach geht es ins ERP.
/// `input` enthält nur die UID (Prüfung wie ANF-025); Handelsregister, Fusszeile und Logo folgen im ERP.
#[server(prefix = "/api/portal")]
pub async fn define_tenant(
    input: CompanyInput,
    adresse: AddressInput,
) -> Result<SaveOutcome, ServerFnError> {
    let user = crate::backend::auth::session::require_user()?;
    let Some(tenant) = user.mandant.as_ref().filter(|t| t.status == STATUS_READY) else {
        return Err(ServerFnError::new(MSG_NOT_READY));
    };
    if tenant.firmenname.is_some() {
        return Err(ServerFnError::new(MSG_ALREADY_DEFINED));
    }

    let input = CompanyInput {
        modus: "neu".into(),
        uid: input.uid,
        ..CompanyInput::default()
    }
    .normalized();
    let mut errors = input.validate();
    let (address, address_errors) =
        crate::backend::company::model::prepare_company_address(adresse);
    errors.extend(address_errors);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    // Die Middleware legt ohne definierten Mandanten keinen Pool in die Anfrage – hier gezielt der eigene
    let pool = crate::db::tenant_pools()?.pool(&tenant.datenbankname);
    // Wiederholung nach einem Abbruch zwischen beiden Datenbanken: bestehende Firma übernehmen
    let company_name = match crate::backend::company::repository::find(&pool).await? {
        Some(company) => company.name,
        None => {
            let mut tx = pool.begin().await?;
            let address_id =
                crate::backend::addresses::repository::insert(&mut *tx, &address).await?;
            crate::backend::company::repository::insert(&mut tx, address_id, &input).await?;
            tx.commit().await?;
            address.firmenname.clone()
        }
    };

    let portal = crate::db::portal_pool()?;
    if repository::define_tenant(&portal, tenant.id, user.portal_id, &company_name).await? == 0 {
        return Err(ServerFnError::new(MSG_ALREADY_DEFINED));
    }
    leptos_axum::redirect("/");
    Ok(SaveOutcome::Saved(tenant.id))
}

#[cfg(test)]
mod tests {
    use leptos::server_fn::ServerFn;

    use super::*;
    use crate::backend::auth::model::PORTAL_API_PREFIX;

    #[test]
    fn reachable_without_tenant() {
        // Ohne Mandant lässt die Middleware nur `/api/portal/…` durch (ANF-026 AK-19)
        for path in [
            <CreateDatabase as ServerFn>::PATH,
            <DefineTenant as ServerFn>::PATH,
        ] {
            assert!(path.starts_with(&format!("{PORTAL_API_PREFIX}/")), "{path}");
        }
    }
}
