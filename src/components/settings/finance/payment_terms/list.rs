//! Einstellungsseite „Zahlungsbedingungen“ (ANF-016 AK-1 bis AK-8). Eingaben im Modal-Fenster
//! (`?neu=1`, `?bearbeiten=<kennung>`).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use super::form::PaymentTermModal;
use crate::backend::finance::payment_term::model::PaymentTerm;
use crate::backend::finance::payment_term::server_fns::list_payment_terms;
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Einstellungsseite „Zahlungsbedingungen“ unter `/einstellungen/finanzen/zahlungsbedingungen`.
#[component]
pub fn PaymentTermsPage() -> impl IntoView {
    let query = use_query_map();
    let param = move |name: &str| query.read().get(name);
    let base = use_base_href();
    let data = Resource::new(data_version, |_| list_payment_terms());

    view! {
        <Title text="Zahlungsbedingungen – Einstellungen – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Zahlungsbedingungen"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neue Zahlungsbedingung"
                </a>
            </div>

            {move || param("gespeichert").map(|_| view! {
                <div role="alert" class="alert alert-success">"Zahlungsbedingung gespeichert."</div>
            })}
            {move || param("geloescht").map(|_| view! {
                <div role="alert" class="alert alert-success">"Zahlungsbedingung wurde gelöscht."</div>
            })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(list) => view! { <PaymentTermTable list base/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?neu=1, ?bearbeiten=<kennung>
        {move || param("neu").map(|_| view! { <PaymentTermModal id=None/> })}
        {move || {
            param("bearbeiten")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <PaymentTermModal id=Some(id)/> })
        }}
    }
}

/// Tabelle der Zahlungsbedingungen; ein Klick auf die Bezeichnung öffnet das Bearbeiten (AK-1).
#[component]
fn PaymentTermTable(list: Vec<PaymentTerm>, base: Memo<String>) -> impl IntoView {
    if list.is_empty() {
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title text-base">"Noch keine Zahlungsbedingungen"</h2>
                    <p class="text-base-content/70">
                        "Legen Sie fest, bis wann Rechnungen zu bezahlen sind und ob es Skonto gibt."
                    </p>
                </div>
            </div>
        }
        .into_any();
    }

    let rows = list
        .into_iter()
        .map(|t| {
            let href = with_params(&base.get_untracked(), &[("bearbeiten", t.id.to_string())]);
            let standard = t.standard.then(|| {
                view! {
                    <span class="badge badge-sm badge-primary badge-soft">"Standard"</span>
                }
            });
            let status = if t.aktiv {
                view! { <span class="badge badge-sm badge-success badge-soft">"Aktiv"</span> }
                    .into_any()
            } else {
                view! { <span class="badge badge-sm badge-ghost">"Inaktiv"</span> }.into_any()
            };
            let used = t.verwendet.then(|| {
                view! {
                    <span class="badge badge-sm badge-ghost">"verwendet"</span>
                }
            });
            view! {
                <tr class="hover:bg-base-200" class:opacity-60=!t.aktiv>
                    <td class="font-medium">
                        <a href=href class="link link-hover">{t.bezeichnung.clone()}</a>
                    </td>
                    <td class="whitespace-nowrap">{t.summary()}</td>
                    <td>{t.rechnungstext.clone()}</td>
                    <td>
                        <div class="flex flex-wrap gap-1">
                            {standard}
                            {status}
                            {used}
                        </div>
                    </td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        <th>"Bezeichnung"</th>
                        <th>"Fälligkeit und Skonto"</th>
                        <th>"Rechnungstext"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
    .into_any()
}
