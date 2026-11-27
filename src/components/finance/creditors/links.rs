//! Kreditoren in anderen Detailansichten: Eingangsrechnungen eines Projekts und offene Kreditoren einer
//! Adresse (ANF-019 AK-15), Eingangsrechnungen eines Einkaufsauftrags mit „bestellt / verrechnet“ und
//! „Eingangsrechnung erfassen“ (AK-8, AK-9).

use leptos::prelude::*;

use super::list::CreditorTableRow;
use crate::backend::finance::creditors::model::billed_vs_ordered;
use crate::backend::finance::creditors::server_fns::{
    CreateCreditorFromOrder, list_order_creditors, list_project_creditors, list_supplier_open,
};
use crate::backend::finance::debtors::model::{OPEN_STATUSES, money, signed};
use crate::forms::{data_version, error_message};

#[component]
fn Head(show_supplier: bool, show_order: bool) -> impl IntoView {
    view! {
        <thead>
            <tr>
                <th>"Nummer"</th>
                <th>"Rechnungs-Nr."</th>
                {show_supplier.then(|| view! { <th>"Lieferant"</th> })}
                {show_order.then(|| view! { <th>"Einkaufsauftrag"</th> })}
                <th>"Datum"</th>
                <th>"Fälligkeit"</th>
                <th>"Skonto bis"</th>
                <th class="text-right">"Gesamtbetrag"</th>
                <th class="text-right">"Offen"</th>
                <th>"Währung"</th>
                <th>"Status"</th>
            </tr>
        </thead>
    }
}

/// Karte mit Titel und Tabelle der Zeilen.
fn table(
    rows: Vec<crate::backend::finance::creditors::model::CreditorRow>,
    show_supplier: bool,
    show_order: bool,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="table">
                <Head show_supplier show_order/>
                <tbody>
                    {rows.into_iter().map(|row| view! {
                        <CreditorTableRow row show_supplier show_order show_project=false/>
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// Eingangsrechnungen eines Projekts (Kopf oder Position).
#[component]
pub fn ProjectCreditors(project_id: i64) -> impl IntoView {
    let data = Resource::new(
        move || (project_id, data_version()),
        |(id, _)| list_project_creditors(id),
    );
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Eingangsrechnungen"</h2>
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match data.await {
                            Ok(rows) if rows.is_empty() => view! {
                                <p class="text-base-content/60">"Noch keine Eingangsrechnungen für dieses Projekt."</p>
                            }.into_any(),
                            Ok(rows) => table(rows, true, true).into_any(),
                            Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                        }
                    })}
                </Transition>
            </div>
        </section>
    }
}

/// Abschnitt „Kreditoren“ einer Adresse: offene Eingangsrechnungen und offener Betrag.
#[component]
pub fn SupplierCreditors(address_id: i64) -> impl IntoView {
    let data = Resource::new(
        move || (address_id, data_version()),
        |(id, _)| list_supplier_open(id),
    );
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Kreditoren"</h2>
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match data.await {
                            Ok(rows) if rows.is_empty() => view! {
                                <p class="text-base-content/60">"Keine offenen Eingangsrechnungen."</p>
                            }.into_any(),
                            Ok(rows) => {
                                let mut sums: Vec<(String, i64)> = Vec::new();
                                for r in rows.iter().filter(|r| OPEN_STATUSES.contains(&r.status.as_str())) {
                                    let v = signed(&r.art, r.offener_betrag);
                                    match sums.iter_mut().find(|(c, _)| *c == r.waehrung) {
                                        Some(e) => e.1 += v,
                                        None => sums.push((r.waehrung.clone(), v)),
                                    }
                                }
                                let text = sums.iter().map(|(c, v)| money(c, *v)).collect::<Vec<_>>().join(" · ");
                                view! {
                                    <p>"Offen gesamt: " <span class="font-semibold tabular-nums">{text}</span></p>
                                    {table(rows, false, true)}
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

/// Eingangsrechnungen eines Einkaufsauftrags mit „bestellt / verrechnet“ (AK-9) und der Übernahme (AK-8).
/// `bestellt`: Gesamtbetrag des Auftrags; `can_invoice`: Auftrag ist bestellt oder geliefert.
#[component]
pub fn OrderCreditors(
    order_id: i64,
    bestellt: i64,
    waehrung: String,
    can_invoice: bool,
) -> impl IntoView {
    let create = ServerAction::<CreateCreditorFromOrder>::new();
    let data = Resource::new(
        move || (order_id, data_version()),
        |(id, _)| list_order_creditors(id),
    );
    let create_error = move || {
        create
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <h2 class="card-title text-base">"Eingangsrechnungen"</h2>
                    {can_invoice.then(|| view! {
                        <button
                            type="button"
                            class="btn btn-sm btn-primary btn-outline"
                            disabled=move || create.pending().get()
                            on:click=move |_| { create.dispatch(CreateCreditorFromOrder { auftrag_id: order_id }); }
                        >
                            "Eingangsrechnung erfassen"
                        </button>
                    })}
                </div>
                {move || create_error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || {
                        let waehrung = waehrung.clone();
                        Suspend::new(async move {
                            match data.await {
                                Ok(rows) if rows.is_empty() => view! {
                                    <p class="text-base-content/60">"Noch keine Eingangsrechnungen zu diesem Auftrag."</p>
                                }.into_any(),
                                Ok(rows) => {
                                    let (verrechnet, zu_viel) = billed_vs_ordered(&rows, bestellt);
                                    view! {
                                        <p class="text-sm">
                                            "Bestellt " <span class="font-medium tabular-nums">{money(&waehrung, bestellt)}</span>
                                            " / verrechnet " <span class="font-medium tabular-nums">{money(&waehrung, verrechnet)}</span>
                                        </p>
                                        {zu_viel.then(|| view! {
                                            <div role="alert" class="alert alert-warning alert-soft">
                                                "Der verrechnete Betrag übersteigt den bestellten Betrag."
                                            </div>
                                        })}
                                        {table(rows, false, false)}
                                    }.into_any()
                                }
                                Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                            }
                        })
                    }}
                </Transition>
            </div>
        </section>
    }
}
