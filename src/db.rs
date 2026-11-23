//! Datenbankzugriff (nur Server).

use sqlx::{
    PgPool, Postgres,
    migrate::{MigrateDatabase, Migrator},
    postgres::PgPoolOptions,
};

/// Migrationen der Mandanten-Datenbank (das ERP).
static ERP_MIGRATIONS: Migrator = sqlx::migrate!("./migrations");
/// Migrationen der Portal-Datenbank (ANF-026 AK-7).
static PORTAL_MIGRATIONS: Migrator = sqlx::migrate!("./migrations_portal");

/// Name der Portal-Datenbank, wenn `PORTAL_DATABASE_URL` fehlt (ANF-026).
pub const PORTAL_DATABASE_NAME: &str = "uniqwork_portal";

/// Verbindungspool der Portal-Datenbank (ANF-026).
/// Eigener Typ, damit er im Leptos-Kontext nicht mit dem ERP-Pool (`PgPool`) verwechselt wird.
#[derive(Clone, Debug)]
pub struct PortalPool(pub PgPool);

/// Legt die ERP-Datenbank bei Bedarf an, baut den Verbindungspool auf
/// und führt ausstehende Migrationen aus.
pub async fn connect() -> Result<PgPool, Box<dyn std::error::Error>> {
    let url = database_url()?;
    open(&url, &ERP_MIGRATIONS, "Datenbank").await
}

/// Legt die Portal-Datenbank bei Bedarf an (ANF-026 AK-9), baut ihren Verbindungspool auf
/// und führt die Migrationen aus `migrations_portal/` aus.
pub async fn connect_portal() -> Result<PortalPool, Box<dyn std::error::Error>> {
    let explicit = std::env::var("PORTAL_DATABASE_URL").ok();
    let url = portal_url(explicit.as_deref(), &database_url()?)?;
    Ok(PortalPool(
        open(&url, &PORTAL_MIGRATIONS, "Portal-Datenbank").await?,
    ))
}

fn database_url() -> Result<String, &'static str> {
    std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL ist nicht gesetzt (.env anlegen, Vorlage: .env.example)")
}

/// Datenbank anlegen, falls sie fehlt, Pool aufbauen und Migrationen ausführen.
async fn open(
    url: &str,
    migrator: &Migrator,
    label: &str,
) -> Result<PgPool, Box<dyn std::error::Error>> {
    if !Postgres::database_exists(url).await? {
        Postgres::create_database(url).await?;
        leptos::logging::log!("{label} angelegt");
    }

    let pool = PgPoolOptions::new().max_connections(5).connect(url).await?;
    migrator.run(&pool).await?;
    Ok(pool)
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

/// Verbindungspool aus dem Leptos-Kontext (in `main.rs` bereitgestellt).
pub fn pool() -> Result<PgPool, leptos::prelude::ServerFnError> {
    leptos::prelude::use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Kein Datenbank-Pool im Kontext"))
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
