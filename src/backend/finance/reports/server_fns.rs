//! Server-Funktionen der Auswertungen (ANF-021): Auswertung abrufen und Auswahllisten.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::model::{ReportPage, ReportQuery};
#[cfg(feature = "ssr")]
use super::{repository, service};
use crate::backend::finance::journal::model::{AccountOption, ProjectOption};

/// Auswahllisten für Konto und Projekt (AK-2, AK-7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportOptions {
    pub konten: Vec<AccountOption>,
    pub projekte: Vec<ProjectOption>,
}

/// Eine Auswertung nach Auswahl (AK-1). Ohne Angabe gelten Vorgaben: Geschäftsjahr und Zeitraum.
#[server]
pub async fn get_report(query: ReportQuery) -> Result<ReportPage, ServerFnError> {
    let pool = crate::db::pool()?;
    let heute = crate::backend::finance::fiscal_year::repository::today(&pool).await?;
    service::build(&pool, &query, &heute).await
}

/// Konten und Projekte für die Auswahl.
#[server]
pub async fn get_report_options() -> Result<ReportOptions, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(ReportOptions {
        konten: repository::all_accounts(&pool).await?,
        projekte: crate::backend::finance::journal::repository::projects(&pool).await?,
    })
}
