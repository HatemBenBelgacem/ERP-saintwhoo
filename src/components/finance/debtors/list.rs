//! Liste der Debitoren (ANF-018 AK-1 bis AK-3) mit Suche, Status- und Zeitraumfilter, Sortierung und
//! Seiten; dazu die Tabellenzeile, die auch Projekt und Adresse verwenden (AK-15, AK-16).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::InvoiceCreateModal;
use crate::backend::finance::debtors::model::{
    DEBTORS_URL, DEFAULT_SORT, InvoiceFilter, InvoicePage, InvoiceRow, PAGE_SIZE, STATUS_FILTERS,
    default_dir, kind_label, signed, status_badge, status_label,
};
use crate::backend::finance::debtors::server_fns::list_invoices;
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::format_amount;
use crate::components::finance::FinanceTabs;
use crate::forms::{data_version, encode, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Webadresse der Liste zu einem Filter; Vorgaben werden weggelassen (AK-2).
fn list_href(f: &InvoiceFilter) -> String {
    let mut params = Vec::new();
    if !f.suche.is_empty() {
        params.push(format!("suche={}", encode(&f.suche)));
    }
    if f.status != "offen" {
        params.push(format!("status={}", encode(&f.status)));
    }
    if !f.von.is_empty() {
        params.push(format!("von={}", f.von));
    }
    if !f.bis.is_empty() {
        params.push(format!("bis={}", f.bis));
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
        DEBTORS_URL.into()
    } else {
        format!("{DEBTORS_URL}?{}", params.join("&"))
    }
}

/// Klick auf einen Spaltenkopf: neue Spalte in Standardrichtung, gleiche Spalte umkehren.
fn sorted_by(f: &InvoiceFilter, key: &str) -> InvoiceFilter {
    let richtung = if f.sortierung == key {
        if f.richtung == "auf" { "ab" } else { "auf" }
    } else {
        default_dir(key)
    };
    InvoiceFilter {
        sortierung: key.into(),
        richtung: richtung.into(),
        seite: 1,
        ..f.clone()
    }
}

#[component]
pub fn InvoiceListPage() -> impl IntoView {
    let query = use_query_map();
    let filter = Memo::new(move |_| {
        let q = query.read();
        InvoiceFilter {
            suche: q.get("suche").unwrap_or_default(),
            status: q.get("status").unwrap_or_default(),
            von: q.get("von").unwrap_or_default(),
            bis: q.get("bis").unwrap_or_default(),
            sortierung: q.get("sortierung").unwrap_or_default(),
            richtung: q.get("richtung").unwrap_or_default(),
            seite: q.get("seite").and_then(|p| p.parse().ok()).unwrap_or(1),
        }
        .normalized()
    });
    let data = Resource::new(
        move || (filter.get(), data_version()),
        |(f, _)| list_invoices(f),
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
        <Title text="Debitoren – Finanzen – uniqwork"/>
        <div class="space-y-4">
            <FinanceTabs active="debitoren"/>
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Debitoren"</h1>
                <div class="flex flex-wrap gap-2">
                    <a href=move || with_params(&base.get(), &[("neu", "rechnung".into())]) class="btn btn-primary">
                        <IconView icon=Icon::Plus/>
                        "Neue Rechnung"
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

            // AK-2: Suche, Status, Zeitraum; die Sortierung bleibt erhalten, die Seite springt auf 1
            <Form method="GET" action=DEBTORS_URL>
                <div class="flex flex-wrap items-end gap-2">
                    <label class="input w-full sm:w-md sm:max-w-md sm:flex-none">
                        <IconView icon=Icon::Search/>
                        <input
                            type="search"
                            name="suche"
                            placeholder="Nummer, Kunde, Projekt oder Referenz suchen …"
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

            // AK-4: Anlegen im Modal-Fenster (?neu=rechnung bzw. ?neu=gutschrift)
            {move || create().map(|art| view! { <InvoiceCreateModal art/> })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(page) => view! { <InvoiceTable page filter=filter.get()/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

#[component]
fn InvoiceTable(page: InvoicePage, filter: InvoiceFilter) -> impl IntoView {
    if page.rows.is_empty() {
        let text = if !filter.suche.is_empty() {
            format!("Keine Belege zu „{}“ gefunden.", filter.suche)
        } else if filter.status == "offen" {
            "Keine offenen Rechnungen.".to_string()
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
        list_href(&InvoiceFilter {
            seite,
            ..filter.clone()
        })
    };
    let prev = (page.page > 1).then(|| with_page(page.page - 1));
    let next = (page.page < page_count).then(|| with_page(page.page + 1));
    let rows = page
        .rows
        .into_iter()
        .map(|row| view! { <InvoiceTableRow row show_customer=true show_project=true/> })
        .collect_view();

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        {header("nummer", "Nummer", false)}
                        <th>"Art"</th>
                        {header("kunde", "Kunde", false)}
                        {header("projekt", "Projekt", false)}
                        {header("rechnungsdatum", "Rechnungsdatum", false)}
                        {header("faelligkeit", "Fälligkeit", false)}
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

/// Fälligkeit mit Markierung „überfällig“ und Tagen seit Fälligkeit (AK-3).
#[component]
pub fn DueDate(faelligkeit: Option<String>, tage: Option<i32>) -> impl IntoView {
    view! {
        <span class:text-error=tage.is_some()>{format_date(faelligkeit.as_deref())}</span>
        {tage.map(|t| {
            let text = if t == 1 { "überfällig · 1 Tag".to_string() } else { format!("überfällig · {t} Tage") };
            view! { <span class="badge badge-sm badge-error badge-soft ml-2" title="Fälligkeit überschritten">{text}</span> }
        })}
    }
}

/// Eine Tabellenzeile; Klick öffnet die Detailansicht. Gutschriften erscheinen mit Minus.
#[component]
pub fn InvoiceTableRow(row: InvoiceRow, show_customer: bool, show_project: bool) -> impl IntoView {
    let navigate = use_navigate();
    let href = format!("{DEBTORS_URL}/{}", row.id);
    let target = href.clone();
    let label = row.nummer.clone().unwrap_or_else(|| "Entwurf".into());
    let total = row.gesamtbetrag.map(|b| format_amount(signed(&row.art, b)));
    let open =
        (row.status != "entwurf").then(|| format_amount(signed(&row.art, row.offener_betrag)));
    view! {
        <tr class="cursor-pointer hover:bg-base-200" on:click=move |_| navigate(&target, Default::default())>
            <td class="whitespace-nowrap">
                <a href=href class="link link-hover font-medium" class:italic=row.nummer.is_none()>{label}</a>
            </td>
            <td>{kind_label(&row.art)}</td>
            {show_customer.then(|| view! { <td>{row.kunde.clone()}</td> })}
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
    fn default_filter_has_plain_href() {
        let f = InvoiceFilter::default().normalized();
        assert_eq!(list_href(&f), "/finanzen/debitoren");
    }

    #[test]
    fn href_keeps_filter_and_toggles_sorting() {
        let f = InvoiceFilter {
            suche: "Müller".into(),
            status: "alle".into(),
            von: "2026-01-01".into(),
            seite: 2,
            ..Default::default()
        }
        .normalized();
        assert_eq!(
            list_href(&f),
            "/finanzen/debitoren?suche=M%C3%BCller&status=alle&von=2026-01-01&seite=2"
        );
        let by_customer = sorted_by(&f, "kunde");
        assert_eq!(
            list_href(&by_customer),
            "/finanzen/debitoren?suche=M%C3%BCller&status=alle&von=2026-01-01&sortierung=kunde"
        );
        assert_eq!(sorted_by(&by_customer, "kunde").richtung, "ab");
    }
}
