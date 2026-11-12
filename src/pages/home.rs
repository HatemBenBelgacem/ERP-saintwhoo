use leptos::prelude::*;

/// Liefert die PostgreSQL-Version – prüft, ob die Datenbank erreichbar ist.
#[server]
pub async fn db_status() -> Result<String, ServerFnError> {
    let pool = use_context::<sqlx::PgPool>()
        .ok_or_else(|| ServerFnError::new("Kein Datenbank-Pool im Kontext"))?;
    let (version,): (String,) = sqlx::query_as("SELECT version()").fetch_one(&pool).await?;
    Ok(version)
}

#[component]
pub fn HomePage() -> impl IntoView {
    let status = Resource::new(|| (), |_| db_status());

    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title text-2xl">"Startseite"</h1>
                <p>"Leptos-Gesamtanwendung mit PostgreSQL, Tailwind CSS und daisyUI."</p>
                <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match status.await {
                            Ok(version) => view! {
                                <div class="alert alert-success">"Datenbank verbunden: " {version}</div>
                            }.into_any(),
                            Err(e) => view! {
                                <div class="alert alert-error">"Datenbankfehler: " {e.to_string()}</div>
                            }.into_any(),
                        }
                    })}
                </Suspense>
            </div>
        </div>
    }
}
