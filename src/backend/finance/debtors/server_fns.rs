//! Server-Funktionen der Debitoren (ANF-018): Liste, Beleg mit Positionen, Buchen, Storno, Gutschrift,
//! Druck und die Verknüpfungen zu Projekt und Adresse.
//!
//! Jede Änderung sperrt den Beleg (`FOR UPDATE`) und prüft seinen Status auf dem Server. Das Buchen
//! schreibt Nummer, Beträge und die Buchung im Journal (ANF-017) in einer Transaktion: Schlägt ein
//! Schritt fehl, bleibt der Entwurf unverändert (AK-9).

use leptos::prelude::*;

use super::model::{
    InvoiceDetail, InvoiceFilter, InvoiceFormData, InvoiceInput, InvoicePage, InvoicePrint,
    InvoiceRow, PickOption, PositionFormData, PositionInput, SaveOutcome,
};

#[cfg(feature = "ssr")]
use super::{
    model::{
        BookingAccounts, DEBTORS_URL, FieldErrors, PAGE_SIZE, RATE_FACTOR, booking_check,
        booking_lines, booking_text, check_position, default_income_account, number_for, totals,
        vat_required_on,
    },
    repository,
};
#[cfg(feature = "ssr")]
use crate::backend::finance::fiscal_year::model::{booking_error, german_date};
#[cfg(feature = "ssr")]
use crate::backend::finance::journal::repository as journal;
#[cfg(feature = "ssr")]
use sqlx::PgConnection;

/// Liste mit Suche, Filter, Sortierung und Seiten (AK-1, AK-2).
#[server]
pub async fn list_invoices(filter: InvoiceFilter) -> Result<InvoicePage, ServerFnError> {
    let filter = filter.normalized();
    let pool = crate::db::pool()?;
    let pattern = crate::db::like_pattern(&filter.suche);
    let total = repository::count(&pool, &filter, &pattern).await?;
    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = filter.seite.min(page_count);
    let mut rows =
        repository::page(&pool, &filter, &pattern, PAGE_SIZE, (page - 1) * PAGE_SIZE).await?;
    fill_draft_totals(&pool, &mut rows).await?;
    Ok(InvoicePage { rows, total, page })
}

/// Ausgangsrechnungen eines Projekts (AK-15).
#[server]
pub async fn list_project_invoices(projekt_id: i64) -> Result<Vec<InvoiceRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut rows = repository::by_project(&pool, projekt_id).await?;
    fill_draft_totals(&pool, &mut rows).await?;
    Ok(rows)
}

/// Offene Belege eines Kunden (AK-16).
#[server]
pub async fn list_customer_open(adresse_id: i64) -> Result<Vec<InvoiceRow>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::open_of_customer(&pool, adresse_id).await?)
}

/// Ein Beleg mit Positionen; `None`, wenn es ihn nicht gibt (AK-17).
#[server]
pub async fn get_invoice(id: i64) -> Result<Option<InvoiceDetail>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(invoice) = repository::find(&pool, id).await? else {
        return Ok(None);
    };
    let positions = repository::positions(&pool, id).await?;
    Ok(Some(InvoiceDetail { invoice, positions }))
}

/// Alles für die Druckansicht (AK-11): Beleg, Firma, Kundenadresse, Bank, MWST-Nummer.
#[server]
pub async fn get_invoice_print(id: i64) -> Result<Option<InvoicePrint>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(invoice) = repository::find(&pool, id).await? else {
        return Ok(None);
    };
    let positions = repository::positions(&pool, id).await?;
    let customer = repository::customer_address(&pool, invoice.adresse_id)
        .await?
        .ok_or_else(|| ServerFnError::new("Die Adresse des Kunden fehlt."))?;
    let company = crate::backend::company::repository::find(&pool).await?;
    let pflicht_ab = journal::mwst_pflicht_ab(&pool).await?;
    let mwst_nummer = company
        .as_ref()
        .and_then(|c| c.uid.as_deref())
        .filter(|_| vat_required_on(pflicht_ab.as_deref(), &invoice.rechnungsdatum))
        .map(crate::backend::company::model::vat_number);
    let bank = repository::standard_bank(&pool).await?;
    Ok(Some(InvoicePrint {
        detail: InvoiceDetail { invoice, positions },
        company,
        customer,
        bank,
        mwst_nummer,
    }))
}

/// Vorgaben für das Formular der Kopfdaten (AK-4).
#[server]
pub async fn get_invoice_form() -> Result<InvoiceFormData, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(InvoiceFormData {
        heute: crate::backend::finance::fiscal_year::repository::today(&pool).await?,
        terms: repository::active_terms(&pool).await?,
    })
}

/// Auswahl für das Positionsformular eines Belegs (AK-5).
#[server]
pub async fn get_position_form(rechnung_id: i64) -> Result<PositionFormData, ServerFnError> {
    let pool = crate::db::pool()?;
    let invoice = repository::find(&pool, rechnung_id)
        .await?
        .ok_or_else(not_found)?;
    let konten = repository::income_accounts(&pool).await?;
    let system = repository::system_account(&pool, "ertrag").await?;
    let pflicht_ab = journal::mwst_pflicht_ab(&pool).await?;
    let datum = invoice.rechnungsdatum;
    let codes = repository::sales_codes(&pool)
        .await?
        .into_iter()
        .filter(|c| crate::backend::finance::journal::model::code_valid_on(c, &datum))
        .collect();
    Ok(PositionFormData {
        vorgabe_konto_id: default_income_account(&konten, system),
        konten,
        codes,
        mwst_pflicht: vat_required_on(pflicht_ab.as_deref(), &datum),
        rechnungsdatum: datum,
    })
}

/// Kunde eines Projekts als Vorschlag (AK-7).
#[server]
pub async fn get_project_customer(projekt_id: i64) -> Result<Option<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::project_customer(&pool, projekt_id).await?)
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
            "Der Beleg ist gebucht. Kunde, Positionen und Beträge sind gesperrt.",
        ))
    }
}

/// Sperrt den Beleg und liefert seinen Status.
#[cfg(feature = "ssr")]
async fn lock(conn: &mut PgConnection, id: i64) -> Result<String, ServerFnError> {
    repository::lock(conn, id).await?.ok_or_else(not_found)
}

#[cfg(feature = "ssr")]
fn redirect_to(id: i64) {
    leptos_axum::redirect(&format!("{DEBTORS_URL}/{id}?gespeichert=1"));
}

#[cfg(feature = "ssr")]
fn field_error(field: &str, message: &str) -> SaveOutcome {
    SaveOutcome::Invalid(FieldErrors::from([(
        field.to_string(),
        message.to_string(),
    )]))
}

/// Fremdschlüssel (Kunde oder Projekt zwischenzeitlich gelöscht) als Feldfehler melden.
#[cfg(feature = "ssr")]
fn reference_outcome(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    match &e {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => {
            let constraint = db.constraint().unwrap_or_default();
            Ok(if constraint.contains("projekt") {
                field_error("projekt_id", "Dieses Projekt gibt es nicht mehr.")
            } else if constraint.contains("zahlungsbedingung") {
                field_error(
                    "zahlungsbedingung_id",
                    "Diese Zahlungsbedingung gibt es nicht mehr.",
                )
            } else {
                field_error("adresse_id", "Diese Adresse gibt es nicht mehr.")
            })
        }
        _ => Err(e.into()),
    }
}

/// Beträge der Entwürfe aus ihren Positionen (gebuchte Belege tragen ihren festen Betrag).
#[cfg(feature = "ssr")]
async fn fill_draft_totals(
    pool: &sqlx::PgPool,
    rows: &mut [InvoiceRow],
) -> Result<(), sqlx::Error> {
    let ids: Vec<i64> = rows
        .iter()
        .filter(|r| r.gesamtbetrag.is_none())
        .map(|r| r.id)
        .collect();
    if ids.is_empty() {
        return Ok(());
    }
    let positions = repository::positions_of(pool, &ids).await?;
    for row in rows.iter_mut().filter(|r| r.gesamtbetrag.is_none()) {
        let own: Vec<_> = positions
            .iter()
            .filter(|(id, _)| *id == row.id)
            .map(|(_, p)| p.clone())
            .collect();
        row.gesamtbetrag = Some(totals(&own, &row.waehrung).total);
    }
    Ok(())
}

/// Zahlungsbedingung für das Speichern: gewählte (aktiv oder bisherige) oder die Vorgabe (AK-4).
#[cfg(feature = "ssr")]
async fn resolve_term(
    conn: &mut PgConnection,
    input: &InvoiceInput,
    bisher: Option<i64>,
) -> Result<Result<i64, SaveOutcome>, ServerFnError> {
    match input.term_id() {
        Some(id) if repository::term_usable(conn, id, bisher).await? => Ok(Ok(id)),
        Some(_) => Ok(Err(field_error(
            "zahlungsbedingung_id",
            "Diese Zahlungsbedingung ist nicht aktiv. Bitte eine aktive wählen.",
        ))),
        None => {
            let customer = input.customer_id().unwrap_or_default();
            match repository::default_term(conn, customer).await? {
                Some(id) => Ok(Ok(id)),
                None => Ok(Err(field_error(
                    "zahlungsbedingung_id",
                    "Es gibt keine Zahlungsbedingung. Bitte zuerst unter Einstellungen → Finanzen → \
                     Zahlungsbedingungen eine erfassen.",
                ))),
            }
        }
    }
}

// ---------- Kopf ----------

/// Neue Rechnung bzw. Gutschrift als Entwurf (AK-4). Ohne Kunde, aber mit Projekt, wird der Kunde des
/// Projekts übernommen (AK-7). Danach öffnet sich die Detailansicht.
#[server]
pub async fn create_invoice(input: InvoiceInput) -> Result<SaveOutcome, ServerFnError> {
    let mut input = input.normalized();
    let pool = crate::db::pool()?;
    if input.adresse_id.is_empty()
        && let Some(projekt_id) = input.project_id()
        && let Some(kunde) = repository::project_customer(&pool, projekt_id).await?
    {
        input.adresse_id = kunde.id.to_string();
    }
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let mut tx = pool.begin().await?;
    if !crate::backend::finance::currency::repository::is_active(&mut tx, &input.waehrung).await? {
        return Ok(field_error(
            "waehrung",
            "Diese Währung ist nicht (mehr) aktiv. Bitte eine aktive Währung wählen.",
        ));
    }
    let term = match resolve_term(&mut tx, &input, None).await? {
        Ok(id) => id,
        Err(outcome) => return Ok(outcome),
    };
    let id = match repository::insert(&mut tx, &input, term, None).await {
        Ok(id) => id,
        Err(e) => return reference_outcome(e),
    };
    tx.commit().await?;
    redirect_to(id);
    Ok(SaveOutcome::Saved(id))
}

/// Kopfdaten ändern. Im Entwurf alle Felder; gebucht nur Referenz, Bemerkung und Texte (AK-10).
#[server]
pub async fn update_invoice(id: i64, input: InvoiceInput) -> Result<SaveOutcome, ServerFnError> {
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
    let input = if current.is_draft() {
        input
    } else {
        input.restricted_to_booked(&current.to_input())
    };
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    // Eine unveränderte Währung darf bleiben, auch wenn sie inzwischen deaktiviert ist.
    if input.waehrung != current.waehrung
        && !crate::backend::finance::currency::repository::is_active(&mut tx, &input.waehrung)
            .await?
    {
        return Ok(field_error(
            "waehrung",
            "Diese Währung ist nicht (mehr) aktiv. Bitte eine aktive Währung wählen.",
        ));
    }
    let term = match resolve_term(&mut tx, &input, Some(current.zahlungsbedingung_id)).await? {
        Ok(term) => term,
        Err(outcome) => return Ok(outcome),
    };
    if let Err(e) = repository::update(&mut tx, id, &input, term).await {
        return reference_outcome(e);
    }
    tx.commit().await?;
    redirect_to(id);
    Ok(SaveOutcome::Saved(id))
}

/// Entwurf löschen (AK-12); gebuchte Belege werden storniert.
#[server]
pub async fn delete_invoice(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    lock(&mut tx, id).await?;
    let Some(art) = repository::delete_draft(&mut tx, id).await? else {
        return Err(ServerFnError::new(
            "Nur Entwürfe lassen sich löschen. Gebuchte Belege werden storniert.",
        ));
    };
    tx.commit().await?;
    leptos_axum::redirect(&format!("{DEBTORS_URL}?geloescht={art}"));
    Ok(())
}

/// Entwurf buchen (AK-8, AK-9): Nummer, Fälligkeit und Skonto (ANF-016), Kurs am Rechnungsdatum
/// (ANF-014), Buchung im Journal mit Herkunft „Debitoren“, Status „offen“.
#[server]
pub async fn book_invoice(id: i64) -> Result<(), ServerFnError> {
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

    // ANF-025 AK-10: ohne eigene Firma kein Buchen
    if !repository::company_exists(&mut *tx).await? {
        return Err(ServerFnError::new(
            "Die eigene Firma ist nicht erfasst. Bitte zuerst unter Einstellungen → Firma \
             (/einstellungen/firma) die Firma erfassen.",
        ));
    }
    // Positionen: Konten, MWST-Codes, Betrag (AK-9)
    let konten = repository::income_accounts(&mut *tx).await?;
    let codes = repository::sales_codes(&mut *tx).await?;
    let pflicht_ab = journal::mwst_pflicht_ab(&mut *tx).await?;
    let sums = totals(&positions, &invoice.waehrung);
    booking_check(
        &positions,
        &konten,
        &codes,
        &datum,
        pflicht_ab.as_deref(),
        sums.total,
    )
    .map_err(ServerFnError::new)?;
    // Geschäftsjahr und Periode (ANF-013)
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
    // Kurs am Rechnungsdatum (ANF-014 AK-4)
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
    // Systemkonten (ANF-011 AK-12)
    let Some(debitoren) = repository::system_account(&mut *tx, "debitoren").await? else {
        return Err(ServerFnError::new(
            "Das Systemkonto „Debitoren“ fehlt (Einstellungen → Finanzen → Kontenplan → Systemkonten).",
        ));
    };
    let rundung = repository::system_account(&mut *tx, "rundungsdifferenzen").await?;
    let (lines, total_chf) = booking_lines(
        &invoice.art,
        &positions,
        &sums,
        &codes,
        BookingAccounts { debitoren, rundung },
        kurs,
        invoice.projekt_id,
    )
    .map_err(ServerFnError::new)?;
    // Zahlungsbedingung: Fälligkeit und Skonto (ANF-016 AK-6)
    let (frist, skontofrist, skonto) = repository::term_days(&mut tx, invoice.zahlungsbedingung_id)
        .await?
        .ok_or_else(|| ServerFnError::new("Die Zahlungsbedingung gibt es nicht mehr."))?;
    let faelligkeit =
        add_days(&datum, frist).ok_or_else(|| ServerFnError::new("Ungültiges Rechnungsdatum."))?;
    let skonto = match (skontofrist, skonto) {
        (Some(tage), Some(prozent)) => add_days(&datum, tage).map(|d| (d, prozent)),
        _ => None,
    };

    // Nummer und Buchung im Journal (ANF-017)
    let jahr = year_of(&datum).unwrap_or_default();
    let laufnummer = repository::next_number(&mut tx, &invoice.art, jahr).await?;
    let nummer = number_for(&invoice.art, jahr, laufnummer);
    let buchung_id = journal::insert_draft(
        &mut tx,
        &journal::NewHead {
            geschaeftsjahr_id: gj_id,
            datum: datum.clone(),
            belegnummer: nummer.clone(),
            text: booking_text(&invoice.art, &nummer, &invoice.kunde),
            herkunft: "debitor".into(),
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

/// Offenen Beleg ohne Zahlungen stornieren (AK-13): Gegenbuchung im Journal am Rechnungsdatum, wenn
/// dessen Periode offen ist, sonst heute; Status „storniert“.
#[server]
pub async fn storno_invoice(id: i64) -> Result<(), ServerFnError> {
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
            "Nur offene Belege ohne Zahlungen lassen sich stornieren. Bitte stattdessen eine Gutschrift erstellen.",
        ));
    }
    let buchung_id = invoice
        .buchung_id
        .ok_or_else(|| ServerFnError::new("Zum Beleg gibt es keine Buchung."))?;
    let orig = journal::find_head(&mut *tx, buchung_id)
        .await?
        .ok_or_else(|| ServerFnError::new("Die Buchung zum Beleg fehlt."))?;
    // Datum der Gegenbuchung: Rechnungsdatum, wenn die Periode offen ist, sonst heute
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

/// Gutschrift-Entwurf aus einer gebuchten Rechnung (AK-14): gleiche Positionen, Verweis auf die Rechnung.
#[server]
pub async fn create_credit_note(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    lock(&mut tx, id).await?;
    let invoice = repository::find(&mut *tx, id)
        .await?
        .ok_or_else(not_found)?;
    if !invoice.can_credit() {
        return Err(ServerFnError::new(
            "Eine Gutschrift lässt sich nur aus einer gebuchten Rechnung erstellen.",
        ));
    }
    let neu = repository::insert_credit_note_from(&mut tx, id).await?;
    tx.commit().await?;
    redirect_to(neu);
    Ok(())
}

// ---------- Positionen ----------

/// Konto und Code einer Position gegen die Stammdaten am Rechnungsdatum prüfen.
#[cfg(feature = "ssr")]
async fn check_master_data(
    conn: &mut PgConnection,
    datum: &str,
    values: &super::model::PositionValues,
) -> Result<FieldErrors, sqlx::Error> {
    let konten = repository::income_accounts(&mut *conn).await?;
    let codes = repository::sales_codes(&mut *conn).await?;
    let pflicht_ab = journal::mwst_pflicht_ab(&mut *conn).await?;
    Ok(check_position(
        values.konto_id,
        values.mwst_code_id,
        &konten,
        &codes,
        datum,
        pflicht_ab.as_deref(),
    ))
}

/// Position am Ende anfügen (AK-5). Nur im Entwurf.
#[server]
pub async fn create_invoice_position(
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
    let errors = check_master_data(&mut tx, &invoice.rechnungsdatum, &values).await?;
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let id = repository::insert_position(&mut tx, rechnung_id, &input, &values).await?;
    repository::touch(&mut tx, rechnung_id).await?;
    tx.commit().await?;
    redirect_to(rechnung_id);
    Ok(SaveOutcome::Saved(id))
}

/// Position ändern (AK-5). Nur im Entwurf.
#[server]
pub async fn update_invoice_position(
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
    let errors = check_master_data(&mut tx, &invoice.rechnungsdatum, &values).await?;
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    repository::update_position(&mut tx, id, &input, &values).await?;
    repository::touch(&mut tx, rechnung_id).await?;
    tx.commit().await?;
    redirect_to(rechnung_id);
    Ok(SaveOutcome::Saved(id))
}

/// Position löschen (AK-5). Nur im Entwurf.
#[server]
pub async fn delete_invoice_position(id: i64) -> Result<(), ServerFnError> {
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

/// Position eine Stelle nach oben/unten schieben (AK-5). Nur im Entwurf.
#[server]
pub async fn move_invoice_position(id: i64, nach_oben: bool) -> Result<(), ServerFnError> {
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
