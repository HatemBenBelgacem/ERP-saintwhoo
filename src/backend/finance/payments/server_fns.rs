//! Server-Funktionen der Zahlungen (ANF-020): Liste, Detail, offene Posten eines Partners, Zahlung bzw.
//! Verrechnung speichern, „Als bezahlt erfassen“, Storno und Altersgliederung.
//!
//! Eine Zahlung sperrt ihre Belege, prüft die Zuordnung gegen die aktuellen offenen Beträge, schreibt die
//! Buchung (Herkunft `zahlung`) und gleicht die Belege aus – alles in einer Transaktion (AK-8).

use leptos::prelude::*;

use super::model::{
    AgingReport, AllocationView, DocumentHint, OpenItem, PaymentDetail, PaymentFilter,
    PaymentFormData, PaymentInput, PaymentPage, SaveOutcome,
};

#[cfg(feature = "ssr")]
use super::{
    model::{
        AllocationInput, BookingItem, FieldErrors, PAGE_SIZE, PAYMENTS_URL, PaymentAccounts,
        PaymentValues, VatShare, aging_by_partner, booking_text, check, parse_key, payment_lines,
    },
    repository,
};
#[cfg(feature = "ssr")]
use crate::backend::finance::debtors::model::{RATE_FACTOR, to_chf};
#[cfg(feature = "ssr")]
use crate::backend::finance::debtors::repository as shared;
#[cfg(feature = "ssr")]
use crate::backend::finance::fiscal_year::model::{booking_error, german_date};
#[cfg(feature = "ssr")]
use crate::backend::finance::journal::repository as journal;
#[cfg(feature = "ssr")]
use crate::forms::PickOption;
#[cfg(feature = "ssr")]
use sqlx::PgConnection;

/// Zahlungsliste (AK-12).
#[server]
pub async fn list_payments(filter: PaymentFilter) -> Result<PaymentPage, ServerFnError> {
    let filter = filter.normalized();
    let pool = crate::db::pool()?;
    let pattern = crate::db::like_pattern(&filter.suche);
    let total = repository::count(&pool, &filter, &pattern).await?;
    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = filter.seite.min(page_count);
    let rows =
        repository::page(&pool, &filter, &pattern, PAGE_SIZE, (page - 1) * PAGE_SIZE).await?;
    Ok(PaymentPage { rows, total, page })
}

/// Eine Zahlung mit Ausgleichszeilen; `None`, wenn es sie nicht gibt.
#[server]
pub async fn get_payment(id: i64) -> Result<Option<PaymentDetail>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(payment) = repository::find_payment(&pool, id).await? else {
        return Ok(None);
    };
    Ok(Some(PaymentDetail {
        payment,
        zeilen: repository::allocation_views(&pool, id).await?,
        verwendungen: repository::allocations_of_prepayment(&pool, id).await?,
    }))
}

/// Vorgaben für das Zahlungsfenster (AK-1).
#[server]
pub async fn get_payment_form() -> Result<PaymentFormData, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(PaymentFormData {
        heute: crate::backend::finance::fiscal_year::repository::today(&pool).await?,
        bankkonten: repository::bank_options(&pool).await?,
        konten: repository::account_choices(&pool).await?,
        waehrungen: repository::active_currencies(&pool).await?,
    })
}

/// Offene Posten eines Partners (AK-2); bei Verrechnung auch Vorauszahlungen (AK-11).
#[server]
pub async fn get_open_items(
    seite: String,
    adresse_id: i64,
    vorauszahlungen: bool,
) -> Result<Vec<OpenItem>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::open_items(&pool, &seite, adresse_id, vorauszahlungen).await?)
}

/// Beleg zur Rechnungsnummer bzw. Kennung (AK-2: „Rechnungsnummer direkt vorwählen“).
#[server]
pub async fn find_document_hint(
    seite: String,
    nummer: String,
    id: Option<i64>,
) -> Result<Option<DocumentHint>, ServerFnError> {
    let pool = crate::db::pool()?;
    let seite = if seite == "kreditoren" {
        "kreditoren"
    } else {
        "debitoren"
    };
    let found = repository::find_document(&pool, seite, nummer.trim(), id).await?;
    Ok(found.map(
        |(beleg_id, adresse_id, an, name, art, waehrung, offen)| DocumentHint {
            seite: seite.into(),
            partner: PickOption {
                id: adresse_id,
                nummer: an,
                name,
            },
            waehrung,
            key: format!("{}:{beleg_id}", super::model::key_prefix(&art)),
            offen,
        },
    ))
}

/// Zahlungen und Verrechnungen eines Belegs (Detailansicht der Rechnung).
#[server]
pub async fn list_document_payments(
    seite: String,
    beleg_id: i64,
) -> Result<Vec<AllocationView>, ServerFnError> {
    let pool = crate::db::pool()?;
    Ok(repository::allocations_of_document(&pool, &seite, beleg_id).await?)
}

/// Offene Posten mit Altersgliederung zum Stichtag (AK-13); leerer Stichtag = heute.
#[server]
pub async fn get_aging(stichtag: String) -> Result<AgingReport, ServerFnError> {
    let pool = crate::db::pool()?;
    let stichtag = if crate::backend::projects::model::is_valid_date(stichtag.trim()) {
        stichtag.trim().to_string()
    } else {
        crate::backend::finance::fiscal_year::repository::today(&pool).await?
    };
    let items = repository::aging_items(&pool, &stichtag).await?;
    Ok(AgingReport {
        debitoren: aging_by_partner(&items, "debitoren"),
        kreditoren: aging_by_partner(&items, "kreditoren"),
        stichtag,
    })
}

// ---------- Speichern ----------

/// Zahlung bzw. Verrechnung speichern (AK-1 bis AK-9, AK-11). Danach zur Detailansicht.
#[server]
pub async fn save_payment(input: PaymentInput) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let user = crate::backend::auth::session::require_user()?.benutzer_id;
    let mut tx = pool.begin().await?;
    let id = match record_payment(&pool, &mut tx, &input, user).await? {
        Ok(id) => id,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    tx.commit().await?;
    leptos_axum::redirect(&format!("{PAYMENTS_URL}/{id}?gespeichert=1"));
    Ok(SaveOutcome::Saved(id))
}

/// „Als bezahlt erfassen“ (AK-7): je Kreditorenrechnung ein Zahlungsausgang über den offenen Betrag, mit
/// Skonto, wenn die Frist am Zahlungsdatum noch läuft. Alle oder keine.
#[server]
pub async fn pay_creditors(
    ids: String,
    datum: String,
    bankkonto_id: String,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let user = crate::backend::auth::session::require_user()?.benutzer_id;
    let ids: Vec<i64> = ids
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    if ids.is_empty() {
        return Err(ServerFnError::new(
            "Bitte mindestens eine Rechnung markieren.",
        ));
    }
    let mut tx = pool.begin().await?;
    let mut anzahl = 0;
    for id in ids {
        let Some(hint) = repository::find_document(&pool, "kreditoren", "", Some(id)).await? else {
            return Err(ServerFnError::new(
                "Eine markierte Rechnung ist nicht mehr offen.",
            ));
        };
        let (_, adresse_id, _, _, art, waehrung, _) = hint;
        if art != "rechnung" {
            continue;
        }
        let items = repository::open_items(&mut *tx, "kreditoren", adresse_id, false).await?;
        let Some(item) = items
            .iter()
            .find(|i| i.art == "rechnung" && i.beleg_id == id)
        else {
            continue;
        };
        let skonto = item.discount_on(datum.trim()).unwrap_or(0);
        let fixed = |c: i64| crate::backend::purchase_orders::model::format_fixed(c, 2);
        let input = PaymentInput {
            art: "ausgang".into(),
            seite: "kreditoren".into(),
            datum: datum.clone(),
            bankkonto_id: bankkonto_id.clone(),
            waehrung,
            betrag: fixed(item.offen - skonto),
            adresse_id: adresse_id.to_string(),
            text: String::new(),
            zeilen: vec![AllocationInput {
                key: item.key(),
                betrag: fixed(item.offen - skonto),
                skonto: fixed(skonto),
                ..Default::default()
            }],
        };
        match record_payment(&pool, &mut tx, &input, user).await? {
            Ok(_) => anzahl += 1,
            Err(errors) => {
                let (key, message) = errors
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| ("zeilen".into(), "Ungültige Eingabe.".into()));
                let field = if key == "datum" || key == "bankkonto_id" {
                    key
                } else {
                    "zeilen".to_string()
                };
                return Ok(SaveOutcome::Invalid(field_error(
                    &field,
                    format!("{}: {message}", item.nummer),
                )));
            }
        }
    }
    tx.commit().await?;
    leptos_axum::redirect(&format!("{PAYMENTS_URL}?gespeichert={anzahl}"));
    Ok(SaveOutcome::Saved(anzahl))
}

/// Zahlung bzw. Verrechnung stornieren (AK-10): Gegenbuchung, offene Beträge und Status der Belege zurück.
#[server]
pub async fn storno_payment(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let user = crate::backend::auth::session::require_user()?.benutzer_id;
    let heute = crate::backend::finance::fiscal_year::repository::today(&pool).await?;
    let mut tx = pool.begin().await?;
    let Some((_, seite)) = repository::lock_payment(&mut tx, id).await? else {
        return Err(ServerFnError::new("Zahlung nicht gefunden."));
    };
    let payment = repository::find_payment(&mut *tx, id)
        .await?
        .ok_or_else(|| ServerFnError::new("Zahlung nicht gefunden."))?;
    if payment.status != "gebucht" {
        return Err(ServerFnError::new("Die Zahlung ist bereits storniert."));
    }
    if !payment.can_storno() {
        return Err(ServerFnError::new(
            "Die Vorauszahlung ist schon verrechnet. Bitte zuerst die Verrechnung stornieren.",
        ));
    }
    let zeilen = repository::allocations_of(&mut tx, id).await?;
    let belege: Vec<i64> = zeilen
        .iter()
        .filter(|z| z.art != "vorauszahlung")
        .map(|z| z.beleg_id)
        .collect();
    repository::lock_documents(&mut tx, &seite, &belege).await?;
    for z in zeilen.iter().filter(|z| z.art == "vorauszahlung") {
        repository::lock_payment(&mut tx, z.beleg_id).await?;
    }
    for z in &zeilen {
        repository::change_open(&mut tx, &seite, &z.art, z.beleg_id, z.cleared()).await?;
    }
    if let Some(buchung_id) = payment.buchung_id {
        let orig = journal::find_head(&mut *tx, buchung_id)
            .await?
            .ok_or_else(|| ServerFnError::new("Die Buchung zur Zahlung fehlt."))?;
        let open_on = |found: &Option<(i64, String, Option<String>)>, datum: &str| {
            booking_error(
                found
                    .as_ref()
                    .map(|(_, status, periode)| (status.as_str(), periode.as_deref())),
                datum,
            )
        };
        let mut datum = payment.datum.clone();
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
    }
    repository::mark_cancelled(&mut tx, id).await?;
    tx.commit().await?;
    Ok(())
}

// ---------- Hilfen (nur Server) ----------

#[cfg(feature = "ssr")]
fn field_error(field: &str, message: String) -> FieldErrors {
    FieldErrors::from([(field.to_string(), message)])
}

/// Prüft und speichert eine Zahlung in der Transaktion des Aufrufers. `Err(FieldErrors)` bei Eingabefehlern
/// (die Transaktion wird dann verworfen), sonst die Kennung der Zahlung.
#[cfg(feature = "ssr")]
async fn record_payment(
    pool: &sqlx::PgPool,
    tx: &mut PgConnection,
    input: &PaymentInput,
    user: Option<i64>,
) -> Result<Result<i64, FieldErrors>, ServerFnError> {
    let verrechnung = input.art == "verrechnung";
    let seite = if verrechnung {
        if input.seite == "kreditoren" {
            "kreditoren"
        } else {
            "debitoren"
        }
    } else {
        super::model::side_of(&input.art)
    };
    let Some(adresse_id) = input.adresse_id.trim().parse::<i64>().ok() else {
        return Ok(Err(field_error(
            "adresse_id",
            "Bitte einen Partner wählen.".into(),
        )));
    };
    // Belege und Vorauszahlungen sperren, dann die offenen Beträge lesen
    let keys: Vec<(&str, i64)> = input
        .zeilen
        .iter()
        .filter_map(|z| parse_key(&z.key))
        .collect();
    let belege: Vec<i64> = keys
        .iter()
        .filter(|(a, _)| *a != "vorauszahlung")
        .map(|(_, id)| *id)
        .collect();
    repository::lock_documents(tx, seite, &belege).await?;
    for (_, id) in keys.iter().filter(|(a, _)| *a == "vorauszahlung") {
        repository::lock_payment(tx, *id).await?;
    }
    let items = repository::open_items(&mut *tx, seite, adresse_id, verrechnung).await?;
    let konten: Vec<i64> = repository::account_choices(&mut *tx)
        .await?
        .into_iter()
        .map(|k| k.id)
        .collect();
    let values = match check(input, &items, &konten) {
        Ok(v) => v,
        Err(errors) => return Ok(Err(errors)),
    };
    let item_of = |a: &super::model::AllocationValues| {
        items
            .iter()
            .find(|i| i.art == a.art && i.beleg_id == a.beleg_id)
            .cloned()
    };

    if verrechnung {
        // AK-11: Ausgleich ohne Bankbewegung und ohne Buchung
        let kurs = values
            .zeilen
            .iter()
            .find_map(|z| item_of(z).filter(|i| i.is_invoice()).map(|i| i.kurs))
            .unwrap_or(RATE_FACTOR);
        let id =
            repository::insert_payment(tx, &values, kurs, to_chf(values.betrag, kurs), None, user)
                .await?;
        for z in &values.zeilen {
            repository::apply_allocation(tx, id, seite, z).await?;
        }
        return Ok(Ok(id));
    }

    // Periode, Bankkonto, Kurs (AK-9) und Systemkonten
    let found = journal::year_for_date(&mut *tx, &values.datum).await?;
    if let Some(message) = booking_error(
        found
            .as_ref()
            .map(|(_, status, periode)| (status.as_str(), periode.as_deref())),
        &values.datum,
    ) {
        return Ok(Err(field_error("datum", message)));
    }
    let Some((gj_id, _, _)) = found else {
        return Ok(Err(field_error(
            "datum",
            "Kein Geschäftsjahr für dieses Datum.".into(),
        )));
    };
    let Some(bank) =
        repository::bank_account_konto(tx, values.bankkonto_id.unwrap_or_default()).await?
    else {
        return Ok(Err(field_error(
            "bankkonto_id",
            "Bitte ein aktives Bankkonto wählen.".into(),
        )));
    };
    let kurs = if values.waehrung == crate::backend::finance::currency::model::BOOKING_CURRENCY {
        RATE_FACTOR
    } else {
        match crate::backend::finance::currency::repository::rate_on(
            pool,
            &values.waehrung,
            &values.datum,
        )
        .await?
        {
            Some(rate) => rate.kurs,
            None => {
                return Ok(Err(field_error(
                    "waehrung",
                    format!(
                        "Für {} gibt es keinen Kurs am oder vor dem {}.",
                        values.waehrung,
                        german_date(&values.datum)
                    ),
                )));
            }
        }
    };
    let (partner_rolle, skonto_rolle) = if seite == "kreditoren" {
        ("kreditoren", "skonto_erhalten")
    } else {
        ("debitoren", "skonto_gewaehrt")
    };
    let Some(partner) = shared::system_account(&mut *tx, partner_rolle).await? else {
        return Err(ServerFnError::new(format!(
            "Das Systemkonto „{}“ fehlt (Einstellungen → Finanzen → Kontenplan → Systemkonten).",
            if seite == "kreditoren" {
                "Kreditoren"
            } else {
                "Debitoren"
            }
        )));
    };
    let accounts = PaymentAccounts {
        bank,
        partner,
        skonto: shared::system_account(&mut *tx, skonto_rolle).await?,
        kursdifferenz: shared::system_account(&mut *tx, "kursdifferenzen").await?,
    };
    let code_konten = repository::code_accounts(&mut *tx).await?;
    let mut booking_items = Vec::new();
    for z in &values.zeilen {
        let Some(item) = item_of(z) else { continue };
        let vat = if z.skonto > 0 {
            vat_shares(tx, seite, z.beleg_id, &code_konten).await?
        } else {
            Vec::new()
        };
        booking_items.push(BookingItem {
            values: z.clone(),
            kurs: item.kurs,
            gesamtbetrag: item.gesamtbetrag,
            vat,
        });
    }
    let lines = match payment_lines(
        seite,
        values.betrag,
        kurs,
        values.vorauszahlung,
        &booking_items,
        accounts,
    ) {
        Ok(lines) => lines,
        Err(message) => return Err(ServerFnError::new(message)),
    };
    let nummern = repository::document_numbers(tx, seite, &belege_of(&values)).await?;
    let partner_name = repository::open_partner_name(tx, values.adresse_id).await?;
    let belegnummer = match nummern.as_slice() {
        [] => "Vorauszahlung".to_string(),
        [one] => one.clone(),
        [first, rest @ ..] => format!("{first} +{}", rest.len()),
    };
    let buchung_id = journal::insert_draft(
        tx,
        &journal::NewHead {
            geschaeftsjahr_id: gj_id,
            datum: values.datum.clone(),
            belegnummer: belegnummer.chars().take(30).collect(),
            text: booking_text(&values.art, &partner_name, &nummern),
            herkunft: "zahlung".into(),
            storno_von_id: None,
            erfasst_von: user,
        },
    )
    .await?;
    for (i, line) in lines.iter().enumerate() {
        journal::insert_line(
            tx,
            buchung_id,
            i32::try_from(i + 1).unwrap_or(i32::MAX),
            line,
        )
        .await?;
    }
    let journalnummer = journal::next_journal_number(tx, gj_id).await?;
    journal::book(tx, buchung_id, journalnummer).await?;
    let id = repository::insert_payment(
        tx,
        &values,
        kurs,
        to_chf(values.betrag, kurs),
        Some(buchung_id),
        user,
    )
    .await?;
    for z in &values.zeilen {
        repository::apply_allocation(tx, id, seite, z).await?;
    }
    Ok(Ok(id))
}

#[cfg(feature = "ssr")]
fn belege_of(values: &PaymentValues) -> Vec<i64> {
    values
        .zeilen
        .iter()
        .filter(|z| z.art != "vorauszahlung")
        .map(|z| z.beleg_id)
        .collect()
}

/// MWST-Anteile einer Rechnung für die Skonto-Korrektur: Brutto je Code aus den Positionen.
#[cfg(feature = "ssr")]
async fn vat_shares(
    conn: &mut PgConnection,
    seite: &str,
    beleg_id: i64,
    code_konten: &[(i64, i64)],
) -> Result<Vec<VatShare>, sqlx::Error> {
    let konto = |code: i64| {
        code_konten
            .iter()
            .find(|(c, _)| *c == code)
            .map(|(_, k)| *k)
    };
    let lines: Vec<(i64, i64, i64)> = if seite == "kreditoren" {
        let positions =
            crate::backend::finance::creditors::repository::positions(&mut *conn, beleg_id).await?;
        crate::backend::finance::creditors::model::totals(&positions, 0, None)
            .vat
            .into_iter()
            .map(|v| (v.code_id, v.rate, v.base + v.vat))
            .collect()
    } else {
        let invoice =
            crate::backend::finance::debtors::repository::find(&mut *conn, beleg_id).await?;
        let positions = shared::positions(&mut *conn, beleg_id).await?;
        let waehrung = invoice.map(|i| i.waehrung).unwrap_or_default();
        crate::backend::finance::debtors::model::totals(&positions, &waehrung)
            .vat
            .into_iter()
            .map(|v| (v.code_id, v.rate, v.base + v.vat))
            .collect()
    };
    Ok(lines
        .into_iter()
        .filter_map(|(code_id, rate, gross)| {
            Some(VatShare {
                code_id,
                rate,
                tax_konto_id: konto(code_id)?,
                gross,
            })
        })
        .collect())
}
