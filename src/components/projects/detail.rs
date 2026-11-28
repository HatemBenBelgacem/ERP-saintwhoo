//! Detailansichten mit Strukturbaum, Fortschritt und Löschen (AK-4 bis AK-6, AK-12 bis AK-15, AK-17).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use super::form::ProjectFormHost;
use crate::backend::projects::model::{
    Level, Node, ProjectTree, Responsible, ResponsibleEntry, descendant_summary, format_date,
    status_badge, status_label, task_stats,
};
use crate::backend::projects::server_fns::{DeleteElement, MoveElement, ToggleTask, get_tree};
use crate::components::inquiries::VorgangInquiries;
use crate::components::purchase_orders::{OrderFromVorgangHost, ProjectOrders, VorgangOrders};
use crate::forms::{Breadcrumbs, data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Offen-Zustand des Löschdialogs (Kopf-Schaltfläche ↔ Dialog).
#[derive(Clone, Copy)]
struct DeleteOpen(RwSignal<bool>);

/// Aktionen im Strukturbaum; nach jeder Aktion wird die Struktur neu geladen.
#[derive(Clone, Copy)]
struct TreeActions {
    toggle: ServerAction<ToggleTask>,
    mv: ServerAction<MoveElement>,
}

/// Lädt die Struktur zum Element in der Webadresse (`:id`) und lädt nach Baum-Aktionen neu.
fn use_tree(
    level: Level,
) -> (
    Resource<Result<Option<ProjectTree>, ServerFnError>>,
    TreeActions,
) {
    let params = use_params_map();
    let actions = TreeActions {
        toggle: ServerAction::<ToggleTask>::new(),
        mv: ServerAction::<MoveElement>::new(),
    };
    provide_context(actions);
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let tree = Resource::new(
        move || {
            (
                id(),
                actions.toggle.version().get(),
                actions.mv.version().get(),
                data_version(),
            )
        },
        move |(id, _, _, _)| async move {
            match id {
                Some(id) => get_tree(level, id).await,
                None => Ok(None),
            }
        },
    );
    (tree, actions)
}

#[component]
pub fn NotFound(level: Level) -> impl IntoView {
    let text = format!("{} nicht gefunden", level.label());
    view! {
        <Title text=format!("{text} – uniqwork")/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">{text}</h1>
                <p class="text-base-content/70">"Dieses Element gibt es nicht oder es wurde gelöscht."</p>
                <div class="card-actions">
                    <a href="/projekte" class="btn">"Zur Projektliste"</a>
                </div>
            </div>
        </div>
    }
}

/// Meldungen nach Speichern oder Löschen eines Kindelements.
#[component]
fn Notices() -> impl IntoView {
    let query = use_query_map();
    view! {
        {move || query.read().get("gespeichert").map(|_| view! {
            <div role="alert" class="alert alert-success">"Gespeichert."</div>
        })}
        {move || query.read().get("geloescht").map(|name| view! {
            <div role="alert" class="alert alert-success">"„" {name} "“ wurde gelöscht."</div>
        })}
    }
}

/// Fehler aus Baum-Aktionen (Abhaken, Verschieben).
#[component]
fn ActionErrors() -> impl IntoView {
    let actions = expect_context::<TreeActions>();
    move || {
        let e = actions
            .toggle
            .value()
            .get()
            .and_then(|r| r.err())
            .or_else(|| actions.mv.value().get().and_then(|r| r.err()));
        e.map(|e| view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> })
    }
}

#[component]
pub fn ProjectDetailPage() -> impl IntoView {
    let (tree, _) = use_tree(Level::Project);
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match tree.await {
                    Ok(Some(t)) => view! { <ProjectView tree=t/> }.into_any(),
                    Ok(None) => view! { <NotFound level=Level::Project/> }.into_any(),
                    Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                }
            })}
        </Transition>
    }
}

#[component]
fn ProjectView(tree: ProjectTree) -> impl IntoView {
    let p = tree.project.clone();
    let (done, total) = task_stats(&tree.children);
    let summary = descendant_summary(&tree.children);
    let scope = if summary.is_empty() {
        "–".to_string()
    } else {
        summary.clone()
    };
    let crumbs = tree.breadcrumbs(&[], false);
    let responsible = tree.responsible_for(&[]);
    provide_context(DeleteOpen(RwSignal::new(false)));

    view! {
        <Title text=format!("{} {} – uniqwork", p.nummer, p.name)/>
        <div class="space-y-4">
            <Breadcrumbs items=crumbs/>
            <Notices/>
            <ProjectFormHost/>
            <ActionErrors/>
            <Header
                title=p.name.clone()
                subtitle=p.nummer.clone()
                status=p.status.clone()
                level=Level::Project
                id=p.id
            />
            <div class="grid gap-4 md:grid-cols-3">
                <InfoCard>
                    <Info label="Kunde">
                        {match (p.adresse_id, p.kunde.clone()) {
                            (Some(id), Some(name)) => view! {
                                <a href=format!("/adressen/{id}") class="link link-primary">{name}</a>
                            }.into_any(),
                            _ => view! { "–" }.into_any(),
                        }}
                    </Info>
                    <Info label="Verantwortlich"><ResponsibleList entries=responsible/></Info>
                    <Info label="Start">{format_date(p.start.as_deref())}</Info>
                    <Info label="Ende">{format_date(p.ende.as_deref())}</Info>
                </InfoCard>
                <InfoCard>
                    <Info label="Fortschritt"><Progress done total/></Info>
                    <Info label="Umfang">{scope.clone()}</Info>
                </InfoCard>
                <InfoCard>
                    <Info label="Beschreibung">
                        <span class="whitespace-pre-line">{p.beschreibung.clone().unwrap_or_else(|| "–".into())}</span>
                    </Info>
                </InfoCard>
            </div>
            <Structure level=Level::Project id=p.id nodes=tree.children.clone()/>
            <ProjectOrders project_id=p.id/>
            <p class="text-sm text-base-content/60">
                "Erstellt am " {p.erstellt_am.clone()} " · Geändert am " {p.geaendert_am.clone()}
            </p>
        </div>
        <DeleteDialog level=Level::Project id=p.id title=format!("{} {}", p.nummer, p.name) summary/>
    }
}

/// Detailansicht für Teilprojekt, Vorgang und Aufgabe.
#[component]
pub fn ElementDetailPage(level: Level) -> impl IntoView {
    let (tree, _) = use_tree(level);
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                let result = tree.await;
                let found = match &result {
                    Ok(Some(t)) => id().and_then(|id| {
                        t.find_path(level, id).map(|path| {
                            let crumbs = t.breadcrumbs(&path, false);
                            // ANF-005 AK-28: Verantwortliche aus der Hierarchie
                            let responsible = t.responsible_for(&path);
                            (crumbs, responsible, path.last().map(|n| (*n).clone()))
                        })
                    }),
                    _ => None,
                };
                match (result, found) {
                    (_, Some((crumbs, responsible, Some(node)))) => {
                        view! { <ElementView node crumbs responsible/> }.into_any()
                    }
                    (Err(e), _) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    _ => view! { <NotFound level/> }.into_any(),
                }
            })}
        </Transition>
    }
}

#[component]
fn ElementView(
    node: Node,
    crumbs: Vec<(String, Option<String>)>,
    responsible: Vec<ResponsibleEntry>,
) -> impl IntoView {
    let level = node.level;
    let (done, total) = node.task_stats();
    let summary = descendant_summary(&node.children);
    let scope = if summary.is_empty() {
        "–".to_string()
    } else {
        summary.clone()
    };
    let is_task = level == Level::Task;
    let title = node.title();
    let description = node.beschreibung.clone().unwrap_or_else(|| "–".into());
    provide_context(DeleteOpen(RwSignal::new(false)));

    view! {
        <Title text=format!("{} – uniqwork", node.title())/>
        <div class="space-y-4">
            <Breadcrumbs items=crumbs/>
            <Notices/>
            <ProjectFormHost/>
            <ActionErrors/>
            <Header
                title=node.title()
                subtitle=String::new()
                status=node.status.clone()
                level
                id=node.id
            />
            // Aufgaben haben nur Titel und Beschreibung; erledigt/offen lässt sich hier umschalten
            {if is_task {
                view! {
                    <TaskDone id=node.id done=node.status == "erledigt"/>
                    <InfoCard>
                        <Info label="Verantwortlich"><ResponsibleList entries=responsible.clone()/></Info>
                        <Info label="Beschreibung">
                            <span class="whitespace-pre-line">{description.clone()}</span>
                        </Info>
                    </InfoCard>
                }.into_any()
            } else {
                view! {
                    <div class="grid gap-4 md:grid-cols-3">
                        <InfoCard>
                            <Info label="Verantwortlich"><ResponsibleList entries=responsible.clone()/></Info>
                            <Info label="Start">{format_date(node.start.as_deref())}</Info>
                            <Info label=level.end_label()>{format_date(node.ende.as_deref())}</Info>
                        </InfoCard>
                        <InfoCard>
                            <Info label="Fortschritt"><Progress done total/></Info>
                            <Info label="Umfang">{scope.clone()}</Info>
                        </InfoCard>
                        <InfoCard>
                            <Info label="Beschreibung">
                                <span class="whitespace-pre-line">{description.clone()}</span>
                            </Info>
                        </InfoCard>
                    </div>
                }.into_any()
            }}
            {(!is_task).then(|| view! { <Structure level id=node.id nodes=node.children.clone()/> })}
            // ANF-029: Anfragen und Einkaufsaufträge aus dem Vorgang
            {(level == Level::Process).then(|| view! {
                <VorgangInquiries vorgang_id=node.id/>
                <VorgangOrders vorgang_id=node.id/>
                <OrderFromVorgangHost vorgang_id=node.id/>
            })}
            <p class="text-sm text-base-content/60">
                "Erstellt am " {node.erstellt_am.clone()} " · Geändert am " {node.geaendert_am.clone()}
            </p>
        </div>
        <DeleteDialog level id=node.id title summary/>
    }
}

/// Kopf mit Titel, Status und Schaltflächen „Bearbeiten“ / „Löschen“.
#[component]
fn Header(title: String, subtitle: String, status: String, level: Level, id: i64) -> impl IntoView {
    let open = expect_context::<DeleteOpen>().0;
    let base = use_base_href();
    let edit = move || edit_href(&base.get(), level, id);
    view! {
        <div class="flex flex-wrap items-start justify-between gap-2">
            <div>
                <h1 class="text-2xl font-semibold">{title}</h1>
                <p class="flex items-center gap-2 text-base-content/70">
                    <span class=level.badge_class()>{level.label()}</span>
                    {subtitle}
                    <span class=status_badge(&status)>{status_label(&status).to_string()}</span>
                </p>
            </div>
            <div class="flex flex-wrap gap-2">
                // ANF-027: Geländeplan gibt es nur für Projekte
                {(level == Level::Project).then(|| view! {
                    <a href=format!("/projekte/{id}/gelaendeplan") class="btn">
                        <IconView icon=Icon::Map/>
                        "Geländeplan"
                    </a>
                })}
                // ANF-021 AK-7: Projektauswertung mit Erlösen, Aufwand und offenen Einkaufsaufträgen
                {(level == Level::Project).then(|| view! {
                    <a href=format!("/finanzen/auswertungen?art=projekte&projekt_id={id}") class="btn">
                        <IconView icon=Icon::Coins/>
                        "Finanzauswertung"
                    </a>
                })}
                <a href=edit class="btn btn-primary">
                    <IconView icon=Icon::Pencil/>
                    "Bearbeiten"
                </a>
                <button
                    type="button"
                    class="btn btn-outline btn-error"
                    on:click=move |_| open.set(true)
                >
                    <IconView icon=Icon::Trash/>
                    "Löschen"
                </button>
            </div>
        </div>
    }
}

/// Erledigt/offen einer Aufgabe in der Detailansicht umschalten.
#[component]
fn TaskDone(id: i64, done: bool) -> impl IntoView {
    let actions = expect_context::<TreeActions>();
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <label class="flex cursor-pointer items-center gap-3">
                    <input
                        type="checkbox"
                        class="checkbox checkbox-primary"
                        checked=done
                        on:change=move |ev| {
                            actions.toggle.dispatch(ToggleTask { id, erledigt: event_target_checked(&ev) });
                        }
                    />
                    <span>"Erledigt"</span>
                </label>
            </div>
        </div>
    }
}

#[component]
fn InfoCard(children: Children) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow">
            <dl class="card-body gap-3">{children()}</dl>
        </div>
    }
}

#[component]
fn Info(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div>
            <dt class="text-sm text-base-content/60">{label}</dt>
            <dd>{children()}</dd>
        </div>
    }
}

/// Fortschrittsbalken: erledigte / alle Aufgaben (AK-5).
#[component]
fn Progress(done: usize, total: usize) -> impl IntoView {
    if total == 0 {
        return view! { <span class="text-base-content/50">"– (keine Aufgaben)"</span> }.into_any();
    }
    let percent = done * 100 / total;
    view! {
        <span class="flex items-center gap-2" title=format!("{done} von {total} Aufgaben erledigt")>
            <progress class="progress progress-primary w-24" value=done max=total></progress>
            <span class="text-sm whitespace-nowrap">{format!("{percent} % ({done}/{total})")}</span>
        </span>
    }
    .into_any()
}

/// Strukturbereich mit Schaltfläche für das nächste Kindelement (AK-8) und Baum (AK-4).
#[component]
fn Structure(level: Level, id: i64, nodes: Vec<Node>) -> impl IntoView {
    let child = level.child().unwrap_or(Level::Task);
    let base = use_base_href();
    let new_url = move || new_child_href(&base.get(), level, id).unwrap_or_default();
    let title = if level == Level::Project {
        "Struktur".to_string()
    } else {
        child.label_plural().to_string()
    };
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <h2 class="card-title text-base">{title}</h2>
                    <a href=new_url class="btn btn-sm btn-primary btn-outline">
                        <IconView icon=Icon::Plus/>
                        {child.new_label()}
                    </a>
                </div>
                {if nodes.is_empty() {
                    view! {
                        <p class="text-base-content/60">{format!("Noch keine {}.", child.label_plural())}</p>
                    }.into_any()
                } else {
                    tree_list(nodes)
                }}
            </div>
        </section>
    }
}

/// Aufklappbarer Baum (rekursiv). Gibt `AnyView` zurück, damit die Rekursion einen festen Typ hat.
fn tree_list(nodes: Vec<Node>) -> AnyView {
    let len = nodes.len();
    view! {
        <ul class="space-y-1">
            {nodes
                .into_iter()
                .enumerate()
                .map(|(i, n)| tree_item(n, i == 0, i + 1 == len))
                .collect_view()}
        </ul>
    }
    .into_any()
}

fn tree_item(node: Node, first: bool, last: bool) -> AnyView {
    let row = view! { <TreeRow node=node.clone() first last/> };
    if node.level == Level::Task {
        return view! { <li class="ml-6">{row}</li> }.into_any();
    }
    let children = node.children.clone();
    let empty = format!(
        "Noch keine {}.",
        node.level.child().map_or("", Level::label_plural)
    );
    view! {
        <li>
            <details open class="group">
                <summary class="flex cursor-pointer list-none items-center gap-1 rounded-field hover:bg-base-200 [&::-webkit-details-marker]:hidden">
                    <span class="inline-block text-base-content/50 transition-transform group-open:rotate-90" aria-hidden="true"><IconView icon=Icon::Chevron/></span>
                    <div class="min-w-0 flex-1">{row}</div>
                </summary>
                <div class="ml-5 border-l border-base-300 pl-2">
                    {if children.is_empty() {
                        view! { <p class="py-1 pl-6 text-sm text-base-content/50">{empty}</p> }.into_any()
                    } else {
                        tree_list(children)
                    }}
                </div>
            </details>
        </li>
    }
    .into_any()
}

/// Eine Zeile im Baum: Nummer, Name, Status, Umfang/Fortschritt, Verschieben (AK-5, AK-12, AK-13).
#[component]
fn TreeRow(node: Node, first: bool, last: bool) -> impl IntoView {
    let actions = expect_context::<TreeActions>();
    let id = node.id;
    let level = node.level;
    let is_task = level == Level::Task;
    let done = node.status == "erledigt";
    let (d, t) = node.task_stats();

    view! {
        <div class=format!(
            "flex flex-wrap items-center gap-x-3 gap-y-1 rounded-r-field py-1 pr-1 pl-3 {} {}",
            level.stripe_class(),
            level.tint_class(),
        )>
            {is_task.then(|| view! {
                <input
                    type="checkbox"
                    class="checkbox checkbox-primary checkbox-sm"
                    checked=done
                    aria-label=format!("{} erledigt", node.title())
                    on:change=move |ev| {
                        actions.toggle.dispatch(ToggleTask { id, erledigt: event_target_checked(&ev) });
                    }
                />
            })}
            <span class=level.badge_class()>{level.label()}</span>
            <span class="font-mono text-sm text-base-content/60">{node.number.clone()}</span>
            <a
                href=level.url(id)
                class="link link-hover min-w-0 flex-1 truncate font-medium"
                class:line-through=is_task && done
                class:opacity-60=is_task && done
            >
                {node.name.clone()}
            </a>
            {node.ende.clone().map(|e| view! {
                <span class="text-sm text-base-content/60 whitespace-nowrap">"bis " {format_date(Some(&e))}</span>
            })}
            // ANF-005 AK-30: nur der hier hinterlegte Benutzer
            {node.responsible.clone().map(|r| view! { <ResponsibleBadge user=r/> })}
            <span class=status_badge(&node.status)>{status_label(&node.status).to_string()}</span>
            {(!is_task).then(|| view! {
                <span class="text-sm text-base-content/60 whitespace-nowrap" title="erledigte / alle Aufgaben">
                    {(d * 100).checked_div(t).map_or_else(|| "–".to_string(), |p| format!("{p} % ({d}/{t})"))}
                </span>
            })}
            <RowActions level id/>
            <span class="join">
                <button
                    type="button"
                    class="btn btn-ghost btn-xs join-item"
                    aria-label="Nach oben"
                    title="Nach oben"
                    disabled=first
                    on:click=move |ev| {
                        ev.prevent_default();
                        actions.mv.dispatch(MoveElement { level, id, nach_oben: true });
                    }
                >"▲"</button>
                <button
                    type="button"
                    class="btn btn-ghost btn-xs join-item"
                    aria-label="Nach unten"
                    title="Nach unten"
                    disabled=last
                    on:click=move |ev| {
                        ev.prevent_default();
                        actions.mv.dispatch(MoveElement { level, id, nach_oben: false });
                    }
                >"▼"</button>
            </span>
        </div>
    }
}

/// Alle Verantwortlichen mit Herkunft (ANF-005 AK-28), gesperrte markiert (AK-33).
#[component]
fn ResponsibleList(entries: Vec<ResponsibleEntry>) -> impl IntoView {
    if entries.is_empty() {
        return view! { "–" }.into_any();
    }
    view! {
        <ul class="space-y-1">
            {entries
                .into_iter()
                .map(|e| view! {
                    <li class="flex flex-wrap items-center gap-x-2">
                        <a href=e.user.url() class="link link-primary">{e.user.display()}</a>
                        {(!e.user.active).then(|| view! { <span class="badge badge-sm badge-error badge-soft">"gesperrt"</span> })}
                        <span class="text-sm text-base-content/60">"– " {e.origin}</span>
                    </li>
                })
                .collect_view()}
        </ul>
    }
    .into_any()
}

/// Kleine Plakette mit dem hinterlegten verantwortlichen Benutzer (Strukturbaum und Übersicht, AK-30).
#[component]
pub(super) fn ResponsibleBadge(user: Responsible) -> impl IntoView {
    let title = if user.active {
        format!("Verantwortlich: {}", user.display())
    } else {
        format!("Verantwortlich: {} – gesperrt", user.display())
    };
    view! {
        <span
            class="badge badge-sm badge-outline whitespace-nowrap"
            class:badge-error=!user.active
            title=title
        >
            {user.name.clone()}
            {(!user.active).then_some(" (gesperrt)")}
        </span>
    }
}

/// Webadresse, die das Modal-Fenster „Bearbeiten“ über der aktuellen Seite öffnet.
pub(super) fn edit_href(base: &str, level: Level, id: i64) -> String {
    with_params(
        base,
        &[("bearbeiten", level.key().into()), ("id", id.to_string())],
    )
}

/// Webadresse, die das Modal-Fenster „Neues Kindelement“ über der aktuellen Seite öffnet.
pub(super) fn new_child_href(base: &str, level: Level, id: i64) -> Option<String> {
    let child = level.child()?;
    Some(with_params(
        base,
        &[("neu", child.key().into()), ("unter", id.to_string())],
    ))
}

/// Kleine Schaltflächen in einer Baumzeile: Unterelement anlegen (+) und Bearbeiten (✎) im Modal-Fenster.
#[component]
pub(super) fn RowActions(level: Level, id: i64) -> impl IntoView {
    let base = use_base_href();
    let child = level.child();
    view! {
        <span class="flex items-center">
            {child.map(|c| view! {
                <a
                    href=move || new_child_href(&base.get(), level, id).unwrap_or_default()
                    class="btn btn-ghost btn-xs"
                    title=c.new_label()
                    aria-label=c.new_label()
                    on:click=|ev| ev.stop_propagation()
                >
                    <IconView icon=Icon::Plus/>
                </a>
            })}
            <a
                href=move || edit_href(&base.get(), level, id)
                class="btn btn-ghost btn-xs"
                title="Bearbeiten"
                aria-label="Bearbeiten"
                on:click=|ev| ev.stop_propagation()
            >
                <IconView icon=Icon::Pencil/>
            </a>
        </span>
    }
}

/// Bestätigungsdialog mit Anzahl der mitgelöschten Elemente (AK-14, AK-15).
#[component]
fn DeleteDialog(level: Level, id: i64, title: String, summary: String) -> impl IntoView {
    let open = expect_context::<DeleteOpen>().0;
    let delete = ServerAction::<DeleteElement>::new();
    let error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    let also = if summary.is_empty() {
        String::new()
    } else {
        format!(" Dabei werden auch {summary} gelöscht.")
    };

    view! {
        <div class="modal max-sm:modal-bottom" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">{format!("{} löschen?", level.label())}</h3>
                <p class="py-4">
                    {format!("Soll „{title}“ wirklich gelöscht werden?{also} Das kann nicht rückgängig gemacht werden.")}
                </p>
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| open.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
                        <input type="hidden" name="level" value=level.key()/>
                        <input type="hidden" name="id" value=id/>
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
