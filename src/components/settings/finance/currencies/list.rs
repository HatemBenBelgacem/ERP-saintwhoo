//! Einstellungsseite „Währungen“ (ANF-014 AK-1 bis AK-5). Eingaben im Modal-Fenster
//! (`?neu=1`, `?bearbeiten=<Code>`, `?kurs=<Code>`).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use super::form::{CurrencyCreateModal, CurrencyEditModal, RateModal};
use crate::backend::finance::currency::model::{Currency, rate_label};
use crate::backend::finance::currency::server_fns::list_currencies;
use crate::backend::finance::fiscal_year::model::german_date;
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Einstellungsseite „Währungen“ unter `/einstellungen/finanzen/waehrungen`.
#[component]
pub fn CurrenciesPage() -> impl IntoView {
    let query = use_query_map();
    let param = move |name: &str| query.read().get(name);
    let base = use_base_href();
    let data = Resource::new(data_version, |_| list_currencies());

    view! {
        <Title text="Währungen – Einstellungen – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Währungen"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neue Währung"
                </a>
            </div>

            {move || param("gespeichert").map(|v| {
                let text = if let Some(code) = v.strip_prefix("kurs ") {
                    format!("Kurs für {code} gespeichert.")
                } else if let Some(code) = v.strip_prefix("waehrung ") {
                    format!("Währung {code} gespeichert.")
                } else {
                    "Gespeichert.".to_string()
                };
                view! { <div role="alert" class="alert alert-success">{text}</div> }
            })}
            {move || param("geloescht").map(|_| view! {
                <div role="alert" class="alert alert-success">"Währung wurde gelöscht."</div>
            })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(list) => view! { <CurrencyTable list base/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?neu=1, ?bearbeiten=<Code>, ?kurs=<Code>
        {move || param("neu").map(|_| view! { <CurrencyCreateModal/> })}
        {move || param("bearbeiten").map(|code| view! { <CurrencyEditModal code/> })}
        {move || param("kurs").map(|code| view! { <RateModal code/> })}
    }
}

/// Tabelle der Währungen mit letztem Kurs; die Buchhaltungswährung steht zuerst (AK-1).
#[component]
fn CurrencyTable(list: Vec<Currency>, base: Memo<String>) -> impl IntoView {
    let rows = list
        .into_iter()
        .map(|c| {
            let href = with_params(&base.get_untracked(), &[("bearbeiten", c.code.clone())]);
            let kurs = match (c.letzter_kurs, c.letzter_kurs_datum.as_deref()) {
                (Some(kurs), Some(datum)) => {
                    format!("{} CHF · {}", rate_label(kurs), german_date(datum))
                }
                _ => "–".to_string(),
            };
            let kurs_link = (!c.buchhaltungswaehrung).then(|| {
                let erfassen = with_params(&base.get_untracked(), &[("kurs", c.code.clone())]);
                view! { <a href=erfassen class="btn btn-xs btn-ghost">"Kurs erfassen"</a> }
            });
            let booking = c.buchhaltungswaehrung.then(|| view! {
                <span class="badge badge-sm badge-primary badge-soft">"Buchhaltungswährung"</span>
            });
            let status = if c.aktiv {
                view! { <span class="badge badge-sm badge-success badge-soft">"Aktiv"</span> }
                    .into_any()
            } else {
                view! { <span class="badge badge-sm badge-ghost">"Inaktiv"</span> }.into_any()
            };
            let used = c.verwendet.then(|| {
                view! {
                    <span class="badge badge-sm badge-ghost">"verwendet"</span>
                }
            });
            view! {
                <tr class="hover:bg-base-200" class:opacity-60=!c.aktiv>
                    <td class="font-medium">
                        <a href=href class="link link-hover">{c.code.clone()}</a>
                    </td>
                    <td>{c.bezeichnung.clone()}</td>
                    <td class="tabular-nums">{c.rundung_label()}</td>
                    <td class="whitespace-nowrap tabular-nums">{kurs}</td>
                    <td>
                        <div class="flex flex-wrap gap-1">
                            {booking}
                            {status}
                            {used}
                        </div>
                    </td>
                    <td class="text-right">{kurs_link}</td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        <th>"Code"</th>
                        <th>"Bezeichnung"</th>
                        <th>"Rundung"</th>
                        <th>"Letzter Kurs (CHF für 1)"</th>
                        <th>"Status"</th>
                        <th></th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}
