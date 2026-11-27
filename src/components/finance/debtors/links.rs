//! Debitoren in anderen Detailansichten: Ausgangsrechnungen eines Projekts (ANF-018 AK-15) und offene
//! Belege eines Kunden (AK-16).

use leptos::prelude::*;

use super::list::InvoiceTableRow;
use crate::backend::finance::debtors::model::{money, open_by_currency};
use crate::backend::finance::debtors::server_fns::{list_customer_open, list_project_invoices};
use crate::forms::{data_version, error_message};

/// Kopfzeile der Tabelle, passend zu `InvoiceTableRow` ohne Kunde bzw. ohne Projekt.
#[component]
fn Head(show_customer: bool) -> impl IntoView {
    view! {
        <thead>
            <tr>
                <th>"Nummer"</th>
                <th>"Art"</th>
                {show_customer.then(|| view! { <th>"Kunde"</th> })}
                <th>"Datum"</th>
                <th>"Fälligkeit"</th>
                <th class="text-right">"Gesamtbetrag"</th>
                <th class="text-right">"Offen"</th>
                <th>"Währung"</th>
                <th>"Status"</th>
            </tr>
        </thead>
    }
}

/// Ausgangsrechnungen eines Projekts (AK-15).
#[component]
pub fn ProjectInvoices(project_id: i64) -> impl IntoView {
    let data = Resource::new(
        move || (project_id, data_version()),
        |(id, _)| list_project_invoices(id),
    );
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Ausgangsrechnungen"</h2>
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match data.await {
                            Ok(rows) if rows.is_empty() => view! {
                                <p class="text-base-content/60">"Noch keine Ausgangsrechnungen für dieses Projekt."</p>
                            }.into_any(),
                            Ok(rows) => view! {
                                <div class="overflow-x-auto">
                                    <table class="table">
                                        <Head show_customer=true/>
                                        <tbody>
                                            {rows.into_iter().map(|row| view! {
                                                <InvoiceTableRow row show_customer=true show_project=false/>
                                            }).collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any(),
                            Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                        }
                    })}
                </Transition>
            </div>
        </section>
    }
}

/// Abschnitt „Debitoren“ einer Adresse: offene Belege und offener Gesamtbetrag (AK-16).
#[component]
pub fn CustomerInvoices(address_id: i64) -> impl IntoView {
    let data = Resource::new(
        move || (address_id, data_version()),
        |(id, _)| list_customer_open(id),
    );
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Debitoren"</h2>
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match data.await {
                            Ok(rows) if rows.is_empty() => view! {
                                <p class="text-base-content/60">"Keine offenen Rechnungen."</p>
                            }.into_any(),
                            Ok(rows) => {
                                let sums = open_by_currency(&rows)
                                    .into_iter()
                                    .map(|(c, v)| money(&c, v))
                                    .collect::<Vec<_>>()
                                    .join(" · ");
                                view! {
                                    <p>"Offen gesamt: " <span class="font-semibold tabular-nums">{sums}</span></p>
                                    <div class="overflow-x-auto">
                                        <table class="table">
                                            <Head show_customer=false/>
                                            <tbody>
                                                {rows.into_iter().map(|row| view! {
                                                    <InvoiceTableRow row show_customer=false show_project=false/>
                                                }).collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }
                            Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                        }
                    })}
                </Transition>
            </div>
        </section>
    }
}
