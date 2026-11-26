//! CSV-Download einer Auswertung unter `/finanzen/auswertungen/csv` (ANF-021 AK-8, nur Server).
//!
//! Die Anfrage läuft durch die Zugangsprüfung wie alle Seiten. Die Datei beginnt mit einer
//! Byte-Reihenfolge-Marke (BOM), damit Excel die Umlaute richtig liest.

use axum::{
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use super::model::{ReportQuery, to_csv};
use super::service;
use crate::backend::finance::fiscal_year::repository as fiscal;

pub async fn csv_response(pool: PgPool, query: ReportQuery) -> Response {
    let heute = match fiscal::today(&pool).await {
        Ok(heute) => heute,
        Err(e) => {
            leptos::logging::error!("Auswertung: Datum konnte nicht gelesen werden: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    match service::build(&pool, &query, &heute).await {
        Ok(page) => {
            let body = format!("\u{feff}{}", to_csv(&page));
            let name = format!("auswertung-{}-{}.csv", page.art, page.period.bis);
            (
                [
                    (header::CONTENT_TYPE, "text/csv; charset=utf-8".to_string()),
                    (
                        header::CONTENT_DISPOSITION,
                        format!("attachment; filename=\"{name}\""),
                    ),
                    (header::X_CONTENT_TYPE_OPTIONS, "nosniff".to_string()),
                ],
                body,
            )
                .into_response()
        }
        Err(e) => (StatusCode::BAD_REQUEST, crate::forms::error_message(&e)).into_response(),
    }
}
