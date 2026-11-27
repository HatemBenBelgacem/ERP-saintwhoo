//! Detailansicht einer Rechnung bzw. Gutschrift (ANF-018): Kopfdaten, Positionen mit Summen, Buchen
//! (AK-8, AK-9), Stornieren (AK-13), Gutschrift erstellen (AK-14), Löschen (AK-12), „nicht gefunden“ (AK-17).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use super::form::{InvoiceEditModal, PositionCreateModal, PositionEditModal};
use super::list::DueDate;
use crate::backend::finance::debtors::model::{
    DEBTORS_URL, Invoice, InvoiceDetail, InvoicePosition, InvoiceTotals, kind_label, money,
    rate_text, status_badge, status_label,
};
use crate::backend::finance::debtors::server_fns::{
    BookInvoice, CreateCreditNote, DeleteInvoice, DeleteInvoicePosition, MoveInvoicePosition,
    StornoInvoice, get_invoice,
};
use crate::backend::finance::vat::model::percent_label;
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::{format_amount, format_trimmed, unit_label};
use crate::components::finance::FinanceTabs;
use crate::forms::{Breadcrumbs, data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Aktionen ohne eigenes Formular; nach jeder Aktion wird der Beleg neu geladen.
#[derive(Clone, Copy)]
struct Actions {
    book: ServerAction<BookInvoice>,
    storno: ServerAction<StornoInvoice>,
    credit: ServerAction<CreateCreditNote>,
    delete_position: ServerAction<DeleteInvoicePosition>,
    move_position: ServerAction<MoveInvoicePosition>,
}

impl Actions {
    fn new() -> Self {
        Self {
            book: ServerAction::new(),
            storno: ServerAction::new(),
            credit: ServerAction::new(),
            delete_position: ServerAction::new(),
            move_position: ServerAction::new(),
        }
    }

    fn versions(&self) -> [usize; 5] {
        [
            self.book.version().get(),
            self.storno.version().get(),
            self.credit.version().get(),
            self.delete_position.version().get(),
            self.move_position.version().get(),
        ]
    }

    /// Erster Fehler der Aktionen ohne eigenen Dialog.
    fn error(&self) -> Option<String> {
        fn err<T>(v: Option<Result<T, ServerFnError>>) -> Option<String> {
            v.and_then(|r| r.err()).map(|e| error_message(&e))
        }
        err(self.credit.value().get())
            .or_else(|| err(self.delete_position.value().get()))
            .or_else(|| err(self.move_position.value().get()))
    }
}

#[component]
pub fn InvoiceDetailPage() -> impl IntoView {
    let params = use_params_map();
    let actions = Actions::new();
    provide_context(actions);
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let data = Resource::new(
        move || (id(), actions.versions(), data_version()),
        |(id, _, _)| async move {
            match id {
                Some(id) => get_invoice(id).await,
                None => Ok(None),
            }
        },
    );
    view! {
        <div class="space-y-4">
            <FinanceTabs active="debitoren"/>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(detail)) => view! { <InvoiceView detail/> }.into_any(),
                        Ok(None) => view! { <NotFound/> }.into_any(),
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

/// AK-17
#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="Rechnung nicht gefunden – uniqwork"/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">"Rechnung nicht gefunden"</h1>
                <p class="text-base-content/70">"Diese Rechnung gibt es nicht oder der Entwurf wurde gelöscht."</p>
                <div class="card-actions">
                    <a href=DEBTORS_URL class="btn">"Zu den Debitoren"</a>
                </div>
            </div>
        </div>
    }
}

/// Welches Modal-Fenster laut Webadresse offen ist.
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
fn InvoiceView(detail: InvoiceDetail) -> impl IntoView {
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
    let has_positions = !positions.is_empty();
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
            Dialog::EditHead if modal_invoice.is_editable() => {
                Some(view! { <InvoiceEditModal invoice=modal_invoice.clone()/> }.into_any())
            }
            Dialog::NewPosition if modal_invoice.is_draft() => Some(
                view! { <PositionCreateModal rechnung_id=modal_invoice.id label=modal_invoice.label()/> }
                    .into_any(),
            ),
            Dialog::EditPosition(pid) if modal_invoice.is_draft() => find(pid).map(|(position, number)| {
                view! { <PositionEditModal rechnung_id=modal_invoice.id position number/> }.into_any()
            }),
            _ => None,
        }
    };

    view! {
        <Title text=format!("{label} – Debitoren – uniqwork")/>
        <Breadcrumbs items=vec![
            ("Finanzen".to_string(), Some("/finanzen".to_string())),
            ("Debitoren".to_string(), Some(DEBTORS_URL.to_string())),
            (label.clone(), None),
        ]/>
        {move || saved().then(|| view! { <div role="alert" class="alert alert-success">"Gespeichert."</div> })}
        {move || actions.error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}

        <div class="flex flex-wrap items-start justify-between gap-2">
            <div>
                <h1 class="text-2xl font-semibold">{label.clone()}</h1>
                <p class="flex flex-wrap items-center gap-2 text-base-content/70">
                    {kind_label(&invoice.art)} " · " {invoice.kunde.clone()}
                    <span class=status_badge(&invoice.status)>{status_label(&invoice.status)}</span>
                </p>
            </div>
            <div class="flex flex-wrap gap-2">
                // AK-8: Buchen
                {is_draft.then(|| view! {
                    <button
                        type="button"
                        class="btn btn-primary"
                        disabled=!has_positions
                        title=(!has_positions).then_some("Zuerst mindestens eine Position erfassen")
                        on:click=move |_| book_open.set(true)
                    >
                        "Buchen"
                    </button>
                })}
                // AK-11: Druckansicht
                <a href=format!("{DEBTORS_URL}/{id}/druck") class="btn btn-outline" target="_blank">"Drucken"</a>
                {invoice.is_editable().then(|| view! {
                    <a href=edit_href class="btn btn-outline">
                        <IconView icon=Icon::Pencil/>
                        "Bearbeiten"
                    </a>
                })}
                // AK-14: Gutschrift aus gebuchter Rechnung
                {invoice.can_credit().then(|| view! {
                    <button
                        type="button"
                        class="btn btn-outline"
                        disabled=move || actions.credit.pending().get()
                        on:click=move |_| { actions.credit.dispatch(CreateCreditNote { id }); }
                    >
                        "Gutschrift erstellen"
                    </button>
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

        {(invoice.status == "offen" && !invoice.can_storno()).then(|| view! {
            <div role="status" class="alert alert-info alert-soft">
                "Zu diesem Beleg gibt es Zahlungen. Er lässt sich nicht stornieren; bitte eine Gutschrift erstellen."
            </div>
        })}

        <InvoiceInfo invoice=invoice.clone()/>
        <PositionsCard invoice=invoice.clone() positions totals/>

        <p class="text-sm text-base-content/60">
            "Erstellt am " {invoice.erstellt_am.clone()} " · Geändert am " {invoice.geaendert_am.clone()}
        </p>

        {modals}
        <ConfirmDialog
            open=book_open
            title="Beleg buchen?"
            text=format!(
                "{} wird mit einer fortlaufenden Nummer gebucht und im Journal erfasst. Danach sind Kunde, \
                 Positionen und Beträge gesperrt.",
                kind_label(&invoice.art)
            )
            button="Buchen"
            button_class="btn btn-primary"
            pending=actions.book.pending().into()
            error=Signal::derive(move || actions.book.value().get().and_then(|r| r.err()).map(|e| error_message(&e)))
            done=Signal::derive(move || matches!(actions.book.value().get(), Some(Ok(()))))
            on_confirm=Callback::new(move |_| { actions.book.dispatch(BookInvoice { id }); })
        />
        <ConfirmDialog
            open=storno_open
            title="Beleg stornieren?"
            text=format!(
                "{label} wird storniert. Im Journal entsteht eine Gegenbuchung; der Beleg bleibt erhalten."
            )
            button="Stornieren"
            button_class="btn btn-warning"
            pending=actions.storno.pending().into()
            error=Signal::derive(move || actions.storno.value().get().and_then(|r| r.err()).map(|e| error_message(&e)))
            done=Signal::derive(move || matches!(actions.storno.value().get(), Some(Ok(()))))
            on_confirm=Callback::new(move |_| { actions.storno.dispatch(StornoInvoice { id }); })
        />
        <DeleteDialog open=delete_open id/>
    }
}

/// Kopfdaten in drei Karten.
#[component]
fn InvoiceInfo(invoice: Invoice) -> impl IntoView {
    let i = invoice;
    let skonto = match (&i.skonto_bis, i.skonto_prozent) {
        (Some(bis), Some(p)) => {
            format!("{} % bis {}", format_trimmed(p, 2), format_date(Some(bis)))
        }
        _ => "–".into(),
    };
    let foreign = i.waehrung != "CHF";
    let draft = i.is_draft();
    let project = match (
        i.projekt_id,
        i.projekt_nummer.clone(),
        i.projekt_name.clone(),
    ) {
        (Some(pid), Some(n), Some(name)) => view! {
            <a href=format!("/projekte/{pid}") class="link link-primary">{format!("{n} {name}")}</a>
        }
        .into_any(),
        _ => view! { "–" }.into_any(),
    };
    let reference = i.bezug_rechnung_id.map(|bid| {
        let nummer = i.bezug_nummer.clone().unwrap_or_else(|| "Rechnung".into());
        view! {
            <Info label="Bezug auf Rechnung">
                <a href=format!("{DEBTORS_URL}/{bid}") class="link link-primary">{nummer}</a>
            </Info>
        }
    });
    let due = if draft {
        view! { <span class="text-base-content/60">"wird beim Buchen berechnet"</span> }.into_any()
    } else {
        view! { <DueDate faelligkeit=i.faelligkeit.clone() tage=i.tage_ueberfaellig/> }.into_any()
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
    let booking = match i.buchung_id {
        Some(b) => view! {
            <a href=format!("/finanzen/journal/{b}") class="link link-primary">"Im Journal anzeigen"</a>
        }
        .into_any(),
        None => view! { "–" }.into_any(),
    };
    let reversal = i.storno_buchung_id.map(|s| {
        view! { " · " <a href=format!("/finanzen/journal/{s}") class="link link-primary">"Gegenbuchung"</a> }
    });
    let has_texts = i.einleitungstext.is_some() || i.schlusstext.is_some() || i.bemerkung.is_some();
    let text = |v: &Option<String>| v.clone().unwrap_or_else(|| "–".into());
    let (intro, outro, remark) = (
        text(&i.einleitungstext),
        text(&i.schlusstext),
        text(&i.bemerkung),
    );
    view! {
        <div class="grid gap-4 md:grid-cols-3">
            <InfoCard>
                <Info label="Kunde">
                    <a href=format!("/adressen/{}", i.adresse_id) class="link link-primary">{i.kunde.clone()}</a>
                    <span class="text-sm text-base-content/60">" · " {i.kunde_nummer.clone()}</span>
                </Info>
                <Info label="Projekt">{project}</Info>
                <Info label="Referenz des Kunden">{text(&i.referenz_kunde)}</Info>
                {reference}
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
                <Info label="Buchung">{booking} {reversal}</Info>
            </InfoCard>
        </div>
        {has_texts.then(|| view! {
            <div class="grid gap-4 md:grid-cols-3">
                <InfoCard>
                    <Info label="Einleitungstext"><span class="whitespace-pre-line">{intro}</span></Info>
                </InfoCard>
                <InfoCard>
                    <Info label="Schlusstext"><span class="whitespace-pre-line">{outro}</span></Info>
                </InfoCard>
                <InfoCard>
                    <Info label="Bemerkung (intern)"><span class="whitespace-pre-line">{remark}</span></Info>
                </InfoCard>
            </div>
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

/// Positionen mit Summen (AK-6); im Entwurf anlegen, bearbeiten, löschen und verschieben (AK-5).
#[component]
fn PositionsCard(
    invoice: Invoice,
    positions: Vec<InvoicePosition>,
    totals: InvoiceTotals,
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
                    <td class="text-right tabular-nums whitespace-nowrap">{format_trimmed(p.menge, 3)} " " {unit_label(&p.einheit).to_string()}</td>
                    <td class="text-right tabular-nums">{format_amount(p.einzelpreis)}</td>
                    <td class="text-right tabular-nums">
                        {(p.rabatt_prozent > 0).then(|| format!("{} %", format_trimmed(p.rabatt_prozent, 2)))}
                    </td>
                    <td class="whitespace-nowrap">
                        {match (p.mwst_code.clone(), p.mwst_satz) {
                            (Some(code), Some(satz)) => format!("{code} ({})", percent_label(satz)),
                            _ => "ohne".into(),
                        }}
                    </td>
                    <td class="text-right tabular-nums font-medium">{format_amount(p.amount())}</td>
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
                                    on:click=move |_| { actions.move_position.dispatch(MoveInvoicePosition { id: pid, nach_oben: true }); }
                                >"▲"</button>
                                <button
                                    type="button"
                                    class="btn btn-ghost btn-xs join-item"
                                    title="Nach unten"
                                    aria-label="Nach unten"
                                    disabled=i + 1 == count
                                    on:click=move |_| { actions.move_position.dispatch(MoveInvoicePosition { id: pid, nach_oben: false }); }
                                >"▼"</button>
                            </span>
                        })}
                    </td>
                </tr>
            }
        })
        .collect_view();

    let label_span = 6;
    let vat_rows = totals
        .vat
        .iter()
        .map(|l| {
            view! {
                <tr>
                    <td colspan=label_span class="text-right text-base-content/70">
                        {format!("MWST {} {} auf {}", l.code, percent_label(l.rate), format_amount(l.base))}
                    </td>
                    <td class="text-right tabular-nums">{format_amount(l.vat)}</td>
                    <td></td>
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
                                        <th>"Bezeichnung / Ertragskonto"</th>
                                        <th class="text-right">"Menge"</th>
                                        <th class="text-right">"Einzelpreis"</th>
                                        <th class="text-right">"Rabatt"</th>
                                        <th>"MWST"</th>
                                        <th class="text-right">"Betrag"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>{rows}</tbody>
                                <tfoot class="text-base text-base-content">
                                    <tr>
                                        <td colspan=label_span class="text-right">"Zwischensumme"</td>
                                        <td class="text-right tabular-nums">{format_amount(totals.subtotal)}</td>
                                        <td></td>
                                    </tr>
                                    {vat_rows}
                                    {(totals.rounding != 0).then(|| view! {
                                        <tr>
                                            <td colspan=label_span class="text-right text-base-content/70">"Rundung"</td>
                                            <td class="text-right tabular-nums">{format_amount(totals.rounding)}</td>
                                            <td></td>
                                        </tr>
                                    })}
                                    <tr class="font-semibold">
                                        <td colspan=label_span class="text-right">{format!("Gesamtbetrag {currency}")}</td>
                                        <td class="text-right tabular-nums">{format_amount(totals.total)}</td>
                                        <td></td>
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
                                actions.delete_position.dispatch(DeleteInvoicePosition { id });
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

/// Bestätigung für Buchen und Stornieren. Schliesst sich nach Erfolg; Fehler bleiben im Fenster sichtbar.
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
                    <button
                        type="button"
                        class=button_class
                        disabled=move || pending.get()
                        on:click=move |_| on_confirm.run(())
                    >
                        {move || if pending.get() { "Bitte warten …" } else { button }}
                    </button>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| open.set(false)></div>
        </div>
    }
}

/// Bestätigung „Entwurf löschen?“ (AK-12).
#[component]
fn DeleteDialog(open: RwSignal<bool>, id: i64) -> impl IntoView {
    let delete = ServerAction::<DeleteInvoice>::new();
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
            parse_dialog(None, s("position"), s("7")),
            Some(Dialog::EditPosition(7))
        );
        assert_eq!(parse_dialog(None, s("position"), None), None);
        assert_eq!(parse_dialog(s("1"), None, None), None);
    }
}
