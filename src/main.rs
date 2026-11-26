#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use uniqwork::app::{App, shell};

    let _ = dotenvy::dotenv();
    // ANF-026: Portal-Datenbank (Registrierung, Anmeldung, Mandanten) und Pools der Mandanten-Datenbanken
    let databases = uniqwork::db::connect()
        .await
        .expect("Verbindung zur Portal-Datenbank fehlgeschlagen");
    // ANF-026 AK-24: abgebrochene Erstellungen aufräumen, Migrationen in allen Mandanten-Datenbanken
    uniqwork::backend::portal::provision::startup(&databases.portal.0, &databases.tenants)
        .await
        .expect("Mandanten-Datenbanken konnten nicht vorbereitet werden");

    let conf = get_configuration(None).expect("Leptos-Konfiguration fehlt");
    // ANF-008: Railway gibt den Port über `PORT` vor; sonst gilt `site-addr` aus Cargo.toml.
    let addr = listen_addr(
        conf.leptos_options.site_addr,
        std::env::var("PORT").ok().as_deref(),
    );
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let databases = databases.clone();
                move || {
                    provide_context(databases.portal.clone());
                    provide_context(databases.tenants.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        // ANF-025: Firmenlogo aus der Datenbank des Mandanten (Pool legt die Middleware in die Anfrage)
        .route(
            uniqwork::backend::company::model::LOGO_URL,
            axum::routing::get(
                |axum::Extension(uniqwork::db::TenantPool(pool)): axum::Extension<
                    uniqwork::db::TenantPool,
                >| uniqwork::backend::company::logo::logo_response(pool),
            ),
        )
        // ANF-021 AK-8: CSV-Download einer Auswertung (Pool der Mandanten-Datenbank wie beim Firmenlogo)
        .route(
            "/finanzen/auswertungen/csv",
            axum::routing::get(
                |axum::Extension(uniqwork::db::TenantPool(pool)): axum::Extension<
                    uniqwork::db::TenantPool,
                >,
                 axum::extract::Query(query): axum::extract::Query<
                    uniqwork::backend::finance::reports::model::ReportQuery,
                >| {
                    uniqwork::backend::finance::reports::csv::csv_response(pool, query)
                },
            ),
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        // ANF-007/ANF-026: Zugangsschutz für alle Seiten und Server-Funktionen (auch die Rückfallroute);
        // Sitzungen liegen in der Portal-Datenbank
        .layer(axum::middleware::from_fn_with_state(
            databases.clone(),
            uniqwork::backend::auth::middleware::require_login,
        ))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log!("Server läuft auf http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Adresse, auf der der Server hört (ANF-008 AK-6).
/// Mit gültigem `PORT` auf allen Schnittstellen (`0.0.0.0:<PORT>`), sonst die Vorgabe.
#[cfg(feature = "ssr")]
fn listen_addr(default: std::net::SocketAddr, port: Option<&str>) -> std::net::SocketAddr {
    match port.and_then(|p| p.trim().parse::<u16>().ok()) {
        Some(port) => std::net::SocketAddr::from(([0, 0, 0, 0], port)),
        None => default,
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::listen_addr;

    #[test]
    fn listen_addr_uses_port() {
        let default = "127.0.0.1:3002".parse().unwrap();
        assert_eq!(
            listen_addr(default, Some("8080")),
            "0.0.0.0:8080".parse().unwrap()
        );
        assert_eq!(listen_addr(default, None), default);
        assert_eq!(listen_addr(default, Some("")), default);
        assert_eq!(listen_addr(default, Some("abc")), default);
    }
}
