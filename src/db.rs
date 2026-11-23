//! Datenbankzugriff (nur Server).
//!
//! Zwei Arten von Datenbanken im selben PostgreSQL-Dienst (ANF-026):
//! - **Portal-Datenbank** `uniqwork_portal`: Zugänge, Sitzungen, Verzeichnis der Mandanten.
//!   Pool `PortalPool`, Migrationen in `migrations_portal/`.
//! - **Mandanten-Datenbanken** `uq_m_…`: je Mandant das ERP, Migrationen in `migrations/`.
//!   Das ERP verbindet sich mit der Rolle `uniqwork_erp` ohne `CREATEDB` (ANF-026 AK-21);
//!   Pools je Mandant verwaltet `TenantPools` (AK-22, AK-27).

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use sqlx::{
    PgPool, Postgres,
    migrate::{MigrateDatabase, Migrator},
    postgres::{PgConnectOptions, PgPoolOptions},
};

/// Migrationen der Mandanten-Datenbanken (das ERP).
pub static ERP_MIGRATIONS: Migrator = sqlx::migrate!("./migrations");
/// Migrationen der Portal-Datenbank (ANF-026 AK-7).
static PORTAL_MIGRATIONS: Migrator = sqlx::migrate!("./migrations_portal");

/// Name der Portal-Datenbank, wenn `PORTAL_DATABASE_URL` fehlt (ANF-026).
pub const PORTAL_DATABASE_NAME: &str = "uniqwork_portal";
/// PostgreSQL-Rolle für den laufenden ERP-Betrieb, ohne `CREATEDB` (ANF-026 AK-21).
pub const ERP_ROLE: &str = "uniqwork_erp";
/// Höchstens so viele Verbindungen je Mandant (ANF-026 AK-27).
pub const TENANT_MAX_CONNECTIONS: u32 = 3;
/// Unbenutzte Verbindungen eines Mandanten werden nach so vielen Sekunden geschlossen (ANF-026 AK-27).
pub const TENANT_IDLE_SECONDS: u64 = 300;

/// Verbindungspool der Portal-Datenbank (ANF-026).
/// Eigener Typ, damit er im Leptos-Kontext nicht mit einem ERP-Pool verwechselt wird.
#[derive(Clone, Debug)]
pub struct PortalPool(pub PgPool);

/// Pool der Mandanten-Datenbank der laufenden Anfrage. Die Middleware legt ihn in die Anfrage,
/// wenn der Zugang einen bereiten und definierten Mandanten hat (ANF-026 AK-22).
#[derive(Clone, Debug)]
pub struct TenantPool(pub PgPool);

/// Verbindungen zu den Mandanten-Datenbanken (ANF-026 AK-22, AK-27).
///
/// - `admin`: Zugangsdaten der Portal-Rolle (mit `CREATEDB`) – nur zum Anlegen, Migrieren und Löschen.
/// - `erp`: Rolle `uniqwork_erp` ohne `CREATEDB` – für den laufenden Betrieb.
///
/// Pools entstehen beim ersten Zugriff ohne Verbindung (`connect_lazy`); unbenutzte Verbindungen werden
/// nach `TENANT_IDLE_SECONDS` geschlossen, ein ruhender Mandant belegt also keine Verbindung.
#[derive(Clone)]
pub struct TenantPools {
    admin: PgConnectOptions,
    erp: PgConnectOptions,
    pools: Arc<Mutex<HashMap<String, PgPool>>>,
}

impl std::fmt::Debug for TenantPools {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TenantPools { … }")
    }
}

impl TenantPools {
    /// Verbindungsdaten der Portal-Rolle für die Datenbank `database` (Anlegen, Migrationen).
    pub fn admin_options(&self, database: &str) -> PgConnectOptions {
        self.admin.clone().database(database)
    }

    /// Pool der ERP-Rolle für die Mandanten-Datenbank `database`; wird beim ersten Aufruf angelegt.
    pub fn pool(&self, database: &str) -> PgPool {
        let mut pools = self.pools.lock().unwrap_or_else(|e| e.into_inner());
        pools
            .entry(database.to_string())
            .or_insert_with(|| {
                PgPoolOptions::new()
                    .max_connections(TENANT_MAX_CONNECTIONS)
                    .min_connections(0)
                    .idle_timeout(Duration::from_secs(TENANT_IDLE_SECONDS))
                    .connect_lazy_with(self.erp.clone().database(database))
            })
            .clone()
    }

    /// Pool eines Mandanten schliessen und vergessen (z. B. bevor seine Datenbank gelöscht wird).
    pub async fn forget(&self, database: &str) {
        let pool = self
            .pools
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(database);
        if let Some(pool) = pool {
            pool.close().await;
        }
    }
}

/// Alle Verbindungen beim Serverstart (ANF-026).
#[derive(Clone, Debug)]
pub struct Databases {
    pub portal: PortalPool,
    pub tenants: TenantPools,
}

/// Legt die Portal-Datenbank bei Bedarf an (ANF-026 AK-9) und führt ihre Migrationen aus, richtet die
/// Rolle `uniqwork_erp` mit einem neuen Zufallspasswort ein (AK-21) und bereitet die Pools der
/// Mandanten vor. Andere Datenbanken legt der Server beim Start nicht an.
///
/// Das Passwort der ERP-Rolle wird bei jedem Start neu erzeugt und nirgends gespeichert. Mehrere
/// gleichzeitig laufende Server teilen sich deshalb keinen PostgreSQL-Dienst (Railway: eine Instanz).
pub async fn connect() -> Result<Databases, Box<dyn std::error::Error>> {
    let explicit = std::env::var("PORTAL_DATABASE_URL").ok();
    let url = portal_url(explicit.as_deref(), &database_url()?)?;
    if !Postgres::database_exists(&url).await? {
        Postgres::create_database(&url).await?;
        leptos::logging::log!("Portal-Datenbank angelegt");
    }
    let portal = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    PORTAL_MIGRATIONS.run(&portal).await?;

    let admin = PgConnectOptions::from_str(&url)?;
    let password = crate::backend::auth::session::new_token();
    crate::backend::portal::repository::ensure_erp_role(&portal, ERP_ROLE, &password).await?;
    let erp = admin.clone().username(ERP_ROLE).password(&password);

    Ok(Databases {
        portal: PortalPool(portal),
        tenants: TenantPools {
            admin,
            erp,
            pools: Arc::default(),
        },
    })
}

fn database_url() -> Result<String, &'static str> {
    std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL ist nicht gesetzt (.env anlegen, Vorlage: .env.example)")
}

/// Webadresse der Portal-Datenbank: `PORTAL_DATABASE_URL`, wenn gesetzt und nicht leer;
/// sonst dieselbe Verbindung wie `DATABASE_URL` mit der Datenbank `uniqwork_portal`
/// (gleicher PostgreSQL-Dienst, ANF-026 offene Fragen).
pub fn portal_url(explicit: Option<&str>, database_url: &str) -> Result<String, &'static str> {
    if let Some(url) = explicit.map(str::trim).filter(|u| !u.is_empty()) {
        return Ok(url.to_string());
    }
    const ERR: &str = "DATABASE_URL hat keinen Datenbanknamen; PORTAL_DATABASE_URL setzen";
    let host_start = database_url.find("://").ok_or(ERR)? + 3;
    let path_start = database_url[host_start..].find('/').ok_or(ERR)? + host_start;
    let path_end = database_url[path_start..]
        .find('?')
        .map_or(database_url.len(), |i| i + path_start);
    if path_end <= path_start + 1 {
        return Err(ERR);
    }
    Ok(format!(
        "{}/{PORTAL_DATABASE_NAME}{}",
        &database_url[..path_start],
        &database_url[path_end..]
    ))
}

/// Pool der Mandanten-Datenbank der laufenden Anfrage (ANF-026 AK-22).
/// Fehlt er, hat der Zugang keinen bereiten und definierten Mandanten.
pub fn pool() -> Result<PgPool, leptos::prelude::ServerFnError> {
    leptos::prelude::use_context::<axum::http::request::Parts>()
        .and_then(|parts| parts.extensions.get::<TenantPool>().map(|p| p.0.clone()))
        .ok_or_else(|| {
            leptos::prelude::ServerFnError::new(crate::backend::auth::model::MSG_NO_TENANT)
        })
}

/// Pools der Mandanten aus dem Leptos-Kontext (Anlegen einer Datenbank, ANF-026 AK-11).
pub fn tenant_pools() -> Result<TenantPools, leptos::prelude::ServerFnError> {
    leptos::prelude::use_context::<TenantPools>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Keine Mandanten-Pools im Kontext"))
}

/// Verbindungspool der Portal-Datenbank aus dem Leptos-Kontext (ANF-026).
pub fn portal_pool() -> Result<PgPool, leptos::prelude::ServerFnError> {
    leptos::prelude::use_context::<PortalPool>()
        .map(|p| p.0)
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Kein Portal-Pool im Kontext"))
}

/// LIKE-Muster „enthält“, Sonderzeichen % _ \ maskiert.
pub fn like_pattern(q: &str) -> String {
    let mut p = String::from("%");
    for c in q.chars() {
        if matches!(c, '%' | '_' | '\\') {
            p.push('\\');
        }
        p.push(c);
    }
    p.push('%');
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn like_pattern_escapes() {
        assert_eq!(like_pattern("ab"), "%ab%");
        assert_eq!(like_pattern("50%_x\\"), "%50\\%\\_x\\\\%");
    }

    #[test]
    fn portal_url_prefers_explicit() {
        let url = portal_url(Some(" postgres://a@h/portal "), "postgres://a@h/erp").unwrap();
        assert_eq!(url, "postgres://a@h/portal");
    }

    #[test]
    fn portal_url_derived_from_database_url() {
        let erp = "postgres://postgres:geheim@localhost:5432/uniqwork";
        let expected = "postgres://postgres:geheim@localhost:5432/uniqwork_portal";
        assert_eq!(portal_url(None, erp).unwrap(), expected);
        assert_eq!(portal_url(Some(""), erp).unwrap(), expected);
    }

    #[test]
    fn portal_url_keeps_query() {
        let erp = "postgres://u:p@db.railway.internal:5432/railway?sslmode=require";
        assert_eq!(
            portal_url(None, erp).unwrap(),
            "postgres://u:p@db.railway.internal:5432/uniqwork_portal?sslmode=require"
        );
    }

    #[test]
    fn portal_url_without_database_name() {
        assert!(portal_url(None, "postgres://u:p@localhost:5432").is_err());
        assert!(portal_url(None, "postgres://u:p@localhost:5432/").is_err());
        assert!(portal_url(None, "postgres://u:p@localhost:5432/?sslmode=disable").is_err());
        assert!(portal_url(None, "kein-url").is_err());
    }
}
