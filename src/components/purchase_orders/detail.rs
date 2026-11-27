//! Detailansicht eines Einkaufsauftrags mit Positionen, Summen und Statuswechseln
//! (AK-6, AK-7, AK-13 bis AK-20, AK-24) sowie die Einkaufsaufträge eines Projekts (AK-22).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use super::form::{DeliveryModal, OrderEditModal, PositionCreateModal, PositionEditModal};
use super::list::OrderTableRow;
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::{
    Order, OrderDetail, Position, Totals, format_amount, format_trimmed, status_badge,
    status_label, unit_label,
};
use crate::backend::purchase_orders::server_fns::{
    CancelOrder, DeleteOrder, DeletePosition, DeliverAll, MovePosition, PlaceOrder, get_order,
    list_project_orders,
};
use crate::components::finance::creditors::OrderCreditors;
use crate::forms::{Breadcrumbs, data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Aktionen ohne eigenes Formular; nach jeder Aktion wird der Auftrag neu geladen.
#[derive(Clone, Copy)]
struct Actions {
    place: ServerAction<PlaceOrder>,
    deliver_all: ServerAction<DeliverAll>,
    cancel: ServerAction<CancelOrder>,
    delete_position: ServerAction<DeletePosition>,
    move_position: ServerAction<MovePosition>,
}

impl Actions {
    fn new() -> Self {
        Self {
            place: ServerAction::new(),
            deliver_all: ServerAction::new(),
            cancel: ServerAction::new(),
            delete_position: ServerAction::new(),
            move_position: ServerAction::new(),
        }
    }

    fn versions(&self) -> [usize; 5] {
        [
            self.place.version().get(),
            self.deliver_all.version().get(),
            self.cancel.version().get(),
            self.delete_position.version().get(),
            self.move_position.version().get(),
        ]
    }

    /// Erster Fehler der Aktionen ohne eigenen Dialog.
    fn error(&self) -> Option<String> {
        fn err<T>(v: Option<Result<T, ServerFnError>>) -> Option<String> {
            v.and_then(|r| r.err()).map(|e| error_message(&e))
        }
        err(self.place.value().get())
            .or_else(|| err(self.deliver_all.value().get()))
            .or_else(|| err(self.delete_position.value().get()))
            .or_else(|| err(self.move_position.value().get()))
    }
}

#[component]
pub fn OrderDetailPage() -> impl IntoView {
    let params = use_params_map();
    let actions = Actions::new();
    provide_context(actions);
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let data = Resource::new(
        move || (id(), actions.versions(), data_version()),
        |(id, _, _)| async move {
            match id {
                Some(id) => get_order(id).await,
                None => Ok(None),
            }
        },
    );

    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(detail)) => view! { <OrderView detail/> }.into_any(),
                    Ok(None) => view! { <NotFound/> }.into_any(),
                    Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                }
            })}
        </Transition>
    }
}

/// AK-24
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="Einkaufsauftrag nicht gefunden – uniqwork"/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">"Einkaufsauftrag nicht gefunden"</h1>
                <p class="text-base-content/70">"Diesen Einkaufsauftrag gibt es nicht oder er wurde gelöscht."</p>
                <div class="card-actions">
                    <a href="/einkauf" class="btn">"Zur Einkaufsliste"</a>
                </div>
            </div>
        </div>
    }
}

/// Welches Modal-Fenster laut Webadresse offen ist.
#[derive(Clone, Debug, PartialEq)]
enum Dialog {
    EditOrder,
    NewPosition,
    EditPosition(i64),
    Delivery(i64),
}

fn parse_dialog(
    neu: Option<String>,
    bearbeiten: Option<String>,
    id: Option<String>,
    wareneingang: Option<String>,
) -> Option<Dialog> {
    if let Some(w) = wareneingang {
        return w.parse().ok().map(Dialog::Delivery);
    }
    if neu.as_deref() == Some("position") {
        return Some(Dialog::NewPosition);
    }
    match bearbeiten.as_deref() {
        Some("position") => id?.parse().ok().map(Dialog::EditPosition),
        Some(_) => Some(Dialog::EditOrder),
        None => None,
    }
}

#[component]
fn OrderView(detail: OrderDetail) -> impl IntoView {
    let actions = expect_context::<Actions>();
    let query = use_query_map();
    let base = use_base_href();
    let order = detail.order.clone();
    let positions = detail.positions.clone();
    let totals = detail.totals();
    let id = order.id;
    let cancel_open = RwSignal::new(false);
    let delete_open = RwSignal::new(false);

    let dialog = Memo::new(move |_| {
        let q = query.read();
        parse_dialog(
            q.get("neu"),
            q.get("bearbeiten"),
            q.get("id"),
            q.get("wareneingang"),
        )
    });
    let saved = move || query.read().get("gespeichert").is_some();
    let edit_href = move || with_params(&base.get(), &[("bearbeiten", "1".into())]);

    let is_draft = order.is_draft();
    let is_ordered = order.is_ordered();
    let can_cancel = order.can_cancel(&positions);
    let has_positions = !positions.is_empty();

    // Modal-Fenster aus der Webadresse
    let modal_order = order.clone();
    let modal_positions = positions.clone();
    let modals = move || {
        let find = |pid: i64| {
            modal_positions
                .iter()
                .enumerate()
                .find(|(_, p)| p.id == pid)
                .map(|(i, p)| (p.clone(), i + 1))
        };
        match dialog.get()? {
            Dialog::EditOrder if modal_order.is_editable() => {
                Some(view! { <OrderEditModal order=modal_order.clone()/> }.into_any())
            }
            Dialog::NewPosition if modal_order.is_draft() => Some(
                view! { <PositionCreateModal order_id=modal_order.id order_number=modal_order.nummer.clone()/> }
                    .into_any(),
            ),
            Dialog::EditPosition(pid) if modal_order.is_draft() => find(pid).map(|(position, number)| {
                view! { <PositionEditModal position number/> }.into_any()
            }),
            Dialog::Delivery(pid) if modal_order.is_ordered() => find(pid).map(|(position, number)| {
                view! { <DeliveryModal position number/> }.into_any()
            }),
            _ => None,
        }
    };

    view! {
        <Title text=format!("{} – Einkauf – uniqwork", order.nummer)/>
        <div class="space-y-4">
            <Breadcrumbs items=vec![
                ("Einkauf".to_string(), Some("/einkauf".to_string())),
                (order.nummer.clone(), None),
            ]/>
            {move || saved().then(|| view! {
                <div role="alert" class="alert alert-success">"Gespeichert."</div>
            })}
            {move || actions.error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}

            // Kopf mit Status und Schaltflächen je Status
            <div class="flex flex-wrap items-start justify-between gap-2">
                <div>
                    <h1 class="text-2xl font-semibold">{order.nummer.clone()}</h1>
                    <p class="flex flex-wrap items-center gap-2 text-base-content/70">
                        {order.lieferant.clone()}
                        <span class=status_badge(&order.status)>{status_label(&order.status).to_string()}</span>
                    </p>
                </div>
                <div class="flex flex-wrap gap-2">
                    // AK-14: Bestellen
                    {is_draft.then(|| view! {
                        <button
                            type="button"
                            class="btn btn-primary"
                            disabled=move || !has_positions || actions.place.pending().get()
                            title=(!has_positions).then_some("Zuerst mindestens eine Position erfassen")
                            on:click=move |_| { actions.place.dispatch(PlaceOrder { id }); }
                        >
                            "Bestellen"
                        </button>
                    })}
                    // Wareneingang: alles geliefert
                    {is_ordered.then(|| view! {
                        <button
                            type="button"
                            class="btn btn-primary"
                            disabled=move || actions.deliver_all.pending().get()
                            on:click=move |_| { actions.deliver_all.dispatch(DeliverAll { id }); }
                        >
                            "Alles geliefert"
                        </button>
                    })}
                    {order.is_editable().then(|| view! {
                        <a href=edit_href class="btn btn-outline">
                            <IconView icon=Icon::Pencil/>
                            "Bearbeiten"
                        </a>
                    })}
                    {can_cancel.then(|| view! {
                        <button type="button" class="btn btn-outline btn-warning" on:click=move |_| cancel_open.set(true)>
                            "Stornieren"
                        </button>
                    })}
                    {is_draft.then(|| view! {
                        <button type="button" class="btn btn-outline btn-error" on:click=move |_| delete_open.set(true)>
                            <IconView icon=Icon::Trash/>
                            "Löschen"
                        </button>
                    })}
                </div>
            </div>

            <OrderInfo order=order.clone()/>
            <PositionsCard order=order.clone() positions=positions.clone() totals=totals.clone()/>
            // ANF-019 AK-8, AK-9: Eingangsrechnungen, „bestellt / verrechnet“, Übernahme
            {(!is_draft).then(|| view! {
                <OrderCreditors
                    order_id=id
                    bestellt=totals.total
                    waehrung=order.waehrung.clone()
                    can_invoice=matches!(order.status.as_str(), "bestellt" | "teilweise_geliefert" | "geliefert")
                />
            })}

            <p class="text-sm text-base-content/60">
                "Erstellt am " {order.erstellt_am.clone()} " · Geändert am " {order.geaendert_am.clone()}
            </p>
        </div>

        {modals}
        <CancelDialog open=cancel_open id nummer=order.nummer.clone()/>
        <DeleteDialog open=delete_open id nummer=order.nummer.clone() count=positions.len()/>
    }
}

/// Kopfdaten in drei Karten.
#[component]
fn OrderInfo(order: Order) -> impl IntoView {
    let o = order;
    view! {
        <div class="grid gap-4 md:grid-cols-3">
            <InfoCard>
                <Info label="Lieferant">
                    <a href=format!("/adressen/{}", o.adresse_id) class="link link-primary">{o.lieferant.clone()}</a>
                    <span class="text-sm text-base-content/60">" · " {o.lieferant_nummer.clone()}</span>
                </Info>
                <Info label="Projekt">
                    {match (o.projekt_id, o.projekt_nummer.clone(), o.projekt_name.clone()) {
                        (Some(pid), Some(n), Some(name)) => view! {
                            <a href=format!("/projekte/{pid}") class="link link-primary">{format!("{n} {name}")}</a>
                        }.into_any(),
                        _ => view! { "–" }.into_any(),
                    }}
                </Info>
                <Info label="Referenz Lieferant">{o.referenz_lieferant.clone().unwrap_or_else(|| "–".into())}</Info>
            </InfoCard>
            <InfoCard>
                <Info label="Bestelldatum">{format_date(o.bestelldatum.as_deref())}</Info>
                <Info label="Liefertermin">
                    {format_date(o.liefertermin.as_deref())}
                    {o.ueberfaellig.then(|| view! {
                        <span class="badge badge-sm badge-error badge-soft ml-2">"überfällig"</span>
                    })}
                </Info>
                <Info label="Währung">{o.waehrung.clone()}</Info>
            </InfoCard>
            <InfoCard>
                <Info label="Abweichende Lieferadresse">
                    <span class="whitespace-pre-line">{o.lieferadresse.clone().unwrap_or_else(|| "–".into())}</span>
                </Info>
                <Info label="Bemerkung">
                    <span class="whitespace-pre-line">{o.bemerkung.clone().unwrap_or_else(|| "–".into())}</span>
                </Info>
            </InfoCard>
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

/// Positionstabelle mit Summen (AK-7) und Schaltflächen je Status (AK-11, AK-13, AK-16).
#[component]
fn PositionsCard(order: Order, positions: Vec<Position>, totals: Totals) -> impl IntoView {
    let actions = expect_context::<Actions>();
    let base = use_base_href();
    let is_draft = order.is_draft();
    let is_ordered = order.is_ordered();
    let show_delivered = !is_draft;
    let currency = order.waehrung.clone();
    let count = positions.len();
    let delete_target = RwSignal::new(None::<(i64, String)>);
    let new_href = move || with_params(&base.get(), &[("neu", "position".into())]);

    let rows = positions
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let pid = p.id;
            let unit = unit_label(&p.einheit).to_string();
            let edit = move || {
                with_params(
                    &base.get(),
                    &[("bearbeiten", "position".into()), ("id", pid.to_string())],
                )
            };
            let receive = move || with_params(&base.get(), &[("wareneingang", pid.to_string())]);
            let name = p.bezeichnung.clone();
            let delivered_class = if p.is_over_delivered() {
                "text-warning"
            } else if p.is_fully_delivered() {
                "text-success"
            } else {
                ""
            };
            view! {
                <tr>
                    <td class="font-mono text-sm text-base-content/60">{i + 1}</td>
                    <td>
                        <div class="font-medium">{p.bezeichnung.clone()}</div>
                        {p.artikelnummer_lieferant.clone().map(|a| view! {
                            <div class="text-sm text-base-content/60">"Art.-Nr. " {a}</div>
                        })}
                    </td>
                    <td class="text-right tabular-nums whitespace-nowrap">{format_trimmed(p.menge, 3)} " " {unit.clone()}</td>
                    <td class="text-right tabular-nums">{format_amount(p.einzelpreis)}</td>
                    <td class="text-right tabular-nums">
                        {(p.rabatt_prozent > 0).then(|| format!("{} %", format_trimmed(p.rabatt_prozent, 2)))}
                    </td>
                    <td class="text-right tabular-nums">{format!("{} %", format_trimmed(p.mwst_satz, 2))}</td>
                    <td class="text-right tabular-nums font-medium">{format_amount(p.amount())}</td>
                    {show_delivered.then(|| view! {
                        <td class=format!("text-right tabular-nums whitespace-nowrap {delivered_class}")>
                            {format_trimmed(p.gelieferte_menge, 3)} " " {unit.clone()}
                            {p.is_over_delivered().then(|| view! {
                                <span class="badge badge-sm badge-warning badge-soft ml-1" title="Mehr geliefert als bestellt">"Überlieferung"</span>
                            })}
                        </td>
                    })}
                    <td class="text-right whitespace-nowrap">
                        {is_draft.then(|| view! {
                            <a href=edit class="btn btn-ghost btn-xs" title="Bearbeiten" aria-label="Bearbeiten">
                                <IconView icon=Icon::Pencil/>
                            </a>
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs"
                                title="Löschen"
                                aria-label="Löschen"
                                on:click=move |_| delete_target.set(Some((pid, name.clone())))
                            >
                                <IconView icon=Icon::Trash/>
                            </button>
                            <span class="join">
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs join-item"
                                    title="Nach oben"
                                    aria-label="Nach oben"
                                    disabled=i == 0
                                    on:click=move |_| { actions.move_position.dispatch(MovePosition { id: pid, nach_oben: true }); }
                                >"▲"</button>
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs join-item"
                                    title="Nach unten"
                                    aria-label="Nach unten"
                                    disabled=i + 1 == count
                                    on:click=move |_| { actions.move_position.dispatch(MovePosition { id: pid, nach_oben: false }); }
                                >"▼"</button>
                            </span>
                        })}
                        {is_ordered.then(|| view! {
                            <a href=receive class="btn btn-outline btn-primary btn-xs">"Wareneingang"</a>
                        })}
                    </td>
                </tr>
            }
        })
        .collect_view();

    // Summenzeilen: Beschriftung über alle Spalten bis „Betrag“
    let label_span = 6;
    let rest_span = if show_delivered { 2 } else { 1 };
    let vat_rows = totals
        .vat
        .iter()
        .map(|l| {
            view! {
                <tr>
                    <td colspan=label_span class="text-right text-base-content/70">
                        {format!("MWST {} % auf {}", format_trimmed(l.rate, 2), format_amount(l.base))}
                    </td>
                    <td class="text-right tabular-nums">{format_amount(l.vat)}</td>
                    <td colspan=rest_span></td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <h2 class="card-title text-base">"Positionen"</h2>
                    {is_draft.then(|| view! {
                        <a href=new_href class="btn btn-sm btn-primary btn-outline">
                            <IconView icon=Icon::Plus/>
                            "Neue Position"
                        </a>
                    })}
                </div>
                {if count == 0 {
                    view! { <p class="text-base-content/60">"Noch keine Positionen."</p> }.into_any()
                } else {
                    view! {
                        <div class="overflow-x-auto">
                            <table class="table">
                                <thead>
                                    <tr>
                                        <th>"Pos."</th>
                                        <th>"Bezeichnung"</th>
                                        <th class="text-right">"Menge"</th>
                                        <th class="text-right">"Einzelpreis"</th>
                                        <th class="text-right">"Rabatt"</th>
                                        <th class="text-right">"MWST"</th>
                                        <th class="text-right">"Betrag"</th>
                                        {show_delivered.then(|| view! { <th class="text-right">"Geliefert"</th> })}
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>{rows}</tbody>
                                <tfoot class="text-base text-base-content">
                                    <tr>
                                        <td colspan=label_span class="text-right">"Zwischensumme"</td>
                                        <td class="text-right tabular-nums">{format_amount(totals.subtotal)}</td>
                                        <td colspan=rest_span></td>
                                    </tr>
                                    {vat_rows}
                                    {(totals.rounding != 0).then(|| view! {
                                        <tr>
                                            <td colspan=label_span class="text-right text-base-content/70">"Rundung"</td>
                                            <td class="text-right tabular-nums">{format_amount(totals.rounding)}</td>
                                            <td colspan=rest_span></td>
                                        </tr>
                                    })}
                                    <tr class="font-semibold">
                                        <td colspan=label_span class="text-right">{format!("Gesamtbetrag {currency}")}</td>
                                        <td class="text-right tabular-nums">{format_amount(totals.total)}</td>
                                        <td colspan=rest_span></td>
                                    </tr>
                                </tfoot>
                            </table>
                        </div>
                    }.into_any()
                }}
            </div>
        </section>
        <DeletePositionDialog target=delete_target/>
    }
}

/// Bestätigung „Position löschen?“.
#[component]
fn DeletePositionDialog(target: RwSignal<Option<(i64, String)>>) -> impl IntoView {
    let actions = expect_context::<Actions>();
    view! {
        <div class="modal max-sm:modal-bottom" class:modal-open=move || target.read().is_some() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Position löschen?"</h3>
                <p class="py-4">
                    {move || target.get().map(|(_, name)| format!("Soll die Position „{name}“ wirklich gelöscht werden?"))}
                </p>
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| target.set(None)>"Abbrechen"</button>
                    <button
                        type="button"
                        class="btn btn-error"
                        on:click=move |_| {
                            if let Some((id, _)) = target.get_untracked() {
                                actions.delete_position.dispatch(DeletePosition { id });
                            }
                            target.set(None);
                        }
                    >
                        "Löschen"
                    </button>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| target.set(None)></div>
        </div>
    }
}

/// Bestätigung „Stornieren?“ (AK-19).
#[component]
fn CancelDialog(open: RwSignal<bool>, id: i64, nummer: String) -> impl IntoView {
    let actions = expect_context::<Actions>();
    let cancel = actions.cancel;
    let error = move || {
        cancel
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    view! {
        <div class="modal max-sm:modal-bottom" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Einkaufsauftrag stornieren?"</h3>
                <p class="py-4">
                    {format!("Soll {nummer} storniert werden? Der Auftrag bleibt erhalten, kann danach aber nicht mehr bearbeitet werden.")}
                </p>
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| open.set(false)>"Abbrechen"</button>
                    <button
                        type="button"
                        class="btn btn-warning"
                        disabled=move || cancel.pending().get()
                        on:click=move |_| { cancel.dispatch(CancelOrder { id }); }
                    >
                        "Stornieren"
                    </button>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| open.set(false)></div>
        </div>
    }
}

/// Bestätigung „Löschen?“ mit Nummer (AK-20).
#[component]
fn DeleteDialog(open: RwSignal<bool>, id: i64, nummer: String, count: usize) -> impl IntoView {
    let delete = ServerAction::<DeleteOrder>::new();
    let error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    let also = match count {
        0 => String::new(),
        1 => " Dabei wird auch 1 Position gelöscht.".to_string(),
        n => format!(" Dabei werden auch {n} Positionen gelöscht."),
    };
    view! {
        <div class="modal max-sm:modal-bottom" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Einkaufsauftrag löschen?"</h3>
                <p class="py-4">
                    {format!("Soll {nummer} wirklich gelöscht werden?{also} Das kann nicht rückgängig gemacht werden.")}
                </p>
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| open.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
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

/// Einkaufsaufträge eines Projekts in dessen Detailansicht (AK-22).
#[component]
pub fn ProjectOrders(project_id: i64) -> impl IntoView {
    let data = Resource::new(
        move || (project_id, data_version()),
        |(id, _)| list_project_orders(id),
    );
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Einkaufsaufträge"</h2>
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match data.await {
                            Ok(rows) if rows.is_empty() => view! {
                                <p class="text-base-content/60">"Noch keine Einkaufsaufträge für dieses Projekt."</p>
                            }.into_any(),
                            Ok(rows) => view! {
                                <div class="overflow-x-auto">
                                    <table class="table">
                                        <thead>
                                            <tr>
                                                <th>"Nummer"</th>
                                                <th>"Lieferant"</th>
                                                <th>"Bestelldatum"</th>
                                                <th>"Liefertermin"</th>
                                                <th>"Status"</th>
                                                <th class="text-right">"Gesamtbetrag"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rows.into_iter().map(|row| view! { <OrderTableRow row show_project=false/> }).collect_view()}
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogs_from_query() {
        let s = |v: &str| Some(v.to_string());
        assert_eq!(
            parse_dialog(None, s("1"), None, None),
            Some(Dialog::EditOrder)
        );
        assert_eq!(
            parse_dialog(s("position"), None, None, None),
            Some(Dialog::NewPosition)
        );
        assert_eq!(
            parse_dialog(None, s("position"), s("7"), None),
            Some(Dialog::EditPosition(7))
        );
        assert_eq!(parse_dialog(None, s("position"), None, None), None);
        assert_eq!(
            parse_dialog(None, None, None, s("3")),
            Some(Dialog::Delivery(3))
        );
        assert_eq!(parse_dialog(s("1"), None, None, None), None);
    }
}
