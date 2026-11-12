//! Zugangsschutz für jede Anfrage (ANF-007 AK-1 bis AK-3, AK-9, AK-14; nur Server).
//!
//! Ermittelt aus dem Sitzungs-Cookie den angemeldeten Benutzer und legt ihn in die Anfrage
//! (`CurrentUser` in den Erweiterungen; Server-Funktionen lesen ihn über `session::current_user`).
//! Ohne gültige Sitzung:
//! - Seiten → Umleitung auf `/anmelden?zurueck=…` (bei abgelaufener Sitzung mit `abgelaufen=1`),
//! - Server-Funktionen unter `/api/…` → 401 „Nicht angemeldet.“
//!
//! Öffentlich: die Anmeldeseite, Server-Funktionen unter `/oeffentlich/…`, `/pkg/…` und feste Dateien.

use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use sqlx::PgPool;

use super::model::{
    LOGIN_PATH, MSG_NOT_LOGGED_IN, PUBLIC_API_PREFIX, login_url, query_param, target_after_login,
};
use super::{repository, session};

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

/// Pfade, die ohne Anmeldung erreichbar sind (AK-3).
pub fn is_public(path: &str) -> bool {
    path == LOGIN_PATH || path.starts_with(&format!("{PUBLIC_API_PREFIX}/")) || is_static(path)
}

pub async fn require_login(State(pool): State<PgPool>, mut req: Request, next: Next) -> Response {
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
        // AK-9: Bereits angemeldet → Anmeldeseite überspringen
        if path == LOGIN_PATH {
            let target = target_after_login(
                req.uri()
                    .query()
                    .and_then(|q| query_param(q, "zurueck"))
                    .as_deref(),
            );
            return Redirect::to(&target).into_response();
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
        assert!(is_public("/pkg/erp-saintwhoo.wasm"));
        assert!(is_public("/favicon.ico"));
        assert!(!is_public("/"));
        assert!(!is_public("/anmelden/x"));
        assert!(!is_public("/api/list_tasks123"));
        assert!(!is_public("/projekte/4"));
        assert!(!is_public("/oeffentlichkeit"));
        assert!(!is_public("/projekte/4.json"));
        assert!(!is_public("/aufgaben/1.x"));
    }
}
