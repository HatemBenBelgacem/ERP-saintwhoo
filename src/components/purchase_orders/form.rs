//! Modal-Fenster: Kopfdaten (AK-8 bis AK-10, AK-13, AK-15), Positionen (AK-11 bis AK-13)
//! und Wareneingang (AK-16 bis AK-18).

use leptos::prelude::*;

use crate::backend::addresses::server_fns::search_addresses;
use crate::backend::finance::vat::model::VatCode;
use crate::backend::finance::vat::server_fns::list_purchase_codes;
use crate::backend::projects::server_fns::search_projects;
use crate::backend::purchase_orders::model::{
    ALT_VAT, CURRENCIES, FieldErrors, Order, OrderInput, PickOption, Position, PositionInput,
    UNITS, format_trimmed, parse_fixed, unit_label,
};
use crate::backend::purchase_orders::server_fns::{
    CreateOrder, CreatePosition, RecordDelivery, UpdateOrder, UpdatePosition,
};
use crate::forms::{
    ErrorText, Modal, SearchPicker, SelectField, TextField, error_message, field_errors, is_saved,
    refresh_when, server_error, use_base_href,
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

/// Schaltflächen „Abbrechen“ und „Speichern“.
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
fn TextArea(field: &'static str, label: &'static str, value: String) -> impl IntoView {
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">{label}</legend>
            <textarea name=format!("input[{field}]") class="textarea h-24 w-full">{value}</textarea>
        </fieldset>
    }
}

// ---------------------------------------------------------------------------------------------
// Kopfdaten
// ---------------------------------------------------------------------------------------------

/// „Neuer Einkaufsauftrag“ (AK-8). Nach dem Speichern öffnet sich die Detailansicht (AK-10).
#[component]
pub fn OrderCreateModal() -> impl IntoView {
    let action = ServerAction::<CreateOrder>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title="Neuer Einkaufsauftrag".to_string() close_href=close.into()>
            <ActionForm action=action>
                <OrderFormBody
                    initial=OrderInput::new_default()
                    supplier=None
                    project=None
                    restricted=false
                    errors=Signal::derive(move || field_errors(action.value().get()))
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

/// Kopfdaten bearbeiten (AK-13); nach dem Bestellen nur noch wenige Felder (AK-15).
#[component]
pub fn OrderEditModal(order: Order) -> impl IntoView {
    let action = ServerAction::<UpdateOrder>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title=format!("{} bearbeiten", order.nummer) close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=order.id/>
                <OrderFormBody
                    initial=order.to_input()
                    supplier=Some(order.supplier())
                    project=order.project()
                    restricted=!order.is_draft()
                    errors=Signal::derive(move || field_errors(action.value().get()))
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

/// Felder der Kopfdaten. `restricted`: nur Liefertermin, Referenz und Bemerkung (AK-15).
#[component]
fn OrderFormBody(
    initial: OrderInput,
    supplier: Option<PickOption>,
    project: Option<PickOption>,
    restricted: bool,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let i = initial;
    view! {
        <div class="flex flex-col gap-4">
            <FormAlerts errors server_error/>
            {if restricted {
                view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Der Auftrag ist bestellt. Lieferant, Positionen und Preise sind gesperrt; \
                         änderbar sind nur Liefertermin, Referenz des Lieferanten und Bemerkung."
                    </div>
                    <div class="grid gap-x-4 md:grid-cols-2">
                        <TextField field="liefertermin" label="Liefertermin" kind="date" value=i.liefertermin.clone() errors/>
                        <TextField field="referenz_lieferant" label="Referenz Lieferant" value=i.referenz_lieferant.clone() errors maxlength=100/>
                    </div>
                    <TextArea field="bemerkung" label="Bemerkung" value=i.bemerkung.clone()/>
                }
                .into_any()
            } else {
                view! {
                    // AK-9: Lieferant Pflicht, Projekt optional
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
                        <TextField field="bestelldatum" label="Bestelldatum" kind="date" value=i.bestelldatum.clone() errors/>
                        <TextField field="liefertermin" label="Liefertermin" kind="date" value=i.liefertermin.clone() errors/>
                        <SelectField field="waehrung" label="Währung *" options=CURRENCIES value=i.waehrung.clone() errors/>
                    </div>
                    <TextField field="referenz_lieferant" label="Referenz Lieferant" value=i.referenz_lieferant.clone() errors maxlength=100/>
                    <div class="grid gap-x-4 md:grid-cols-2">
                        <TextArea field="lieferadresse" label="Abweichende Lieferadresse" value=i.lieferadresse.clone()/>
                        <TextArea field="bemerkung" label="Bemerkung" value=i.bemerkung.clone()/>
                    </div>
                }
                .into_any()
            }}
            <FormActions cancel_href pending/>
        </div>
    }
}

// ---------------------------------------------------------------------------------------------
// Positionen
// ---------------------------------------------------------------------------------------------

/// „Neue Position“ (AK-11), Webadresse `?neu=position`.
#[component]
pub fn PositionCreateModal(order_id: i64, order_number: String) -> impl IntoView {
    let action = ServerAction::<CreatePosition>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title=format!("Neue Position in {order_number}") close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="auftrag_id" value=order_id/>
                <PositionFormLoader
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

/// Position bearbeiten (AK-13), Webadresse `?bearbeiten=position&id=<kennung>`.
#[component]
pub fn PositionEditModal(position: Position, number: usize) -> impl IntoView {
    let action = ServerAction::<UpdatePosition>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    view! {
        <Modal title=format!("Position {number} bearbeiten") close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=position.id/>
                <PositionFormLoader
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

/// Lädt die wählbaren MWST-Codes (ANF-012 AK-9) und zeigt das Formular, sobald sie vorliegen.
/// Bei einer neuen Position ist der erste Code vorausgewählt.
#[component]
fn PositionFormLoader(
    initial: PositionInput,
    creating: bool,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let codes = Resource::new(|| (), |_| list_purchase_codes());
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || {
                let initial = initial.clone();
                Suspend::new(async move {
                    match codes.await {
                        Ok(list) => {
                            let mut initial = initial;
                            if creating
                                && initial.mwst_code.is_empty()
                                && let Some(first) = list.first()
                            {
                                initial.mwst_code = first.id.to_string();
                            }
                            view! {
                                <PositionFormBody initial codes=list errors server_error pending cancel_href/>
                            }
                            .into_any()
                        }
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })
            }}
        </Transition>
    }
}

#[component]
fn PositionFormBody(
    initial: PositionInput,
    codes: Vec<VatCode>,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let i = initial;
    let required = Signal::stored(true);
    // Eine Position mit einem Code, der nicht (mehr) wählbar ist, bleibt sichtbar; speichern verlangt eine neue Wahl.
    let stale = i
        .mwst_code
        .parse::<i64>()
        .is_ok_and(|id| !codes.iter().any(|c| c.id == id));
    let no_codes = codes.is_empty();
    view! {
        <div class="flex flex-col gap-4">
            <FormAlerts errors server_error/>
            <div class="grid gap-x-4 md:grid-cols-3">
                <div class="md:col-span-2">
                    <TextField field="bezeichnung" label="Bezeichnung" value=i.bezeichnung.clone() errors required maxlength=200/>
                </div>
                <TextField field="artikelnummer_lieferant" label="Artikelnummer Lieferant" value=i.artikelnummer_lieferant.clone() errors maxlength=50/>
            </div>
            <div class="grid gap-x-4 md:grid-cols-5">
                <TextField field="menge" label="Menge" value=i.menge.clone() errors required inputmode="decimal" maxlength=20/>
                <SelectField field="einheit" label="Einheit *" options=UNITS value=i.einheit.clone() errors/>
                <TextField field="einzelpreis" label="Einzelpreis (ohne MWST)" value=i.einzelpreis.clone() errors required inputmode="decimal" maxlength=20/>
                <TextField field="rabatt_prozent" label="Rabatt %" value=i.rabatt_prozent.clone() errors inputmode="decimal" maxlength=6/>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"MWST *"</legend>
                    <select
                        name="input[mwst_code]"
                        class="select w-full"
                        class:select-error=move || errors.read().contains_key("mwst_code")
                    >
                        {(i.mwst_code == ALT_VAT).then(|| view! {
                            <option value=ALT_VAT selected=true>"Bisheriger Satz (ohne Code)"</option>
                        })}
                        {stale.then(|| view! {
                            <option value=i.mwst_code.clone() selected=true>"Bisheriger Code – bitte neu wählen"</option>
                        })}
                        <option value="" selected=i.mwst_code.is_empty()>"ohne MWST"</option>
                        {codes
                            .iter()
                            .map(|c| {
                                let value = c.id.to_string();
                                let selected = i.mwst_code == value;
                                view! { <option value=value selected=selected>{c.choice_label()}</option> }
                            })
                            .collect_view()}
                    </select>
                    <ErrorText field="mwst_code" errors/>
                    {no_codes.then(|| view! {
                        <p class="label">
                            "Keine MWST-Codes wählbar: Die MWST-Pflicht ist ausgeschaltet oder es gibt keinen gültigen "
                            "Vorsteuer-Code (Einstellungen → Finanzen → MWST)."
                        </p>
                    })}
                </fieldset>
            </div>
            <FormActions cancel_href pending/>
        </div>
    }
}

// ---------------------------------------------------------------------------------------------
// Wareneingang
// ---------------------------------------------------------------------------------------------

/// Gelieferte Menge einer Position erfassen (AK-16), Webadresse `?wareneingang=<kennung>`.
/// Bei Überlieferung erscheint ein Hinweis, Speichern bleibt erlaubt (AK-18).
#[component]
pub fn DeliveryModal(position: Position, number: usize) -> impl IntoView {
    let action = ServerAction::<RecordDelivery>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let server_error = Signal::derive(move || server_error(action.value().get()));
    let pending: Signal<bool> = action.pending().into();

    let unit = unit_label(&position.einheit).to_string();
    let ordered = position.menge;
    let ordered_text = format!("{} {unit}", format_trimmed(ordered, 3));
    let over_hint = format!(
        "Überlieferung: Es wurde mehr geliefert als bestellt ({ordered_text}). Speichern ist trotzdem möglich."
    );
    let value = RwSignal::new(format_trimmed(position.gelieferte_menge, 3));
    let over = move || parse_fixed(&value.get(), 3).is_some_and(|v| v > ordered);
    let has_error = move || errors.read().contains_key("geliefert");

    view! {
        <Modal title=format!("Wareneingang Position {number}") close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=position.id/>
                <div class="flex flex-col gap-4">
                    <FormAlerts errors server_error/>
                    <dl class="grid gap-2 sm:grid-cols-2">
                        <div>
                            <dt class="text-sm text-base-content/60">"Bezeichnung"</dt>
                            <dd class="font-medium">{position.bezeichnung.clone()}</dd>
                        </div>
                        <div>
                            <dt class="text-sm text-base-content/60">"Bestellt"</dt>
                            <dd>{ordered_text}</dd>
                        </div>
                    </dl>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Gelieferte Menge gesamt (" {unit} ") *"</legend>
                        <input
                            type="text"
                            name="geliefert"
                            inputmode="decimal"
                            maxlength="20"
                            class="input w-full sm:w-64"
                            class:input-error=has_error
                            prop:value=move || value.get()
                            on:input=move |ev| value.set(event_target_value(&ev))
                        />
                        <ErrorText field="geliefert" errors/>
                    </fieldset>
                    {move || over().then(|| view! {
                        <div role="alert" class="alert alert-warning">
                            {over_hint.clone()}
                        </div>
                    })}
                    <FormActions cancel_href=close.into() pending/>
                </div>
            </ActionForm>
        </Modal>
    }
}
