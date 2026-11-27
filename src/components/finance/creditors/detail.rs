//! Detailansicht einer Eingangsrechnung bzw. Lieferantengutschrift (ANF-019): Kopfdaten, Positionen mit
//! Kontrollbetrag (AK-7), Rundung ausgleichen, Buchen (AK-10, AK-11), Stornieren (AK-14), Löschen (AK-13),
//! „nicht gefunden“ (AK-16).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use super::form::{
    CreditorEditModal, CreditorPositionCreateModal, CreditorPositionEditModal, kind_title,
};
use super::list::DiscountDate;
use crate::backend::finance::creditors::model::{
    CREDITORS_URL, CreditorDetail, CreditorInvoice, CreditorPosition, CreditorTotals,
};
use crate::backend::finance::creditors::server_fns::{
    BalanceRounding, BookCreditor, DeleteCreditor, DeleteCreditorPosition, MoveCreditorPosition,
    StornoCreditor, get_creditor,
};
use crate::backend::finance::debtors::model::{money, rate_text, status_badge, status_label};
use crate::backend::finance::vat::model::percent_label;
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::{format_amount, format_trimmed};
use crate::components::finance::FinanceTabs;
use crate::components::finance::debtors::DueDate;
use crate::forms::{Breadcrumbs, data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

#[derive(Clone, Copy)]
struct Actions {
    book: ServerAction<BookCreditor>,
    storno: ServerAction<StornoCreditor>,
    rounding: ServerAction<BalanceRounding>,
    delete_position: ServerAction<DeleteCreditorPosition>,
    move_position: ServerAction<MoveCreditorPosition>,
}

impl Actions {
    fn new() -> Self {
        Self {
            book: ServerAction::new(),
            storno: ServerAction::new(),
            rounding: ServerAction::new(),
            delete_position: ServerAction::new(),
            move_position: ServerAction::new(),
        }
    }

    fn versions(&self) -> [usize; 5] {
        [
            self.book.version().get(),
            self.storno.version().get(),
            self.rounding.version().get(),
            self.delete_position.version().get(),
            self.move_position.version().get(),
        ]
    }

    fn error(&self) -> Option<String> {
        fn err<T>(v: Option<Result<T, ServerFnError>>) -> Option<String> {
            v.and_then(|r| r.err()).map(|e| error_message(&e))
        }
        err(self.rounding.value().get())
            .or_else(|| err(self.delete_position.value().get()))
            .or_else(|| err(self.move_position.value().get()))
    }
}

#[component]
pub fn CreditorDetailPage() -> impl IntoView {
    let params = use_params_map();
    let actions = Actions::new();
    provide_context(actions);
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let data = Resource::new(
        move || (id(), actions.versions(), data_version()),
        |(id, _, _)| async move {
            match id {
                Some(id) => get_creditor(id).await,
                None => Ok(None),
            }
        },
    );
    view! {
        <div class="space-y-4">
            <FinanceTabs active="kreditoren"/>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(detail)) => view! { <CreditorView detail/> }.into_any(),
                        Ok(None) => view! { <NotFound/> }.into_any(),
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

/// AK-16
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="Rechnung nicht gefunden – uniqwork"/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">"Rechnung nicht gefunden"</h1>
                <p class="text-base-content/70">"Diese Eingangsrechnung gibt es nicht oder der Entwurf wurde gelöscht."</p>
                <div class="card-actions">
                    <a href=CREDITORS_URL class="btn">"Zu den Kreditoren"</a>
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Dialog {
    EditHead,
    NewPosition,
    EditPosition(i64),
}

fn parse_dialog(
    neu: Option<String>,
    bearbeiten: Option<String>,
    id: Option<String>,
) -> Option<Dialog> {
    if neu.as_deref() == Some("position") {
        return Some(Dialog::NewPosition);
    }
    match bearbeiten.as_deref() {
        Some("position") => id?.parse().ok().map(Dialog::EditPosition),
        Some(_) => Some(Dialog::EditHead),
        None => None,
    }
}

#[component]
fn CreditorView(detail: CreditorDetail) -> impl IntoView {
    let actions = expect_context::<Actions>();
    let query = use_query_map();
    let base = use_base_href();
    let totals = detail.totals();
    let invoice = detail.invoice.clone();
    let positions = detail.positions.clone();
    let id = invoice.id;
    let label = invoice.label();
    let book_open = RwSignal::new(false);
    let storno_open = RwSignal::new(false);
    let delete_open = RwSignal::new(false);

    let dialog = Memo::new(move |_| {
        let q = query.read();
        parse_dialog(q.get("neu"), q.get("bearbeiten"), q.get("id"))
    });
    let saved = move || query.read().get("gespeichert").is_some();
    let edit_href = move || with_params(&base.get(), &[("bearbeiten", "1".into())]);

    let is_draft = invoice.is_draft();
    // AK-7: Buchen nur mit Positionen, Rechnungsnummer und Differenz 0.00
    let blocker = if !is_draft {
        None
    } else if positions.is_empty() {
        Some("Zuerst mindestens eine Position erfassen")
    } else if invoice.rechnungsnummer.is_none() {
        Some("Zuerst die Rechnungsnummer des Lieferanten eingeben")
    } else if !totals.is_balanced() {
        Some("Der Betrag gemäss Rechnung stimmt nicht mit dem Total überein")
    } else {
        None
    };
    let roundable = is_draft.then(|| totals.roundable()).flatten();
    let computed_total = totals.total;
    let modal_invoice = invoice.clone();
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
            Dialog::EditHead if modal_invoice.is_editable() => Some(
                view! { <CreditorEditModal invoice=modal_invoice.clone() computed_total/> }.into_any(),
            ),
            Dialog::NewPosition if modal_invoice.is_draft() => Some(
                view! { <CreditorPositionCreateModal rechnung_id=modal_invoice.id label=modal_invoice.label()/> }
                    .into_any(),
            ),
            Dialog::EditPosition(pid) if modal_invoice.is_draft() => find(pid).map(|(position, number)| {
                view! { <CreditorPositionEditModal rechnung_id=modal_invoice.id position number/> }.into_any()
            }),
            _ => None,
        }
    };

    view! {
        <Title text=format!("{label} – Kreditoren – uniqwork")/>
        <Breadcrumbs items=vec![
            ("Finanzen".to_string(), Some("/finanzen".to_string())),
            ("Kreditoren".to_string(), Some(CREDITORS_URL.to_string())),
            (label.clone(), None),
        ]/>
        {move || saved().then(|| view! { <div role="alert" class="alert alert-success">"Gespeichert."</div> })}
        {move || actions.error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}

        <div class="flex flex-wrap items-start justify-between gap-2">
            <div>
                <h1 class="text-2xl font-semibold">{label.clone()}</h1>
                <p class="flex flex-wrap items-center gap-2 text-base-content/70">
                    {kind_title(&invoice.art)} " · " {invoice.lieferant.clone()}
                    {invoice.rechnungsnummer.clone().map(|n| format!(" · Rechnung {n}"))}
                    <span class=status_badge(&invoice.status)>{status_label(&invoice.status)}</span>
                </p>
            </div>
            <div class="flex flex-wrap gap-2">
                {is_draft.then(|| view! {
                    <button
                        type="button"
                        class="btn btn-primary"
                        disabled=blocker.is_some()
                        title=blocker
                        on:click=move |_| book_open.set(true)
                    >
                        "Buchen"
                    </button>
                })}
                // AK-7: Rundungsdifferenz bis 0.05 ausgleichen
                {roundable.map(|r| view! {
                    <button
                        type="button"
                        class="btn btn-outline"
                        disabled=move || actions.rounding.pending().get()
                        on:click=move |_| { actions.rounding.dispatch(BalanceRounding { id }); }
                    >
                        {format!("Rundung ausgleichen ({})", format_amount(r))}
                    </button>
                })}
                {invoice.is_editable().then(|| view! {
                    <a href=edit_href class="btn btn-outline">
                        <IconView icon=Icon::Pencil/>
                        "Bearbeiten"
                    </a>
                })}
                {invoice.can_storno().then(|| view! {
                    <button type="button" class="btn btn-outline btn-warning" on:click=move |_| storno_open.set(true)>
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

        {blocker.map(|b| view! { <div role="status" class="alert alert-info alert-soft">{format!("{b}, dann lässt sich der Beleg buchen.")}</div> })}

        <CreditorInfo invoice=invoice.clone()/>
        <PositionsCard invoice=invoice.clone() positions totals/>

        <p class="text-sm text-base-content/60">
            "Erstellt am " {invoice.erstellt_am.clone()} " · Geändert am " {invoice.geaendert_am.clone()}
        </p>

        {modals}
        <ConfirmDialog
            open=book_open
            title="Beleg buchen?"
            text="Der Beleg erhält eine interne Nummer und wird im Journal gebucht. Danach sind Lieferant, Positionen und Beträge gesperrt.".to_string()
            button="Buchen"
            button_class="btn btn-primary"
            pending=actions.book.pending().into()
            error=Signal::derive(move || actions.book.value().get().and_then(|r| r.err()).map(|e| error_message(&e)))
            done=Signal::derive(move || matches!(actions.book.value().get(), Some(Ok(()))))
            on_confirm=Callback::new(move |_| { actions.book.dispatch(BookCreditor { id }); })
        />
        <ConfirmDialog
            open=storno_open
            title="Beleg stornieren?"
            text=format!("{label} wird storniert. Im Journal entsteht eine Gegenbuchung; der Beleg bleibt erhalten.")
            button="Stornieren"
            button_class="btn btn-warning"
            pending=actions.storno.pending().into()
            error=Signal::derive(move || actions.storno.value().get().and_then(|r| r.err()).map(|e| error_message(&e)))
            done=Signal::derive(move || matches!(actions.storno.value().get(), Some(Ok(()))))
            on_confirm=Callback::new(move |_| { actions.storno.dispatch(StornoCreditor { id }); })
        />
        <DeleteDialog open=delete_open id/>
    }
}

#[component]
fn CreditorInfo(invoice: CreditorInvoice) -> impl IntoView {
    let i = invoice;
    let foreign = i.waehrung != "CHF";
    let draft = i.is_draft();
    let link = |href: Option<String>, text: Option<String>| match (href, text) {
        (Some(h), Some(t)) => view! { <a href=h class="link link-primary">{t}</a> }.into_any(),
        _ => view! { "–" }.into_any(),
    };
    let project = link(
        i.projekt_id.map(|p| format!("/projekte/{p}")),
        i.projekt_nummer
            .clone()
            .map(|n| format!("{n} {}", i.projekt_name.clone().unwrap_or_default())),
    );
    let order = link(
        i.einkaufsauftrag_id.map(|e| format!("/einkauf/{e}")),
        i.einkaufsauftrag_nummer.clone(),
    );
    let due = if draft && i.faelligkeit.is_none() {
        view! { <span class="text-base-content/60">"aus der Zahlungsbedingung beim Buchen"</span> }
            .into_any()
    } else {
        view! { <DueDate faelligkeit=i.faelligkeit.clone() tage=i.tage_ueberfaellig/> }.into_any()
    };
    let skonto = match (&i.skonto_bis, i.skonto_prozent) {
        (Some(_), Some(p)) => view! {
            {format!("{} % bis ", format_trimmed(p, 2))}
            <DiscountDate skonto_bis=i.skonto_bis.clone() laeuft=i.skonto_laeuft/>
        }
        .into_any(),
        _ => view! { "–" }.into_any(),
    };
    let rate = foreign.then(|| match i.kurs {
        Some(k) => format!(" · Kurs {}", rate_text(k)),
        None => " · Kurs am Rechnungsdatum beim Buchen".to_string(),
    });
    let chf = i
        .gesamtbetrag_chf
        .filter(|_| foreign)
        .map(|chf| view! { <Info label="Gesamtbetrag in CHF">{money("CHF", chf)}</Info> });
    let open = (!draft).then(|| {
        let text = money(&i.waehrung, i.offener_betrag);
        view! { <Info label="Offener Betrag">{text}</Info> }
    });
    let booking = link(
        i.buchung_id.map(|b| format!("/finanzen/journal/{b}")),
        i.buchung_id.map(|_| "Im Journal anzeigen".to_string()),
    );
    let reversal = i.storno_buchung_id.map(|s| {
        view! { " · " <a href=format!("/finanzen/journal/{s}") class="link link-primary">"Gegenbuchung"</a> }
    });
    let text = |v: &Option<String>| v.clone().unwrap_or_else(|| "–".into());
    view! {
        <div class="grid gap-4 md:grid-cols-3">
            <InfoCard>
                <Info label="Lieferant">
                    <a href=format!("/adressen/{}", i.adresse_id) class="link link-primary">{i.lieferant.clone()}</a>
                    <span class="text-sm text-base-content/60">" · " {i.lieferant_nummer.clone()}</span>
                </Info>
                <Info label="Rechnungsnummer des Lieferanten">{text(&i.rechnungsnummer)}</Info>
                <Info label="Einkaufsauftrag">{order}</Info>
                <Info label="Projekt">{project}</Info>
            </InfoCard>
            <InfoCard>
                <Info label="Rechnungsdatum">{format_date(Some(&i.rechnungsdatum))}</Info>
                <Info label="Fälligkeit">{due}</Info>
                <Info label="Zahlungsbedingung">{i.zahlungsbedingung.clone()}</Info>
                <Info label="Skonto">{skonto}</Info>
            </InfoCard>
            <InfoCard>
                <Info label="Währung">{i.waehrung.clone()} {rate}</Info>
                {chf}
                {open}
                <Info label="Zahlungsreferenz">{text(&i.zahlungsreferenz)}</Info>
                <Info label="Buchung">{booking} {reversal}</Info>
            </InfoCard>
        </div>
        {i.bemerkung.clone().map(|b| view! {
            <InfoCard>
                <Info label="Bemerkung"><span class="whitespace-pre-line">{b}</span></Info>
            </InfoCard>
        })}
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

/// Positionen mit Summen, Kontrollbetrag und Differenz (AK-6, AK-7).
#[component]
fn PositionsCard(
    invoice: CreditorInvoice,
    positions: Vec<CreditorPosition>,
    totals: CreditorTotals,
) -> impl IntoView {
    let actions = expect_context::<Actions>();
    let base = use_base_href();
    let is_draft = invoice.is_draft();
    let currency = invoice.waehrung.clone();
    let count = positions.len();
    let delete_target = RwSignal::new(None::<(i64, String)>);
    let new_href = move || with_params(&base.get(), &[("neu", "position".into())]);

    let rows = positions
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let pid = p.id;
            let edit = move || {
                with_params(
                    &base.get(),
                    &[("bearbeiten", "position".into()), ("id", pid.to_string())],
                )
            };
            let name = p.bezeichnung.clone();
            view! {
                <tr>
                    <td class="font-mono text-sm text-base-content/60">{i + 1}</td>
                    <td>
                        <div class="font-medium">{p.bezeichnung.clone()}</div>
                        <div class="text-sm text-base-content/60">{format!("{} · {}", p.konto_nummer, p.konto_bezeichnung)}</div>
                    </td>
                    <td class="whitespace-nowrap">{p.projekt_nummer.clone().unwrap_or_default()}</td>
                    <td class="whitespace-nowrap">
                        {match (p.mwst_code.clone(), p.mwst_satz) {
                            (Some(code), Some(satz)) => format!("{code} ({})", percent_label(satz)),
                            _ => "ohne".into(),
                        }}
                    </td>
                    <td class="text-right tabular-nums">{format_amount(p.netto)}</td>
                    <td class="text-right tabular-nums">{format_amount(p.mwst_betrag)}</td>
                    <td class="text-right tabular-nums font-medium">{format_amount(p.brutto())}</td>
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
                                    on:click=move |_| { actions.move_position.dispatch(MoveCreditorPosition { id: pid, nach_oben: true }); }
                                >"▲"</button>
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs join-item"
                                    title="Nach unten"
                                    aria-label="Nach unten"
                                    disabled=i + 1 == count
                                    on:click=move |_| { actions.move_position.dispatch(MoveCreditorPosition { id: pid, nach_oben: false }); }
                                >"▼"</button>
                            </span>
                        })}
                    </td>
                </tr>
            }
        })
        .collect_view();

    let label_span = 6;
    let line = |text: String, amount: i64, strong: bool| {
        view! {
            <tr class:font-semibold=strong>
                <td colspan=label_span class="text-right" class:text-base-content=strong>{text}</td>
                <td class="text-right tabular-nums">{format_amount(amount)}</td>
                <td></td>
            </tr>
        }
    };
    let vat_rows = totals
        .vat
        .iter()
        .map(|v| {
            line(
                format!(
                    "MWST {} {} auf {}",
                    v.code,
                    percent_label(v.rate),
                    format_amount(v.base)
                ),
                v.vat,
                false,
            )
        })
        .collect_view();
    let control = invoice.betrag_gemaess_rechnung;
    let difference = totals.difference;

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
                                        <th>"Bezeichnung / Aufwandskonto"</th>
                                        <th>"Projekt"</th>
                                        <th>"MWST"</th>
                                        <th class="text-right">"Netto"</th>
                                        <th class="text-right">"MWST"</th>
                                        <th class="text-right">"Brutto"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>{rows}</tbody>
                                <tfoot class="text-base text-base-content">
                                    {line("Zwischensumme (netto)".into(), totals.subtotal, false)}
                                    {vat_rows}
                                    {(totals.rounding != 0).then(|| line("Rundung".into(), totals.rounding, false))}
                                    {line(format!("Gesamtbetrag {currency}"), totals.total, true)}
                                </tfoot>
                            </table>
                        </div>
                    }.into_any()
                }}
                // AK-7: Kontrollbetrag und Differenz
                <div class="flex flex-wrap justify-end gap-x-6 gap-y-1 text-sm">
                    <span>
                        "Betrag gemäss Rechnung: "
                        <span class="font-medium tabular-nums">
                            {control.map(format_amount).unwrap_or_else(|| "nicht erfasst".into())}
                        </span>
                    </span>
                    {difference.map(|d| view! {
                        <span class:text-success=d == 0 class:text-error=d != 0>
                            "Differenz: " <span class="font-medium tabular-nums">{format_amount(d)}</span>
                        </span>
                    })}
                </div>
            </div>
        </section>
        <DeletePositionDialog target=delete_target/>
    }
}

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
                                actions.delete_position.dispatch(DeleteCreditorPosition { id });
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

#[allow(clippy::too_many_arguments)]
#[component]
fn ConfirmDialog(
    open: RwSignal<bool>,
    title: &'static str,
    text: String,
    button: &'static str,
    button_class: &'static str,
    pending: Signal<bool>,
    error: Signal<Option<String>>,
    done: Signal<bool>,
    on_confirm: Callback<()>,
) -> impl IntoView {
    Effect::new(move |_| {
        if done.get() {
            open.set(false);
        }
    });
    view! {
        <div class="modal max-sm:modal-bottom" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">{title}</h3>
                <p class="py-4">{text}</p>
                {move || error.get().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| open.set(false)>"Abbrechen"</button>
                    <button type="button" class=button_class disabled=move || pending.get() on:click=move |_| on_confirm.run(())>
                        {move || if pending.get() { "Bitte warten …" } else { button }}
                    </button>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| open.set(false)></div>
        </div>
    }
}

#[component]
fn DeleteDialog(open: RwSignal<bool>, id: i64) -> impl IntoView {
    let delete = ServerAction::<DeleteCreditor>::new();
    let error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    view! {
        <div class="modal max-sm:modal-bottom" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Entwurf löschen?"</h3>
                <p class="py-4">"Soll dieser Entwurf mit allen Positionen wirklich gelöscht werden? Das kann nicht rückgängig gemacht werden."</p>
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogs_from_query() {
        let s = |v: &str| Some(v.to_string());
        assert_eq!(parse_dialog(None, s("1"), None), Some(Dialog::EditHead));
        assert_eq!(
            parse_dialog(s("position"), None, None),
            Some(Dialog::NewPosition)
        );
        assert_eq!(
            parse_dialog(None, s("position"), s("4")),
            Some(Dialog::EditPosition(4))
        );
        assert_eq!(parse_dialog(None, None, None), None);
    }
}
