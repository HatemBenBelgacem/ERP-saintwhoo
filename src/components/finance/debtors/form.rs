//! Modal-Fenster der Debitoren: Kopfdaten (ANF-018 AK-4, AK-7, AK-10) und Positionen (AK-5).

use leptos::prelude::*;

use crate::backend::addresses::server_fns::search_addresses;
use crate::backend::finance::currency::server_fns::list_currencies;
use crate::backend::finance::debtors::model::{
    FieldErrors, Invoice, InvoiceFormData, InvoiceInput, InvoicePosition, PickOption,
    PositionFormData, PositionInput, kind_label,
};
use crate::backend::finance::debtors::server_fns::{
    CreateInvoice, CreateInvoicePosition, UpdateInvoice, UpdateInvoicePosition, get_invoice_form,
    get_position_form, get_project_customer,
};
use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::vat::model::percent_label;
use crate::backend::projects::server_fns::search_projects;
use crate::backend::purchase_orders::model::UNITS;
use crate::forms::{
    ErrorText, Modal, OptionField, SearchPicker, SelectField, TextField, error_message,
    field_errors, is_saved, refresh_when, server_error, use_base_href,
};

/// Meldungen über dem Formular (Server-Fehler, Hinweis auf markierte Felder).
#[component]
fn FormAlerts(errors: Signal<FieldErrors>, server_error: Signal<Option<String>>) -> impl IntoView {
    view! {
        {move || server_error.get().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
        {move || (!errors.read().is_empty()).then(|| view! {
            <div role="alert" class="alert alert-warning">"Bitte die markierten Felder korrigieren."</div>
        })}
    }
}

/// „Abbrechen“ und „Speichern“.
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
fn TextArea(
    field: &'static str,
    label: &'static str,
    value: String,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">{label}</legend>
            <textarea
                name=format!("input[{field}]")
                maxlength="2000"
                class="textarea h-24 w-full"
                class:textarea-error=move || errors.read().contains_key(field)
            >
                {value}
            </textarea>
            <ErrorText field errors/>
        </fieldset>
    }
}

/// Lädt die Vorgaben (heutiges Datum, Zahlungsbedingungen) und zeigt dann das Formular.
#[component]
fn WithFormData<F, V>(render: F) -> impl IntoView
where
    F: Fn(InvoiceFormData) -> V + Send + Sync + 'static,
    V: IntoView + 'static,
{
    let data = Resource::new(|| (), |_| get_invoice_form());
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

/// „Neue Rechnung“ bzw. „Neue Gutschrift“ (AK-4). Nach dem Speichern öffnet sich die Detailansicht.
#[component]
pub fn InvoiceCreateModal(art: String) -> impl IntoView {
    let action = ServerAction::<CreateInvoice>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let title = format!("Neue {}", kind_label(&art));
    view! {
        <Modal title close_href=close.into()>
            <ActionForm action=action>
                <WithFormData render=move |data: InvoiceFormData| {
                    let initial = InvoiceInput::new_default(&art, &data.heute);
                    view! {
                        <InvoiceFormBody
                            initial
                            customer=None
                            project=None
                            restricted=false
                            terms=data.terms
                            current_term=None
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

/// Kopfdaten bearbeiten; nach dem Buchen nur Referenz, Bemerkung und Texte (AK-10).
#[component]
pub fn InvoiceEditModal(invoice: Invoice) -> impl IntoView {
    let action = ServerAction::<UpdateInvoice>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let title = format!("{} bearbeiten", invoice.label());
    let id = invoice.id;
    view! {
        <Modal title close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=id/>
                <WithFormData render=move |data: InvoiceFormData| {
                    let i = invoice.clone();
                    view! {
                        <InvoiceFormBody
                            initial=i.to_input()
                            customer=Some(i.customer())
                            project=i.project()
                            restricted=!i.is_draft()
                            terms=data.terms
                            current_term=Some((i.zahlungsbedingung_id, i.zahlungsbedingung.clone()))
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

/// Währung aus den Einstellungen (ANF-014): aktive Währungen und die bisherige des Belegs.
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

/// Felder der Kopfdaten. `restricted`: nur Referenz, Bemerkung und Texte (AK-10).
#[allow(clippy::too_many_arguments)]
#[component]
fn InvoiceFormBody(
    initial: InvoiceInput,
    customer: Option<PickOption>,
    project: Option<PickOption>,
    restricted: bool,
    terms: Vec<crate::backend::finance::debtors::model::TermOption>,
    /// Bisherige Zahlungsbedingung (auch wenn sie inzwischen inaktiv ist)
    current_term: Option<(i64, String)>,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let i = initial;
    let texts = view! {
        <TextField field="referenz_kunde" label="Referenz des Kunden" value=i.referenz_kunde.clone() errors maxlength=100/>
        <div class="grid gap-x-4 md:grid-cols-2">
            <TextArea field="einleitungstext" label="Einleitungstext (Druck)" value=i.einleitungstext.clone() errors/>
            <TextArea field="schlusstext" label="Schlusstext (Druck)" value=i.schlusstext.clone() errors/>
        </div>
        <TextArea field="bemerkung" label="Bemerkung (intern)" value=i.bemerkung.clone() errors/>
    };
    if restricted {
        return view! {
            <div class="flex flex-col gap-4">
                <FormAlerts errors server_error/>
                <div role="status" class="alert alert-info alert-soft">
                    "Der Beleg ist gebucht. Kunde, Positionen und Beträge sind gesperrt; änderbar sind nur \
                     die Referenz, die Texte für den Druck und die Bemerkung."
                </div>
                {texts}
                <FormActions cancel_href pending/>
            </div>
        }
        .into_any();
    }

    // AK-4: Zahlungsbedingung – Vorgabe der Adresse bzw. Standard, oder eine aktive Bedingung
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

    // AK-7: Projekt zuerst gewählt → Kunde des Projekts vorschlagen, solange kein Kunde gewählt ist
    let chosen = RwSignal::new(customer.clone());
    let suggestion = RwSignal::new(customer);
    let picker_key = RwSignal::new(0_u32);
    let on_project = Callback::new(move |p: Option<PickOption>| {
        let Some(p) = p else { return };
        if chosen.get_untracked().is_some() {
            return;
        }
        leptos::task::spawn_local(async move {
            if let Ok(Some(kunde)) = get_project_customer(p.id).await
                && chosen.get_untracked().is_none()
            {
                chosen.set(Some(kunde.clone()));
                suggestion.set(Some(kunde));
                picker_key.update(|k| *k += 1);
            }
        });
    });
    let on_customer = Callback::new(move |c: Option<PickOption>| chosen.set(c));

    view! {
        <div class="flex flex-col gap-4">
            <FormAlerts errors server_error/>
            <input type="hidden" name="input[art]" value=i.art.clone()/>
            <div class="grid gap-x-4 md:grid-cols-2">
                {move || {
                    picker_key.track();
                    view! {
                        <SearchPicker
                            field="adresse_id"
                            label="Kunde"
                            placeholder="Adresse suchen (Name, Nummer, Ort) …"
                            initial=suggestion.get_untracked()
                            errors
                            search=search_addresses
                            required=true
                            on_change=on_customer
                        />
                    }
                }}
                <SearchPicker
                    field="projekt_id"
                    label="Projekt"
                    placeholder="Projekt suchen (Nummer, Name) …"
                    initial=project
                    errors
                    search=search_projects
                    on_change=on_project
                />
            </div>
            <div class="grid gap-x-4 md:grid-cols-3">
                <TextField field="rechnungsdatum" label="Rechnungsdatum" kind="date" value=i.rechnungsdatum.clone() errors required=true/>
                <CurrencySelect current=i.waehrung.clone() errors/>
                <OptionField field="zahlungsbedingung_id" label="Zahlungsbedingung" options=term_options value=i.zahlungsbedingung_id.clone() errors/>
            </div>
            {texts}
            <FormActions cancel_href pending/>
        </div>
    }
    .into_any()
}

// ---------------------------------------------------------------------------------------------
// Positionen
// ---------------------------------------------------------------------------------------------

/// „Neue Position“ (AK-5), Webadresse `?neu=position`.
#[component]
pub fn PositionCreateModal(rechnung_id: i64, label: String) -> impl IntoView {
    let action = ServerAction::<CreateInvoicePosition>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title=format!("Neue Position in {label}") close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="rechnung_id" value=rechnung_id/>
                <PositionFormLoader
                    rechnung_id
                    initial=PositionInput::new_default()
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

/// Position bearbeiten (AK-5), Webadresse `?bearbeiten=position&id=<Kennung>`.
#[component]
pub fn PositionEditModal(
    rechnung_id: i64,
    position: InvoicePosition,
    number: usize,
) -> impl IntoView {
    let action = ServerAction::<UpdateInvoicePosition>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title=format!("Position {number} bearbeiten") close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=position.id/>
                <PositionFormLoader
                    rechnung_id
                    initial=position.to_input()
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

/// Lädt Konten und Codes zum Rechnungsdatum; eine neue Position erhält das Vorgabe-Ertragskonto und
/// dessen MWST-Code bzw. den ersten Code (AK-5).
#[component]
fn PositionFormLoader(
    rechnung_id: i64,
    initial: PositionInput,
    creating: bool,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let data = Resource::new(move || rechnung_id, get_position_form);
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || {
                let initial = initial.clone();
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
                            view! { <PositionFormBody initial data=d errors server_error pending cancel_href/> }.into_any()
                        }
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })
            }}
        </Transition>
    }
}

#[component]
fn PositionFormBody(
    initial: PositionInput,
    data: PositionFormData,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let i = initial;
    let required = Signal::stored(true);
    // Ein Konto oder Code, der nicht (mehr) wählbar ist, bleibt sichtbar; Speichern verlangt eine neue Wahl.
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
                "Am {} ist kein Umsatzsteuer-Code gültig (Einstellungen → Finanzen → MWST).",
                german_date(&data.rechnungsdatum)
            )
        })
    } else {
        Some(format!(
            "Am {} gilt keine MWST-Pflicht; die Position wird ohne MWST erfasst.",
            german_date(&data.rechnungsdatum)
        ))
    };
    view! {
        <div class="flex flex-col gap-4">
            <FormAlerts errors server_error/>
            <TextField field="bezeichnung" label="Bezeichnung" value=i.bezeichnung.clone() errors required maxlength=200/>
            <div class="grid gap-x-4 md:grid-cols-4">
                <TextField field="menge" label="Menge" value=i.menge.clone() errors required inputmode="decimal" maxlength=20/>
                <SelectField field="einheit" label="Einheit *" options=UNITS value=i.einheit.clone() errors/>
                <TextField field="einzelpreis" label="Einzelpreis (ohne MWST)" value=i.einzelpreis.clone() errors required inputmode="decimal" maxlength=20/>
                <TextField field="rabatt_prozent" label="Rabatt %" value=i.rabatt_prozent.clone() errors inputmode="decimal" maxlength=6/>
            </div>
            <div class="grid gap-x-4 md:grid-cols-2">
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Ertragskonto *"</legend>
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
                    >
                        {stale_code.then(|| view! {
                            <option value=i.mwst_code_id.clone() selected=true>"Bisheriger Code – bitte neu wählen"</option>
                        })}
                        {(!data.mwst_pflicht || i.mwst_code_id.is_empty()).then(|| view! {
                            <option value="" selected=i.mwst_code_id.is_empty()>
                                {if data.mwst_pflicht { "Bitte wählen …" } else { "ohne MWST" }}
                            </option>
                        })}
                        {data
                            .mwst_pflicht
                            .then(|| {
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
            <FormActions cancel_href pending/>
        </div>
    }
}
