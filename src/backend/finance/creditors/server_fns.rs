//! Server-Funktionen der Kreditoren (ANF-019): Liste, Beleg mit Positionen, Übernahme aus dem
//! Einkaufsauftrag, Kontrollbetrag mit Rundung, Buchen, Storno und die Verknüpfungen.
//!
//! Aufbau wie bei den Debitoren (ANF-018): Jede Änderung sperrt den Beleg; das Buchen schreibt Nummer,
//! Beträge und die Buchung im Journal in einer Transaktion.

use leptos::prelude::*;

use super::model::{
    CreditorDetail, CreditorFilter, CreditorFormData, CreditorInput, CreditorPage, CreditorRow,
    PickOption, PositionFormData, PositionInput, SaveOutcome,
};

#[cfg(feature = "ssr")]
use super::{
    model::{
        BookingAccounts, CREDITORS_URL, FieldErrors, PAGE_SIZE, booking_check, booking_lines,
        booking_text, check_position, default_expense_account, number_for, split_amount, totals,
    },
    repository::{self, PositionWrite},
};
#[cfg(feature = "ssr")]
use crate::backend::finance::debtors::model::{RATE_FACTOR, vat_required_on};
#[cfg(feature = "ssr")]
use crate::backend::finance::debtors::repository as shared;
#[cfg(feature = "ssr")]
use crate::backend::finance::fiscal_year::model::{booking_error, german_date};
#[cfg(feature = "ssr")]
use crate::backend::finance::journal::model::code_valid_on;
#[cfg(feature = "ssr")]
use crate::backend::finance::journal::repository as journal;
#[cfg(feature = "ssr")]
use sqlx::PgConnection;

/// Liste (AK-1, AK-2).
#[server]
pub async fn list_creditors(filter: CreditorFilter) -> Result<CreditorPage, ServerFnError> {
    let filter = filter.normalized();
    let pool = crate::db::pool()?;
    let pattern = crate::db::like_pattern(&filter.suche);
    let total = repository::count(&pool, &filter, &pattern).await?;
    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = filter.seite.min(page_count);
    let rows =
        repository::page(&pool, &filter, &pattern, PAGE_SIZE, (page - 1) * PAGE_SIZE).await?;
    Ok(CreditorPage { rows, total, page })
}

/// Eingangsrechnungen eines Projekts (AK-15).
#[server]
pub async fn list_project_creditors(projekt_id: i64) -> Result<Vec<CreditorRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::by_project(&pool, projekt_id).await?)
}

/// Offene Kreditoren eines Lieferanten (AK-15).
#[server]
pub async fn list_supplier_open(adresse_id: i64) -> Result<Vec<CreditorRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::open_of_supplier(&pool, adresse_id).await?)
}

/// Eingangsrechnungen eines Einkaufsauftrags (AK-9).
#[server]
pub async fn list_order_creditors(auftrag_id: i64) -> Result<Vec<CreditorRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::by_order(&pool, auftrag_id).await?)
}

/// Eine Eingangsrechnung mit Positionen; `None`, wenn es sie nicht gibt (AK-16).
#[server]
pub async fn get_creditor(id: i64) -> Result<Option<CreditorDetail>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(invoice) = repository::find(&pool, id).await? else {
        return Ok(None);
    };
    let positions = repository::positions(&pool, id).await?;
    Ok(Some(CreditorDetail { invoice, positions }))
}

/// Vorgaben für das Kopfformular (AK-4).
#[server]
pub async fn get_creditor_form() -> Result<CreditorFormData, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(CreditorFormData {
        heute: crate::backend::finance::fiscal_year::repository::today(&pool).await?,
        terms: shared::active_terms(&pool).await?,
    })
}

/// Auswahl für das Positionsformular (AK-6).
#[server]
pub async fn get_creditor_position_form(
    rechnung_id: i64,
) -> Result<PositionFormData, ServerFnError> {
    let pool = crate::db::pool()?;
    let invoice = repository::find(&pool, rechnung_id)
        .await?
        .ok_or_else(not_found)?;
    let konten = repository::expense_accounts(&pool).await?;
    let system = shared::system_account(&pool, "aufwand").await?;
    let pflicht_ab = journal::mwst_pflicht_ab(&pool).await?;
    let datum = invoice.rechnungsdatum;
    let codes = repository::input_tax_codes(&pool)
        .await?
        .into_iter()
        .filter(|c| code_valid_on(c, &datum))
        .collect();
    Ok(PositionFormData {
        vorgabe_konto_id: default_expense_account(&konten, system),
        konten,
        codes,
        mwst_pflicht: vat_required_on(pflicht_ab.as_deref(), &datum),
        rechnungsdatum: datum,
    })
}

/// Einkaufsaufträge für die Auswahl im Kopfformular (bestellt oder geliefert).
#[server]
pub async fn search_purchase_orders(suche: String) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    Ok(repository::search_orders(&pool, &suche, &pattern).await?)
}

// ---------- Hilfen ----------

#[cfg(feature = "ssr")]
fn not_found() -> ServerFnError {
    ServerFnError::new("Rechnung nicht gefunden.")
}

#[cfg(feature = "ssr")]
fn require_draft(status: &str) -> Result<(), ServerFnError> {
    if status == "entwurf" {
        Ok(())
    } else {
        Err(ServerFnError::new(
            "Der Beleg ist gebucht. Lieferant, Positionen und Beträge sind gesperrt.",
        ))
    }
}

#[cfg(feature = "ssr")]
async fn lock(conn: &mut PgConnection, id: i64) -> Result<String, ServerFnError> {
    repository::lock(conn, id).await?.ok_or_else(not_found)
}

#[cfg(feature = "ssr")]
fn redirect_to(id: i64) {
    leptos_axum::redirect(&format!("{CREDITORS_URL}/{id}?gespeichert=1"));
}

#[cfg(feature = "ssr")]
fn field_error(field: &str, message: &str) -> SaveOutcome {
    SaveOutcome::Invalid(FieldErrors::from([(
        field.to_string(),
        message.to_string(),
    )]))
}

#[cfg(feature = "ssr")]
const DUPLICATE: &str =
    "Rechnung bereits erfasst: Dieser Lieferant hat schon eine Rechnung mit dieser Nummer.";

/// Datenbankfehler beim Speichern des Kopfs als Feldfehler melden (Verweise, doppelte Rechnung).
#[cfg(feature = "ssr")]
fn save_outcome(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    match &e {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23505") => {
            Ok(field_error("rechnungsnummer", DUPLICATE))
        }
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => {
            let constraint = db.constraint().unwrap_or_default();
            Ok(if constraint.contains("projekt") {
                field_error("projekt_id", "Dieses Projekt gibt es nicht mehr.")
            } else if constraint.contains("einkaufsauftrag") {
                field_error(
                    "einkaufsauftrag_id",
                    "Diesen Einkaufsauftrag gibt es nicht mehr.",
                )
            } else {
                field_error("adresse_id", "Diese Adresse gibt es nicht mehr.")
            })
        }
        _ => Err(e.into()),
    }
}

/// Zahlungsbedingung: gewählte (aktiv oder bisherige) oder die Vorgabe der Adresse bzw. Standard.
#[cfg(feature = "ssr")]
async fn resolve_term(
    conn: &mut PgConnection,
    input: &CreditorInput,
    bisher: Option<i64>,
) -> Result<Result<i64, SaveOutcome>, ServerFnError> {
    match input.term_id() {
        Some(id) if shared::term_usable(conn, id, bisher).await? => Ok(Ok(id)),
        Some(_) => Ok(Err(field_error(
            "zahlungsbedingung_id",
            "Diese Zahlungsbedingung ist nicht aktiv. Bitte eine aktive wählen.",
        ))),
        None => match shared::default_term(conn, input.supplier_id().unwrap_or_default()).await? {
            Some(id) => Ok(Ok(id)),
            None => Ok(Err(field_error(
                "zahlungsbedingung_id",
                "Es gibt keine Zahlungsbedingung. Bitte zuerst unter Einstellungen → Finanzen → \
                 Zahlungsbedingungen eine erfassen.",
            ))),
        },
    }
}

/// Gemeinsame Prüfungen von Anlegen und Bearbeiten im Entwurf: Lieferant aus dem Einkaufsauftrag,
/// Auftrag passend, Währung aktiv, keine doppelte Rechnung (AK-4, AK-5).
#[cfg(feature = "ssr")]
async fn check_head(
    conn: &mut PgConnection,
    input: &mut CreditorInput,
    current: Option<&super::model::CreditorInvoice>,
) -> Result<Option<SaveOutcome>, ServerFnError> {
    if let Some(auftrag_id) = input.order_id() {
        let Some((lieferant, _, _, status)) = repository::order_head(conn, auftrag_id).await?
        else {
            return Ok(Some(field_error(
                "einkaufsauftrag_id",
                "Diesen Einkaufsauftrag gibt es nicht mehr.",
            )));
        };
        if input.adresse_id.is_empty() {
            input.adresse_id = lieferant.to_string();
        }
        let unchanged = current.is_some_and(|c| c.einkaufsauftrag_id == Some(auftrag_id));
        if !unchanged
            && !matches!(
                status.as_str(),
                "bestellt" | "teilweise_geliefert" | "geliefert"
            )
        {
            return Ok(Some(field_error(
                "einkaufsauftrag_id",
                "Nur bestellte oder gelieferte Einkaufsaufträge lassen sich zuordnen.",
            )));
        }
        if input.supplier_id() != Some(lieferant) {
            return Ok(Some(field_error(
                "einkaufsauftrag_id",
                "Der Einkaufsauftrag gehört zu einem anderen Lieferanten.",
            )));
        }
    }
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(Some(SaveOutcome::Invalid(errors)));
    }
    let same_currency = current.is_some_and(|c| c.waehrung == input.waehrung);
    if !same_currency
        && !crate::backend::finance::currency::repository::is_active(conn, &input.waehrung).await?
    {
        return Ok(Some(field_error(
            "waehrung",
            "Diese Währung ist nicht (mehr) aktiv. Bitte eine aktive Währung wählen.",
        )));
    }
    if repository::duplicate_exists(
        conn,
        input.supplier_id().unwrap_or_default(),
        &input.rechnungsnummer,
        current.map(|c| c.id),
    )
    .await?
    {
        return Ok(Some(field_error("rechnungsnummer", DUPLICATE)));
    }
    Ok(None)
}

// ---------- Kopf ----------

/// Neue Eingangsrechnung bzw. Gutschrift als Entwurf (AK-4, AK-5).
#[server]
pub async fn create_creditor(input: CreditorInput) -> Result<SaveOutcome, ServerFnError> {
    let mut input = input.normalized();
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    if let Some(outcome) = check_head(&mut tx, &mut input, None).await? {
        return Ok(outcome);
    }
    let term = match resolve_term(&mut tx, &input, None).await? {
        Ok(id) => id,
        Err(outcome) => return Ok(outcome),
    };
    let id = match repository::insert(&mut tx, &input, term).await {
        Ok(id) => id,
        Err(e) => return save_outcome(e),
    };
    tx.commit().await?;
    redirect_to(id);
    Ok(SaveOutcome::Saved(id))
}

/// Kopfdaten ändern; gebucht nur Fälligkeit und Bemerkung (AK-12).
#[server]
pub async fn update_creditor(id: i64, input: CreditorInput) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    lock(&mut tx, id).await?;
    let current = repository::find(&mut *tx, id)
        .await?
        .ok_or_else(not_found)?;
    if !current.is_editable() {
        return Err(ServerFnError::new(
            "Der Beleg ist storniert und kann nicht mehr geändert werden.",
        ));
    }
    let mut input = input.normalized();
    input.art = current.art.clone();
    if current.is_draft() {
        if let Some(outcome) = check_head(&mut tx, &mut input, Some(&current)).await? {
            return Ok(outcome);
        }
    } else {
        input = input.restricted_to_booked(&current.to_input());
        let mut errors = input.validate();
        // Eine gebuchte Rechnung braucht eine Fälligkeit
        if input.faelligkeit.is_empty() {
            errors.insert(
                "faelligkeit".into(),
                "Bitte eine Fälligkeit eingeben.".into(),
            );
        }
        if !errors.is_empty() {
            return Ok(SaveOutcome::Invalid(errors));
        }
    }
    let term = match resolve_term(&mut tx, &input, Some(current.zahlungsbedingung_id)).await? {
        Ok(term) => term,
        Err(outcome) => return Ok(outcome),
    };
    if let Err(e) = repository::update(&mut tx, id, &input, term).await {
        return save_outcome(e);
    }
    tx.commit().await?;
    redirect_to(id);
    Ok(SaveOutcome::Saved(id))
}

/// Entwurf löschen (AK-13).
#[server]
pub async fn delete_creditor(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    lock(&mut tx, id).await?;
    let Some(art) = repository::delete_draft(&mut tx, id).await? else {
        return Err(ServerFnError::new(
            "Nur Entwürfe lassen sich löschen. Gebuchte Belege werden storniert.",
        ));
    };
    tx.commit().await?;
    leptos_axum::redirect(&format!("{CREDITORS_URL}?geloescht={art}"));
    Ok(())
}

/// Rundungsdifferenz bis 0.05 ausgleichen (AK-7).
#[server]
pub async fn balance_rounding(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    require_draft(&lock(&mut tx, id).await?)?;
    let invoice = repository::find(&mut *tx, id)
        .await?
        .ok_or_else(not_found)?;
    let positions = repository::positions(&mut *tx, id).await?;
    let sums = totals(&positions, invoice.rundung, invoice.betrag_gemaess_rechnung);
    let Some(rundung) = sums.roundable() else {
        return Err(ServerFnError::new(
            "Ausgleichen lassen sich nur Differenzen bis 0.05.",
        ));
    };
    repository::set_rounding(&mut tx, id, rundung).await?;
    tx.commit().await?;
    Ok(())
}

/// Entwurf buchen (AK-10, AK-11): interne Nummer, Fälligkeit (gewählt oder aus der Zahlungsbedingung),
/// Skonto, Kurs am Rechnungsdatum, Buchung mit Herkunft „Kreditoren“, Status „offen“.
#[server]
pub async fn book_creditor(id: i64) -> Result<(), ServerFnError> {
    use crate::backend::finance::date::{add_days, year_of};

    let pool = crate::db::pool()?;
    let user = crate::backend::auth::session::require_user()?.benutzer_id;
    let mut tx = pool.begin().await?;
    require_draft(&lock(&mut tx, id).await?)?;
    let invoice = repository::find(&mut *tx, id)
        .await?
        .ok_or_else(not_found)?;
    let positions = repository::positions(&mut *tx, id).await?;
    let datum = invoice.rechnungsdatum.clone();
    let Some(rechnungsnummer) = invoice.rechnungsnummer.clone() else {
        return Err(ServerFnError::new(
            "Bitte zuerst die Rechnungsnummer des Lieferanten eingeben (Bearbeiten).",
        ));
    };

    let konten = repository::expense_accounts(&mut *tx).await?;
    let codes = repository::input_tax_codes(&mut *tx).await?;
    let pflicht_ab = journal::mwst_pflicht_ab(&mut *tx).await?;
    let sums = totals(&positions, invoice.rundung, invoice.betrag_gemaess_rechnung);
    booking_check(
        &positions,
        &sums,
        &konten,
        &codes,
        &datum,
        pflicht_ab.as_deref(),
    )
    .map_err(ServerFnError::new)?;
    let found = journal::year_for_date(&mut *tx, &datum).await?;
    if let Some(message) = booking_error(
        found
            .as_ref()
            .map(|(_, status, periode)| (status.as_str(), periode.as_deref())),
        &datum,
    ) {
        return Err(ServerFnError::new(message));
    }
    let Some((gj_id, _, _)) = found else {
        return Err(ServerFnError::new("Kein Geschäftsjahr für dieses Datum."));
    };
    let kurs = if invoice.waehrung == crate::backend::finance::currency::model::BOOKING_CURRENCY {
        RATE_FACTOR
    } else {
        match crate::backend::finance::currency::repository::rate_on(
            &pool,
            &invoice.waehrung,
            &datum,
        )
        .await?
        {
            Some(rate) => rate.kurs,
            None => {
                return Err(ServerFnError::new(format!(
                    "Für {} gibt es keinen Kurs am oder vor dem {}. Bitte unter Einstellungen → \
                     Finanzen → Währungen einen Kurs erfassen.",
                    invoice.waehrung,
                    german_date(&datum)
                )));
            }
        }
    };
    let Some(kreditoren) = shared::system_account(&mut *tx, "kreditoren").await? else {
        return Err(ServerFnError::new(
            "Das Systemkonto „Kreditoren“ fehlt (Einstellungen → Finanzen → Kontenplan → Systemkonten).",
        ));
    };
    let rundung = shared::system_account(&mut *tx, "rundungsdifferenzen").await?;
    let (lines, total_chf) = booking_lines(
        &invoice.art,
        &positions,
        &sums,
        &codes,
        BookingAccounts {
            kreditoren,
            rundung,
        },
        kurs,
        invoice.projekt_id,
    )
    .map_err(ServerFnError::new)?;
    let (frist, skontofrist, skonto) = shared::term_days(&mut tx, invoice.zahlungsbedingung_id)
        .await?
        .ok_or_else(|| ServerFnError::new("Die Zahlungsbedingung gibt es nicht mehr."))?;
    // AK-4: eine gewählte Fälligkeit hat Vorrang vor der Zahlungsbedingung
    let faelligkeit = match invoice.faelligkeit.clone() {
        Some(f) => f,
        None => add_days(&datum, frist)
            .ok_or_else(|| ServerFnError::new("Ungültiges Rechnungsdatum."))?,
    };
    let skonto = match (skontofrist, skonto) {
        (Some(tage), Some(prozent)) => add_days(&datum, tage).map(|d| (d, prozent)),
        _ => None,
    };

    let jahr = year_of(&datum).unwrap_or_default();
    let nummer = number_for(jahr, repository::next_number(&mut tx, jahr).await?);
    let buchung_id = journal::insert_draft(
        &mut tx,
        &journal::NewHead {
            geschaeftsjahr_id: gj_id,
            datum: datum.clone(),
            belegnummer: nummer.clone(),
            text: booking_text(&invoice.art, &nummer, &invoice.lieferant, &rechnungsnummer),
            herkunft: "kreditor".into(),
            storno_von_id: None,
            erfasst_von: user,
        },
    )
    .await?;
    for (i, line) in lines.iter().enumerate() {
        let reihenfolge = i32::try_from(i + 1).unwrap_or(i32::MAX);
        journal::insert_line(&mut tx, buchung_id, reihenfolge, line).await?;
    }
    let journalnummer = journal::next_journal_number(&mut tx, gj_id).await?;
    journal::book(&mut tx, buchung_id, journalnummer).await?;
    repository::mark_booked(
        &mut tx,
        id,
        &repository::BookedValues {
            nummer,
            faelligkeit,
            skonto,
            kurs,
            gesamtbetrag: sums.total,
            gesamtbetrag_chf: total_chf,
            buchung_id,
        },
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Offenen Beleg ohne Zahlungen stornieren (AK-14): Gegenbuchung am Rechnungsdatum, wenn die Periode offen
/// ist, sonst heute.
#[server]
pub async fn storno_creditor(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let user = crate::backend::auth::session::require_user()?.benutzer_id;
    let heute = crate::backend::finance::fiscal_year::repository::today(&pool).await?;
    let mut tx = pool.begin().await?;
    lock(&mut tx, id).await?;
    let invoice = repository::find(&mut *tx, id)
        .await?
        .ok_or_else(not_found)?;
    if !invoice.can_storno() {
        return Err(ServerFnError::new(
            "Nur offene Belege ohne Zahlungen lassen sich stornieren.",
        ));
    }
    let buchung_id = invoice
        .buchung_id
        .ok_or_else(|| ServerFnError::new("Zum Beleg gibt es keine Buchung."))?;
    let orig = journal::find_head(&mut *tx, buchung_id)
        .await?
        .ok_or_else(|| ServerFnError::new("Die Buchung zum Beleg fehlt."))?;
    let open_on = |found: &Option<(i64, String, Option<String>)>, datum: &str| {
        booking_error(
            found
                .as_ref()
                .map(|(_, status, periode)| (status.as_str(), periode.as_deref())),
            datum,
        )
    };
    let mut datum = invoice.rechnungsdatum.clone();
    let mut found = journal::year_for_date(&mut *tx, &datum).await?;
    if open_on(&found, &datum).is_some() {
        datum = heute;
        found = journal::year_for_date(&mut *tx, &datum).await?;
    }
    if let Some(message) = open_on(&found, &datum) {
        return Err(ServerFnError::new(message));
    }
    let Some((gj_id, _, _)) = found else {
        return Err(ServerFnError::new("Kein Geschäftsjahr für dieses Datum."));
    };
    crate::backend::finance::journal::server_fns::write_reversal(
        &mut tx, &orig, gj_id, datum, user,
    )
    .await?;
    repository::mark_cancelled(&mut tx, id).await?;
    tx.commit().await?;
    Ok(())
}

/// Entwurf aus einem Einkaufsauftrag (AK-8): Lieferant, Projekt, Währung und Positionen mit gelieferter Menge
/// × Einzelpreis abzüglich Rabatt; MWST-Code der Position bzw. ein Vorsteuer-Code mit gleichem Satz.
#[server]
pub async fn create_creditor_from_order(auftrag_id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let heute = crate::backend::finance::fiscal_year::repository::today(&pool).await?;
    let mut tx = pool.begin().await?;
    let Some((lieferant, projekt_id, waehrung, status)) =
        repository::order_head(&mut tx, auftrag_id).await?
    else {
        return Err(ServerFnError::new("Einkaufsauftrag nicht gefunden."));
    };
    if !matches!(
        status.as_str(),
        "bestellt" | "teilweise_geliefert" | "geliefert"
    ) {
        return Err(ServerFnError::new(
            "Eine Eingangsrechnung lässt sich nur aus einem bestellten oder gelieferten Auftrag erfassen.",
        ));
    }
    let konten = repository::expense_accounts(&mut *tx).await?;
    let system = shared::system_account(&mut *tx, "aufwand").await?;
    let Some(konto_id) = default_expense_account(&konten, system) else {
        return Err(ServerFnError::new(
            "Es gibt kein Vorgabe-Aufwandskonto. Bitte das Systemkonto „Aufwand (Vorgabe Kreditoren)“ \
             festlegen (Einstellungen → Finanzen → Kontenplan → Systemkonten).",
        ));
    };
    let Some(term) = shared::default_term(&mut tx, lieferant).await? else {
        return Err(ServerFnError::new(
            "Es gibt keine Zahlungsbedingung. Bitte zuerst eine erfassen.",
        ));
    };
    let pflicht = vat_required_on(journal::mwst_pflicht_ab(&mut *tx).await?.as_deref(), &heute);
    let codes: Vec<_> = repository::input_tax_codes(&mut *tx)
        .await?
        .into_iter()
        .filter(|c| code_valid_on(c, &heute))
        .collect();
    let order_positions = repository::order_positions(&mut tx, auftrag_id).await?;
    let id =
        repository::insert_from_order(&mut tx, auftrag_id, lieferant, projekt_id, &waehrung, term)
            .await?;
    for (einkaufsposition_id, bezeichnung, netto, code_id, satz) in order_positions {
        if netto <= 0 {
            continue;
        }
        let code = if pflicht {
            code_id
                .and_then(|cid| codes.iter().find(|c| c.id == cid))
                .or_else(|| codes.iter().find(|c| c.satz == satz && satz > 0))
        } else {
            None
        };
        let (netto, mwst) = split_amount(netto, code.map(|c| c.satz), false);
        repository::insert_position(
            &mut tx,
            id,
            &PositionWrite {
                bezeichnung: &bezeichnung,
                konto_id,
                mwst_code_id: code.map(|c| c.id),
                netto,
                mwst_betrag: mwst,
                projekt_id: None,
                einkaufsposition_id: Some(einkaufsposition_id),
            },
        )
        .await?;
    }
    tx.commit().await?;
    leptos_axum::redirect(&format!("{CREDITORS_URL}/{id}?gespeichert=1&bearbeiten=1"));
    Ok(())
}

// ---------- Positionen ----------

/// Position prüfen und Netto/MWST berechnen (AK-6).
#[cfg(feature = "ssr")]
async fn position_values(
    conn: &mut PgConnection,
    datum: &str,
    values: &super::model::PositionValues,
) -> Result<Result<(i64, i64), FieldErrors>, sqlx::Error> {
    let konten = repository::expense_accounts(&mut *conn).await?;
    let codes = repository::input_tax_codes(&mut *conn).await?;
    let pflicht_ab = journal::mwst_pflicht_ab(&mut *conn).await?;
    let errors = check_position(
        values.konto_id,
        values.mwst_code_id,
        &konten,
        &codes,
        datum,
        pflicht_ab.as_deref(),
    );
    if !errors.is_empty() {
        return Ok(Err(errors));
    }
    let satz = values
        .mwst_code_id
        .and_then(|id| codes.iter().find(|c| c.id == id))
        .map(|c| c.satz);
    Ok(Ok(split_amount(values.betrag, satz, values.brutto)))
}

#[server]
pub async fn create_creditor_position(
    rechnung_id: i64,
    input: PositionInput,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let values = match input.validate() {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    require_draft(&lock(&mut tx, rechnung_id).await?)?;
    let invoice = repository::find(&mut *tx, rechnung_id)
        .await?
        .ok_or_else(not_found)?;
    let (netto, mwst) = match position_values(&mut tx, &invoice.rechnungsdatum, &values).await? {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let write = PositionWrite {
        bezeichnung: &input.bezeichnung,
        konto_id: values.konto_id,
        mwst_code_id: values.mwst_code_id,
        netto,
        mwst_betrag: mwst,
        projekt_id: values.projekt_id,
        einkaufsposition_id: None,
    };
    let id = match repository::insert_position(&mut tx, rechnung_id, &write).await {
        Ok(id) => id,
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            return Ok(field_error(
                "projekt_id",
                "Dieses Projekt gibt es nicht mehr.",
            ));
        }
        Err(e) => return Err(e.into()),
    };
    repository::touch(&mut tx, rechnung_id).await?;
    tx.commit().await?;
    redirect_to(rechnung_id);
    Ok(SaveOutcome::Saved(id))
}

#[server]
pub async fn update_creditor_position(
    id: i64,
    input: PositionInput,
) -> Result<SaveOutcome, ServerFnError> {
    let input = input.normalized();
    let values = match input.validate() {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (rechnung_id, status) = repository::lock_of_position(&mut tx, id)
        .await?
        .ok_or_else(|| ServerFnError::new("Position nicht gefunden."))?;
    require_draft(&status)?;
    let invoice = repository::find(&mut *tx, rechnung_id)
        .await?
        .ok_or_else(not_found)?;
    let (netto, mwst) = match position_values(&mut tx, &invoice.rechnungsdatum, &values).await? {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let write = PositionWrite {
        bezeichnung: &input.bezeichnung,
        konto_id: values.konto_id,
        mwst_code_id: values.mwst_code_id,
        netto,
        mwst_betrag: mwst,
        projekt_id: values.projekt_id,
        einkaufsposition_id: None,
    };
    match repository::update_position(&mut tx, id, &write).await {
        Ok(()) => {}
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            return Ok(field_error(
                "projekt_id",
                "Dieses Projekt gibt es nicht mehr.",
            ));
        }
        Err(e) => return Err(e.into()),
    }
    repository::touch(&mut tx, rechnung_id).await?;
    tx.commit().await?;
    redirect_to(rechnung_id);
    Ok(SaveOutcome::Saved(id))
}

#[server]
pub async fn delete_creditor_position(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (rechnung_id, status) = repository::lock_of_position(&mut tx, id)
        .await?
        .ok_or_else(|| ServerFnError::new("Position nicht gefunden."))?;
    require_draft(&status)?;
    repository::delete_position(&mut tx, id).await?;
    repository::touch(&mut tx, rechnung_id).await?;
    tx.commit().await?;
    Ok(())
}

#[server]
pub async fn move_creditor_position(id: i64, nach_oben: bool) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (rechnung_id, status) = repository::lock_of_position(&mut tx, id)
        .await?
        .ok_or_else(|| ServerFnError::new("Position nicht gefunden."))?;
    require_draft(&status)?;
    repository::move_position(&mut tx, rechnung_id, id, nach_oben).await?;
    tx.commit().await?;
    Ok(())
}
