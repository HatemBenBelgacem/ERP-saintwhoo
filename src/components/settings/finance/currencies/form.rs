//! Modal-Fenster der Währungen: anlegen, bearbeiten, löschen und Kurs erfassen (ANF-014 AK-2, AK-3, AK-5).

use leptos::prelude::*;

use crate::backend::finance::currency::model::{
    Currency, CurrencyInput, FieldErrors, NAME_MAX, ROUNDINGS, rate_label,
};
use crate::backend::finance::currency::server_fns::{
    CreateCurrency, DeleteCurrency, SaveRate, UpdateCurrency, list_currencies, today_date,
};
use crate::backend::finance::fiscal_year::model::german_date;
use crate::forms::{
    ErrorText, Modal, SaveButtons, SelectField, ServerAlert, TextField, data_version,
    error_message, field_errors, is_saved, refresh_when, server_error, use_base_href,
};
use crate::icons::{Icon, IconView};

/// „Neue Währung“ (AK-2): ISO-Code, Bezeichnung und Rundung sind Pflicht.
#[component]
pub fn CurrencyCreateModal() -> impl IntoView {
    let action = ServerAction::<CreateCurrency>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    view! {
        <Modal title="Neue Währung".to_string() close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="zurueck" value=move || close.get()/>
                <div class="flex flex-col gap-4">
                    <ServerAlert message=Signal::derive(move || server_error(action.value().get()))/>
                    <CurrencyFields initial=CurrencyInput::new_default() neu=true booking=false errors/>
                    <SaveButtons cancel_href=close.into() pending=action.pending().into()/>
                </div>
            </ActionForm>
        </Modal>
    }
}

/// Währung bearbeiten (Bezeichnung, Rundung, Aktiv) und löschen, wenn sie unbenutzt ist (AK-5).
#[component]
pub fn CurrencyEditModal(code: String) -> impl IntoView {
    let close = use_base_href();
    let title = format!("Währung {code} bearbeiten");
    let data = Resource::new(data_version, move |_| load_currency(code.clone()));
    view! {
        <Modal title close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(currency)) => view! { <CurrencyEditForm currency/> }.into_any(),
                        Ok(None) => view! {
                            <p>"Diese Währung gibt es nicht oder sie wurde gelöscht."</p>
                        }
                        .into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </Modal>
    }
}

async fn load_currency(code: String) -> Result<Option<Currency>, ServerFnError> {
    Ok(list_currencies()
        .await?
        .into_iter()
        .find(|c| c.code == code))
}

#[component]
fn CurrencyEditForm(currency: Currency) -> impl IntoView {
    let action = ServerAction::<UpdateCurrency>::new();
    refresh_when(move || is_saved(action.value().get()));
    let delete = ServerAction::<DeleteCurrency>::new();
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let confirm = RwSignal::new(false);
    let delete_error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    let booking = currency.buchhaltungswaehrung;
    let used = currency.verwendet;
    let code = currency.code.clone();
    let code_delete = StoredValue::new(currency.code.clone());
    let initial = currency.to_input();

    view! {
        <ActionForm action=action>
            <input type="hidden" name="code" value=code.clone()/>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerAlert message=Signal::derive(move || server_error(action.value().get()))/>
                {booking.then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Die Buchhaltungswährung bleibt immer aktiv und lässt sich nicht löschen."
                    </div>
                })}
                {used.then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Diese Währung wird in Einkaufsaufträgen verwendet: Sie kann nicht gelöscht, nur deaktiviert werden."
                    </div>
                })}
                <CurrencyFields initial neu=false booking errors/>
                <div class="modal-action mt-0">
                    {(!booking && !used).then(|| view! {
                        <button type="button" class="btn btn-outline btn-error mr-auto" on:click=move |_| confirm.set(true)>
                            <IconView icon=Icon::Trash/>
                            "Löschen"
                        </button>
                    })}
                    <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                    <button type="submit" class="btn btn-primary" disabled=move || action.pending().get()>
                        "Speichern"
                    </button>
                </div>
            </div>
        </ActionForm>
        <Show when=move || confirm.get()>
            <div role="alertdialog" class="mt-4 space-y-3 rounded-box border border-error/40 p-4">
                <p>"Soll die Währung " {code_delete.get_value()} " wirklich gelöscht werden? Ihre Kurse gehen dabei verloren."</p>
                {move || delete_error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="flex justify-end gap-2">
                    <button type="button" class="btn" on:click=move |_| confirm.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
                        <input type="hidden" name="code" value=code_delete.get_value()/>
                        <input type="hidden" name="zurueck" value=move || close.get()/>
                        <button type="submit" class="btn btn-error" disabled=move || delete.pending().get()>
                            "Endgültig löschen"
                        </button>
                    </ActionForm>
                </div>
            </div>
        </Show>
    }
}

/// Felder einer Währung. Beim Bearbeiten ist der Code fest; die Buchhaltungswährung bleibt aktiv.
#[component]
fn CurrencyFields(
    initial: CurrencyInput,
    neu: bool,
    booking: bool,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    let aktiv = !initial.aktiv.is_empty();
    view! {
        <div class="grid gap-x-4 md:grid-cols-3">
            {if neu {
                view! {
                    <TextField field="code" label="Code (ISO 4217) *" value=initial.code.clone() errors required=true maxlength=3/>
                }
                .into_any()
            } else {
                view! {
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Code"</legend>
                        <p class="input w-full bg-base-200 font-medium">{initial.code.clone()}</p>
                    </fieldset>
                }
                .into_any()
            }}
            <div class="md:col-span-2">
                <TextField field="bezeichnung" label="Bezeichnung" value=initial.bezeichnung.clone() errors required=true maxlength=NAME_MAX/>
            </div>
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            <SelectField field="rundung" label="Rundung *" options=ROUNDINGS value=initial.rundung.clone() errors/>
        </div>
        {if booking {
            view! {
                <input type="hidden" name="input[aktiv]" value="1"/>
            }
            .into_any()
        } else {
            view! {
                <fieldset class="fieldset">
                    <label class="label cursor-pointer gap-2">
                        <input type="checkbox" name="input[aktiv]" value="1" class="checkbox checkbox-primary" checked=aktiv/>
                        "Aktiv – bei Einkaufsaufträgen wählbar"
                    </label>
                    <ErrorText field="aktiv" errors/>
                </fieldset>
            }
            .into_any()
        }}
    }
}

/// „Kurs erfassen“ (AK-3): CHF für eine Einheit ab einem Datum.
#[component]
pub fn RateModal(code: String) -> impl IntoView {
    let close = use_base_href();
    let title = format!("Kurs für {code} erfassen");
    let data = Resource::new(|| (), move |_| load_rate_form(code.clone()));
    view! {
        <Modal title close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(form) => view! { <RateForm form/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </Modal>
    }
}

/// Daten für das Kurs-Formular: die Währung und das heutige Datum als Vorgabe.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct RateFormData {
    currency: Currency,
    today: String,
}

async fn load_rate_form(code: String) -> Result<RateFormData, ServerFnError> {
    let currency = list_currencies()
        .await?
        .into_iter()
        .find(|c| c.code == code)
        .ok_or_else(|| ServerFnError::new("Währung nicht gefunden."))?;
    let today = today_date().await?;
    Ok(RateFormData { currency, today })
}

#[component]
fn RateForm(form: RateFormData) -> impl IntoView {
    let action = ServerAction::<SaveRate>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let code = form.currency.code.clone();
    let letzter = match (
        form.currency.letzter_kurs,
        form.currency.letzter_kurs_datum.clone(),
    ) {
        (Some(kurs), Some(datum)) => Some(format!(
            "Letzter Kurs: {} CHF am {}.",
            rate_label(kurs),
            german_date(&datum)
        )),
        _ => None,
    };

    view! {
        <ActionForm action=action>
            <input type="hidden" name="code" value=code.clone()/>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerAlert message=Signal::derive(move || server_error(action.value().get()))/>
                <p>"Wie viele Franken kostet 1 " {code.clone()} "? Der Kurs gilt ab dem angegebenen Datum."</p>
                {letzter.map(|text| view! { <p class="text-sm text-base-content/70">{text}</p> })}
                <div class="grid gap-x-4 md:grid-cols-2">
                    <TextField field="datum" label="Gültig ab *" value=form.today.clone() errors required=true kind="date"/>
                    <TextField field="kurs" label="Kurs in CHF für 1 Einheit *" value=String::new() errors required=true inputmode="decimal" maxlength=20/>
                </div>
                <SaveButtons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}
