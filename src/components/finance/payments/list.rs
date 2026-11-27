//! Reiter „Zahlungen“ (ANF-020): Zahlungsliste mit Suche, Filtern und Sortierung (AK-12) und die Ansicht
//! „Offene Posten“ mit Altersgliederung zu einem Stichtag (AK-13).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::PaymentModal;
use crate::backend::finance::payments::model::{
    AGING_LABELS, KIND_FILTERS, PAGE_SIZE, PAYMENTS_URL, PartnerAging, PaymentFilter, PaymentPage,
    PaymentRow, aging_totals, default_dir, kind_label, status_badge,
};
use crate::backend::finance::payments::server_fns::{get_aging, get_payment_form, list_payments};
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::format_amount;
use crate::components::finance::FinanceTabs;
use crate::forms::{data_version, encode, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

fn list_href(f: &PaymentFilter) -> String {
    let mut params = Vec::new();
    if !f.suche.is_empty() {
        params.push(format!("suche={}", encode(&f.suche)));
    }
    if !f.art.is_empty() {
        params.push(format!("art={}", f.art));
    }
    for (k, v) in [("von", &f.von), ("bis", &f.bis)] {
        if !v.is_empty() {
            params.push(format!("{k}={v}"));
        }
    }
    if f.bankkonto_id > 0 {
        params.push(format!("bankkonto_id={}", f.bankkonto_id));
    }
    if f.sortierung != "datum" {
        params.push(format!("sortierung={}", f.sortierung));
    }
    if f.richtung != default_dir(&f.sortierung) {
        params.push(format!("richtung={}", f.richtung));
    }
    if f.seite > 1 {
        params.push(format!("seite={}", f.seite));
    }
    if params.is_empty() {
        PAYMENTS_URL.into()
    } else {
        format!("{PAYMENTS_URL}?{}", params.join("&"))
    }
}

fn sorted_by(f: &PaymentFilter, key: &str) -> PaymentFilter {
    let richtung = if f.sortierung == key {
        if f.richtung == "auf" { "ab" } else { "auf" }
    } else {
        default_dir(key)
    };
    PaymentFilter {
        sortierung: key.into(),
        richtung: richtung.into(),
        seite: 1,
        ..f.clone()
    }
}

/// Welches Zahlungsfenster laut `?neu=` offen ist: (Art, Seite).
fn dialog_of(neu: &str) -> Option<(&'static str, &'static str)> {
    match neu {
        "eingang" => Some(("eingang", "debitoren")),
        "ausgang" => Some(("ausgang", "kreditoren")),
        "verrechnung-debitoren" => Some(("verrechnung", "debitoren")),
        "verrechnung-kreditoren" => Some(("verrechnung", "kreditoren")),
        _ => None,
    }
}

#[component]
pub fn PaymentsPage() -> impl IntoView {
    let query = use_query_map();
    let base = use_base_href();
    let aging_view = move || query.read().get("ansicht").as_deref() == Some("offene-posten");
    let dialog = move || {
        let q = query.read();
        let (art, seite) = dialog_of(&q.get("neu")?)?;
        Some((art, seite, q.get("beleg")))
    };
    let saved = move || query.read().get("gespeichert");
    let new_href = move |neu: &'static str| with_params(&base.get(), &[("neu", neu.into())]);

    view! {
        <Title text="Zahlungen – Finanzen – uniqwork"/>
        <div class="space-y-4">
            <FinanceTabs active="zahlungen"/>
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Zahlungen"</h1>
                <div class="flex flex-wrap gap-2">
                    <a href=move || new_href("eingang") class="btn btn-primary">
                        <IconView icon=Icon::Plus/>
                        "Zahlungseingang"
                    </a>
                    <a href=move || new_href("ausgang") class="btn btn-primary btn-outline">
                        <IconView icon=Icon::Plus/>
                        "Zahlungsausgang"
                    </a>
                    <a href=move || new_href("verrechnung-debitoren") class="btn btn-outline">"Verrechnen Debitoren"</a>
                    <a href=move || new_href("verrechnung-kreditoren") class="btn btn-outline">"Verrechnen Kreditoren"</a>
                </div>
            </div>
            {move || saved().filter(|s| s != "1").map(|n| view! {
                <div role="alert" class="alert alert-success">{format!("{n} Zahlungsausgänge erfasst.")}</div>
            })}
            <div role="tablist" class="tabs tabs-box w-fit">
                <a role="tab" href=PAYMENTS_URL class="tab" class:tab-active=move || !aging_view()>"Zahlungen"</a>
                <a role="tab" href=format!("{PAYMENTS_URL}?ansicht=offene-posten") class="tab" class:tab-active=aging_view>"Offene Posten"</a>
            </div>
            {move || if aging_view() {
                view! { <AgingView/> }.into_any()
            } else {
                view! { <PaymentList/> }.into_any()
            }}
            {move || dialog().map(|(art, seite, beleg)| view! { <PaymentModal art seite beleg/> })}
        </div>
    }
}

#[component]
fn PaymentList() -> impl IntoView {
    let query = use_query_map();
    let filter = Memo::new(move |_| {
        let q = query.read();
        PaymentFilter {
            suche: q.get("suche").unwrap_or_default(),
            art: q.get("art").unwrap_or_default(),
            von: q.get("von").unwrap_or_default(),
            bis: q.get("bis").unwrap_or_default(),
            bankkonto_id: q
                .get("bankkonto_id")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0),
            sortierung: q.get("sortierung").unwrap_or_default(),
            richtung: q.get("richtung").unwrap_or_default(),
            seite: q.get("seite").and_then(|p| p.parse().ok()).unwrap_or(1),
        }
        .normalized()
    });
    let data = Resource::new(
        move || (filter.get(), data_version()),
        |(f, _)| list_payments(f),
    );
    let form = Resource::new(|| (), |_| get_payment_form());
    view! {
        <Form method="GET" action=PAYMENTS_URL>
            <div class="flex flex-wrap items-end gap-2">
                <label class="input w-full sm:w-md sm:max-w-md sm:flex-none">
                    <IconView icon=Icon::Search/>
                    <input type="search" name="suche" placeholder="Partner, Beleg oder Text suchen …" prop:value=move || filter.read().suche.clone()/>
                </label>
                <select name="art" class="select w-44" aria-label="Art" onchange="this.form.requestSubmit()">
                    {KIND_FILTERS.iter().map(|(k, l)| view! {
                        <option value=*k selected=move || filter.read().art == *k>{*l}</option>
                    }).collect_view()}
                </select>
                <label class="input w-44" title="Datum von">
                    <span class="label">"von"</span>
                    <input type="date" name="von" prop:value=move || filter.read().von.clone()/>
                </label>
                <label class="input w-44" title="Datum bis">
                    <span class="label">"bis"</span>
                    <input type="date" name="bis" prop:value=move || filter.read().bis.clone()/>
                </label>
                <Transition fallback=|| ()>
                    {move || Suspend::new(async move {
                        let banks = form.await.map(|f| f.bankkonten).unwrap_or_default();
                        view! {
                            <select name="bankkonto_id" class="select w-52" aria-label="Bankkonto" onchange="this.form.requestSubmit()">
                                <option value="0">"Alle Bankkonten"</option>
                                {banks.into_iter().map(|b| {
                                    let id = b.id;
                                    view! { <option value=id selected=move || filter.read().bankkonto_id == id>{b.bezeichnung}</option> }
                                }).collect_view()}
                            </select>
                        }
                    })}
                </Transition>
                {move || {
                    let f = filter.get();
                    let sort = (f.sortierung != "datum").then(|| f.sortierung.clone());
                    let dir = (f.richtung != default_dir(&f.sortierung)).then(|| f.richtung.clone());
                    view! {
                        {sort.map(|v| view! { <input type="hidden" name="sortierung" value=v/> })}
                        {dir.map(|v| view! { <input type="hidden" name="richtung" value=v/> })}
                    }
                }}
                <button type="submit" class="btn">"Suchen"</button>
            </div>
        </Form>
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(page) => view! { <PaymentTable page filter=filter.get()/> }.into_any(),
                    Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                }
            })}
        </Transition>
    }
}

#[component]
fn PaymentTable(page: PaymentPage, filter: PaymentFilter) -> impl IntoView {
    if page.rows.is_empty() {
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">"Keine Zahlungen für diesen Filter."</div>
            </div>
        }
        .into_any();
    }
    let page_count = page.page_count();
    let first = (page.page - 1) * PAGE_SIZE + 1;
    let last = first + page.rows.len() as i64 - 1;
    let header = |key: &'static str, label: &'static str, right: bool| {
        let active = filter.sortierung == key;
        let arrow = match (active, filter.richtung.as_str()) {
            (true, "ab") => " ▼",
            (true, _) => " ▲",
            _ => "",
        };
        view! {
            <th class:text-right=right>
                <a href=list_href(&sorted_by(&filter, key)) class="link link-hover" class:text-secondary=active>{label}{arrow}</a>
            </th>
        }
    };
    let with_page = |seite: i64| {
        list_href(&PaymentFilter {
            seite,
            ..filter.clone()
        })
    };
    let prev = (page.page > 1).then(|| with_page(page.page - 1));
    let next = (page.page < page_count).then(|| with_page(page.page + 1));
    let rows = page
        .rows
        .into_iter()
        .map(|row| view! { <PaymentTableRow row/> })
        .collect_view();
    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        {header("datum", "Datum", false)}
                        {header("art", "Art", false)}
                        {header("partner", "Partner", false)}
                        {header("bankkonto", "Bankkonto", false)}
                        {header("betrag", "Betrag", true)}
                        <th>"Ausgeglichen"</th>
                        {header("status", "Status", false)}
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-2 text-sm">
            <span class="text-base-content/70">{format!("{first}–{last} von {} Zahlungen", page.total)}</span>
            {(page_count > 1).then(|| view! {
                <div class="join">
                    {match prev {
                        Some(h) => view! { <a href=h class="join-item btn btn-sm">"‹ Zurück"</a> }.into_any(),
                        None => view! { <button class="join-item btn btn-sm" disabled>"‹ Zurück"</button> }.into_any(),
                    }}
                    <span class="join-item btn btn-sm btn-ghost pointer-events-none">{format!("Seite {} von {page_count}", page.page)}</span>
                    {match next {
                        Some(h) => view! { <a href=h class="join-item btn btn-sm">"Weiter ›"</a> }.into_any(),
                        None => view! { <button class="join-item btn btn-sm" disabled>"Weiter ›"</button> }.into_any(),
                    }}
                </div>
            })}
        </div>
    }
    .into_any()
}

#[component]
fn PaymentTableRow(row: PaymentRow) -> impl IntoView {
    let navigate = use_navigate();
    let href = format!("{PAYMENTS_URL}/{}", row.id);
    let target = href.clone();
    let seite = if row.seite == "kreditoren" {
        "Kreditoren"
    } else {
        "Debitoren"
    };
    view! {
        <tr class="cursor-pointer hover:bg-base-200" on:click=move |_| navigate(&target, Default::default())>
            <td class="whitespace-nowrap"><a href=href class="link link-hover font-medium">{format_date(Some(&row.datum))}</a></td>
            <td class="whitespace-nowrap">{kind_label(&row.art)} <span class="text-sm text-base-content/60">{format!(" · {seite}")}</span></td>
            <td>{row.partner.clone()}</td>
            <td>{row.bankkonto.clone().unwrap_or_else(|| "–".into())}</td>
            <td class="whitespace-nowrap text-right tabular-nums">
                <span class="text-base-content/60">{row.waehrung.clone()} " "</span>
                {format_amount(row.betrag)}
            </td>
            <td>
                {row.belege.clone().unwrap_or_default()}
                {(row.vorauszahlung_offen > 0).then(|| view! {
                    <span class="badge badge-sm badge-secondary badge-soft ml-1">{format!("Vorauszahlung {}", format_amount(row.vorauszahlung_offen))}</span>
                })}
            </td>
            <td><span class=status_badge(&row.status)>{if row.status == "storniert" { "Storniert" } else { "Gebucht" }}</span></td>
        </tr>
    }
}

/// Offene Posten mit Altersgliederung (AK-13).
#[component]
fn AgingView() -> impl IntoView {
    let query = use_query_map();
    let stichtag = Memo::new(move |_| query.read().get("stichtag").unwrap_or_default());
    let data = Resource::new(
        move || (stichtag.get(), data_version()),
        |(s, _)| get_aging(s),
    );
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(report) => {
                        let hinweis = format!(
                            "Offen am {} in CHF zum Buchungskurs; Gutschriften und Vorauszahlungen mit Minus. Spalten: Tage überfällig.",
                            format_date(Some(&report.stichtag))
                        );
                        view! {
                        <Form method="GET" action=PAYMENTS_URL>
                            <input type="hidden" name="ansicht" value="offene-posten"/>
                            <div class="flex flex-wrap items-end gap-2">
                                <label class="input w-52" title="Stichtag">
                                    <span class="label">"Stichtag"</span>
                                    <input type="date" name="stichtag" value=report.stichtag.clone()/>
                                </label>
                                <button type="submit" class="btn">"Anzeigen"</button>
                            </div>
                        </Form>
                        <p class="text-sm text-base-content/60">
                            {hinweis}
                        </p>
                        <AgingTable title="Debitoren" rows=report.debitoren seite="debitoren"/>
                        <AgingTable title="Kreditoren" rows=report.kreditoren seite="kreditoren"/>
                    }.into_any()
                    }
                    Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                }
            })}
        </Transition>
    }
}

#[component]
fn AgingTable(title: &'static str, rows: Vec<PartnerAging>, seite: &'static str) -> impl IntoView {
    let (summen, total) = aging_totals(&rows);
    let cell = |v: i64| view! { <td class="text-right tabular-nums" class:text-error=v < 0>{(v != 0).then(|| format_amount(v))}</td> };
    let neu = if seite == "kreditoren" {
        "ausgang"
    } else {
        "eingang"
    };
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">{title}</h2>
                {if rows.is_empty() {
                    view! { <p class="text-base-content/60">"Keine offenen Posten."</p> }.into_any()
                } else {
                    view! {
                        <div class="overflow-x-auto">
                            <table class="table">
                                <thead>
                                    <tr>
                                        <th>"Partner"</th>
                                        {AGING_LABELS.iter().map(|l| view! { <th class="text-right">{*l}</th> }).collect_view()}
                                        <th class="text-right">"Total CHF"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {rows.into_iter().map(|r| view! {
                                        <tr>
                                            <td><a href=format!("/adressen/{}", r.adresse_id) class="link link-hover">{r.partner.clone()}</a></td>
                                            {r.klassen.into_iter().map(cell).collect_view()}
                                            <td class="text-right font-medium tabular-nums">{format_amount(r.total)}</td>
                                            <td class="text-right">
                                                <a href=format!("{PAYMENTS_URL}?ansicht=offene-posten&neu={neu}") class="btn btn-ghost btn-xs">"Zahlung"</a>
                                            </td>
                                        </tr>
                                    }).collect_view()}
                                </tbody>
                                <tfoot class="text-base text-base-content">
                                    <tr class="font-semibold">
                                        <td>"Summe"</td>
                                        {summen.into_iter().map(cell).collect_view()}
                                        <td class="text-right tabular-nums">{format_amount(total)}</td>
                                        <td></td>
                                    </tr>
                                </tfoot>
                            </table>
                        </div>
                    }.into_any()
                }}
            </div>
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hrefs_and_dialogs() {
        assert_eq!(
            list_href(&PaymentFilter::default().normalized()),
            "/finanzen/zahlungen"
        );
        let f = PaymentFilter {
            art: "ausgang".into(),
            bankkonto_id: 2,
            ..Default::default()
        }
        .normalized();
        assert_eq!(
            list_href(&f),
            "/finanzen/zahlungen?art=ausgang&bankkonto_id=2"
        );
        assert_eq!(
            list_href(&sorted_by(&f, "partner")),
            "/finanzen/zahlungen?art=ausgang&bankkonto_id=2&sortierung=partner"
        );
        assert_eq!(
            dialog_of("verrechnung-kreditoren"),
            Some(("verrechnung", "kreditoren"))
        );
        assert_eq!(dialog_of("x"), None);
    }
}
