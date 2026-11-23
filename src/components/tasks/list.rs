//! Navigation „Aufgaben“: alle Aufgaben eines Benutzers mit Filtern (ANF-006 AK-1 bis AK-9).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::TaskCreateModal;
use crate::backend::auth::server_fns::get_current_user;
use crate::backend::projects::model::Level;
use crate::backend::projects::server_fns::ToggleTask;
use crate::backend::tasks::model::{BASE, KIND_FILTERS, STATUS_FILTERS, TaskItem};
use crate::backend::tasks::server_fns::list_tasks;
use crate::backend::users::server_fns::{get_user_option, search_active_users};
use crate::forms::{
    FieldErrors, PickOption, SearchPicker, data_version, encode, error_message, use_base_href,
    with_params,
};
use crate::icons::{Icon, IconView};

/// Zustand der Liste – kommt vollständig aus der Webadresse (AK-2, AK-5).
#[derive(Clone, Debug, PartialEq)]
struct ListState {
    user: Option<i64>,
    status: String,
    kind: String,
    search: String,
}

impl ListState {
    fn href(&self) -> String {
        let mut params = Vec::new();
        if let Some(u) = self.user {
            params.push(format!("benutzer={u}"));
        }
        if self.status != STATUS_FILTERS[0].0 {
            params.push(format!("status={}", encode(&self.status)));
        }
        if self.kind != KIND_FILTERS[0].0 {
            params.push(format!("art={}", encode(&self.kind)));
        }
        if !self.search.is_empty() {
            params.push(format!("suche={}", encode(&self.search)));
        }
        if params.is_empty() {
            BASE.into()
        } else {
            format!("{BASE}?{}", params.join("&"))
        }
    }

    fn with_user(&self, user: Option<i64>) -> Self {
        Self {
            user,
            ..self.clone()
        }
    }
}

/// Gewählter Benutzer aus der Webadresse, sonst der angemeldete Benutzer (ANF-007 AK-19).
async fn effective_user(selected: Option<i64>) -> Option<i64> {
    match selected {
        Some(id) => Some(id),
        None => get_current_user()
            .await
            .ok()
            .flatten()
            .and_then(|u| u.benutzer_id),
    }
}

#[component]
pub fn TaskListPage() -> impl IntoView {
    let query = use_query_map();
    let state = Memo::new(move |_| {
        let q = query.read();
        ListState {
            user: q.get("benutzer").and_then(|s| s.parse().ok()),
            status: q
                .get("status")
                .unwrap_or_else(|| STATUS_FILTERS[0].0.into()),
            kind: q.get("art").unwrap_or_else(|| KIND_FILTERS[0].0.into()),
            search: q.get("suche").unwrap_or_default(),
        }
    });
    let user_id = Memo::new(move |_| state.read().user);
    // Gewählter Benutzer als Anfangswert der Auswahl (auch gesperrte, AK-20)
    let user_option = Resource::new(
        move || user_id.get(),
        |id| async move {
            match effective_user(id).await {
                Some(id) => get_user_option(id).await,
                None => Ok(None),
            }
        },
    );
    // AK-8: Abhaken direkt in der Liste; danach wird neu geladen
    let toggle = ServerAction::<ToggleTask>::new();
    let data = Resource::new(
        move || (state.get(), toggle.version().get(), data_version()),
        |(s, _, _)| async move {
            match effective_user(s.user).await {
                Some(id) => list_tasks(id, s.status, s.kind, s.search).await.map(Some),
                None => Ok(None),
            }
        },
    );
    let base = use_base_href();
    let create_open = move || query.read().get("neu").is_some();
    let saved = move || query.read().get("gespeichert").is_some();
    let deleted = move || query.read().get("geloescht");
    let navigate = use_navigate();
    let choose_user = Callback::new(move |o: Option<PickOption>| {
        let href = state.get_untracked().with_user(o.map(|p| p.id)).href();
        navigate(&href, Default::default());
    });

    view! {
        <Title text="Aufgaben – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Aufgaben"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neue Aufgabe"
                </a>
            </div>

            {move || saved().then(|| view! {
                <div role="alert" class="alert alert-success">"Aufgabe gespeichert."</div>
            })}
            {move || deleted().map(|name| view! {
                <div role="alert" class="alert alert-success">"Aufgabe „" {name} "“ wurde gelöscht."</div>
            })}

            // AK-2: Benutzer-Auswahl (Standard: angemeldeter Benutzer, ANF-007 AK-19); AK-10: Modal-Fenster vorbelegt
            <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    let selected = user_option.await.ok().flatten();
                    let initial = selected.clone();
                    view! {
                        <div class="sm:w-md">
                            <SearchPicker
                                field="benutzer"
                                label="Aufgaben von"
                                placeholder="Aktiven Benutzer suchen (Name, Benutzername) …"
                                initial
                                errors=Signal::stored(FieldErrors::new())
                                search=search_active_users
                                on_change=choose_user
                            />
                        </div>
                        {move || create_open().then(|| view! { <TaskCreateModal user=selected.clone()/> })}
                    }
                })}
            </Suspense>

            <Filters state/>

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    let s = state.get();
                    match data.await {
                        Ok(Some(rows)) => view! { <TaskTable rows state=s toggle/> }.into_any(),
                        Ok(None) => view! {
                            <div class="card bg-base-100 shadow">
                                <div class="card-body items-center text-center text-base-content/70">
                                    "Bitte einen Benutzer wählen."
                                </div>
                            </div>
                        }
                            .into_any(),
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

/// Suche, Status und Art (AK-5). Der gewählte Benutzer bleibt erhalten.
#[component]
fn Filters(state: Memo<ListState>) -> impl IntoView {
    view! {
        <Form method="GET" action=BASE>
            <input type="hidden" name="benutzer" value=move || state.read().user.map(|u| u.to_string())/>
            <div class="flex flex-wrap gap-2">
                <label class="input w-full sm:w-md sm:max-w-md sm:flex-none">
                    <IconView icon=Icon::Search/>
                    <input
                        type="search"
                        name="suche"
                        placeholder="Titel, Beschreibung oder Projekt suchen …"
                        prop:value=move || state.read().search.clone()
                    />
                </label>
                <select
                    name="status"
                    class="select w-40"
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
                <select
                    name="art"
                    class="select w-44"
                    aria-label="Art"
                    onchange="this.form.requestSubmit()"
                    prop:value=move || state.read().kind.clone()
                >
                    {KIND_FILTERS
                        .iter()
                        .map(|(k, l)| view! {
                            <option value=*k selected=move || state.read().kind == *k>{*l}</option>
                        })
                        .collect_view()}
                </select>
                <button type="submit" class="btn">"Suchen"</button>
            </div>
        </Form>
    }
}

#[component]
fn TaskTable(
    rows: Vec<TaskItem>,
    state: ListState,
    toggle: ServerAction<ToggleTask>,
) -> impl IntoView {
    // AK-7: Hinweis statt leerer Tabelle
    if rows.is_empty() {
        let filtered = !state.search.is_empty() || state.kind != KIND_FILTERS[0].0;
        let text = match (filtered, state.status.as_str()) {
            (true, _) => "Keine Aufgaben zu diesen Filtern gefunden.",
            (false, "offen") => "Keine offenen Aufgaben.",
            (false, "erledigt") => "Keine erledigten Aufgaben.",
            _ => "Keine Aufgaben zugeordnet.",
        };
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">{text}</div>
            </div>
        }
        .into_any();
    }
    let count = rows.len();

    let body = rows
        .into_iter()
        .map(|t| {
            let id = t.id;
            let done = t.is_done();
            let standalone = t.is_standalone();
            let href = t.url();
            view! {
                <tr class=if standalone { "" } else { Level::Task.tint_class() }>
                    <td class="w-0">
                        <input
                            type="checkbox"
                            class="checkbox checkbox-primary checkbox-sm"
                            checked=done
                            aria-label=format!("{} erledigt", t.name)
                            on:change=move |ev| {
                                toggle.dispatch(ToggleTask { id, erledigt: event_target_checked(&ev) });
                            }
                        />
                    </td>
                    <td class="min-w-60">
                        <a
                            href=href
                            class="link link-hover font-medium"
                            class:line-through=done
                            class:opacity-60=done
                        >
                            {t.name.clone()}
                        </a>
                    </td>
                    <td class="whitespace-nowrap">
                        {match t.projekt_id {
                            Some(pid) => view! {
                                <a href=Level::Project.url(pid) class="link link-hover">{t.project_label()}</a>
                            }.into_any(),
                            // AK-4: Aufgaben ohne Projekt mit eigener Plakette
                            None => view! { <span class="badge badge-sm badge-secondary badge-soft">"ohne Projekt"</span> }.into_any(),
                        }}
                    </td>
                    <td class="whitespace-nowrap font-mono text-sm text-base-content/60">{t.nummer.clone()}</td>
                    <td class="whitespace-nowrap text-sm text-base-content/70">{t.herkunft.clone()}</td>
                    <td class="whitespace-nowrap text-sm">{t.geaendert_am.clone()}</td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        <th><span class="sr-only">"Erledigt"</span></th>
                        <th>"Titel"</th>
                        <th>"Projekt"</th>
                        <th>"Nummer"</th>
                        <th>"Herkunft"</th>
                        <th>"Geändert am"</th>
                    </tr>
                </thead>
                <tbody>{body}</tbody>
            </table>
        </div>
        <p class="text-sm text-base-content/70">
            {if count == 1 { "1 Aufgabe".to_string() } else { format!("{count} Aufgaben") }}
        </p>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> ListState {
        ListState {
            user: None,
            status: "offen".into(),
            kind: "alle".into(),
            search: String::new(),
        }
    }

    #[test]
    fn default_href_is_plain() {
        assert_eq!(state().href(), "/aufgaben");
    }

    #[test]
    fn href_keeps_filters_when_user_changes() {
        let s = ListState {
            status: "alle".into(),
            kind: "ohne".into(),
            search: "Offerte nachfassen".into(),
            ..state()
        };
        assert_eq!(
            s.with_user(Some(3)).href(),
            "/aufgaben?benutzer=3&status=alle&art=ohne&suche=Offerte%20nachfassen"
        );
        assert_eq!(
            s.with_user(None).href(),
            "/aufgaben?status=alle&art=ohne&suche=Offerte%20nachfassen"
        );
    }
}
