//! SQL-Zugriffe für den Systemzustand (nur Server).

use sqlx::PgPool;

/// Versionstext des Datenbankservers.
pub async fn database_version(pool: &PgPool) -> Result<String, sqlx::Error> {
    let (version,): (String,) = sqlx::query_as("SELECT version()").fetch_one(pool).await?;
    Ok(version)
}
