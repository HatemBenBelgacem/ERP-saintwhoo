//! Liste der Kreditoren (ANF-019 AK-1 bis AK-3) mit Suche, Filtern (Status, Zeitraum, fällig bis),
//! Sortierung und Seiten; dazu die Tabellenzeile für Projekt, Adresse und Einkaufsauftrag.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::CreditorCreateModal;
use crate::backend::finance::creditors::model::{
    CREDITORS_URL, CreditorFilter, CreditorPage, CreditorRow, DEFAULT_SORT, PAGE_SIZE,
    STATUS_FILTERS, default_dir,
};
use crate::backend::finance::creditors::server_fns::list_creditors;
use crate::backend::finance::debtors::model::{kind_label, signed, status_badge, status_label};
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::format_amount;
use crate::components::finance::FinanceTabs;
use crate::components::finance::debtors::DueDate;
use crate::components::finance::payments::PayCreditorsModal;
use crate::forms::{data_version, encode, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Webadresse der Liste; Vorgaben werden weggelassen (AK-2).
fn list_href(f: &CreditorFilter) -> String {
    let mut params = Vec::new();
    if !f.suche.is_empty() {
        params.push(format!("suche={}", encode(&f.suche)));
    }
    if f.status != "offen" {
        params.push(format!("status={}", encode(&f.status)));
    }
    for (key, value) in [
        ("von", &f.von),
        ("bis", &f.bis),
        ("faellig_bis", &f.faellig_bis),
    ] {
        if !value.is_empty() {
            params.push(format!("{key}={value}"));
        }
    }
    if f.sortierung != DEFAULT_SORT {
        params.push(format!("sortierung={}", f.sortierung));
    }
    if f.richtung != default_dir(&f.sortierung) {
        params.push(format!("richtung={}", f.richtung));
    }
    if f.seite > 1 {
        params.push(format!("seite={}", f.seite));
    }
    if params.is_empty() {
        CREDITORS_URL.into()
    } else {
        format!("{CREDITORS_URL}?{}", params.join("&"))
    }
}

fn sorted_by(f: &CreditorFilter, key: &str) -> CreditorFilter {
    let richtung = if f.sortierung == key {
        if f.richtung == "auf" { "ab" } else { "auf" }
    } else {
        default_dir(key)
    };
    CreditorFilter {
        sortierung: key.into(),
        richtung: richtung.into(),
        seite: 1,
        ..f.clone()
    }
}

#[component]
pub fn CreditorListPage() -> impl IntoView {
    let query = use_query_map();
    let filter = Memo::new(move |_| {
        let q = query.read();
        CreditorFilter {
            suche: q.get("suche").unwrap_or_default(),
            status: q.get("status").unwrap_or_default(),
            von: q.get("von").unwrap_or_default(),
            bis: q.get("bis").unwrap_or_default(),
            faellig_bis: q.get("faellig_bis").unwrap_or_default(),
            sortierung: q.get("sortierung").unwrap_or_default(),
            richtung: q.get("richtung").unwrap_or_default(),
            seite: q.get("seite").and_then(|p| p.parse().ok()).unwrap_or(1),
        }
        .normalized()
    });
    let data = Resource::new(
        move || (filter.get(), data_version()),
        |(f, _)| list_creditors(f),
    );
    let base = use_base_href();
    let create = move || {
        query
            .read()
            .get("neu")
            .filter(|n| n == "rechnung" || n == "gutschrift")
    };
    let deleted = move || query.read().get("geloescht");

    view! {
        <Title text="Kreditoren – Finanzen – uniqwork"/>
        <div class="space-y-4">
            <FinanceTabs active="kreditoren"/>
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Kreditoren"</h1>
                <div class="flex flex-wrap gap-2">
                    <a href=move || with_params(&base.get(), &[("neu", "rechnung".into())]) class="btn btn-primary">
                        <IconView icon=Icon::Plus/>
                        "Neue Eingangsrechnung"
                    </a>
                    <a href=move || with_params(&base.get(), &[("neu", "gutschrift".into())]) class="btn btn-outline">
                        <IconView icon=Icon::Plus/>
                        "Neue Gutschrift"
                    </a>
                </div>
            </div>

            {move || deleted().map(|art| view! {
                <div role="alert" class="alert alert-success">
                    {format!("Der Entwurf ({}) wurde gelöscht.", kind_label(&art))}
                </div>
            })}

            <Form method="GET" action=CREDITORS_URL>
                <div class="flex flex-wrap items-end gap-2">
                    <label class="input w-full sm:w-md sm:max-w-md sm:flex-none">
                        <IconView icon=Icon::Search/>
                        <input
                            type="search"
                            name="suche"
                            placeholder="Nummer, Rechnungs-Nr., Lieferant, Auftrag oder Projekt …"
                            prop:value=move || filter.read().suche.clone()
                        />
                    </label>
                    <select
                        name="status"
                        class="select w-60"
                        aria-label="Status"
                        onchange="this.form.requestSubmit()"
                        prop:value=move || filter.read().status.clone()
                    >
                        {STATUS_FILTERS
                            .iter()
                            .map(|(k, l)| view! {
                                <option value=*k selected=move || filter.read().status == *k>{*l}</option>
                            })
                            .collect_view()}
                    </select>
                    <label class="input w-44" title="Rechnungsdatum von">
                        <span class="label">"von"</span>
                        <input type="date" name="von" prop:value=move || filter.read().von.clone()/>
                    </label>
                    <label class="input w-44" title="Rechnungsdatum bis">
                        <span class="label">"bis"</span>
                        <input type="date" name="bis" prop:value=move || filter.read().bis.clone()/>
                    </label>
                    <label class="input w-52" title="Fällig bis">
                        <span class="label">"fällig bis"</span>
                        <input type="date" name="faellig_bis" prop:value=move || filter.read().faellig_bis.clone()/>
                    </label>
                    {move || {
                        let f = filter.get();
                        let sort = (f.sortierung != DEFAULT_SORT).then(|| f.sortierung.clone());
                        let dir = (f.richtung != default_dir(&f.sortierung)).then(|| f.richtung.clone());
                        view! {
                            {sort.map(|v| view! { <input type="hidden" name="sortierung" value=v/> })}
                            {dir.map(|v| view! { <input type="hidden" name="richtung" value=v/> })}
                        }
                    }}
                    <button type="submit" class="btn">"Suchen"</button>
                </div>
            </Form>

            {move || create().map(|art| view! { <CreditorCreateModal art/> })}
            {move || query.read().get("bezahlt").filter(|ids| !ids.is_empty()).map(|ids| view! { <PayCreditorsModal ids/> })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(page) => view! { <CreditorTable page filter=filter.get()/> }.into_any(),
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

#[component]
fn CreditorTable(page: CreditorPage, filter: CreditorFilter) -> impl IntoView {
    if page.rows.is_empty() {
        let text = if !filter.suche.is_empty() {
            format!("Keine Belege zu „{}“ gefunden.", filter.suche)
        } else if filter.status == "offen" {
            "Keine offenen Eingangsrechnungen.".to_string()
        } else {
            "Keine Belege für diesen Filter.".to_string()
        };
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">{text}</div>
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
            <th
                class:text-right=right
                aria-sort=active.then_some(if filter.richtung == "ab" { "descending" } else { "ascending" })
            >
                <a href=list_href(&sorted_by(&filter, key)) class="link link-hover" class:text-secondary=active>
                    {label}
                    {arrow}
                </a>
            </th>
        }
    };
    let with_page = |seite: i64| {
        list_href(&CreditorFilter {
            seite,
            ..filter.clone()
        })
    };
    let prev = (page.page > 1).then(|| with_page(page.page - 1));
    let next = (page.page < page_count).then(|| with_page(page.page + 1));
    // ANF-020 AK-7: offene Rechnungen markieren und gemeinsam als bezahlt erfassen
    let selected = RwSignal::new(Vec::<i64>::new());
    let base = use_base_href();
    let pay_href = move || {
        let ids = selected
            .get()
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        with_params(&base.get(), &[("bezahlt", ids)])
    };
    let rows = page
        .rows
        .into_iter()
        .map(|row| view! { <CreditorTableRow row show_supplier=true show_order=true show_project=true selected/> })
        .collect_view();
    view! {
        <div class="flex flex-wrap items-center gap-2">
            <a
                href=pay_href
                class="btn btn-sm btn-primary"
                class:btn-disabled=move || selected.read().is_empty()
                aria-disabled=move || selected.read().is_empty().then_some("true")
            >
                {move || format!("Als bezahlt erfassen ({})", selected.read().len())}
            </a>
            <span class="text-sm text-base-content/60">"Offene Rechnungen links markieren."</span>
        </div>
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        <th><span class="sr-only">"Markieren"</span></th>
                        {header("nummer", "Nummer", false)}
                        {header("rechnungsnummer", "Rechnungs-Nr.", false)}
                        {header("lieferant", "Lieferant", false)}
                        {header("auftrag", "Einkaufsauftrag", false)}
                        {header("projekt", "Projekt", false)}
                        {header("rechnungsdatum", "Rechnungsdatum", false)}
                        {header("faelligkeit", "Fälligkeit", false)}
                        {header("skonto", "Skonto bis", false)}
                        {header("betrag", "Gesamtbetrag", true)}
                        {header("offen", "Offen", true)}
                        <th>"Währung"</th>
                        {header("status", "Status", false)}
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-2 text-sm">
            <span class="text-base-content/70">{format!("{first}–{last} von {} Belegen", page.total)}</span>
            {(page_count > 1).then(|| view! {
                <div class="join">
                    {match prev {
                        Some(h) => view! { <a href=h class="join-item btn btn-sm">"‹ Zurück"</a> }.into_any(),
                        None => view! { <button class="join-item btn btn-sm" disabled>"‹ Zurück"</button> }.into_any(),
                    }}
                    <span class="join-item btn btn-sm btn-ghost pointer-events-none">
                        {format!("Seite {} von {page_count}", page.page)}
                    </span>
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

/// Skontodatum; eine laufende Skontofrist ist hervorgehoben (AK-3).
#[component]
pub(super) fn DiscountDate(skonto_bis: Option<String>, laeuft: bool) -> impl IntoView {
    view! {
        <span class:text-secondary=laeuft class:font-medium=laeuft title=laeuft.then_some("Skontofrist läuft")>
            {format_date(skonto_bis.as_deref())}
        </span>
    }
}

/// Eine Tabellenzeile; Klick öffnet die Detailansicht. Gutschriften erscheinen mit Minus.
#[component]
pub fn CreditorTableRow(
    row: CreditorRow,
    show_supplier: bool,
    show_order: bool,
    show_project: bool,
    /// Markierte Rechnungen (nur in der Liste, ANF-020 AK-7)
    #[prop(optional)]
    selected: Option<RwSignal<Vec<i64>>>,
) -> impl IntoView {
    let id = row.id;
    let payable =
        row.art == "rechnung" && matches!(row.status.as_str(), "offen" | "teilweise_bezahlt");
    let navigate = use_navigate();
    let href = format!("{CREDITORS_URL}/{}", row.id);
    let target = href.clone();
    let label = row.nummer.clone().unwrap_or_else(|| "Entwurf".into());
    let total = row.gesamtbetrag.map(|b| format_amount(signed(&row.art, b)));
    let open =
        (row.status != "entwurf").then(|| format_amount(signed(&row.art, row.offener_betrag)));
    view! {
        <tr class="cursor-pointer hover:bg-base-200" on:click=move |_| navigate(&target, Default::default())>
            {selected.map(|sel| view! {
                <td on:click=|ev| ev.stop_propagation()>
                    {payable.then(|| view! {
                        <input
                            type="checkbox"
                            class="checkbox checkbox-sm checkbox-primary"
                            aria-label="Markieren"
                            prop:checked=move || sel.read().contains(&id)
                            on:change=move |_| sel.update(|s| if let Some(p) = s.iter().position(|x| *x == id) { s.remove(p); } else { s.push(id); })
                        />
                    })}
                </td>
            })}
            <td class="whitespace-nowrap">
                <a href=href class="link link-hover font-medium" class:italic=row.nummer.is_none()>{label}</a>
                {(row.art == "gutschrift").then(|| view! { <span class="badge badge-sm badge-ghost ml-1">"Gutschrift"</span> })}
            </td>
            <td class="whitespace-nowrap">{row.rechnungsnummer.clone().unwrap_or_else(|| "–".into())}</td>
            {show_supplier.then(|| view! { <td>{row.lieferant.clone()}</td> })}
            {show_order.then(|| view! {
                <td class="whitespace-nowrap">
                    {match (row.einkaufsauftrag_id, row.einkaufsauftrag_nummer.clone()) {
                        (Some(id), Some(n)) => view! {
                            <a href=format!("/einkauf/{id}") class="link link-hover" on:click=|ev| ev.stop_propagation()>{n}</a>
                        }.into_any(),
                        _ => view! { "–" }.into_any(),
                    }}
                </td>
            })}
            {show_project.then(|| view! {
                <td class="whitespace-nowrap">
                    {match (row.projekt_id, row.projekt_nummer.clone()) {
                        (Some(id), Some(n)) => view! {
                            <a href=format!("/projekte/{id}") class="link link-hover" on:click=|ev| ev.stop_propagation()>{n}</a>
                        }.into_any(),
                        _ => view! { "–" }.into_any(),
                    }}
                </td>
            })}
            <td class="whitespace-nowrap">{format_date(Some(&row.rechnungsdatum))}</td>
            <td class="whitespace-nowrap"><DueDate faelligkeit=row.faelligkeit.clone() tage=row.tage_ueberfaellig/></td>
            <td class="whitespace-nowrap"><DiscountDate skonto_bis=row.skonto_bis.clone() laeuft=row.skonto_laeuft/></td>
            <td class="whitespace-nowrap text-right tabular-nums">{total.unwrap_or_else(|| "–".into())}</td>
            <td class="whitespace-nowrap text-right tabular-nums">{open.unwrap_or_else(|| "–".into())}</td>
            <td>{row.waehrung.clone()}</td>
            <td><span class=status_badge(&row.status)>{status_label(&row.status)}</span></td>
        </tr>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hrefs_keep_filters() {
        assert_eq!(
            list_href(&CreditorFilter::default().normalized()),
            "/finanzen/kreditoren"
        );
        let f = CreditorFilter {
            status: "alle".into(),
            faellig_bis: "2026-12-31".into(),
            ..Default::default()
        }
        .normalized();
        assert_eq!(
            list_href(&f),
            "/finanzen/kreditoren?status=alle&faellig_bis=2026-12-31"
        );
        assert_eq!(
            list_href(&sorted_by(&f, "skonto")),
            "/finanzen/kreditoren?status=alle&faellig_bis=2026-12-31&sortierung=skonto"
        );
    }
}
