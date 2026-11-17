//! Adressliste mit Suche, Sortierung und Seiten (AK-1 bis AK-6).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::AddressCreateModal;
use crate::backend::addresses::model::{AddressPage, PAGE_SIZE, SORT_COLUMNS};
use crate::backend::addresses::server_fns::list_addresses;
use crate::forms::error_message;
use crate::forms::{encode, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Zustand der Liste – kommt vollständig aus der Webadresse (AK-6).
#[derive(Clone, Debug, PartialEq)]
struct ListState {
    search: String,
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
        if self.sort != "name" {
            params.push(format!("sortierung={}", encode(&self.sort)));
        }
        if self.dir == "ab" {
            params.push("richtung=ab".to_string());
        }
        if self.page > 1 {
            params.push(format!("seite={}", self.page));
        }
        if params.is_empty() {
            "/adressen".into()
        } else {
            format!("/adressen?{}", params.join("&"))
        }
    }

    /// Webadresse nach Klick auf einen Spaltenkopf: gleiche Spalte → Richtung umkehren (AK-3).
    fn sorted_by(&self, key: &str) -> Self {
        let dir = if self.sort == key && self.dir != "ab" {
            "ab"
        } else {
            "auf"
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
pub fn AddressListPage() -> impl IntoView {
    let query = use_query_map();
    let state = Memo::new(move |_| {
        let q = query.read();
        ListState {
            search: q.get("suche").unwrap_or_default(),
            sort: q.get("sortierung").unwrap_or_else(|| "name".into()),
            dir: q.get("richtung").unwrap_or_else(|| "auf".into()),
            page: q.get("seite").and_then(|p| p.parse().ok()).unwrap_or(1),
        }
    });
    let data = Resource::new(
        move || state.get(),
        |s| list_addresses(s.search, s.sort, s.dir, s.page),
    );
    let deleted = move || query.read().get("geloescht");
    let base = use_base_href();
    let create_open = move || query.read().get("neu").is_some();

    view! {
        <Title text="Adressen – ERP-saintwhoo"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Adressen"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neue Adresse"
                </a>
            </div>

            {move || {
                deleted()
                    .map(|nummer| {
                        view! {
                            <div role="alert" class="alert alert-success">
                                "Adresse " {nummer} " wurde gelöscht."
                            </div>
                        }
                    })
            }}

            // AK-2: Suche (Enter oder Schaltfläche); Sortierung bleibt erhalten, Seite springt auf 1
            <Form method="GET" action="/adressen">
                <div class="flex gap-2">
                    <label class="input min-w-0 flex-1 sm:max-w-md sm:flex-none sm:w-md">
                        <IconView icon=Icon::Search/>
                        <input
                            type="search"
                            name="suche"
                            placeholder="Nummer, Name, Ort, PLZ oder E-Mail suchen …"
                            prop:value=move || state.read().search.clone()
                        />
                    </label>
                    {move || {
                        let s = state.get();
                        // Nur abweichende Sortierung mitschicken – hält die Webadresse kurz
                        let sort = (s.sort != "name").then(|| s.sort.clone());
                        let dir = (s.dir != "auf").then(|| s.dir.clone());
                        view! {
                            {sort.map(|v| view! { <input type="hidden" name="sortierung" value=v/> })}
                            {dir.map(|v| view! { <input type="hidden" name="richtung" value=v/> })}
                        }
                    }}
                    <button type="submit" class="btn">"Suchen"</button>
                </div>
            </Form>

            // AK-8: Eingabe im Modal-Fenster (?neu=1)
            {move || create_open().then(|| view! { <AddressCreateModal/> })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(page) => view! { <AddressTable page state=state.get()/> }.into_any(),
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
fn AddressTable(page: AddressPage, state: ListState) -> impl IntoView {
    // AK-5: Hinweis statt leerer Tabelle
    if page.rows.is_empty() {
        let text = if state.search.is_empty() {
            "Noch keine Adressen erfasst.".to_string()
        } else {
            format!("Keine Adressen zu „{}“ gefunden.", state.search)
        };
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">{text}</div>
            </div>
        }
        .into_any();
    }

    let navigate = use_navigate();
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
            view! {
                <th aria-sort=active.then_some(if state.dir == "ab" { "descending" } else { "ascending" })>
                    <a href=state.sorted_by(key).href() class="link link-hover" class:text-secondary=active>
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
        .map(|row| {
            let href = format!("/adressen/{}", row.id);
            let nav = navigate.clone();
            let target = href.clone();
            view! {
                // AK-7: Klick auf die Zeile öffnet die Detailansicht
                <tr
                    class="cursor-pointer hover:bg-base-200"
                    on:click=move |_| nav(&target, Default::default())
                >
                    <td class="whitespace-nowrap">
                        <a href=href.clone() class="link link-hover">{row.nummer}</a>
                    </td>
                    <td class="font-medium">
                        <a href=href class="link link-hover">{row.anzeigename}</a>
                    </td>
                    <td>{row.plz}</td>
                    <td>{row.ort}</td>
                    <td>{row.land}</td>
                    <td>{row.email}</td>
                    <td class="whitespace-nowrap">{row.telefon}</td>
                </tr>
            }
        })
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
        // AK-4: Seiten
        <div class="flex flex-wrap items-center justify-between gap-2 text-sm">
            <span class="text-base-content/70">
                {format!("{first}–{last} von {} Adressen", page.total)}
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

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ListState {
        ListState {
            search: String::new(),
            sort: "name".into(),
            dir: "auf".into(),
            page: 1,
        }
    }

    #[test]
    fn default_state_has_plain_href() {
        assert_eq!(state().href(), "/adressen");
    }

    #[test]
    fn href_encodes_search() {
        let s = ListState {
            search: "Müller & Co".into(),
            page: 2,
            ..state()
        };
        assert_eq!(s.href(), "/adressen?suche=M%C3%BCller%20%26%20Co&seite=2");
    }

    #[test]
    fn sorting_toggles_direction_and_resets_page() {
        let s = state().with_page(3);
        let by_name = s.sorted_by("name");
        assert_eq!((by_name.dir.as_str(), by_name.page), ("ab", 1));
        assert_eq!(by_name.sorted_by("name").dir, "auf");
        assert_eq!(by_name.sorted_by("ort").dir, "auf");
    }
}
