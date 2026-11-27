//! Modal-Fenster der Zahlungen (ANF-020): Zahlungseingang, Zahlungsausgang und Verrechnung mit den offenen
//! Posten des Partners (AK-1 bis AK-6, AK-11) sowie „Als bezahlt erfassen“ (AK-7).

use leptos::prelude::*;

use crate::backend::addresses::server_fns::search_addresses;
use crate::backend::finance::payments::model::{
    AllocationInput, OpenItem, PaymentFormData, PaymentInput, PickOption, SaveOutcome, item_label,
    propose,
};
use crate::backend::finance::payments::server_fns::{
    PayCreditors, SavePayment, find_document_hint, get_open_items, get_payment_form,
};
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::{format_amount, format_fixed, parse_fixed};
use crate::forms::{
    ErrorText, FieldErrors, Modal, SearchPicker, error_message, field_errors, is_saved,
    refresh_when, server_error, use_base_href,
};

/// Zahlungsfenster. `art`: eingang, ausgang oder verrechnung; `seite`: debitoren oder kreditoren;
/// `beleg`: Schlüssel eines Belegs zum Vorwählen (`r:12`, `g:5`).
#[component]
pub fn PaymentModal(
    art: &'static str,
    seite: &'static str,
    beleg: Option<String>,
) -> impl IntoView {
    let close = use_base_href();
    let title = match (art, seite) {
        ("eingang", _) => "Zahlungseingang",
        ("ausgang", _) => "Zahlungsausgang",
        (_, "kreditoren") => "Verrechnen – Kreditoren",
        _ => "Verrechnen – Debitoren",
    };
    let data = Resource::new(|| (), |_| get_payment_form());
    view! {
        <Modal title=title.to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || {
                    let beleg = beleg.clone();
                    Suspend::new(async move {
                        match data.await {
                            Ok(form) => view! { <PaymentBody art seite beleg form/> }.into_any(),
                            Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                        }
                    })
                }}
            </Transition>
        </Modal>
    }
}

#[component]
fn PaymentBody(
    art: &'static str,
    seite: &'static str,
    beleg: Option<String>,
    form: PaymentFormData,
) -> impl IntoView {
    let verrechnung = art == "verrechnung";
    let close = use_base_href();
    let save = ServerAction::<SavePayment>::new();
    refresh_when(move || is_saved(save.value().get()));
    let errors = Signal::derive(move || field_errors(save.value().get()));
    let server_msg = Signal::derive(move || server_error(save.value().get()));

    let datum = RwSignal::new(form.heute.clone());
    let bank = RwSignal::new(
        form.bankkonten
            .iter()
            .find(|b| b.standard)
            .or(form.bankkonten.first())
            .map(|b| b.id.to_string())
            .unwrap_or_default(),
    );
    let waehrung = RwSignal::new("CHF".to_string());
    let betrag = RwSignal::new(String::new());
    let text = RwSignal::new(String::new());
    let nummer = RwSignal::new(String::new());
    let lookup_msg = RwSignal::new(None::<String>);
    let partner = RwSignal::new(None::<PickOption>);
    let picker_key = RwSignal::new(0_u32);
    let items = RwSignal::new(Vec::<OpenItem>::new());
    let allocs = RwSignal::new(Vec::<AllocationInput>::new());
    // Beleg, der nach dem Laden der Posten voll vorgewählt wird (AK-2)
    let preselect = RwSignal::new(None::<String>);
    let konten = StoredValue::new(form.konten.clone());

    // Offene Posten laden, sobald der Partner wechselt
    let load = move |p: Option<PickOption>| {
        partner.set(p.clone());
        items.set(Vec::new());
        allocs.set(Vec::new());
        let Some(p) = p else { return };
        leptos::task::spawn_local(async move {
            let Ok(list) = get_open_items(seite.to_string(), p.id, verrechnung).await else {
                return;
            };
            let mut rows: Vec<AllocationInput> = list
                .iter()
                .map(|i| AllocationInput {
                    key: i.key(),
                    ..Default::default()
                })
                .collect();
            if let Some(key) = preselect.get_untracked()
                && let Some((row, item)) = rows.iter_mut().zip(&list).find(|(r, _)| r.key == key)
            {
                row.betrag = format_fixed(item.offen, 2);
                if !verrechnung && betrag.get_untracked().is_empty() {
                    betrag.set(format_fixed(item.offen, 2));
                }
                preselect.set(None);
            }
            items.set(list);
            allocs.set(rows);
        });
    };

    // Vorwahl über die Rechnungsnummer bzw. `?beleg=` (AK-2)
    let apply_hint = move |nummer: String, id: Option<i64>| {
        leptos::task::spawn_local(async move {
            match find_document_hint(seite.to_string(), nummer, id).await {
                Ok(Some(hint)) => {
                    lookup_msg.set(None);
                    waehrung.set(hint.waehrung.clone());
                    preselect.set(Some(hint.key.clone()));
                    picker_key.update(|k| *k += 1);
                    load(Some(hint.partner));
                }
                Ok(None) => lookup_msg.set(Some("Kein offener Beleg mit dieser Nummer.".into())),
                Err(e) => lookup_msg.set(Some(error_message(&e))),
            }
        });
    };
    if let Some(id) = beleg
        .as_deref()
        .and_then(crate::backend::finance::payments::model::parse_key)
        .map(|(_, id)| id)
    {
        Effect::new(move |done: Option<()>| {
            if done.is_none() {
                apply_hint(String::new(), Some(id));
            }
        });
    }

    // AK-2, AK-3: Betrag auf die Rechnungen verteilen
    let distribute = move |_| {
        let Some(total) = parse_fixed(&betrag.get_untracked(), 2) else {
            return;
        };
        let cur = waehrung.get_untracked();
        let list: Vec<OpenItem> = items
            .get_untracked()
            .into_iter()
            .filter(|i| i.waehrung == cur)
            .collect();
        let proposal = propose(&list, total, &datum.get_untracked());
        allocs.update(|rows| {
            for row in rows.iter_mut() {
                let p = proposal.iter().find(|p| p.key == row.key);
                row.betrag = p.map(|p| p.betrag.clone()).unwrap_or_default();
                row.skonto = p.map(|p| p.skonto.clone()).unwrap_or_default();
            }
        });
    };

    let amount = |s: &str| parse_fixed(s.trim(), 2).unwrap_or(0);
    let assigned = move || {
        allocs.with(|rows| {
            items.with(|its| {
                rows.iter()
                    .zip(its)
                    .map(|(r, i)| {
                        let b = amount(&r.betrag);
                        if i.is_invoice() { b } else { -b }
                    })
                    .sum::<i64>()
            })
        })
    };

    let submit = move |_| {
        save.dispatch(SavePayment {
            input: PaymentInput {
                art: art.into(),
                seite: seite.into(),
                datum: datum.get_untracked(),
                bankkonto_id: bank.get_untracked(),
                waehrung: waehrung.get_untracked(),
                betrag: betrag.get_untracked(),
                adresse_id: partner
                    .get_untracked()
                    .map(|p| p.id.to_string())
                    .unwrap_or_default(),
                text: text.get_untracked(),
                zeilen: allocs.get_untracked(),
            },
        });
    };

    let partner_label = if seite == "kreditoren" {
        "Lieferant"
    } else {
        "Kunde"
    };
    let input_class = move |field: &'static str| {
        move || {
            if errors.read().contains_key(field) {
                "input input-error w-full"
            } else {
                "input w-full"
            }
        }
    };
    let banks = form.bankkonten.clone();
    let currencies = form.waehrungen.clone();

    view! {
        <div class="flex flex-col gap-4">
            {move || server_msg.get().map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
            <div class="grid gap-x-4 md:grid-cols-4">
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Datum *"</legend>
                    <input type="date" class=input_class("datum") prop:value=move || datum.get() on:input=move |ev| datum.set(event_target_value(&ev))/>
                    <ErrorText field="datum" errors/>
                </fieldset>
                {(!verrechnung).then(|| view! {
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Bankkonto *"</legend>
                        <select class="select w-full" on:change=move |ev| bank.set(event_target_value(&ev))>
                            {banks.iter().map(|b| {
                                let id = b.id.to_string();
                                let selected = bank.get_untracked() == id;
                                view! { <option value=id selected=selected>{b.bezeichnung.clone()}</option> }
                            }).collect_view()}
                        </select>
                        <ErrorText field="bankkonto_id" errors/>
                    </fieldset>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Betrag *"</legend>
                        <input type="text" inputmode="decimal" maxlength="20" class=input_class("betrag") prop:value=move || betrag.get() on:input=move |ev| betrag.set(event_target_value(&ev))/>
                        <ErrorText field="betrag" errors/>
                    </fieldset>
                })}
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Währung *"</legend>
                    <select class="select w-full" prop:value=move || waehrung.get() on:change=move |ev| waehrung.set(event_target_value(&ev))>
                        {currencies.iter().map(|c| view! { <option value=c.clone()>{c.clone()}</option> }).collect_view()}
                    </select>
                    <ErrorText field="waehrung" errors/>
                </fieldset>
            </div>
            <div class="grid gap-x-4 md:grid-cols-2">
                {move || {
                    picker_key.track();
                    view! {
                        <SearchPicker
                            field="adresse_id"
                            label=partner_label
                            placeholder="Adresse suchen (Name, Nummer, Ort) …"
                            initial=partner.get_untracked()
                            errors
                            search=search_addresses
                            required=true
                            on_change=Callback::new(load)
                        />
                    }
                }}
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"oder Rechnungsnummer"</legend>
                    <div class="join w-full">
                        <input
                            type="text"
                            class="input join-item w-full"
                            placeholder=if seite == "kreditoren" { "K-2026-0001 oder Nummer des Lieferanten" } else { "R-2026-0001" }
                            prop:value=move || nummer.get()
                            on:input=move |ev| nummer.set(event_target_value(&ev))
                            on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default(); apply_hint(nummer.get_untracked(), None); }
                        />
                        <button type="button" class="btn join-item" on:click=move |_| apply_hint(nummer.get_untracked(), None)>"Übernehmen"</button>
                    </div>
                    {move || lookup_msg.get().map(|m| view! { <p class="label text-warning">{m}</p> })}
                </fieldset>
            </div>

            // Offene Posten des Partners
            {move || {
                let list = items.get();
                if partner.read().is_none() {
                    return view! { <p class="text-base-content/60">"Bitte zuerst den Partner wählen."</p> }.into_any();
                }
                if list.is_empty() {
                    return view! { <p class="text-base-content/60">"Keine offenen Posten."</p> }.into_any();
                }
                let konten = konten.get_value();
                view! {
                    <div class="flex flex-wrap items-center justify-between gap-2">
                        <h4 class="font-semibold">"Offene Posten"</h4>
                        {(!verrechnung).then(|| view! {
                            <button type="button" class="btn btn-sm btn-outline" on:click=distribute>"Betrag verteilen"</button>
                        })}
                    </div>
                    <div class="overflow-x-auto">
                        <table class="table table-sm">
                            <thead>
                                <tr>
                                    <th>"Beleg"</th>
                                    <th>"Fällig"</th>
                                    <th class="text-right">"Offen"</th>
                                    <th>"Betrag"</th>
                                    {(!verrechnung).then(|| view! { <th>"Skonto"</th><th>"Differenz ausbuchen"</th> })}
                                </tr>
                            </thead>
                            <tbody>
                                {list.into_iter().enumerate().map(|(i, item)| {
                                    view! { <ItemRow i item allocs datum waehrung verrechnung konten=konten.clone() errors/> }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>
                }.into_any()
            }}
            <ErrorText field="zeilen" errors/>

            // Zusammenfassung: zugeordnet und Vorauszahlung (AK-5)
            {move || {
                let zugeordnet = assigned();
                if verrechnung {
                    return view! {
                        <p class="text-sm" class:text-success=zugeordnet == 0 class:text-warning=zugeordnet != 0>
                            {format!("Rechnungen minus Gutschriften/Vorauszahlungen: {}", format_amount(zugeordnet))}
                        </p>
                    }.into_any();
                }
                let total = parse_fixed(betrag.get().trim(), 2).unwrap_or(0);
                let rest = total - zugeordnet;
                view! {
                    <p class="text-sm">
                        {format!("Zugeordnet {} · ", format_amount(zugeordnet))}
                        {if rest > 0 {
                            view! { <span class="text-secondary font-medium">{format!("Vorauszahlung {}", format_amount(rest))}</span> }.into_any()
                        } else if rest < 0 {
                            view! { <span class="text-error font-medium">{format!("{} mehr zugeordnet als bezahlt", format_amount(-rest))}</span> }.into_any()
                        } else {
                            view! { <span class="text-success">"vollständig zugeordnet"</span> }.into_any()
                        }}
                    </p>
                }.into_any()
            }}

            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Text"</legend>
                <input type="text" maxlength="200" class="input w-full" prop:value=move || text.get() on:input=move |ev| text.set(event_target_value(&ev))/>
                <ErrorText field="text" errors/>
            </fieldset>
            <p class="text-sm text-base-content/60">"* Pflichtfeld"</p>
            <div class="modal-action mt-0">
                <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                <button type="button" class="btn btn-primary" disabled=move || save.pending().get() on:click=submit>
                    {move || if save.pending().get() { "Speichern …" } else if verrechnung { "Verrechnen" } else { "Zahlung buchen" }}
                </button>
            </div>
        </div>
    }
}

/// Eine Zeile der offenen Posten mit Eingaben für Betrag, Skonto und Differenz.
#[allow(clippy::too_many_arguments)]
#[component]
fn ItemRow(
    i: usize,
    item: OpenItem,
    allocs: RwSignal<Vec<AllocationInput>>,
    datum: RwSignal<String>,
    waehrung: RwSignal<String>,
    verrechnung: bool,
    konten: Vec<crate::backend::finance::payments::model::AccountChoice>,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    let key = item.key();
    let field = move |name: &str| format!("{key}_{name}");
    let (fb, fs, fd) = (field("betrag"), field("skonto"), field("differenz"));
    let get = move |f: fn(&AllocationInput) -> String| {
        move || allocs.with(|a| a.get(i).map(f).unwrap_or_default())
    };
    let update = move |f: fn(&mut AllocationInput, String), v: String| {
        allocs.update(|a| {
            if let Some(row) = a.get_mut(i) {
                f(row, v);
            }
        });
    };
    let offen = item.offen;
    let other_currency = item.waehrung.clone();
    let enabled = Memo::new(move |_| waehrung.get() == other_currency);
    let discount = {
        let item = item.clone();
        move || item.discount_on(&datum.get())
    };
    let invoice = item.is_invoice();
    let error_of = move |f: String| {
        move || {
            errors
                .read()
                .get(&f)
                .cloned()
                .map(|m| view! { <p class="label text-error">{m}</p> })
        }
    };
    view! {
        <tr class:opacity-50=move || !enabled.get()>
            <td>
                <div class="font-medium">{format!("{} {}", item_label(&item.art), item.nummer)}</div>
                <div class="text-xs text-base-content/60">
                    {format_date(Some(&item.datum))}
                    {item.referenz.clone().map(|r| format!(" · {r}"))}
                    {(item.waehrung != "CHF").then(|| format!(" · {}", item.waehrung))}
                </div>
            </td>
            <td class="whitespace-nowrap">{format_date(item.faelligkeit.as_deref())}</td>
            <td class="text-right tabular-nums">
                {format_amount(if invoice { offen } else { -offen })}
            </td>
            <td class="min-w-32">
                <div class="join">
                    <input type="text" inputmode="decimal" class="input input-sm join-item w-28" disabled=move || !enabled.get() prop:value=get(|a| a.betrag.clone()) on:input=move |ev| update(|a, v| a.betrag = v, event_target_value(&ev))/>
                    <button type="button" class="btn btn-sm join-item" title="Offenen Betrag übernehmen" disabled=move || !enabled.get()
                        on:click=move |_| allocs.update(|a| if let Some(r) = a.get_mut(i) { r.betrag = format_fixed(offen, 2); r.skonto.clear(); r.differenz.clear(); })>
                        "voll"
                    </button>
                </div>
                {error_of(fb)}
            </td>
            {(!verrechnung).then(|| view! {
                <td class="min-w-32">
                    {invoice.then(|| view! {
                        <input type="text" inputmode="decimal" class="input input-sm w-24" disabled=move || !enabled.get() prop:value=get(|a| a.skonto.clone()) on:input=move |ev| update(|a, v| a.skonto = v, event_target_value(&ev))/>
                        {move || discount().map(|s| view! { <p class="label text-secondary">{format!("Skonto bis {}", format_amount(s))}</p> })}
                    })}
                    {error_of(fs)}
                </td>
                <td class="min-w-56">
                    {invoice.then(|| view! {
                        <div class="flex gap-1">
                            <input type="text" inputmode="decimal" class="input input-sm w-24" disabled=move || !enabled.get() prop:value=get(|a| a.differenz.clone()) on:input=move |ev| update(|a, v| a.differenz = v, event_target_value(&ev))/>
                            <select class="select select-sm w-40" disabled=move || !enabled.get() on:change=move |ev| update(|a, v| a.differenz_konto_id = v, event_target_value(&ev))>
                                <option value="">"Konto …"</option>
                                {konten.iter().map(|k| view! { <option value=k.id.to_string()>{format!("{} · {}", k.nummer, k.bezeichnung)}</option> }).collect_view()}
                            </select>
                        </div>
                    })}
                    {error_of(fd)}
                </td>
            })}
        </tr>
    }
}

/// „Als bezahlt erfassen“ (AK-7) für markierte Kreditorenrechnungen (`ids` durch Kommas getrennt).
#[component]
pub fn PayCreditorsModal(ids: String) -> impl IntoView {
    let action = ServerAction::<PayCreditors>::new();
    refresh_when(move || matches!(action.value().get(), Some(Ok(SaveOutcome::Saved(_)))));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let server_msg = Signal::derive(move || server_error(action.value().get()));
    let anzahl = ids.split(',').filter(|s| !s.trim().is_empty()).count();
    let data = Resource::new(|| (), |_| get_payment_form());
    view! {
        <Modal title="Als bezahlt erfassen".to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || {
                    let ids = ids.clone();
                    Suspend::new(async move {
                        match data.await {
                            Ok(form) => view! {
                                <ActionForm action=action>
                                    <input type="hidden" name="ids" value=ids/>
                                    <div class="flex flex-col gap-4">
                                        {move || server_msg.get().map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
                                        <p>{format!("Für {anzahl} markierte Rechnungen wird je ein Zahlungsausgang über den offenen Betrag gebucht; läuft die Skontofrist am Zahlungsdatum noch, mit Skontoabzug.")}</p>
                                        <div class="grid gap-x-4 md:grid-cols-2">
                                            <fieldset class="fieldset">
                                                <legend class="fieldset-legend">"Zahlungsdatum *"</legend>
                                                <input type="date" name="datum" class="input w-full" value=form.heute.clone()/>
                                                <ErrorText field="datum" errors/>
                                            </fieldset>
                                            <fieldset class="fieldset">
                                                <legend class="fieldset-legend">"Bankkonto *"</legend>
                                                <select name="bankkonto_id" class="select w-full">
                                                    {form.bankkonten.iter().map(|b| view! { <option value=b.id.to_string() selected=b.standard>{b.bezeichnung.clone()}</option> }).collect_view()}
                                                </select>
                                                <ErrorText field="bankkonto_id" errors/>
                                            </fieldset>
                                        </div>
                                        <ErrorText field="zeilen" errors/>
                                        <div class="modal-action mt-0">
                                            <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                                            <button type="submit" class="btn btn-primary" disabled=move || action.pending().get()>"Als bezahlt erfassen"</button>
                                        </div>
                                    </div>
                                </ActionForm>
                            }.into_any(),
                            Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                        }
                    })
                }}
            </Transition>
        </Modal>
    }
}
