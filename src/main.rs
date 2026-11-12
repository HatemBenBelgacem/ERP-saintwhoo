#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use erp_saintwhoo::app::{App, shell};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};

    let _ = dotenvy::dotenv();
    let pool = erp_saintwhoo::db::connect()
        .await
        .expect("Verbindung zur Datenbank fehlgeschlagen");

    let conf = get_configuration(None).expect("Leptos-Konfiguration fehlt");
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let pool = pool.clone();
                move || provide_context(pool.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        // ANF-007: Zugangsschutz für alle Seiten und Server-Funktionen (auch die Rückfallroute)
        .layer(axum::middleware::from_fn_with_state(
            pool.clone(),
            erp_saintwhoo::backend::auth::middleware::require_login,
        ))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log!("Server läuft auf http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
