//! Auswertungen zusammensetzen (ANF-021, nur Server): Geschäftsjahr und Zeitraum prüfen, Salden
//! aus dem Journal holen und die Modelle aus `model` bauen.

use leptos::prelude::ServerFnError;
use sqlx::PgPool;

use super::model::{
    ARTEN, AccountBalance, BalanceSheet, Period, ReportKind, ReportPage, ReportQuery,
    balance_sheet, default_range, income_statement, previous_day, project_report, shift_years,
    trial_balance, with_vorjahr_groups, with_vorjahr_income,
};
use super::repository;
use crate::backend::finance::fiscal_year::model::FiscalYear;
use crate::backend::finance::fiscal_year::repository as fiscal;
use crate::backend::projects::model::is_valid_date;

/// Baut die Auswertung für die Auswahl. `heute` bestimmt das Vorgabe-Geschäftsjahr und den Zeitraum.
pub async fn build(
    pool: &PgPool,
    query: &ReportQuery,
    heute: &str,
) -> Result<ReportPage, ServerFnError> {
    let years = fiscal::list_years(pool).await?;
    let gj = pick_year(&years, query.geschaeftsjahr_id, heute)
        .ok_or_else(|| ServerFnError::new("Es gibt noch kein Geschäftsjahr."))?;

    let art = if ARTEN.iter().any(|(k, _)| *k == query.art) {
        query.art.clone()
    } else {
        "bilanz".to_string()
    };
    let (vorgabe_von, vorgabe_bis) = default_range(&gj.beginn, &gj.ende, heute);
    let von = if is_valid_date(&query.von) {
        query.von.clone()
    } else {
        vorgabe_von
    };
    let bis = if is_valid_date(&query.bis) {
        query.bis.clone()
    } else {
        vorgabe_bis
    };
    if von < gj.beginn || bis > gj.ende || von > bis {
        return Err(ServerFnError::new(format!(
            "Der Zeitraum muss innerhalb des Geschäftsjahres {} liegen und von vor bis sein.",
            gj.bezeichnung
        )));
    }

    // Vorjahresvergleich gibt es für Bilanz und Erfolgsrechnung (AK-6)
    let vergleich = query.vergleich() && matches!(art.as_str(), "bilanz" | "erfolgsrechnung");
    let period = Period {
        geschaeftsjahr_id: gj.id,
        geschaeftsjahr: gj.bezeichnung.clone(),
        beginn: gj.beginn.clone(),
        ende: gj.ende.clone(),
        von: von.clone(),
        bis: bis.clone(),
        vorjahr: vergleich.then(|| {
            (
                shift_years(&von, -1).unwrap_or_default(),
                shift_years(&bis, -1).unwrap_or_default(),
            )
        }),
    };

    let kind = match art.as_str() {
        "kontoauszug" => kontoauszug(pool, query.konto_id, &von, &bis).await?,
        "saldenliste" => {
            let anfang = salden_bis(pool, previous_day(&von).as_deref()).await?;
            let bewegung = repository::balances(pool, Some(&von), Some(&bis), None).await?;
            ReportKind::Saldenliste(trial_balance(&anfang, &bewegung))
        }
        "bilanz" => ReportKind::Bilanz(bilanz(pool, &gj.beginn, &bis, vergleich).await?),
        "erfolgsrechnung" => {
            let erfolg = repository::balances(pool, Some(&von), Some(&bis), None).await?;
            let mut er = income_statement(&erfolg);
            if vergleich {
                let vj_von = shift_years(&von, -1).unwrap_or_default();
                let vj_bis = shift_years(&bis, -1).unwrap_or_default();
                let vj = repository::balances(pool, Some(&vj_von), Some(&vj_bis), None).await?;
                with_vorjahr_income(&mut er, &income_statement(&vj));
            }
            ReportKind::Erfolgsrechnung(er)
        }
        _ => {
            let projekt_id = (query.projekt_id != 0).then_some(query.projekt_id);
            let rows = repository::project_rows(pool, projekt_id, Some(&von), Some(&bis)).await?;
            ReportKind::Projekte(project_report(rows, projekt_id.is_some()))
        }
    };
    Ok(ReportPage { art, period, kind })
}

/// Kontoauszug mit Anfangssaldo; ohne gewähltes Konto (oder Konto ohne Zeile) ist die Seite leer.
async fn kontoauszug(
    pool: &PgPool,
    konto_id: i64,
    von: &str,
    bis: &str,
) -> Result<ReportKind, ServerFnError> {
    if konto_id == 0 {
        return Ok(ReportKind::Leer);
    }
    let bewegung = repository::balances(pool, Some(von), Some(bis), None).await?;
    let Some(konto) = bewegung.iter().find(|k| k.konto_id == konto_id).cloned() else {
        return Ok(ReportKind::Leer);
    };
    let anfang = salden_bis(pool, previous_day(von).as_deref()).await?;
    let anfangssaldo = anfang
        .iter()
        .find(|k| k.konto_id == konto_id)
        .map_or(0, AccountBalance::saldo);
    let lines = repository::statement_lines(pool, konto_id, Some(von), Some(bis)).await?;
    let mut ids: Vec<i64> = lines.iter().map(|l| l.buchung_id).collect();
    ids.sort_unstable();
    ids.dedup();
    let counter = repository::counter_lines(pool, &ids).await?;
    Ok(ReportKind::Kontoauszug(super::model::statement(
        &konto,
        anfangssaldo,
        &lines,
        &counter,
    )))
}

/// Bilanz zum Stichtag. Mit Vergleich zusätzlich der Stichtag des Vorjahres (AK-6).
async fn bilanz(
    pool: &PgPool,
    gj_beginn: &str,
    stichtag: &str,
    vergleich: bool,
) -> Result<BalanceSheet, ServerFnError> {
    let alle = repository::balances(pool, None, Some(stichtag), None).await?;
    let frueher = erfolg_bis(pool, previous_day(gj_beginn).as_deref()).await?;
    let mut sheet = balance_sheet(stichtag, &alle, frueher);
    if vergleich {
        let vj_stichtag = shift_years(stichtag, -1).unwrap_or_default();
        let vj_beginn = shift_years(gj_beginn, -1).unwrap_or_default();
        let alle_vj = repository::balances(pool, None, Some(&vj_stichtag), None).await?;
        let frueher_vj = erfolg_bis(pool, previous_day(&vj_beginn).as_deref()).await?;
        let prev = balance_sheet(&vj_stichtag, &alle_vj, frueher_vj);
        with_vorjahr_groups(&mut sheet.aktiven, &prev.aktiven);
        with_vorjahr_groups(&mut sheet.passiven, &prev.passiven);
        sheet.vorjahr_aktiven = Some(prev.summe_aktiven);
        sheet.vorjahr_passiven = Some(prev.summe_passiven);
    }
    Ok(sheet)
}

/// Salden aller Konten bis zum Datum; ohne Datum (z. B. vor dem ersten Tag) leer.
async fn salden_bis(
    pool: &PgPool,
    bis: Option<&str>,
) -> Result<Vec<AccountBalance>, ServerFnError> {
    match bis {
        Some(bis) => Ok(repository::balances(pool, None, Some(bis), None).await?),
        None => Ok(Vec::new()),
    }
}

/// Erfolg (Ertrag minus Aufwand, positiv = Gewinn) bis zum Datum; ohne Datum 0.
async fn erfolg_bis(pool: &PgPool, bis: Option<&str>) -> Result<i64, ServerFnError> {
    let salden = salden_bis(pool, bis).await?;
    Ok(salden
        .iter()
        .filter(|k| k.is_erfolg())
        .map(|k| -k.saldo())
        .sum())
}

/// Geschäftsjahr: die Auswahl, sonst das laufende, sonst das neueste.
fn pick_year<'a>(years: &'a [FiscalYear], id: i64, heute: &str) -> Option<&'a FiscalYear> {
    years
        .iter()
        .find(|y| y.id == id)
        .or_else(|| {
            years
                .iter()
                .find(|y| y.beginn.as_str() <= heute && heute <= y.ende.as_str())
        })
        .or_else(|| years.iter().max_by(|a, b| a.beginn.cmp(&b.beginn)))
}
