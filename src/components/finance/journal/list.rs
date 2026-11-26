//! Buchungsjournal unter `/finanzen/journal` (ANF-017 AK-1, AK-2): Liste mit Filter, Seiten und
//! ausklappbaren Sammelbuchungen. Eingaben öffnen Modal-Fenster (`?neu=1`, `?bearbeiten=`,
//! `?kopieren=`, `?storno=`, `?loeschen=`).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
    params::ParamsMap,
};

use super::form::{BookingFormModal, FormMode};
use super::storno::{DeleteDraftModal, StornoModal};
use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::fiscal_year::server_fns::get_fiscal_years;
use crate::backend::finance::journal::model::{
    HERKUNFT, JournalFilter, JournalPage, SEARCH_MAX, SORTS, STATUSES, amount_cell, herkunft_label,
    status_badge, status_label,
};
use crate::backend::finance::journal::server_fns::{JOURNAL_URL, get_journal};
use crate::backend::purchase_orders::model::format_amount;
use crate::components::finance::FinanceTabs;
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Filter aus der Webadresse (AK-2). Unbekannte Werte prüft der Server.
fn filter_from(q: &ParamsMap) -> JournalFilter {
    JournalFilter {
        geschaeftsjahr_id: q.get("gj").and_then(|v| v.parse().ok()).unwrap_or(0),
        suche: q.get("suche").unwrap_or_default(),
        herkunft: q.get("herkunft").unwrap_or_default(),
        status: q.get("status").unwrap_or_default(),
        von: q.get("von").unwrap_or_default(),
        bis: q.get("bis").unwrap_or_default(),
        sortierung: q.get("sortierung").unwrap_or_default(),
        seite: q.get("seite").and_then(|v| v.parse().ok()).unwrap_or(1),
    }
}

/// Webadresse der Liste mit dem Filter und einer Seite.
fn list_url(f: &JournalFilter, seite: i64) -> String {
    let mut p: Vec<(&str, String)> = Vec::new();
    if f.geschaeftsjahr_id > 0 {
        p.push(("gj", f.geschaeftsjahr_id.to_string()));
    }
    for (name, value) in [
        ("suche", &f.suche),
        ("herkunft", &f.herkunft),
        ("status", &f.status),
        ("von", &f.von),
        ("bis", &f.bis),
        ("sortierung", &f.sortierung),
    ] {
        if !value.is_empty() {
            p.push((name, value.clone()));
        }
    }
    if seite > 1 {
        p.push(("seite", seite.to_string()));
    }
    with_params(JOURNAL_URL, &p)
}

#[component]
pub fn JournalListPage() -> impl IntoView {
    let query = use_query_map();
    let filter = Memo::new(move |_| filter_from(&query.read()));
    let data = Resource::new(
        move || (filter.get(), data_version()),
        |(f, _)| get_journal(f),
    );
    let years = Resource::new(|| (), |_| get_fiscal_years());
    let base = use_base_href();
    let param = move |name: &str| query.read().get(name);

    view! {
        <Title text="Journal – Finanzen – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Buchungsjournal"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neue Buchung"
                </a>
            </div>
            <FinanceTabs active="journal"/>

            {move || param("gespeichert").map(|v| {
                let text = if v == "entwurf" {
                    "Entwurf gespeichert.".to_string()
                } else if let Some(n) = v.strip_prefix("buchung-") {
                    format!("Buchung {n} gebucht.")
                } else if let Some(n) = v.strip_prefix("storno-") {
                    format!("Storno gebucht, Journalnummer {n}.")
                } else {
                    "Gespeichert.".to_string()
                };
                view! { <div role="alert" class="alert alert-success">{text}</div> }
            })}
            {move || param("geloescht").map(|_| view! {
                <div role="alert" class="alert alert-success">"Entwurf gelöscht."</div>
            })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    let jahre = years.await;
                    let page = data.await;
                    match (jahre, page) {
                        (Ok(overview), Ok(page)) => {
                            let jahre: Vec<(i64, String)> = overview
                                .jahre
                                .iter()
                                .map(|j| (j.id, j.bezeichnung.clone()))
                                .collect();
                            let f = filter.get_untracked();
                            view! {
                                <FilterForm filter=f jahre aktiv=page.geschaeftsjahr_id/>
                                <JournalTable page filter=filter.get_untracked()/>
                            }
                            .into_any()
                        }
                        (Err(e), _) | (_, Err(e)) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?neu=1, ?bearbeiten=<kennung>, ?kopieren=<kennung>, ?storno=<kennung>, ?loeschen=<kennung>
        {move || param("neu").map(|_| view! { <BookingFormModal mode=FormMode::Neu/> })}
        {move || {
            param("bearbeiten")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <BookingFormModal mode=FormMode::Bearbeiten(id)/> })
        }}
        {move || {
            param("kopieren")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <BookingFormModal mode=FormMode::Kopie(id)/> })
        }}
        {move || {
            param("storno")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <StornoModal id/> })
        }}
        {move || {
            param("loeschen")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <DeleteDraftModal id/> })
        }}
    }
}

/// Filterleiste (AK-2). Auswahlfelder senden sofort; Suchtext mit Schaltfläche.
#[component]
fn FilterForm(filter: JournalFilter, jahre: Vec<(i64, String)>, aktiv: i64) -> impl IntoView {
    view! {
        <Form method="GET" action=JOURNAL_URL>
            <div class="flex flex-wrap items-center gap-2">
                <select
                    name="gj"
                    class="select w-full sm:w-56"
                    aria-label="Geschäftsjahr"
                    onchange="this.form.requestSubmit()"
                >
                    {jahre
                        .into_iter()
                        .map(|(id, name)| view! {
                            <option value=id.to_string() selected={id == aktiv}>{name}</option>
                        })
                        .collect_view()}
                </select>
                <label class="input w-full sm:w-72 sm:flex-none">
                    <IconView icon=Icon::Search/>
                    <input
                        type="search"
                        name="suche"
                        maxlength=SEARCH_MAX
                        placeholder="Text, Beleg, Konto oder Betrag"
                        value=filter.suche.clone()
                    />
                </label>
                <select name="herkunft" class="select w-full sm:w-40" aria-label="Herkunft" onchange="this.form.requestSubmit()">
                    <option value="" selected=filter.herkunft.is_empty()>"Alle Herkünfte"</option>
                    {HERKUNFT
                        .iter()
                        .map(|(k, l)| view! { <option value=*k selected={filter.herkunft == *k}>{*l}</option> })
                        .collect_view()}
                </select>
                <select name="status" class="select w-full sm:w-36" aria-label="Status" onchange="this.form.requestSubmit()">
                    <option value="" selected=filter.status.is_empty()>"Alle Status"</option>
                    {STATUSES
                        .iter()
                        .map(|(k, l)| view! { <option value=*k selected={filter.status == *k}>{*l}</option> })
                        .collect_view()}
                </select>
                <input type="date" name="von" class="input w-full sm:w-40" aria-label="Datum von" value=filter.von.clone()/>
                <input type="date" name="bis" class="input w-full sm:w-40" aria-label="Datum bis" value=filter.bis.clone()/>
                <select name="sortierung" class="select w-full sm:w-44" aria-label="Sortierung" onchange="this.form.requestSubmit()">
                    {SORTS
                        .iter()
                        .map(|(k, l)| view! {
                            <option value=*k selected={filter.sortierung == *k || (filter.sortierung.is_empty() && *k == "datum_ab")}>{*l}</option>
                        })
                        .collect_view()}
                </select>
                <button type="submit" class="btn">"Suchen"</button>
            </div>
        </Form>
    }
}

/// Buchungen als Tabelle mit Seiten. Eine Zeile öffnet die Detailansicht; Sammelbuchungen klappen auf (AK-1).
#[component]
fn JournalTable(page: JournalPage, filter: JournalFilter) -> impl IntoView {
    if page.rows.is_empty() {
        let text = if filter.suche.is_empty()
            && filter.herkunft.is_empty()
            && filter.status.is_empty()
            && filter.von.is_empty()
            && filter.bis.is_empty()
        {
            "Noch keine Buchungen in diesem Geschäftsjahr. Mit „Neue Buchung“ beginnen."
        } else {
            "Keine Buchungen zu dieser Auswahl."
        };
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">{text}</div>
            </div>
        }
        .into_any();
    }

    let navigate = use_navigate();
    let anzahl = page.anzahl;
    let (seite, seiten) = (page.seite, page.seiten);
    let zurueck = list_url(&filter, seite - 1);
    let weiter = list_url(&filter, seite + 1);
    let mut body: Vec<AnyView> = Vec::new();
    for row in page.rows {
        let href = format!("/finanzen/journal/{}", row.head.id);
        let (soll, haben) = row.konto_labels();
        let collective = row.zeilen.iter().filter(|z| !z.ist_mwst_zeile).count() > 2;
        let nav = navigate.clone();
        let target = href.clone();
        let storniert = row.head.status == "storniert";
        body.push(
            view! {
                <tr
                    class="cursor-pointer hover:bg-base-200"
                    class:opacity-60=storniert
                    on:click=move |_| nav(&target, Default::default())
                >
                    <td class="whitespace-nowrap">{german_date(&row.head.datum)}</td>
                    <td class="tabular-nums">
                        {row.head.journalnummer.map_or("–".to_string(), |n| n.to_string())}
                    </td>
                    <td class="whitespace-nowrap">
                        <a href=href.clone() class="link link-hover">{row.head.belegnummer.clone()}</a>
                    </td>
                    <td class="max-w-xs truncate">{row.head.text.clone()}</td>
                    <td class="whitespace-nowrap tabular-nums">{soll}</td>
                    <td class="whitespace-nowrap tabular-nums">{haben}</td>
                    <td class="whitespace-nowrap text-right tabular-nums">{format_amount(row.summe)}</td>
                    <td class="whitespace-nowrap">{herkunft_label(&row.head.herkunft)}</td>
                    <td>
                        <span class=status_badge(&row.head.status)>{status_label(&row.head.status)}</span>
                    </td>
                </tr>
            }
            .into_any(),
        );
        if collective {
            let zeilen = row
                .zeilen
                .iter()
                .map(|z| {
                    view! {
                        <tr>
                            <td>{z.seite()}</td>
                            <td class="whitespace-nowrap">{format!("{} · {}", z.konto_nummer, z.konto_bezeichnung)}</td>
                            <td class="text-right tabular-nums">
                                {amount_cell(z.soll)}
                            </td>
                            <td class="text-right tabular-nums">
                                {amount_cell(z.haben)}
                            </td>
                            <td>
                                {if z.ist_mwst_zeile {
                                    "Steuerzeile".to_string()
                                } else {
                                    z.mwst_code.clone().unwrap_or_default()
                                }}
                            </td>
                            <td>{z.projekt_nummer.clone().unwrap_or_default()}</td>
                        </tr>
                    }
                })
                .collect_view();
            body.push(
                view! {
                    <tr>
                        <td colspan="9" class="bg-base-200/50 p-0">
                            <details class="px-4 py-2">
                                <summary class="cursor-pointer text-sm font-medium">"Zeilen anzeigen"</summary>
                                <table class="table table-xs mt-2">
                                    <thead>
                                        <tr>
                                            <th>"Seite"</th>
                                            <th>"Konto"</th>
                                            <th class="text-right">"Soll"</th>
                                            <th class="text-right">"Haben"</th>
                                            <th>"MWST"</th>
                                            <th>"Projekt"</th>
                                        </tr>
                                    </thead>
                                    <tbody>{zeilen}</tbody>
                                </table>
                            </details>
                        </td>
                    </tr>
                }
                .into_any(),
            );
        }
    }

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th>"Datum"</th>
                        <th>"Journal"</th>
                        <th>"Beleg"</th>
                        <th>"Text"</th>
                        <th>"Soll"</th>
                        <th>"Haben"</th>
                        <th class="text-right">"Betrag"</th>
                        <th>"Herkunft"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>{body}</tbody>
            </table>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-2 text-sm text-base-content/70">
            <span>{if anzahl == 1 { "1 Buchung".to_string() } else { format!("{anzahl} Buchungen") }}</span>
            {(seiten > 1).then(|| view! {
                <div class="join">
                    {(seite > 1).then(|| view! { <a class="join-item btn btn-sm" href=zurueck>"‹ Zurück"</a> })}
                    <span class="join-item btn btn-sm btn-disabled">{format!("Seite {seite} von {seiten}")}</span>
                    {(seite < seiten).then(|| view! { <a class="join-item btn btn-sm" href=weiter>"Weiter ›"</a> })}
                </div>
            })}
        </div>
    }
    .into_any()
}
