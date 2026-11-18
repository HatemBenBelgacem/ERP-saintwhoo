//! Auslieferung des Firmenlogos unter `/firma/logo` (ANF-025 AK-7, nur Server).
//!
//! Das Logo liegt in der Datenbank. SVG-Dateien können Skripte enthalten; die Antwort verbietet
//! deshalb jede Ausführung (Content-Security-Policy mit `sandbox`) und das Erraten des Typs.

use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use super::repository;

pub async fn logo_response(pool: PgPool) -> Response {
    match repository::logo(&pool).await {
        Ok(Some((bytes, typ))) => (
            [
                (header::CONTENT_TYPE, typ),
                // Die Webadresse enthält die Version (?v=…) – darf lange zwischengespeichert werden.
                (
                    header::CACHE_CONTROL,
                    "private, max-age=31536000, immutable".to_string(),
                ),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
                (
                    header::CONTENT_SECURITY_POLICY,
                    "default-src 'none'; style-src 'unsafe-inline'; sandbox".to_string(),
                ),
            ],
            bytes,
        )
            .into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            leptos::logging::error!("Logo konnte nicht gelesen werden: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
