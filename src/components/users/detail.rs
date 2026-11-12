//! Detailansicht eines Benutzers mit Verantwortung und Löschen (ANF-005 AK-6 bis AK-8, AK-18, AK-22, AK-31).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use super::form::{PasswordModal, UserEditModal};
use crate::backend::projects::model::Level;
use crate::backend::tasks::model::{TaskItem, user_tasks_url};
use crate::backend::users::model::{
    AssignedElement, Responsibility, User, UserDetail, status_badge, status_label,
};
use crate::backend::users::server_fns::{DeleteUser, get_user};
use crate::forms::{Breadcrumbs, data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

#[component]
pub fn UserDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let data = Resource::new(
        move || (id(), data_version()),
        |(id, _)| async move {
            match id {
                Some(id) => get_user(id).await,
                None => Ok(None),
            }
        },
    );

    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(detail)) => view! { <UserView detail/> }.into_any(),
                    Ok(None) => view! { <NotFound/> }.into_any(),
                    Err(e) => view! {
                        <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                    }
                        .into_any(),
                }
            })}
        </Transition>
    }
}

/// AK-22
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="Benutzer nicht gefunden – ERP-saintwhoo"/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">"Benutzer nicht gefunden"</h1>
                <p class="text-base-content/70">"Diesen Benutzer gibt es nicht oder er wurde gelöscht."</p>
                <div class="card-actions">
                    <a href="/einstellungen/benutzer" class="btn">"Zur Benutzerliste"</a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn UserView(detail: UserDetail) -> impl IntoView {
    let UserDetail {
        user,
        responsibility,
    } = detail;
    let query = use_query_map();
    let base = use_base_href();
    let confirm_open = RwSignal::new(false);
    let edit_open = move || query.read().get("bearbeiten").is_some();
    let password_open = move || query.read().get("passwort").is_some();
    let saved = move || query.read().get("gespeichert");
    let edit_user = user.clone();
    let password_user = user.clone();
    let u = user.clone();
    let address_user = user.clone();
    let last_login = user.letzte_anmeldung.clone();

    view! {
        <Title text=format!("{} – Benutzer – ERP-saintwhoo", user.benutzername)/>
        <div class="space-y-4">
            <Breadcrumbs items=vec![
                ("Einstellungen".to_string(), Some("/einstellungen".to_string())),
                ("Benutzer".to_string(), Some("/einstellungen/benutzer".to_string())),
                (user.benutzername.clone(), None),
            ]/>

            {move || saved().map(|what| view! {
                <div role="alert" class="alert alert-success">
                    {if what == "passwort" { "Passwort gesetzt." } else { "Benutzer gespeichert." }}
                </div>
            })}

            <div class="flex flex-wrap items-start justify-between gap-2">
                <div>
                    <h1 class="text-2xl font-semibold">{user.name.clone()}</h1>
                    <p class="flex items-center gap-2 text-base-content/70">
                        {user.benutzername.clone()}
                        <span class=status_badge(user.aktiv)>{status_label(user.aktiv)}</span>
                    </p>
                </div>
                <div class="flex flex-wrap gap-2">
                    <a href=move || with_params(&base.get(), &[("bearbeiten", "1".into())]) class="btn btn-primary">
                        <IconView icon=Icon::Pencil/>
                        "Bearbeiten"
                    </a>
                    <a href=move || with_params(&base.get(), &[("passwort", "1".into())]) class="btn btn-outline">
                        "Passwort setzen"
                    </a>
                    <button type="button" class="btn btn-outline btn-error" on:click=move |_| confirm_open.set(true)>
                        <IconView icon=Icon::Trash/>
                        "Löschen"
                    </button>
                </div>
            </div>

            <div class="grid gap-4 md:grid-cols-2">
                <Section title="Zugang">
                    <Field label="Benutzername" value=Some(u.benutzername.clone())/>
                    <Field label="Status" value=Some(status_label(u.aktiv).to_string())/>
                    <Field label="Letzte Anmeldung" value=last_login/>
                </Section>
                <AddressSection user=address_user/>
            </div>

            <ResponsibilitySection responsibility user_id=u.id/>

            <p class="text-sm text-base-content/60">
                "Erstellt am " {u.erstellt_am.clone()} " · Geändert am " {u.geaendert_am.clone()}
            </p>
        </div>

        // AK-14, AK-15: Bearbeiten und Passwort im Modal-Fenster
        {move || edit_open().then(|| view! { <UserEditModal user=edit_user.clone()/> })}
        {move || password_open().then(|| view! { <PasswordModal user=password_user.clone()/> })}

        <DeleteDialog user open=confirm_open/>
    }
}

/// Angaben der verknüpften Adresse (AK-6, AK-7).
#[component]
fn AddressSection(user: User) -> impl IntoView {
    view! {
        <Section title="Adresse">
            <div>
                <dt class="text-sm text-base-content/60">"Name"</dt>
                <dd>
                    <a href=format!("/adressen/{}", user.adresse_id) class="link link-primary">{user.name.clone()}</a>
                    <span class="text-base-content/60">" · " {user.adresse_nummer.clone()}</span>
                </dd>
            </div>
            <div>
                <dt class="text-sm text-base-content/60">"E-Mail"</dt>
                <dd>
                    {match user.email.clone() {
                        Some(m) => view! { <a href=format!("mailto:{m}") class="link link-primary">{m.clone()}</a> }.into_any(),
                        None => view! { "–" }.into_any(),
                    }}
                </dd>
            </div>
            <Field label="Telefon" value=user.telefon.clone()/>
            <Field label="Mobiltelefon" value=user.mobiltelefon.clone()/>
        </Section>
    }
}

/// Wo der Benutzer hinterlegt ist und für welche offenen Aufgaben er verantwortlich ist (AK-31).
#[component]
fn ResponsibilitySection(responsibility: Responsibility, user_id: i64) -> impl IntoView {
    let Responsibility {
        assigned,
        open_tasks,
    } = responsibility;
    let task_count = open_tasks.len();

    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Verantwortung"</h2>
                {if assigned.is_empty() {
                    view! {
                        <p class="text-base-content/60">
                            "Bei keinem Projekt, Teilprojekt oder Vorgang als verantwortlich hinterlegt."
                        </p>
                    }
                    .into_any()
                } else {
                    view! { <AssignedTable rows=assigned/> }.into_any()
                }}

                <div class="divider my-0"></div>
                // ANF-006 AK-18: Verweis auf die Seite „Aufgaben“ dieses Benutzers
                <a href=user_tasks_url(user_id) class="link link-primary text-sm">"Alle Aufgaben dieses Benutzers anzeigen"</a>
                {if task_count == 0 {
                    view! { <p class="text-base-content/60">"Keine offenen Aufgaben."</p> }.into_any()
                } else {
                    view! {
                        <details class="group">
                            <summary class="flex cursor-pointer list-none items-center gap-1 font-medium [&::-webkit-details-marker]:hidden">
                                <span class="inline-block text-base-content/50 transition-transform group-open:rotate-90" aria-hidden="true">
                                    <IconView icon=Icon::Chevron/>
                                </span>
                                {format!("Offene Aufgaben: {task_count}")}
                                <span class="text-sm font-normal text-base-content/60"> " (Projektaufgaben hinterlegt oder geerbt, Aufgaben ohne Projekt)"</span>
                            </summary>
                            <TaskTable rows=open_tasks/>
                        </details>
                    }
                    .into_any()
                }}
            </div>
        </section>
    }
}

#[component]
fn AssignedTable(rows: Vec<AssignedElement>) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="table table-sm">
                <thead>
                    <tr><th>"Ebene"</th><th>"Nummer"</th><th>"Name"</th><th>"Projekt"</th></tr>
                </thead>
                <tbody>
                    {rows
                        .into_iter()
                        .map(|r| view! {
                            <tr>
                                <td><span class=r.level.badge_class()>{r.level.label()}</span></td>
                                <td class="whitespace-nowrap font-mono text-base-content/70">{r.nummer.clone()}</td>
                                <td><a href=r.level.url(r.id) class="link link-hover font-medium">{r.name.clone()}</a></td>
                                <td>
                                    <a href=Level::Project.url(r.projekt_id) class="link link-hover">
                                        {format!("{} {}", r.projekt_nummer, r.projekt_name)}
                                    </a>
                                </td>
                            </tr>
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

#[component]
fn TaskTable(rows: Vec<TaskItem>) -> impl IntoView {
    view! {
        <div class="mt-2 overflow-x-auto">
            <table class="table table-sm">
                <thead>
                    <tr><th>"Titel"</th><th>"Projekt"</th><th>"Nummer"</th><th>"Herkunft"</th></tr>
                </thead>
                <tbody>
                    {rows
                        .into_iter()
                        .map(|t| view! {
                            <tr>
                                <td><a href=t.url() class="link link-hover">{t.name.clone()}</a></td>
                                <td>
                                    {match t.projekt_id {
                                        Some(pid) => view! {
                                            <a href=Level::Project.url(pid) class="link link-hover">{t.project_label()}</a>
                                        }.into_any(),
                                        None => view! { <span class="badge badge-sm badge-ghost">"ohne Projekt"</span> }.into_any(),
                                    }}
                                </td>
                                <td class="whitespace-nowrap font-mono text-base-content/70">{t.nummer.clone()}</td>
                                <td class="text-sm text-base-content/70">{t.herkunft.clone()}</td>
                            </tr>
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// Bestätigungsdialog (AK-18). Ist der Benutzer noch in Projekten hinterlegt, meldet der Server das (AK-32).
#[component]
fn DeleteDialog(user: User, open: RwSignal<bool>) -> impl IntoView {
    let delete = ServerAction::<DeleteUser>::new();
    let error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    view! {
        <div class="modal" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Benutzer löschen?"</h3>
                <p class="py-4">
                    {format!(
                        "Soll der Benutzer „{}“ ({}) wirklich gelöscht werden? Die Adresse bleibt erhalten. \
                         Das kann nicht rückgängig gemacht werden.",
                        user.benutzername, user.name
                    )}
                </p>
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| open.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
                        <input type="hidden" name="id" value=user.id/>
                        <button type="submit" class="btn btn-error" disabled=move || delete.pending().get()>
                            "Endgültig löschen"
                        </button>
                    </ActionForm>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| open.set(false)></div>
        </div>
    }
}

#[component]
fn Section(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h2 class="card-title text-base">{title}</h2>
                <dl class="space-y-2">{children()}</dl>
            </div>
        </div>
    }
}

#[component]
fn Field(label: &'static str, value: Option<String>) -> impl IntoView {
    view! {
        <div>
            <dt class="text-sm text-base-content/60">{label}</dt>
            <dd>{value.unwrap_or_else(|| "–".into())}</dd>
        </div>
    }
}
