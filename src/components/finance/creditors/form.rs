//! Modal-Fenster der Kreditoren: Kopfdaten (ANF-019 AK-4, AK-5, AK-7, AK-12) und Positionen (AK-6).

use leptos::prelude::*;

use crate::backend::addresses::server_fns::search_addresses;
use crate::backend::finance::creditors::model::{
    AMOUNT_KINDS, CreditorFormData, CreditorInput, CreditorInvoice, CreditorPosition, FieldErrors,
    PickOption, PositionFormData, PositionInput, split_amount,
};
use crate::backend::finance::creditors::server_fns::{
    CreateCreditor, CreateCreditorPosition, UpdateCreditor, UpdateCreditorPosition,
    get_creditor_form, get_creditor_position_form, search_purchase_orders,
};
use crate::backend::finance::currency::server_fns::list_currencies;
use crate::backend::finance::debtors::model::TermOption;
use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::vat::model::percent_label;
use crate::backend::projects::server_fns::search_projects;
use crate::backend::purchase_orders::model::{format_amount, parse_fixed};
use crate::forms::{
    ErrorText, Modal, OptionField, SearchPicker, SelectField, TextField, error_message,
    field_errors, is_saved, refresh_when, server_error, use_base_href,
};

#[component]
fn FormAlerts(errors: Signal<FieldErrors>, server_error: Signal<Option<String>>) -> impl IntoView {
    view! {
        {move || server_error.get().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
        {move || (!errors.read().is_empty()).then(|| view! {
            <div role="alert" class="alert alert-warning">"Bitte die markierten Felder korrigieren."</div>
        })}
    }
}

#[component]
fn FormActions(cancel_href: Signal<String>, pending: Signal<bool>) -> impl IntoView {
    view! {
        <p class="text-sm text-base-content/60">"* Pflichtfeld"</p>
        <div class="modal-action mt-0">
            <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
            <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                {move || if pending.get() { "Speichern …" } else { "Speichern" }}
            </button>
        </div>
    }
}

#[component]
fn Remark(value: String, errors: Signal<FieldErrors>) -> impl IntoView {
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Bemerkung"</legend>
            <textarea
                name="input[bemerkung]"
                maxlength="2000"
                class="textarea h-24 w-full"
                class:textarea-error=move || errors.read().contains_key("bemerkung")
            >
                {value}
            </textarea>
            <ErrorText field="bemerkung" errors/>
        </fieldset>
    }
}

/// Lädt die Vorgaben (heute, Zahlungsbedingungen) und zeigt dann das Formular.
#[component]
fn WithFormData<F, V>(render: F) -> impl IntoView
where
    F: Fn(CreditorFormData) -> V + Send + Sync + 'static,
    V: IntoView + 'static,
{
    let data = Resource::new(|| (), |_| get_creditor_form());
    let render = std::sync::Arc::new(render);
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || {
                let render = render.clone();
                Suspend::new(async move {
                    match data.await {
                        Ok(d) => render(d).into_any(),
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })
            }}
        </Transition>
    }
}

/// „Neue Eingangsrechnung“ bzw. „Neue Gutschrift“ (AK-4).
#[component]
pub fn CreditorCreateModal(art: String) -> impl IntoView {
    let action = ServerAction::<CreateCreditor>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let title = if art == "gutschrift" {
        "Neue Gutschrift eines Lieferanten".to_string()
    } else {
        "Neue Eingangsrechnung".to_string()
    };
    view! {
        <Modal title close_href=close.into()>
            <ActionForm action=action>
                <WithFormData render=move |data: CreditorFormData| view! {
                    <CreditorFormBody
                        initial=CreditorInput::new_default(&art, &data.heute)
                        supplier=None
                        order=None
                        project=None
                        restricted=false
                        terms=data.terms
                        current_term=None
                        computed_total=None
                        errors=Signal::derive(move || field_errors(action.value().get()))
                        server_error=Signal::derive(move || server_error(action.value().get()))
                        pending=action.pending().into()
                        cancel_href=close.into()
                    />
                }/>
            </ActionForm>
        </Modal>
    }
}

/// Kopfdaten bearbeiten; gebucht nur Fälligkeit und Bemerkung (AK-12). `computed_total` zeigt die Differenz
/// zum Kontrollbetrag (AK-7).
#[component]
pub fn CreditorEditModal(invoice: CreditorInvoice, computed_total: i64) -> impl IntoView {
    let action = ServerAction::<UpdateCreditor>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let title = format!("{} bearbeiten", invoice.label());
    let id = invoice.id;
    view! {
        <Modal title close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=id/>
                <WithFormData render=move |data: CreditorFormData| {
                    let i = invoice.clone();
                    view! {
                        <CreditorFormBody
                            initial=i.to_input()
                            supplier=Some(i.supplier())
                            order=i.order()
                            project=i.project()
                            restricted=!i.is_draft()
                            terms=data.terms
                            current_term=Some((i.zahlungsbedingung_id, i.zahlungsbedingung.clone()))
                            computed_total=Some(computed_total)
                            errors=Signal::derive(move || field_errors(action.value().get()))
                            server_error=Signal::derive(move || server_error(action.value().get()))
                            pending=action.pending().into()
                            cancel_href=close.into()
                        />
                    }
                }/>
            </ActionForm>
        </Modal>
    }
}

#[component]
fn CurrencySelect(current: String, errors: Signal<FieldErrors>) -> impl IntoView {
    let currencies = Resource::new(|| (), |_| list_currencies());
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || {
                let current = current.clone();
                Suspend::new(async move {
                    match currencies.await {
                        Ok(list) => {
                            let options = list
                                .into_iter()
                                .filter(|c| c.aktiv || c.code == current)
                                .map(|c| (c.code.clone(), format!("{} – {}", c.code, c.bezeichnung)))
                                .collect::<Vec<_>>();
                            view! { <OptionField field="waehrung" label="Währung *" options value=current.clone() errors/> }.into_any()
                        }
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })
            }}
        </Transition>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn CreditorFormBody(
    initial: CreditorInput,
    supplier: Option<PickOption>,
    order: Option<PickOption>,
    project: Option<PickOption>,
    restricted: bool,
    terms: Vec<TermOption>,
    current_term: Option<(i64, String)>,
    /// Berechnetes Total aus den Positionen (Bearbeiten im Entwurf)
    computed_total: Option<i64>,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let i = initial;
    if restricted {
        return view! {
            <div class="flex flex-col gap-4">
                <FormAlerts errors server_error/>
                <div role="status" class="alert alert-info alert-soft">
                    "Der Beleg ist gebucht. Lieferant, Positionen und Beträge sind gesperrt; änderbar sind nur \
                     die Fälligkeit und die Bemerkung."
                </div>
                <TextField field="faelligkeit" label="Fälligkeit" kind="date" value=i.faelligkeit.clone() errors required=true/>
                <Remark value=i.bemerkung.clone() errors/>
                <FormActions cancel_href pending/>
            </div>
        }
        .into_any();
    }

    let mut term_options = vec![(
        String::new(),
        "Vorgabe (Adresse, sonst Standard)".to_string(),
    )];
    if let Some((id, name)) = &current_term
        && !terms.iter().any(|t| t.id == *id)
    {
        term_options.push((id.to_string(), format!("{name} (inaktiv)")));
    }
    term_options.extend(terms.iter().map(|t| {
        let label = if t.standard {
            format!("{} (Standard)", t.bezeichnung)
        } else {
            t.bezeichnung.clone()
        };
        (t.id.to_string(), label)
    }));

    // AK-7: Differenz zum berechneten Total, während der Kontrollbetrag eingegeben wird
    let control = RwSignal::new(i.betrag_gemaess_rechnung.clone());
    let difference = move || {
        let total = computed_total?;
        let value = parse_fixed(&control.get(), 2)?;
        Some((total, value - total))
    };

    view! {
        <div class="flex flex-col gap-4">
            <FormAlerts errors server_error/>
            <input type="hidden" name="input[art]" value=i.art.clone()/>
            <div class="grid gap-x-4 md:grid-cols-2">
                <SearchPicker
                    field="adresse_id"
                    label="Lieferant"
                    placeholder="Adresse suchen (Name, Nummer, Ort) …"
                    initial=supplier
                    errors
                    search=search_addresses
                    required=true
                />
                <TextField field="rechnungsnummer" label="Rechnungsnummer des Lieferanten" value=i.rechnungsnummer.clone() errors required=true maxlength=50/>
            </div>
            <div class="grid gap-x-4 md:grid-cols-3">
                <TextField field="rechnungsdatum" label="Rechnungsdatum" kind="date" value=i.rechnungsdatum.clone() errors required=true/>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Betrag gemäss Rechnung *"</legend>
                    <input
                        type="text"
                        name="input[betrag_gemaess_rechnung]"
                        inputmode="decimal"
                        maxlength="20"
                        class="input w-full"
                        class:input-error=move || errors.read().contains_key("betrag_gemaess_rechnung")
                        prop:value=move || control.get()
                        on:input=move |ev| control.set(event_target_value(&ev))
                    />
                    <ErrorText field="betrag_gemaess_rechnung" errors/>
                    {move || difference().map(|(total, d)| view! {
                        <p class="label" class:text-success=d == 0 class:text-warning=d != 0>
                            {format!("Berechnet {} · Differenz {}", format_amount(total), format_amount(d))}
                        </p>
                    })}
                </fieldset>
                <CurrencySelect current=i.waehrung.clone() errors/>
            </div>
            <div class="grid gap-x-4 md:grid-cols-2">
                <SearchPicker
                    field="einkaufsauftrag_id"
                    label="Einkaufsauftrag"
                    placeholder="Auftrag suchen (Nummer, Lieferant) …"
                    initial=order
                    errors
                    search=search_purchase_orders
                />
                <SearchPicker
                    field="projekt_id"
                    label="Projekt"
                    placeholder="Projekt suchen (Nummer, Name) …"
                    initial=project
                    errors
                    search=search_projects
                />
            </div>
            <div class="grid gap-x-4 md:grid-cols-3">
                <OptionField field="zahlungsbedingung_id" label="Zahlungsbedingung" options=term_options value=i.zahlungsbedingung_id.clone() errors/>
                <TextField field="faelligkeit" label="Fälligkeit (leer = aus Zahlungsbedingung)" kind="date" value=i.faelligkeit.clone() errors/>
                <TextField field="zahlungsreferenz" label="Zahlungsreferenz" value=i.zahlungsreferenz.clone() errors maxlength=40/>
            </div>
            <Remark value=i.bemerkung.clone() errors/>
            <FormActions cancel_href pending/>
        </div>
    }
    .into_any()
}

// ---------------------------------------------------------------------------------------------
// Positionen
// ---------------------------------------------------------------------------------------------

/// „Neue Position“ (AK-6), Webadresse `?neu=position`.
#[component]
pub fn CreditorPositionCreateModal(rechnung_id: i64, label: String) -> impl IntoView {
    let action = ServerAction::<CreateCreditorPosition>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title=format!("Neue Position in {label}") close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="rechnung_id" value=rechnung_id/>
                <PositionFormLoader
                    rechnung_id
                    initial=PositionInput::new_default()
                    project=None
                    creating=true
                    errors=Signal::derive(move || field_errors(action.value().get()))
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

/// Position bearbeiten, Webadresse `?bearbeiten=position&id=<Kennung>`.
#[component]
pub fn CreditorPositionEditModal(
    rechnung_id: i64,
    position: CreditorPosition,
    number: usize,
) -> impl IntoView {
    let action = ServerAction::<UpdateCreditorPosition>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title=format!("Position {number} bearbeiten") close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=position.id/>
                <PositionFormLoader
                    rechnung_id
                    initial=position.to_input()
                    project=position.project()
                    creating=false
                    errors=Signal::derive(move || field_errors(action.value().get()))
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn PositionFormLoader(
    rechnung_id: i64,
    initial: PositionInput,
    project: Option<PickOption>,
    creating: bool,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let data = Resource::new(move || rechnung_id, get_creditor_position_form);
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || {
                let initial = initial.clone();
                let project = project.clone();
                Suspend::new(async move {
                    match data.await {
                        Ok(d) => {
                            let mut initial = initial;
                            if creating {
                                if initial.konto_id.is_empty()
                                    && let Some(id) = d.vorgabe_konto_id
                                {
                                    initial.konto_id = id.to_string();
                                }
                                if initial.mwst_code_id.is_empty() && d.mwst_pflicht {
                                    let vorgabe = d
                                        .konten
                                        .iter()
                                        .find(|k| k.id.to_string() == initial.konto_id)
                                        .and_then(|k| k.vorgabe_mwst)
                                        .filter(|c| d.codes.iter().any(|x| x.id == *c));
                                    if let Some(code) = vorgabe.or_else(|| d.codes.first().map(|c| c.id)) {
                                        initial.mwst_code_id = code.to_string();
                                    }
                                }
                            }
                            view! { <PositionFormBody initial project data=d errors server_error pending cancel_href/> }.into_any()
                        }
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })
            }}
        </Transition>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
fn PositionFormBody(
    initial: PositionInput,
    project: Option<PickOption>,
    data: PositionFormData,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let i = initial;
    let required = Signal::stored(true);
    let stale_konto =
        !i.konto_id.is_empty() && !data.konten.iter().any(|k| k.id.to_string() == i.konto_id);
    let stale_code = !i.mwst_code_id.is_empty()
        && !data
            .codes
            .iter()
            .any(|c| c.id.to_string() == i.mwst_code_id);
    let hint = if data.mwst_pflicht {
        data.codes.is_empty().then(|| {
            format!(
                "Am {} ist kein Vorsteuer-Code gültig (Einstellungen → Finanzen → MWST).",
                german_date(&data.rechnungsdatum)
            )
        })
    } else {
        Some(format!(
            "Am {} gilt keine MWST-Pflicht; die Position wird ohne MWST erfasst.",
            german_date(&data.rechnungsdatum)
        ))
    };

    // AK-6: Vorschau Netto / MWST / Brutto während der Eingabe
    let betrag = RwSignal::new(i.betrag.clone());
    let art = RwSignal::new(i.betragsart.clone());
    let code = RwSignal::new(i.mwst_code_id.clone());
    let rates: Vec<(String, i64)> = data
        .codes
        .iter()
        .map(|c| (c.id.to_string(), c.satz))
        .collect();
    let preview = move || {
        let value = parse_fixed(&betrag.get(), 2)?;
        let satz = rates
            .iter()
            .find(|(id, _)| *id == code.get())
            .map(|(_, s)| *s);
        let (netto, mwst) = split_amount(value, satz, art.get() == "brutto");
        Some(format!(
            "Netto {} · MWST {} · Brutto {}",
            format_amount(netto),
            format_amount(mwst),
            format_amount(netto + mwst)
        ))
    };

    view! {
        <div class="flex flex-col gap-4">
            <FormAlerts errors server_error/>
            <TextField field="bezeichnung" label="Bezeichnung" value=i.bezeichnung.clone() errors required maxlength=200/>
            <div class="grid gap-x-4 md:grid-cols-2">
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Aufwandskonto *"</legend>
                    <select
                        name="input[konto_id]"
                        class="select w-full"
                        class:select-error=move || errors.read().contains_key("konto_id")
                    >
                        {stale_konto.then(|| view! {
                            <option value=i.konto_id.clone() selected=true>"Bisheriges Konto – bitte neu wählen"</option>
                        })}
                        {i.konto_id.is_empty().then(|| view! { <option value="" selected=true>"Bitte wählen …"</option> })}
                        {data
                            .konten
                            .iter()
                            .map(|k| {
                                let value = k.id.to_string();
                                let selected = i.konto_id == value;
                                view! { <option value=value selected=selected>{format!("{} · {}", k.nummer, k.bezeichnung)}</option> }
                            })
                            .collect_view()}
                    </select>
                    <ErrorText field="konto_id" errors/>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{if data.mwst_pflicht { "MWST-Code *" } else { "MWST-Code" }}</legend>
                    <select
                        name="input[mwst_code_id]"
                        class="select w-full"
                        class:select-error=move || errors.read().contains_key("mwst_code_id")
                        on:change=move |ev| code.set(event_target_value(&ev))
                    >
                        {stale_code.then(|| view! {
                            <option value=i.mwst_code_id.clone() selected=true>"Bisheriger Code – bitte neu wählen"</option>
                        })}
                        {(!data.mwst_pflicht || i.mwst_code_id.is_empty()).then(|| view! {
                            <option value="" selected=i.mwst_code_id.is_empty()>
                                {if data.mwst_pflicht { "Bitte wählen …" } else { "ohne MWST" }}
                            </option>
                        })}
                        {data.mwst_pflicht.then(|| {
                            data.codes
                                .iter()
                                .map(|c| {
                                    let value = c.id.to_string();
                                    let selected = i.mwst_code_id == value;
                                    let label = format!("{} – {} ({})", c.code, c.bezeichnung, percent_label(c.satz));
                                    view! { <option value=value selected=selected>{label}</option> }
                                })
                                .collect_view()
                        })}
                    </select>
                    <ErrorText field="mwst_code_id" errors/>
                    {hint.map(|h| view! { <p class="label">{h}</p> })}
                </fieldset>
            </div>
            <div class="grid gap-x-4 md:grid-cols-3">
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Betrag *"</legend>
                    <input
                        type="text"
                        name="input[betrag]"
                        inputmode="decimal"
                        maxlength="20"
                        class="input w-full"
                        class:input-error=move || errors.read().contains_key("betrag")
                        prop:value=move || betrag.get()
                        on:input=move |ev| betrag.set(event_target_value(&ev))
                    />
                    <ErrorText field="betrag" errors/>
                    {move || preview().map(|p| view! { <p class="label tabular-nums">{p}</p> })}
                </fieldset>
                <div on:change=move |ev| art.set(event_target_value(&ev))>
                    <SelectField field="betragsart" label="Betrag ist" options=AMOUNT_KINDS value=i.betragsart.clone() errors/>
                </div>
                <SearchPicker
                    field="projekt_id"
                    label="Projekt (leer = Projekt der Rechnung)"
                    placeholder="Projekt suchen …"
                    initial=project
                    errors
                    search=search_projects
                />
            </div>
            <FormActions cancel_href pending/>
        </div>
    }
}

/// Bezeichnung der Art für Titel, z. B. „Eingangsrechnung“.
pub fn kind_title(art: &str) -> &'static str {
    if art == "gutschrift" {
        "Lieferantengutschrift"
    } else {
        "Eingangsrechnung"
    }
}
