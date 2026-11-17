//! Detailansicht einer Aufgabe ohne Projekt mit Bearbeiten und Löschen (ANF-006 AK-14 bis AK-17).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use super::form::TaskEditModal;
use crate::backend::projects::model::{Level, status_badge, status_label};
use crate::backend::projects::server_fns::ToggleTask;
use crate::backend::tasks::model::{BASE, StandaloneTask, TaskLookup, user_tasks_url};
use crate::backend::tasks::server_fns::{DeleteTask, get_task};
use crate::forms::{Breadcrumbs, data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

#[component]
pub fn TaskDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let toggle = ServerAction::<ToggleTask>::new();
    let data = Resource::new(
        move || (id(), toggle.version().get(), data_version()),
        |(id, _, _)| async move {
            match id {
                Some(id) => get_task(id).await,
                None => Ok(TaskLookup::Missing),
            }
        },
    );

    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(TaskLookup::Found(task)) => view! { <TaskView task toggle/> }.into_any(),
                    Ok(TaskLookup::InProject(id)) => view! { <InProject id/> }.into_any(),
                    Ok(TaskLookup::Missing) => view! { <NotFound/> }.into_any(),
                    Err(e) => view! {
                        <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                    }
                        .into_any(),
                }
            })}
        </Transition>
    }
}

/// AK-17
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="Aufgabe nicht gefunden – ERP-saintwhoo"/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">"Aufgabe nicht gefunden"</h1>
                <p class="text-base-content/70">"Diese Aufgabe gibt es nicht oder sie wurde gelöscht."</p>
                <div class="card-actions">
                    <a href=BASE class="btn">"Zu den Aufgaben"</a>
                </div>
            </div>
        </div>
    }
}

/// AK-17: Projektaufgaben werden im Projekt angezeigt.
#[component]
fn InProject(id: i64) -> impl IntoView {
    view! {
        <Title text="Projektaufgabe – ERP-saintwhoo"/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">"Diese Aufgabe gehört zu einem Projekt"</h1>
                <p class="text-base-content/70">"Projektaufgaben werden im Projekt angezeigt und bearbeitet."</p>
                <div class="card-actions">
                    <a href=Level::Task.url(id) class="btn btn-primary">"Im Projekt öffnen"</a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn TaskView(task: StandaloneTask, toggle: ServerAction<ToggleTask>) -> impl IntoView {
    let query = use_query_map();
    let base = use_base_href();
    let confirm_open = RwSignal::new(false);
    let edit_open = move || query.read().get("bearbeiten").is_some();
    let saved = move || query.read().get("gespeichert").is_some();
    let assignee = task.assignee();
    let done = task.status == "erledigt";
    let id = task.id;
    let edit_task = task.clone();
    let delete = ServerAction::<DeleteTask>::new();
    let delete_error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    let toggle_error = move || {
        toggle
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };

    view! {
        <Title text=format!("{} – Aufgaben – ERP-saintwhoo", task.name)/>
        <div class="space-y-4">
            <Breadcrumbs items=vec![
                ("Aufgaben".to_string(), Some(user_tasks_url(assignee.id))),
                (task.name.clone(), None),
            ]/>
            {move || saved().then(|| view! {
                <div role="alert" class="alert alert-success">"Aufgabe gespeichert."</div>
            })}
            {move || toggle_error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}

            <div class="flex flex-wrap items-start justify-between gap-2">
                <div>
                    <h1 class="text-2xl font-semibold">{task.name.clone()}</h1>
                    <p class="flex items-center gap-2 text-base-content/70">
                        <span class="badge badge-sm badge-secondary badge-soft">"ohne Projekt"</span>
                        <span class=status_badge(&task.status)>{status_label(&task.status).to_string()}</span>
                    </p>
                </div>
                <div class="flex gap-2">
                    <a href=move || with_params(&base.get(), &[("bearbeiten", "1".into())]) class="btn btn-primary">
                        <IconView icon=Icon::Pencil/>
                        "Bearbeiten"
                    </a>
                    <button type="button" class="btn btn-outline btn-error" on:click=move |_| confirm_open.set(true)>
                        <IconView icon=Icon::Trash/>
                        "Löschen"
                    </button>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <label class="flex cursor-pointer items-center gap-3">
                        <input
                            type="checkbox"
                            class="checkbox checkbox-primary"
                            checked=done
                            on:change=move |ev| {
                                toggle.dispatch(ToggleTask { id, erledigt: event_target_checked(&ev) });
                            }
                        />
                        <span>"Erledigt"</span>
                    </label>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <dl class="card-body gap-3">
                    <div>
                        <dt class="text-sm text-base-content/60">"Zuständig"</dt>
                        <dd class="flex flex-wrap items-center gap-2">
                            <a href=assignee.url() class="link link-primary">{assignee.display()}</a>
                            {(!assignee.active).then(|| view! {
                                <span class="badge badge-sm badge-error badge-soft">"gesperrt"</span>
                            })}
                        </dd>
                    </div>
                    <div>
                        <dt class="text-sm text-base-content/60">"Beschreibung"</dt>
                        <dd class="whitespace-pre-line">{task.beschreibung.clone().unwrap_or_else(|| "–".into())}</dd>
                    </div>
                </dl>
            </div>

            <p class="text-sm text-base-content/60">
                "Erstellt am " {task.erstellt_am.clone()} " · Geändert am " {task.geaendert_am.clone()}
            </p>
        </div>

        // AK-15: Bearbeiten im Modal-Fenster (?bearbeiten=1)
        {move || edit_open().then(|| view! { <TaskEditModal task=edit_task.clone()/> })}

        // AK-16: Bestätigungsdialog
        <div class="modal" class:modal-open=move || confirm_open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Aufgabe löschen?"</h3>
                <p class="py-4">
                    {format!("Soll die Aufgabe „{}“ wirklich gelöscht werden? Das kann nicht rückgängig gemacht werden.", task.name)}
                </p>
                {move || delete_error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| confirm_open.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
                        <input type="hidden" name="id" value=id/>
                        <button type="submit" class="btn btn-error" disabled=move || delete.pending().get()>
                            "Endgültig löschen"
                        </button>
                    </ActionForm>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| confirm_open.set(false)></div>
        </div>
    }
}
