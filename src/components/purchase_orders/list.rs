//! Liste der Einkaufsaufträge mit Suche, Statusfilter, Sortierung und Seiten (AK-1 bis AK-5).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::OrderCreateModal;
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::{
    OrderPage, OrderRow, PAGE_SIZE, SORT_COLUMNS, STATUS_FILTERS, format_amount, status_badge,
    status_label,
};
use crate::backend::purchase_orders::server_fns::list_orders;
use crate::forms::{data_version, encode, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Standardfilter: alle offenen Aufträge (AK-2).
const DEFAULT_STATUS: &str = "offen";

/// Standard-Sortierrichtung je Spalte: Nummer absteigend (neueste zuerst), sonst aufsteigend.
fn default_dir(sort: &str) -> &'static str {
    if sort == "nummer" { "ab" } else { "auf" }
}

/// Zustand der Liste – vollständig aus der Webadresse (AK-3).
#[derive(Clone, Debug, PartialEq)]
struct ListState {
    search: String,
    status: String,
    sort: String,
    dir: String,
    page: i64,
}

impl ListState {
    fn href(&self) -> String {
        let mut params = Vec::new();
        if !self.search.is_empty() {
            params.push(format!("suche={}", encode(&self.search)));
        }
        if self.status != DEFAULT_STATUS {
            params.push(format!("status={}", encode(&self.status)));
        }
        if self.sort != "nummer" {
            params.push(format!("sortierung={}", encode(&self.sort)));
        }
        if self.dir != default_dir(&self.sort) {
            params.push(format!("richtung={}", encode(&self.dir)));
        }
        if self.page > 1 {
            params.push(format!("seite={}", self.page));
        }
        if params.is_empty() {
            "/einkauf".into()
        } else {
            format!("/einkauf?{}", params.join("&"))
        }
    }

    /// Klick auf Spaltenkopf: neue Spalte in Standardrichtung, gleiche Spalte umkehren.
    fn sorted_by(&self, key: &str) -> Self {
        let dir = if self.sort == key {
            if self.dir == "auf" { "ab" } else { "auf" }
        } else {
            default_dir(key)
        };
        Self {
            sort: key.into(),
            dir: dir.into(),
            page: 1,
            ..self.clone()
        }
    }

    fn with_page(&self, page: i64) -> Self {
        Self {
            page,
            ..self.clone()
        }
    }
}

#[component]
pub fn OrderListPage() -> impl IntoView {
    let query = use_query_map();
    let state = Memo::new(move |_| {
        let q = query.read();
        let sort = q.get("sortierung").unwrap_or_else(|| "nummer".into());
        ListState {
            search: q.get("suche").unwrap_or_default(),
            status: q.get("status").unwrap_or_else(|| DEFAULT_STATUS.into()),
            dir: q
                .get("richtung")
                .unwrap_or_else(|| default_dir(&sort).into()),
            sort,
            page: q.get("seite").and_then(|p| p.parse().ok()).unwrap_or(1),
        }
    });
    let data = Resource::new(
        move || (state.get(), data_version()),
        |(s, _)| list_orders(s.search, s.status, s.sort, s.dir, s.page),
    );
    let deleted = move || query.read().get("geloescht");
    let base = use_base_href();
    let create_open = move || query.read().get("neu").is_some();

    view! {
        <Title text="Einkauf – ERP-saintwhoo"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Einkauf"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neuer Einkaufsauftrag"
                </a>
            </div>

            {move || deleted().map(|n| view! {
                <div role="alert" class="alert alert-success">"Einkaufsauftrag " {n} " wurde gelöscht."</div>
            })}

            // AK-2: Suche und Statusfilter; Sortierung bleibt erhalten, Seite springt auf 1
            <Form method="GET" action="/einkauf">
                <div class="flex flex-wrap gap-2">
                    <label class="input min-w-0 flex-1 sm:w-md sm:max-w-md sm:flex-none">
                        <IconView icon=Icon::Search/>
                        <input
                            type="search"
                            name="suche"
                            placeholder="Nummer, Lieferant, Projekt oder Referenz suchen …"
                            prop:value=move || state.read().search.clone()
                        />
                    </label>
                    <select
                        name="status"
                        class="select w-52"
                        aria-label="Status"
                        onchange="this.form.requestSubmit()"
                        prop:value=move || state.read().status.clone()
                    >
                        {STATUS_FILTERS
                            .iter()
                            .map(|(k, l)| view! {
                                <option value=*k selected=move || state.read().status == *k>{*l}</option>
                            })
                            .collect_view()}
                    </select>
                    {move || {
                        let s = state.get();
                        // Nur abweichende Sortierung mitschicken – hält die Webadresse kurz
                        let sort = (s.sort != "nummer").then(|| s.sort.clone());
                        let dir = (s.dir != default_dir(&s.sort)).then(|| s.dir.clone());
                        view! {
                            {sort.map(|v| view! { <input type="hidden" name="sortierung" value=v/> })}
                            {dir.map(|v| view! { <input type="hidden" name="richtung" value=v/> })}
                        }
                    }}
                    <button type="submit" class="btn">"Suchen"</button>
                </div>
            </Form>

            // AK-8: Anlegen im Modal-Fenster (?neu=1)
            {move || create_open().then(|| view! { <OrderCreateModal/> })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(page) => view! { <OrderTable page state=state.get()/> }.into_any(),
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
fn OrderTable(page: OrderPage, state: ListState) -> impl IntoView {
    // AK-4: Hinweis statt leerer Tabelle
    if page.rows.is_empty() {
        let text = if !state.search.is_empty() {
            format!("Keine Einkaufsaufträge zu „{}“ gefunden.", state.search)
        } else if state.status == DEFAULT_STATUS {
            "Keine offenen Einkaufsaufträge.".to_string()
        } else {
            "Keine Einkaufsaufträge mit diesem Status.".to_string()
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

    let headers = SORT_COLUMNS
        .iter()
        .map(|(key, label)| {
            let active = state.sort == *key;
            let arrow = match (active, state.dir.as_str()) {
                (true, "ab") => " ▼",
                (true, _) => " ▲",
                _ => "",
            };
            let right = *key == "betrag";
            view! {
                <th
                    class:text-right=right
                    aria-sort=active.then_some(if state.dir == "ab" { "descending" } else { "ascending" })
                >
                    <a href=state.sorted_by(key).href() class="link link-hover" class:text-primary=active>
                        {*label}
                        {arrow}
                    </a>
                </th>
            }
        })
        .collect_view();

    let rows = page
        .rows
        .into_iter()
        .map(|row| view! { <OrderTableRow row show_project=true/> })
        .collect_view();

    let prev = (page.page > 1).then(|| state.with_page(page.page - 1).href());
    let next = (page.page < page_count).then(|| state.with_page(page.page + 1).href());

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>{headers}</tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
        // AK-3: Seiten
        <div class="flex flex-wrap items-center justify-between gap-2 text-sm">
            <span class="text-base-content/70">
                {format!("{first}–{last} von {} Einkaufsaufträgen", page.total)}
            </span>
            {(page_count > 1)
                .then(|| {
                    view! {
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
                    }
                })}
        </div>
    }
    .into_any()
}

/// Eine Tabellenzeile; Klick öffnet die Detailansicht (AK-6). Auch in der Projekt-Detailansicht verwendet (AK-22).
#[component]
pub(super) fn OrderTableRow(row: OrderRow, show_project: bool) -> impl IntoView {
    let navigate = use_navigate();
    let href = format!("/einkauf/{}", row.id);
    let target = href.clone();
    view! {
        <tr
            class="cursor-pointer hover:bg-base-200"
            on:click=move |_| navigate(&target, Default::default())
        >
            <td class="whitespace-nowrap">
                <a href=href class="link link-hover font-medium">{row.nummer}</a>
            </td>
            <td>{row.lieferant}</td>
            {show_project.then(|| view! {
                <td class="whitespace-nowrap">
                    {match (row.projekt_id, row.projekt_nummer) {
                        (Some(id), Some(n)) => view! {
                            <a href=format!("/projekte/{id}") class="link link-hover" on:click=|ev| ev.stop_propagation()>{n}</a>
                        }.into_any(),
                        _ => view! { "–" }.into_any(),
                    }}
                </td>
            })}
            <td class="whitespace-nowrap">{format_date(row.bestelldatum.as_deref())}</td>
            // AK-5: überfälliger Liefertermin
            <td class="whitespace-nowrap">
                {format_date(row.liefertermin.as_deref())}
                {row.ueberfaellig.then(|| view! {
                    <span class="badge badge-sm badge-error badge-soft ml-2" title="Liefertermin überschritten">"überfällig"</span>
                })}
            </td>
            <td><span class=status_badge(&row.status)>{status_label(&row.status).to_string()}</span></td>
            <td class="whitespace-nowrap text-right tabular-nums">
                <span class="text-base-content/60">{row.waehrung} " "</span>
                {format_amount(row.total)}
            </td>
        </tr>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ListState {
        ListState {
            search: String::new(),
            status: DEFAULT_STATUS.into(),
            sort: "nummer".into(),
            dir: "ab".into(),
            page: 1,
        }
    }

    #[test]
    fn default_state_has_plain_href() {
        assert_eq!(state().href(), "/einkauf");
    }

    #[test]
    fn href_keeps_filter_and_sorting() {
        let s = ListState {
            search: "Müller".into(),
            status: "alle".into(),
            page: 2,
            ..state()
        };
        assert_eq!(s.href(), "/einkauf?suche=M%C3%BCller&status=alle&seite=2");
        let by_supplier = s.sorted_by("lieferant");
        assert_eq!(
            by_supplier.href(),
            "/einkauf?suche=M%C3%BCller&status=alle&sortierung=lieferant"
        );
        assert_eq!(by_supplier.sorted_by("lieferant").dir, "ab");
    }
}
