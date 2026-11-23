//! Zugangsschutz für jede Anfrage (ANF-007 AK-1 bis AK-3, AK-9, AK-14; ANF-026 AK-19; nur Server).
//!
//! Ermittelt aus dem Sitzungs-Cookie den angemeldeten Zugang in der Portal-Datenbank und legt ihn in die
//! Anfrage (`CurrentUser` in den Erweiterungen; Server-Funktionen lesen ihn über `session::current_user`).
//! Ohne gültige Sitzung:
//! - Seiten → Umleitung auf `/anmelden?zurueck=…` (bei abgelaufener Sitzung mit `abgelaufen=1`),
//! - Server-Funktionen unter `/api/…` → 401 „Nicht angemeldet.“
//!
//! Mit Sitzung, aber ohne bereiten und definierten Mandanten (ANF-026 AK-19) ist nur das Portal erreichbar:
//! - Seiten → Umleitung auf `/portal`,
//! - Server-Funktionen des ERP unter `/api/…` → 403 „Kein Mandant gewählt.“ (nur `/api/portal/…` geht).
//!
//! Öffentlich: Anmeldung, Registrierung, Server-Funktionen unter `/oeffentlich/…`, `/pkg/…` und feste Dateien.

use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};

use super::model::{
    LOGIN_PATH, MSG_NO_TENANT, MSG_NOT_LOGGED_IN, PORTAL_API_PREFIX, PORTAL_PATH,
    PUBLIC_API_PREFIX, REGISTER_PATH, login_url, query_param, target_after_login,
};
use super::{repository, session};
use crate::db::PortalPool;

/// Dateiendungen fester Dateien aus `public/` (Bilder, Seitensymbol, Schriften …).
const STATIC_EXTENSIONS: &[&str] = &[
    "ico",
    "png",
    "svg",
    "jpg",
    "jpeg",
    "webp",
    "gif",
    "css",
    "js",
    "wasm",
    "txt",
    "woff",
    "woff2",
    "webmanifest",
];

/// Feste Dateien: alles unter `/pkg/` (CSS, JavaScript, WebAssembly) und Dateien mit bekannter Endung.
/// Seiten haben nie eine solche Endung – sonst würden sie am Zugangsschutz vorbeigeleitet.
fn is_static(path: &str) -> bool {
    path.starts_with("/pkg/")
        || path
            .rsplit_once('.')
            .is_some_and(|(rest, ext)| !rest.ends_with('/') && STATIC_EXTENSIONS.contains(&ext))
}

/// Pfade, die ohne Anmeldung erreichbar sind (ANF-007 AK-3, ANF-026 AK-1).
pub fn is_public(path: &str) -> bool {
    path == LOGIN_PATH
        || path == REGISTER_PATH
        || path.starts_with(&format!("{PUBLIC_API_PREFIX}/"))
        || is_static(path)
}

/// Pfade, die angemeldet, aber ohne Zugang zum ERP erreichbar sind (ANF-026 AK-19).
pub fn is_portal(path: &str) -> bool {
    path == PORTAL_PATH || path.starts_with(&format!("{PORTAL_API_PREFIX}/")) || is_public(path)
}

pub async fn require_login(
    State(PortalPool(pool)): State<PortalPool>,
    mut req: Request,
    next: Next,
) -> Response {
    let path = req.uri().path().to_string();
    if is_static(&path) {
        return next.run(req).await;
    }

    let token = session::token_from_headers(req.headers());
    let user = match &token {
        Some(t) => match repository::user_for_session(&pool, &session::token_hash(t)).await {
            Ok(user) => user,
            Err(e) => {
                leptos::logging::error!("Sitzung konnte nicht geprüft werden: {e}");
                return (StatusCode::INTERNAL_SERVER_ERROR, "Datenbankfehler").into_response();
            }
        },
        None => None,
    };

    if let Some(user) = user {
        let erp_access = user.erp_access();
        // ANF-007 AK-9: Bereits angemeldet → Anmeldung und Registrierung überspringen
        if path == LOGIN_PATH || path == REGISTER_PATH {
            let target = if erp_access {
                target_after_login(
                    req.uri()
                        .query()
                        .and_then(|q| query_param(q, "zurueck"))
                        .as_deref(),
                )
            } else {
                PORTAL_PATH.to_string()
            };
            return Redirect::to(&target).into_response();
        }
        // ANF-026 AK-19: ohne bereiten und definierten Mandanten nur das Portal
        if !erp_access && !is_portal(&path) {
            return if path.starts_with("/api/") {
                (StatusCode::FORBIDDEN, MSG_NO_TENANT).into_response()
            } else {
                Redirect::to(PORTAL_PATH).into_response()
            };
        }
        req.extensions_mut().insert(user);
        return next.run(req).await;
    }

    if is_public(&path) {
        return next.run(req).await;
    }

    // Nicht angemeldet: Server-Funktionen abweisen (AK-2), Seiten umleiten (AK-1)
    let expired = token.is_some();
    let mut response = if path.starts_with("/api/") {
        (StatusCode::UNAUTHORIZED, MSG_NOT_LOGGED_IN).into_response()
    } else {
        let target = match req.uri().query() {
            Some(q) => format!("{path}?{q}"),
            None => path.clone(),
        };
        Redirect::to(&login_url(&target, expired)).into_response()
    };
    // Ungültiges Cookie entfernen
    if expired && let Ok(v) = HeaderValue::from_str(&session::clear_cookie()) {
        response.headers_mut().append(header::SET_COOKIE, v);
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_paths() {
        assert!(is_public("/anmelden"));
        assert!(is_public("/oeffentlich/login123"));
        assert!(is_public("/pkg/uniqwork.wasm"));
        assert!(is_public("/favicon.ico"));
        assert!(!is_public("/"));
        assert!(!is_public("/anmelden/x"));
        assert!(!is_public("/api/list_tasks123"));
        assert!(!is_public("/projekte/4"));
        assert!(!is_public("/oeffentlichkeit"));
        assert!(!is_public("/projekte/4.json"));
        assert!(!is_public("/aufgaben/1.x"));
        assert!(is_public("/registrieren"));
        assert!(!is_public("/registrieren/x"));
        assert!(!is_public("/portal"));
        assert!(!is_public("/api/portal/logout123"));
    }

    #[test]
    fn portal_paths() {
        assert!(is_portal("/portal"));
        assert!(is_portal("/api/portal/logout123"));
        assert!(is_portal("/oeffentlich/get_current_user123"));
        assert!(is_portal("/pkg/uniqwork.css"));
        assert!(!is_portal("/"));
        assert!(!is_portal("/adressen"));
        assert!(!is_portal("/portalx"));
        assert!(!is_portal("/api/portalx"));
        assert!(!is_portal("/api/list_addresses123"));
    }
}
