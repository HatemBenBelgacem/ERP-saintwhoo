//! Server-Funktionen des Buchungsjournals (ANF-017): Liste, Detail, Entwurf speichern und buchen,
//! Entwurf löschen, Storno.
//!
//! Prüfen die Eingaben (`model`), öffnen die Transaktion, vergeben die Journalnummer und leiten nach dem
//! Speichern zurück zur Liste. Buchungen sind danach unveränderlich; Korrekturen laufen über den Storno.

use leptos::prelude::*;

use super::model::{
    BookingDetail, BookingForm, BookingInput, JournalFilter, JournalPage, SaveOutcome,
};

#[cfg(feature = "ssr")]
use super::{
    model::{
        BookingRow, EffectiveLine, FieldErrors, LineView, PAGE_SIZE, TEXT_MAX, check, expand,
        storno_hint,
    },
    repository::{self, NewHead},
};
#[cfg(feature = "ssr")]
use crate::backend::finance::fiscal_year::model::booking_error;
#[cfg(feature = "ssr")]
use crate::backend::finance::fiscal_year::repository as fiscal;
#[cfg(feature = "ssr")]
use crate::backend::purchase_orders::model::parse_fixed;

/// Webadresse der Journal-Liste.
pub const JOURNAL_URL: &str = "/finanzen/journal";

/// Liste der Buchungen nach Filter und Seite (AK-1, AK-2).
#[server]
pub async fn get_journal(filter: JournalFilter) -> Result<JournalPage, ServerFnError> {
    let mut filter = filter.normalized();
    let pool = crate::db::pool()?;
    if filter.geschaeftsjahr_id == 0 {
        filter.geschaeftsjahr_id = default_year(&pool).await?.unwrap_or(0);
    }
    if filter.geschaeftsjahr_id == 0 {
        return Ok(JournalPage {
            geschaeftsjahr_id: 0,
            rows: Vec::new(),
            anzahl: 0,
            seite: 1,
            seiten: 1,
        });
    }
    let pattern = crate::db::like_pattern(&filter.suche);
    // Suchtext als Betrag (z. B. „108.10“) zusätzlich in den Zeilen suchen (AK-2)
    let betrag = parse_fixed(&filter.suche, 2)
        .map(|c| crate::backend::purchase_orders::model::format_fixed(c, 2));
    let anzahl = repository::count(&pool, &filter, &pattern, betrag.as_deref()).await?;
    let seiten = ((anzahl + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let seite = filter.seite.min(seiten);
    let heads = repository::page(
        &pool,
        &filter,
        &pattern,
        betrag.as_deref(),
        (seite - 1) * PAGE_SIZE,
        PAGE_SIZE,
    )
    .await?;
    let ids: Vec<i64> = heads.iter().map(|h| h.id).collect();
    let lines = repository::lines_for(&pool, &ids).await?;
    let rows = heads
        .into_iter()
        .map(|head| {
            let id = head.id;
            let zeilen: Vec<LineView> = lines
                .iter()
                .filter(|l| l.buchung_id == id)
                .cloned()
                .collect();
            let summe = zeilen.iter().map(|l| l.soll).sum();
            BookingRow {
                head,
                zeilen,
                summe,
            }
        })
        .collect();
    Ok(JournalPage {
        geschaeftsjahr_id: filter.geschaeftsjahr_id,
        rows,
        anzahl,
        seite,
        seiten,
    })
}

/// Vorgaben für „Neue Buchung“: heutiges Datum, nächste freie Belegnummer, Auswahllisten (AK-3, AK-4).
#[server]
pub async fn get_booking_form() -> Result<BookingForm, ServerFnError> {
    let pool = crate::db::pool()?;
    let heute = fiscal::today(&pool).await?;
    let jahr = crate::backend::finance::date::year_of(&heute).unwrap_or(0);
    let belegnummer = next_belegnummer(&pool, jahr).await?;
    load_form(&pool, heute, belegnummer).await
}

/// Eine Buchung mit Zeilen und Verweisen (AK-13); `None`, wenn es sie nicht gibt.
#[server]
pub async fn get_booking(id: i64) -> Result<Option<BookingDetail>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(head) = repository::find_head(&pool, id).await? else {
        return Ok(None);
    };
    let zeilen = repository::lines_for(&pool, &[id]).await?;
    let summe = zeilen.iter().map(|l| l.soll).sum();
    let storno_von = repository::storno_original(&pool, id).await?;
    let storniert_durch = repository::storniert_durch(&pool, id).await?;
    let herkunftsbeleg = match head.herkunft.as_str() {
        "debitor" => repository::debtor_document(&pool, id).await?,
        "kreditor" => repository::creditor_document(&pool, id).await?,
        _ => None,
    };
    Ok(Some(BookingDetail {
        head,
        zeilen,
        storno_von,
        storniert_durch,
        herkunftsbeleg,
        summe,
    }))
}

/// Vorgabe für das Datum des Storno: das Originaldatum, wenn dessen Periode offen ist, sonst heute (AK-11).
#[server]
pub async fn get_storno_default(id: i64) -> Result<String, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(orig) = repository::find_head(&pool, id).await? else {
        return Err(ServerFnError::new("Buchung nicht gefunden."));
    };
    let offen = repository::year_for_date(&pool, &orig.datum)
        .await?
        .is_some_and(|(_, status, periode)| {
            booking_error(Some((status.as_str(), periode.as_deref())), &orig.datum).is_none()
        });
    if offen {
        Ok(orig.datum)
    } else {
        Ok(fiscal::today(&pool).await?)
    }
}

/// Buchung speichern (`id` = 0: neu, sonst Entwurf ändern). Mit `buchen` wird sie gebucht (AK-6, AK-9),
/// sonst bleibt sie Entwurf (AK-10). Danach zurück zur Liste (`zurueck`).
#[server]
pub async fn save_booking(
    id: i64,
    input: BookingInput,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let form = load_form(&pool, String::new(), String::new()).await?;
    let values = match check(&input, &form) {
        Ok(values) => values,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let user = crate::backend::auth::session::require_user()?.benutzer_id;
    let mut tx = pool.begin().await?;

    // Geschäftsjahr und Periode zum Datum (AK-6, AK-7); Entwürfe brauchen nur ein Geschäftsjahr
    let found = repository::year_for_date(&mut *tx, &values.datum).await?;
    let message = match &found {
        Some((_, status, periode)) if values.buchen => {
            booking_error(Some((status.as_str(), periode.as_deref())), &values.datum)
        }
        Some(_) => None,
        None => booking_error(None, &values.datum),
    };
    let Some((gj_id, _, _)) = found else {
        return Ok(datum_error(message));
    };
    if message.is_some() {
        return Ok(datum_error(message));
    }

    let head = NewHead {
        geschaeftsjahr_id: gj_id,
        datum: values.datum.clone(),
        belegnummer: values.belegnummer.clone(),
        text: values.text.clone(),
        herkunft: "manuell".into(),
        storno_von_id: None,
        erfasst_von: user,
    };
    let buchung_id = if id == 0 {
        repository::insert_draft(&mut tx, &head).await?
    } else {
        let Some(existing) = repository::find_head(&mut *tx, id).await? else {
            return Err(ServerFnError::new("Buchung nicht gefunden."));
        };
        if !existing.is_editable() {
            return Err(ServerFnError::new(
                "Diese Buchung ist gebucht und kann nicht mehr geändert werden.",
            ));
        }
        repository::update_draft_head(&mut tx, id, &head).await?;
        repository::delete_lines(&mut tx, id).await?;
        id
    };
    for (i, line) in expand(&values, &form.codes).iter().enumerate() {
        repository::insert_line(&mut tx, buchung_id, reihenfolge(i), line).await?;
    }
    let journalnummer = if values.buchen {
        let nummer = repository::next_journal_number(&mut tx, gj_id).await?;
        repository::book(&mut tx, buchung_id, nummer).await?;
        Some(nummer)
    } else {
        None
    };
    if let Err(e) = tx.commit().await {
        // Die Datenbank prüft den Ausgleich am Ende der Transaktion (ANF-017 AK-15)
        return match e {
            sqlx::Error::Database(db) if db.message().contains("nicht ausgeglichen") => {
                Ok(SaveOutcome::Invalid(FieldErrors::from([(
                    "ausgleich".to_string(),
                    "Soll und Haben sind nicht ausgeglichen.".to_string(),
                )])))
            }
            other => Err(other.into()),
        };
    }
    let meldung = journalnummer.map_or("entwurf".to_string(), |n| format!("buchung-{n}"));
    redirect_back(&zurueck, "gespeichert", &meldung);
    Ok(SaveOutcome::Saved(buchung_id))
}

/// Entwurf löschen (AK-10). Gebuchte Buchungen lassen sich nicht löschen (ANF-010 AK-7).
#[server]
pub async fn delete_draft(id: i64, zurueck: String) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    if repository::delete_draft(&mut tx, id).await? == 0 {
        return Err(ServerFnError::new(
            "Nur ein Entwurf lässt sich löschen. Gebuchte Buchungen werden storniert.",
        ));
    }
    tx.commit().await?;
    redirect_back(&zurueck, "geloescht", "entwurf");
    Ok(())
}

/// Gebuchte manuelle Buchung stornieren (AK-11): Gegenbuchung mit vertauschtem Soll und Haben,
/// das Original wird „storniert“. Beide verweisen aufeinander.
#[server]
pub async fn storno_booking(
    id: i64,
    datum: String,
    zurueck: String,
) -> Result<SaveOutcome, ServerFnError> {
    let pool = crate::db::pool()?;
    let user = crate::backend::auth::session::require_user()?.benutzer_id;
    let datum = datum.trim().to_string();
    let mut tx = pool.begin().await?;
    let Some(orig) = repository::find_head(&mut *tx, id).await? else {
        return Err(ServerFnError::new("Buchung nicht gefunden."));
    };
    if let Some(hint) = storno_hint(&orig) {
        return Err(ServerFnError::new(hint));
    }
    let found = repository::year_for_date(&mut *tx, &datum).await?;
    let message = match &found {
        Some((_, status, periode)) => {
            booking_error(Some((status.as_str(), periode.as_deref())), &datum)
        }
        None => booking_error(None, &datum),
    };
    let Some((gj_id, _, _)) = found else {
        return Ok(datum_error(message));
    };
    if message.is_some() {
        return Ok(datum_error(message));
    }

    let (neu, nummer) = write_reversal(&mut tx, &orig, gj_id, datum, user).await?;
    tx.commit().await?;
    redirect_back(&zurueck, "gespeichert", &format!("storno-{nummer}"));
    Ok(SaveOutcome::Saved(neu))
}

// ---------------------------------------------------------------------------------------------
// Hilfsfunktionen (nur Server)
// ---------------------------------------------------------------------------------------------

/// Gegenbuchung zu einer gebuchten Buchung schreiben und buchen: Soll und Haben vertauscht, MWST-Zeilen
/// bleiben Steuerzeilen, das Original wird „storniert“ (AK-11). Auch für den Storno von Belegen
/// (ANF-018 AK-13). Liefert die Kennung und die Journalnummer der Gegenbuchung.
#[cfg(feature = "ssr")]
pub async fn write_reversal(
    tx: &mut sqlx::PgConnection,
    orig: &super::model::BookingHead,
    geschaeftsjahr_id: i64,
    datum: String,
    erfasst_von: Option<i64>,
) -> Result<(i64, i32), sqlx::Error> {
    let lines: Vec<LineView> = repository::lines_for(&mut *tx, &[orig.id]).await?;
    let text = format!("Storno: {}", orig.text)
        .chars()
        .take(TEXT_MAX)
        .collect::<String>();
    let neu = repository::insert_draft(
        &mut *tx,
        &NewHead {
            geschaeftsjahr_id,
            datum,
            belegnummer: orig.belegnummer.clone(),
            text,
            herkunft: "manuell".into(),
            storno_von_id: Some(orig.id),
            erfasst_von,
        },
    )
    .await?;
    for (i, z) in lines.iter().enumerate() {
        let (soll, betrag) = if z.soll > 0 {
            (false, z.soll)
        } else {
            (true, z.haben)
        };
        let line = EffectiveLine {
            soll,
            konto_id: z.konto_id,
            betrag,
            mwst_code_id: z.mwst_code_id,
            mwst_basis: z.mwst_basis,
            ist_mwst_zeile: z.ist_mwst_zeile,
            projekt_id: z.projekt_id,
        };
        repository::insert_line(&mut *tx, neu, reihenfolge(i), &line).await?;
    }
    let nummer = repository::next_journal_number(&mut *tx, geschaeftsjahr_id).await?;
    repository::book(&mut *tx, neu, nummer).await?;
    repository::mark_storniert(&mut *tx, orig.id).await?;
    Ok((neu, nummer))
}

/// Stammdaten für das Formular und die Prüfung: aktive Konten, Codes, Projekte, MWST-Pflicht.
#[cfg(feature = "ssr")]
async fn load_form(
    pool: &sqlx::PgPool,
    heute: String,
    belegnummer: String,
) -> Result<BookingForm, ServerFnError> {
    Ok(BookingForm {
        heute,
        belegnummer,
        konten: repository::active_accounts(pool).await?,
        codes: repository::active_codes(pool).await?,
        projekte: repository::projects(pool).await?,
        mwst_pflicht_ab: repository::mwst_pflicht_ab(pool).await?,
    })
}

/// Nächste freie Belegnummer `M-JJJJ-NNNN` des Jahres (Vorschlag, AK-3 und Offene Frage ANF-017).
#[cfg(feature = "ssr")]
async fn next_belegnummer(pool: &sqlx::PgPool, jahr: i32) -> Result<String, sqlx::Error> {
    let prefix = format!("M-{jahr}-");
    let vorhanden = repository::belegnummern_like(pool, &format!("{prefix}%")).await?;
    let hoechste = vorhanden
        .iter()
        .filter_map(|b| b.strip_prefix(prefix.as_str())?.parse::<u32>().ok())
        .max()
        .unwrap_or(0);
    Ok(format!("{prefix}{:04}", hoechste + 1))
}

/// Standard-Geschäftsjahr: das laufende, sonst das neueste (AK-2).
#[cfg(feature = "ssr")]
async fn default_year(pool: &sqlx::PgPool) -> Result<Option<i64>, ServerFnError> {
    let heute = fiscal::today(pool).await?;
    let jahre = fiscal::list_years(pool).await?;
    Ok(jahre
        .iter()
        .find(|y| y.beginn.as_str() <= heute.as_str() && heute.as_str() <= y.ende.as_str())
        .or_else(|| jahre.iter().max_by(|a, b| a.beginn.cmp(&b.beginn)))
        .map(|y| y.id))
}

/// Reihenfolge der Zeile (ab 1).
#[cfg(feature = "ssr")]
fn reihenfolge(index: usize) -> i32 {
    i32::try_from(index + 1).unwrap_or(i32::MAX)
}

/// Fehlermeldung zum Datum (oder keine Meldung, wenn das Datum passt).
#[cfg(feature = "ssr")]
fn datum_error(message: Option<String>) -> SaveOutcome {
    SaveOutcome::Invalid(FieldErrors::from([(
        "datum".to_string(),
        message.unwrap_or_else(|| "Das Datum passt nicht zu einem Geschäftsjahr.".to_string()),
    )]))
}

/// Zurück zur Journal-Liste mit Meldung. Die Rücksprung-Adresse nur innerhalb des Journals annehmen.
#[cfg(feature = "ssr")]
fn redirect_back(zurueck: &str, param: &str, value: &str) {
    let base = crate::forms::safe_return_url(zurueck)
        .filter(|u| u.starts_with(JOURNAL_URL))
        .unwrap_or(JOURNAL_URL);
    leptos_axum::redirect(&crate::forms::with_params(
        base,
        &[(param, value.to_string())],
    ));
}
