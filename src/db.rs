//! Datenbankzugriff (nur Server).

use sqlx::{PgPool, Postgres, migrate::MigrateDatabase, postgres::PgPoolOptions};

/// Legt die Datenbank bei Bedarf an, baut den Verbindungspool auf
/// und führt ausstehende Migrationen aus.
pub async fn connect() -> Result<PgPool, Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL ist nicht gesetzt (.env anlegen, Vorlage: .env.example)")?;

    if !Postgres::database_exists(&url).await? {
        Postgres::create_database(&url).await?;
        leptos::logging::log!("Datenbank angelegt");
    }

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

/// Verbindungspool aus dem Leptos-Kontext (in `main.rs` bereitgestellt).
pub fn pool() -> Result<PgPool, leptos::prelude::ServerFnError> {
    leptos::prelude::use_context::<PgPool>()
        .ok_or_else(|| leptos::prelude::ServerFnError::new("Kein Datenbank-Pool im Kontext"))
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
}
