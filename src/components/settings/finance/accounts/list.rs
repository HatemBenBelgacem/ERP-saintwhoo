//! Einstellungsseite „Kontenplan“: Liste nach Kontenklassen, Suche, Filter, Vorlage und
//! Systemkonten (ANF-011 AK-1 bis AK-3, AK-11, AK-12).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{
    components::Form,
    hooks::{use_navigate, use_query_map},
};

use super::form::{AccountModal, SystemAccountsModal, TemplateModal};
use crate::backend::finance::accounts::model::{
    Account, ChartSummary, KIND_FILTERS, class_name, class_of, kind_badge, kind_label, role_label,
};
use crate::backend::finance::accounts::server_fns::{
    ACCOUNTS_URL, get_chart_summary, list_accounts,
};
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Zustand der Liste – kommt vollständig aus der Webadresse (AK-2).
#[derive(Clone, Debug, PartialEq)]
struct ListState {
    search: String,
    kind: String,
    inactive: bool,
}

impl ListState {
    fn is_filtered(&self) -> bool {
        !self.search.is_empty() || self.kind != "alle" || self.inactive
    }
}

#[component]
pub fn AccountsPage() -> impl IntoView {
    let query = use_query_map();
    let state = Memo::new(move |_| {
        let q = query.read();
        ListState {
            search: q.get("suche").unwrap_or_default(),
            kind: q.get("kontoart").unwrap_or_else(|| "alle".into()),
            inactive: q.get("inaktive").is_some_and(|v| !v.is_empty()),
        }
    });
    let data = Resource::new(
        move || (state.get(), data_version()),
        |(s, _)| list_accounts(s.search, s.kind, s.inactive),
    );
    let summary = Resource::new(data_version, |_| get_chart_summary());
    let base = use_base_href();
    let param = move |name: &str| query.read().get(name);

    view! {
        <Title text="Kontenplan – Einstellungen – ERP-saintwhoo"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Kontenplan"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neues Konto"
                </a>
            </div>

            {move || param("gespeichert").map(|v| {
                let text = match v.as_str() {
                    "vorlage" => "KMU-Kontenrahmen übernommen, Systemkonten vorbelegt.".to_string(),
                    "systemkonten" => "Systemkonten gespeichert.".to_string(),
                    nummer => format!("Konto {nummer} gespeichert."),
                };
                view! { <div role="alert" class="alert alert-success">{text}</div> }
            })}
            {move || param("geloescht").map(|nummer| view! {
                <div role="alert" class="alert alert-success">"Konto " {nummer} " wurde gelöscht."</div>
            })}

            <Transition fallback=|| ()>
                {move || Suspend::new(async move {
                    match summary.await {
                        Ok(s) => view! { <SummaryCards summary=s base/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                            .into_any(),
                    }
                })}
            </Transition>

            // AK-2: Suche, Kontoart, inaktive Konten
            <Form method="GET" action=ACCOUNTS_URL>
                <div class="flex flex-wrap items-center gap-2">
                    <label class="input min-w-0 flex-1 sm:w-md sm:max-w-md sm:flex-none">
                        <IconView icon=Icon::Search/>
                        <input
                            type="search"
                            name="suche"
                            placeholder="Nummer, Bezeichnung oder Gruppe suchen …"
                            prop:value=move || state.read().search.clone()
                        />
                    </label>
                    <select
                        name="kontoart"
                        class="select w-44"
                        aria-label="Kontoart"
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
                    <label class="label cursor-pointer gap-2">
                        <input
                            type="checkbox"
                            name="inaktive"
                            value="1"
                            class="checkbox checkbox-primary checkbox-sm"
                            onchange="this.form.requestSubmit()"
                            prop:checked=move || state.read().inactive
                        />
                        "Inaktive anzeigen"
                    </label>
                    <button type="submit" class="btn">"Suchen"</button>
                </div>
            </Form>

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(rows) => view! { <AccountTable rows filtered=state.get().is_filtered()/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                            .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?neu=1, ?bearbeiten=<kennung>, ?vorlage=1, ?systemkonten=1
        {move || param("neu").map(|_| view! { <AccountModal id=None/> })}
        {move || {
            param("bearbeiten")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <AccountModal id=Some(id)/> })
        }}
        {move || param("vorlage").map(|_| view! { <TemplateModal/> })}
        {move || param("systemkonten").map(|_| view! { <SystemAccountsModal/> })}
    }
}

/// Leerer Kontenplan mit Vorlage (AK-3), Warnung fehlender Systemkonten (AK-12) und
/// Übersicht der Systemkonten (AK-11).
#[component]
fn SummaryCards(summary: ChartSummary, base: Memo<String>) -> impl IntoView {
    let missing = summary.missing_required();
    let empty = summary.anzahl == 0;
    let rows = summary
        .systemkonten
        .iter()
        .map(|s| {
            let label = role_label(&s.rolle);
            let account = s.konto.as_ref().map(Account::label);
            view! {
                <tr>
                    <td class="whitespace-nowrap">{label}</td>
                    <td>
                        {match account {
                            Some(a) => view! { <span>{a}</span> }.into_any(),
                            None => view! { <span class="text-base-content/50">"nicht zugeordnet"</span> }.into_any(),
                        }}
                    </td>
                </tr>
            }
        })
        .collect_view();

    view! {
        {empty.then(|| view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title text-base">"Noch keine Konten erfasst"</h2>
                    <p class="text-base-content/70">
                        "Übernehmen Sie den Schweizer KMU-Kontenrahmen als Ausgangspunkt und passen Sie ihn an, "
                        "oder erfassen Sie die Konten einzeln."
                    </p>
                    <div class="card-actions">
                        <a href=move || with_params(&base.get(), &[("vorlage", "1".into())]) class="btn btn-primary">
                            "KMU-Kontenrahmen übernehmen"
                        </a>
                    </div>
                </div>
            </div>
        })}
        {(!missing.is_empty()).then(|| view! {
            <div role="alert" class="alert alert-warning">
                <span>
                    "Pflicht-Systemkonten fehlen: " {missing.join(", ")} ". "
                    "Automatische Buchungen aus Debitoren und Kreditoren sind gesperrt, bis sie zugeordnet sind."
                </span>
            </div>
        })}
        <details class="collapse-arrow collapse bg-base-100 shadow" open=!missing.is_empty() && !empty>
            <summary class="collapse-title font-semibold">"Systemkonten"</summary>
            <div class="collapse-content space-y-2">
                <p class="text-sm text-base-content/70">
                    "Diese Konten verwenden Debitoren, Kreditoren, Zahlungen, MWST und Abschluss für automatische Buchungen."
                </p>
                <div class="overflow-x-auto">
                    <table class="table table-sm">
                        <tbody>{rows}</tbody>
                    </table>
                </div>
                <a href=move || with_params(&base.get(), &[("systemkonten", "1".into())]) class="btn btn-sm">
                    <IconView icon=Icon::Pencil/>
                    "Systemkonten bearbeiten"
                </a>
            </div>
        </details>
    }
}

/// Konten gruppiert nach Kontenklasse (AK-1). Klick auf eine Zeile öffnet das Bearbeiten.
#[component]
fn AccountTable(rows: Vec<Account>, filtered: bool) -> impl IntoView {
    if rows.is_empty() {
        let text = if filtered {
            "Keine Konten zu dieser Suche gefunden."
        } else {
            "Der Kontenplan ist leer."
        };
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">{text}</div>
            </div>
        }
        .into_any();
    }

    let navigate = use_navigate();
    let base = use_base_href();
    let count = rows.len();
    let mut current_class = None;
    let mut body = Vec::new();
    for row in rows {
        let class = class_of(&row.nummer);
        if class != current_class {
            current_class = class;
            if let Some(c) = class {
                body.push(
                    view! {
                        <tr class="bg-base-200">
                            <th colspan="6">"Klasse " {c} " · " {class_name(c)}</th>
                        </tr>
                    }
                    .into_any(),
                );
            }
        }
        let href = with_params(&base.get_untracked(), &[("bearbeiten", row.id.to_string())]);
        let nav = navigate.clone();
        let target = href.clone();
        let roles = row
            .rollen
            .iter()
            .map(|r| view! { <span class="badge badge-sm badge-secondary badge-soft">{role_label(r)}</span> })
            .collect_view();
        body.push(
            view! {
                <tr
                    class="cursor-pointer hover:bg-base-200"
                    class:opacity-60=!row.aktiv
                    on:click=move |_| nav(&target, Default::default())
                >
                    <td class="whitespace-nowrap font-medium tabular-nums">
                        <a href=href class="link link-hover">{row.nummer.clone()}</a>
                    </td>
                    <td>{row.bezeichnung.clone()}</td>
                    <td><span class=kind_badge(&row.kontoart)>{kind_label(&row.kontoart)}</span></td>
                    <td class="text-base-content/70">{row.kontengruppe.clone().unwrap_or_default()}</td>
                    <td><div class="flex flex-wrap gap-1">{roles}</div></td>
                    <td>
                        {(!row.aktiv).then(|| view! { <span class="badge badge-sm badge-ghost">"Inaktiv"</span> })}
                    </td>
                </tr>
            }
            .into_any(),
        );
    }

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        <th>"Nummer"</th>
                        <th>"Bezeichnung"</th>
                        <th>"Kontoart"</th>
                        <th>"Kontengruppe"</th>
                        <th>"Systemkonto"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>{body}</tbody>
            </table>
        </div>
        <p class="text-sm text-base-content/70">
            {if count == 1 { "1 Konto".to_string() } else { format!("{count} Konten") }}
        </p>
    }
    .into_any()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtered_state() {
        let plain = ListState {
            search: String::new(),
            kind: "alle".into(),
            inactive: false,
        };
        assert!(!plain.is_filtered());
        assert!(
            ListState {
                inactive: true,
                ..plain.clone()
            }
            .is_filtered()
        );
        assert!(
            ListState {
                kind: "aktiv".into(),
                ..plain
            }
            .is_filtered()
        );
    }
}
