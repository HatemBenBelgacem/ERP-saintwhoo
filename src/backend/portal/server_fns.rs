//! Server-Funktionen des Portals (ANF-026). Alle unter `/api/portal`: angemeldet, aber ohne Mandant erreichbar.

use leptos::prelude::*;

#[cfg(feature = "ssr")]
use super::{
    model::{
        CreationDecision, MSG_FAILED, MSG_LIMIT, MSG_NOT_ADMIN, MSG_RUNNING, STATUS_FAILED,
        STATUS_READY, creation_decision,
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
    leptos_axum::redirect(crate::backend::auth::model::PORTAL_PATH);
    Ok(())
}

#[cfg(test)]
mod tests {
    use leptos::server_fn::ServerFn;

    use super::*;
    use crate::backend::auth::model::PORTAL_API_PREFIX;

    #[test]
    fn reachable_without_tenant() {
        // Ohne Mandant lässt die Middleware nur `/api/portal/…` durch (ANF-026 AK-19)
        let path = <CreateDatabase as ServerFn>::PATH;
        assert!(path.starts_with(&format!("{PORTAL_API_PREFIX}/")), "{path}");
    }
}
