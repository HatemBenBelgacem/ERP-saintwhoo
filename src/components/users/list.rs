//! Benutzerliste mit Suche, Statusfilter und Sortierung (ANF-005 AK-1 bis AK-5).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::UserCreateModal;
use crate::backend::users::model::{
    SORT_COLUMNS, STATUS_FILTERS, UserRow, status_badge, status_label,
};
use crate::backend::users::server_fns::{list_users, user_url};
use crate::forms::{data_version, encode, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

const BASE: &str = "/einstellungen/benutzer";

/// Zustand der Liste – kommt vollständig aus der Webadresse (AK-5).
#[derive(Clone, Debug, PartialEq)]
struct ListState {
    search: String,
    status: String,
    sort: String,
    dir: String,
}

impl ListState {
    fn href(&self) -> String {
        let mut params = Vec::new();
        if !self.search.is_empty() {
            params.push(format!("suche={}", encode(&self.search)));
        }
        if self.status != "alle" {
            params.push(format!("status={}", encode(&self.status)));
        }
        if self.sort != "benutzername" {
            params.push(format!("sortierung={}", encode(&self.sort)));
        }
        if self.dir == "ab" {
            params.push("richtung=ab".to_string());
        }
        if params.is_empty() {
            BASE.into()
        } else {
            format!("{BASE}?{}", params.join("&"))
        }
    }

    /// Klick auf einen Spaltenkopf: gleiche Spalte → Richtung umkehren (AK-4).
    fn sorted_by(&self, key: &str) -> Self {
        let dir = if self.sort == key && self.dir != "ab" {
            "ab"
        } else {
            "auf"
        };
        Self {
            sort: key.into(),
            dir: dir.into(),
            ..self.clone()
        }
    }
}

#[component]
pub fn UserListPage() -> impl IntoView {
    let query = use_query_map();
    let state = Memo::new(move |_| {
        let q = query.read();
        ListState {
            search: q.get("suche").unwrap_or_default(),
            status: q.get("status").unwrap_or_else(|| "alle".into()),
            sort: q.get("sortierung").unwrap_or_else(|| "benutzername".into()),
            dir: q.get("richtung").unwrap_or_else(|| "auf".into()),
        }
    });
    let data = Resource::new(
        move || (state.get(), data_version()),
        |(s, _)| list_users(s.search, s.status, s.sort, s.dir),
    );
    let deleted = move || query.read().get("geloescht");
    let base = use_base_href();
    let create_open = move || query.read().get("neu").is_some();

    view! {
        <Title text="Benutzer – Einstellungen – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Benutzer"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neuer Benutzer"
                </a>
            </div>

            {move || deleted().map(|name| view! {
                <div role="alert" class="alert alert-success">"Benutzer „" {name} "“ wurde gelöscht."</div>
            })}

            // AK-2, AK-3: Suche und Statusfilter; Sortierung bleibt erhalten
            <Form method="GET" action=BASE>
                <div class="flex flex-wrap gap-2">
                    <label class="input w-full sm:w-md sm:max-w-md sm:flex-none">
                        <IconView icon=Icon::Search/>
                        <input
                            type="search"
                            name="suche"
                            placeholder="Benutzername, Name oder E-Mail suchen …"
                            prop:value=move || state.read().search.clone()
                        />
                    </label>
                    <select
                        name="status"
                        class="select w-44"
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
                        let sort = (s.sort != "benutzername").then(|| s.sort.clone());
                        let dir = (s.dir == "ab").then(|| s.dir.clone());
                        view! {
                            {sort.map(|v| view! { <input type="hidden" name="sortierung" value=v/> })}
                            {dir.map(|v| view! { <input type="hidden" name="richtung" value=v/> })}
                        }
                    }}
                    <button type="submit" class="btn">"Suchen"</button>
                </div>
            </Form>

            // AK-9: Anlegen im Modal-Fenster (?neu=1)
            {move || create_open().then(|| view! { <UserCreateModal/> })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(rows) => view! { <UserTable rows state=state.get()/> }.into_any(),
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
fn UserTable(rows: Vec<UserRow>, state: ListState) -> impl IntoView {
    // AK-5: Hinweis statt leerer Tabelle
    if rows.is_empty() {
        let filtered = !state.search.is_empty() || state.status != "alle";
        let text = if filtered {
            "Keine Benutzer zu dieser Suche gefunden."
        } else {
            "Noch keine Benutzer erfasst."
        };
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">{text}</div>
            </div>
        }
        .into_any();
    }

    let navigate = use_navigate();
    let count = rows.len();
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

    let body = rows
        .into_iter()
        .map(|row| {
            let href = user_url(row.id);
            let nav = navigate.clone();
            let target = href.clone();
            view! {
                // AK-6: Klick auf die Zeile öffnet die Detailansicht
                <tr class="cursor-pointer hover:bg-base-200" on:click=move |_| nav(&target, Default::default())>
                    <td class="whitespace-nowrap font-medium">
                        <a href=href.clone() class="link link-hover">{row.benutzername}</a>
                    </td>
                    <td><a href=href class="link link-hover">{row.name}</a></td>
                    <td>{row.email}</td>
                    <td><span class=status_badge(row.aktiv)>{status_label(row.aktiv)}</span></td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>{headers}</tr>
                </thead>
                <tbody>{body}</tbody>
            </table>
        </div>
        <p class="text-sm text-base-content/70">
            {if count == 1 { "1 Benutzer".to_string() } else { format!("{count} Benutzer") }}
        </p>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ListState {
        ListState {
            search: String::new(),
            status: "alle".into(),
            sort: "benutzername".into(),
            dir: "auf".into(),
        }
    }

    #[test]
    fn default_href_is_plain() {
        assert_eq!(state().href(), "/einstellungen/benutzer");
    }

    #[test]
    fn filter_search_and_sorting_in_href() {
        let s = ListState {
            search: "Anna Muster".into(),
            status: "gesperrt".into(),
            ..state()
        };
        assert_eq!(
            s.href(),
            "/einstellungen/benutzer?suche=Anna%20Muster&status=gesperrt"
        );
        let by_name = state().sorted_by("name");
        assert_eq!(by_name.href(), "/einstellungen/benutzer?sortierung=name");
        assert_eq!(
            by_name.sorted_by("name").href(),
            "/einstellungen/benutzer?sortierung=name&richtung=ab"
        );
        assert_eq!(
            state().sorted_by("benutzername").href(),
            "/einstellungen/benutzer?richtung=ab"
        );
    }
}
