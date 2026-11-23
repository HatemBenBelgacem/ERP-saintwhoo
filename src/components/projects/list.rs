//! Projektliste mit Suche, Statusfilter, Sortierung und Seiten (AK-1 bis AK-3).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use std::collections::HashSet;

use super::detail::{ResponsibleBadge, RowActions};
use super::form::ProjectFormHost;
use crate::backend::projects::model::{
    Level, Node, PAGE_SIZE, PROJECT_STATUSES, ProjectPage, ProjectRow, SORT_COLUMNS, format_date,
    status_badge, status_label, task_stats,
};
use crate::backend::projects::server_fns::{ToggleTask, list_projects};
use crate::forms::{data_version, encode, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Standardfilter: nur aktive Projekte (AK-2). „alle“ = kein Filter.
const DEFAULT_STATUS: &str = "aktiv";

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
            "/projekte".into()
        } else {
            format!("/projekte?{}", params.join("&"))
        }
    }

    /// Klick auf Spaltenkopf: neue Spalte aufsteigend (Nummer absteigend), gleiche Spalte umkehren.
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

    /// Statusfilter für die Datenbank: leer = alle.
    fn status_filter(&self) -> String {
        if self.status == "alle" {
            String::new()
        } else {
            self.status.clone()
        }
    }
}

#[component]
pub fn ProjectListPage() -> impl IntoView {
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
    // Abhaken einer Aufgabe direkt in der Übersicht; danach wird neu geladen
    let toggle = ServerAction::<ToggleTask>::new();
    let data = Resource::new(
        move || (state.get(), toggle.version().get(), data_version()),
        |(s, _, _)| {
            let status = s.status_filter();
            list_projects(s.search, status, s.sort, s.dir, s.page)
        },
    );
    let deleted = move || query.read().get("geloescht");
    let saved = move || query.read().get("gespeichert").is_some();
    let base = use_base_href();
    // Zugeklappte Zeilen (Schlüssel „ebene-kennung“); bleibt beim Neuladen erhalten
    let collapsed = RwSignal::new(HashSet::<String>::new());

    view! {
        <Title text="Projekte – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Projekte"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "projekt".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neues Projekt"
                </a>
            </div>

            <ProjectFormHost/>
            {move || saved().then(|| view! {
                <div role="alert" class="alert alert-success">"Gespeichert."</div>
            })}
            {move || deleted().map(|n| view! {
                <div role="alert" class="alert alert-success">"Projekt " {n} " wurde gelöscht."</div>
            })}

            <Form method="GET" action="/projekte">
                <div class="flex flex-wrap gap-2">
                    <label class="input w-full sm:w-md sm:max-w-md sm:flex-none">
                        <IconView icon=Icon::Search/>
                        <input
                            type="search"
                            name="suche"
                            placeholder="Nummer, Name oder Kunde suchen …"
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
                        <option value="alle">"Alle Status"</option>
                        {PROJECT_STATUSES
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

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(page) => view! { <ProjectTable page state=state.get() collapsed toggle/> }.into_any(),
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

/// Eine Zeile der Übersicht: Projekt oder Element darunter.
#[derive(Clone)]
enum Item {
    Project(ProjectRow),
    Node(Node),
}

/// Flache Zeilenliste mit Einrückungstiefe und den Schlüsseln aller übergeordneten Zeilen.
struct FlatRow {
    item: Item,
    depth: usize,
    key: String,
    ancestors: Vec<String>,
    has_children: bool,
    /// (erledigte, alle) Aufgaben darunter; `None` bei Aufgaben
    stats: Option<(usize, usize)>,
}

fn row_key(level: Level, id: i64) -> String {
    format!("{}-{id}", level.key())
}

/// Projekte mit Struktur in Tabellenzeilen auflösen (Projekt, darunter eingerückt die Elemente).
fn flatten(rows: Vec<ProjectRow>) -> Vec<FlatRow> {
    fn walk(nodes: Vec<Node>, depth: usize, ancestors: &[String], out: &mut Vec<FlatRow>) {
        for mut n in nodes {
            let key = row_key(n.level, n.id);
            let stats = (n.level != Level::Task).then(|| task_stats(&n.children));
            let children = std::mem::take(&mut n.children);
            out.push(FlatRow {
                item: Item::Node(n),
                depth,
                key: key.clone(),
                ancestors: ancestors.to_vec(),
                has_children: !children.is_empty(),
                stats,
            });
            let mut next = ancestors.to_vec();
            next.push(key);
            walk(children, depth + 1, &next, out);
        }
    }
    let mut out = Vec::new();
    for mut p in rows {
        let key = row_key(Level::Project, p.id);
        let stats = Some(task_stats(&p.children));
        let children = std::mem::take(&mut p.children);
        out.push(FlatRow {
            item: Item::Project(p),
            depth: 0,
            key: key.clone(),
            ancestors: vec![],
            has_children: !children.is_empty(),
            stats,
        });
        walk(children, 1, &[key], &mut out);
    }
    out
}

#[component]
fn ProjectTable(
    page: ProjectPage,
    state: ListState,
    collapsed: RwSignal<HashSet<String>>,
    toggle: ServerAction<ToggleTask>,
) -> impl IntoView {
    if page.rows.is_empty() {
        let mut text = if state.search.is_empty() {
            "Keine Projekte".to_string()
        } else {
            format!("Keine Projekte zu „{}“", state.search)
        };
        if state.status != "alle" {
            text.push_str(&format!(" mit Status „{}“", status_label(&state.status)));
        }
        text.push_str(" gefunden.");
        let all = (state.status != "alle").then(|| {
            ListState {
                status: "alle".into(),
                page: 1,
                ..state.clone()
            }
            .href()
        });
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">
                    {text}
                    {all.map(|h| view! { <a href=h class="link link-primary">"Alle Status anzeigen"</a> })}
                </div>
            </div>
        }
        .into_any();
    }

    let navigate = use_navigate();
    let page_count = page.page_count();
    let first = (page.page - 1) * PAGE_SIZE + 1;
    let last = first + page.rows.len() as i64 - 1;
    let project_keys: Vec<String> = page
        .rows
        .iter()
        .map(|r| row_key(Level::Project, r.id))
        .collect();

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

    let rows = flatten(page.rows)
        .into_iter()
        .map(|row| {
            let FlatRow { item, depth, key, ancestors, has_children, stats } = row;
            let hidden = move || ancestors.iter().any(|a| collapsed.read().contains(a));
            let toggle_key = key.clone();
            let is_open = {
                let k = key.clone();
                Memo::new(move |_| !collapsed.read().contains(&k))
            };
            let (level, id, href) = match &item {
                Item::Project(p) => (Level::Project, p.id, Level::Project.url(p.id)),
                Item::Node(n) => (n.level, n.id, n.level.url(n.id)),
            };
            let nav = navigate.clone();
            let target = href.clone();
            let indent = format!("padding-left: {}rem", 0.5 + depth as f32 * 1.5);

            // Auf-/Zuklappen
            let expander = if has_children {
                view! {
                    <button
                        type="button"
                        class="btn btn-ghost btn-xs btn-square"
                        aria-label=move || if is_open.get() { "Zuklappen" } else { "Aufklappen" }
                        aria-expanded=move || is_open.get().to_string()
                        on:click=move |ev| {
                            ev.stop_propagation();
                            let k = toggle_key.clone();
                            collapsed.update(|c| { if !c.remove(&k) { c.insert(k); } });
                        }
                    >
                        <span class="inline-block transition-transform" class:rotate-90=is_open><IconView icon=Icon::Chevron/></span>
                    </button>
                }
                .into_any()
            } else {
                view! { <span class="inline-block w-6"></span> }.into_any()
            };

            let responsible = match &item {
                Item::Project(p) => p.responsible(),
                Item::Node(n) => n.responsible.clone(),
            };
            let (number, name, customer, status, start, end) = match item {
                Item::Project(p) => (p.nummer, p.name, p.kunde.unwrap_or_else(|| "–".into()), p.status, p.start, p.ende),
                Item::Node(n) => (n.number, n.name, String::new(), n.status, n.start, n.ende),
            };
            let is_project = level == Level::Project;
            let is_task = level == Level::Task;
            let done = status == "erledigt";

            view! {
                <tr
                    class=format!("cursor-pointer hover:bg-base-200 {}", level.tint_class())
                    class:hidden=hidden
                    class:border-t-2=is_project
                    class:border-base-300=is_project
                    class:text-sm=!is_project
                    on:click=move |_| nav(&target, Default::default())
                >
                    <td class=format!(
                        "{} {}",
                        level.stripe_class(),
                        if is_project { "whitespace-nowrap" } else { "whitespace-nowrap font-mono text-base-content/60" },
                    )>
                        <a href=href.clone() class="link link-hover" on:click=|ev| ev.stop_propagation()>{number}</a>
                    </td>
                    <td style=indent class="min-w-72">
                        <div class="flex items-center gap-1">
                            {expander}
                            {is_task.then(|| view! {
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-primary checkbox-xs mr-1"
                                    checked=done
                                    aria-label="Erledigt"
                                    on:click=|ev| ev.stop_propagation()
                                    on:change=move |ev| {
                                        toggle.dispatch(ToggleTask { id, erledigt: event_target_checked(&ev) });
                                    }
                                />
                            })}
                            <span class=level.badge_class()>{level.label()}</span>
                            <a
                                href=href
                                class="link link-hover whitespace-nowrap"
                                class:font-semibold=is_project
                                class:line-through=is_task && done
                                class:opacity-60=is_task && done
                                on:click=|ev| ev.stop_propagation()
                            >
                                {name}
                            </a>
                        </div>
                    </td>
                    <td class="whitespace-nowrap">{customer}</td>
                    <td><span class=status_badge(&status)>{status_label(&status).to_string()}</span></td>
                    <td class="whitespace-nowrap">{format_date(start.as_deref())}</td>
                    <td class="whitespace-nowrap">{format_date(end.as_deref())}</td>
                    // ANF-005 AK-30: nur der hier hinterlegte Benutzer
                    <td>{responsible.map(|r| view! { <ResponsibleBadge user=r/> })}</td>
                    <td class="whitespace-nowrap">
                        {stats.map(|(d, t)| match (d * 100).checked_div(t) {
                            Some(p) => view! {
                                <span class="flex items-center gap-2" title=format!("{d} von {t} Aufgaben erledigt")>
                                    <progress class="progress progress-primary w-16" value=d max=t></progress>
                                    <span class="text-xs">{format!("{p} %")}</span>
                                </span>
                            }.into_any(),
                            None => view! { <span class="text-base-content/40">"–"</span> }.into_any(),
                        })}
                    </td>
                    <td class="w-0"><RowActions level id/></td>
                </tr>
            }
        })
        .collect_view();

    let prev = (page.page > 1).then(|| state.with_page(page.page - 1).href());
    let next = (page.page < page_count).then(|| state.with_page(page.page + 1).href());

    view! {
        <div class="flex flex-wrap items-center justify-between gap-2">
            // Legende der Ebenen-Farben
            <div class="flex flex-wrap items-center gap-2 text-sm" aria-label="Ebenen">
                {[Level::Project, Level::Subproject, Level::Process, Level::Task]
                    .into_iter()
                    .map(|l| view! { <span class=l.badge_class()>{l.label()}</span> })
                    .collect_view()}
            </div>
            <div class="flex gap-2">
            <button type="button" class="btn btn-ghost btn-sm" on:click=move |_| collapsed.set(HashSet::new())>
                "Alle aufklappen"
            </button>
            <button
                type="button"
                class="btn btn-ghost btn-sm"
                on:click=move |_| collapsed.set(project_keys.iter().cloned().collect())
            >
                "Alle zuklappen"
            </button>
            </div>
        </div>
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        {headers}
                        <th>"Verantwortlich"</th>
                        <th>"Fortschritt"</th>
                        <th><span class="sr-only">"Aktionen"</span></th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
        <div class="flex flex-wrap items-center justify-between gap-2 text-sm">
            <span class="text-base-content/70">{format!("{first}–{last} von {} Projekten", page.total)}</span>
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
    fn default_href_is_plain() {
        assert_eq!(state().href(), "/projekte");
    }

    #[test]
    fn status_all_and_sorting() {
        let s = ListState {
            status: "alle".into(),
            ..state()
        };
        assert_eq!(s.href(), "/projekte?status=alle");
        assert_eq!(s.status_filter(), "");
        let by_name = s.sorted_by("name");
        assert_eq!(by_name.href(), "/projekte?status=alle&sortierung=name");
        assert_eq!(
            by_name.sorted_by("name").href(),
            "/projekte?status=alle&sortierung=name&richtung=ab"
        );
        assert_eq!(state().sorted_by("nummer").href(), "/projekte?richtung=auf");
    }
}
