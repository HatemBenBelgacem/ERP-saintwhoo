//! Formular zum Anlegen und Bearbeiten (AK-8 bis AK-14).

use leptos::prelude::*;

use crate::backend::addresses::model::{Address, AddressInput, COUNTRIES, FieldErrors};
use crate::backend::addresses::server_fns::{CreateAddress, UpdateAddress};
use crate::forms::{
    ErrorText, Modal, TextField, field_errors, is_saved, refresh_when, server_error, use_base_href,
};

/// Modal-Fenster „Neue Adresse“ (AK-8). Nach dem Speichern öffnet sich die Detailansicht (AK-10).
#[component]
pub fn AddressCreateModal() -> impl IntoView {
    let action = ServerAction::<CreateAddress>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();

    view! {
        <Modal title="Neue Adresse".to_string() close_href=close.into()>
            <ActionForm action=action>
                <FormBody
                    initial=AddressInput::new_default()
                    errors=Signal::derive(move || field_errors(action.value().get()))
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

/// Modal-Fenster „Adresse bearbeiten“ (AK-12). Nach dem Speichern erscheint die Detailansicht neu (AK-13).
#[component]
pub fn AddressEditModal(address: Address) -> impl IntoView {
    let action = ServerAction::<UpdateAddress>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();

    view! {
        <Modal title=format!("{} bearbeiten", address.anzeigename) close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=address.id/>
                <FormBody
                    initial=address.to_input()
                    errors=Signal::derive(move || field_errors(action.value().get()))
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

/// Felder und Schaltflächen – gemeinsam für Anlegen und Bearbeiten.
#[component]
fn FormBody(
    initial: AddressInput,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    let typ = RwSignal::new(initial.typ.clone());
    let is_company = Signal::derive(move || typ.get() == "firma");
    let is_person = Signal::derive(move || typ.get() == "person");
    let i = initial;

    view! {
        <div>
            <div class="flex flex-col gap-4">
                {move || server_error.get().map(|e| view! {
                    <div role="alert" class="alert alert-error">{e}</div>
                })}
                {move || (!errors.read().is_empty()).then(|| view! {
                    <div role="alert" class="alert alert-warning">"Bitte die markierten Felder korrigieren."</div>
                })}

                // Typ (AK-9)
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Typ *"</legend>
                    <div class="flex gap-6">
                        {[("firma", "Firma"), ("person", "Person")]
                            .into_iter()
                            .map(|(value, label)| view! {
                                <label class="flex cursor-pointer items-center gap-2">
                                    <input
                                        type="radio"
                                        name="input[typ]"
                                        value=value
                                        class="radio radio-primary radio-sm"
                                        checked=i.typ == value
                                        on:change=move |_| typ.set(value.to_string())
                                    />
                                    {label}
                                </label>
                            })
                            .collect_view()}
                    </div>
                    <ErrorText field="typ" errors/>
                </fieldset>

                <div class="grid gap-x-4 md:grid-cols-2">
                    <div class="md:col-span-2" class:hidden=move || !is_company.get()>
                        <TextField field="firmenname" label="Firmenname" value=i.firmenname.clone() errors required=is_company maxlength=200/>
                    </div>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Anrede"</legend>
                        <select name="input[anrede]" class="select w-full">
                            {[("", "–"), ("herr", "Herr"), ("frau", "Frau")]
                                .into_iter()
                                .map(|(v, l)| view! { <option value=v selected=i.anrede == v>{l}</option> })
                                .collect_view()}
                        </select>
                    </fieldset>
                    <div class="hidden md:block"></div>
                    <TextField field="vorname" label="Vorname" value=i.vorname.clone() errors required=is_person maxlength=100/>
                    <TextField field="nachname" label="Nachname" value=i.nachname.clone() errors required=is_person maxlength=100/>
                    <div class="md:col-span-2">
                        <TextField field="zusatz" label="Zusatz (z. B. Abteilung, c/o)" value=i.zusatz.clone() errors maxlength=200/>
                    </div>
                    <TextField field="strasse" label="Strasse und Hausnummer" value=i.strasse.clone() errors maxlength=200/>
                    <TextField field="postfach" label="Postfach" value=i.postfach.clone() errors maxlength=50/>
                    <div class="grid grid-cols-[8rem_1fr] gap-x-4">
                        <TextField field="plz" label="PLZ" value=i.plz.clone() errors maxlength=20/>
                        <TextField field="ort" label="Ort" value=i.ort.clone() errors maxlength=100/>
                    </div>
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Land *"</legend>
                        <select
                            name="input[land]"
                            class="select w-full"
                            class:select-error=move || errors.read().contains_key("land")
                        >
                            {COUNTRIES
                                .iter()
                                .map(|(code, name)| view! {
                                    <option value=*code selected=i.land == *code>{format!("{name} ({code})")}</option>
                                })
                                .collect_view()}
                        </select>
                        <ErrorText field="land" errors/>
                    </fieldset>
                </div>

                <div class="divider my-0">"Kontakt"</div>
                <div class="grid gap-x-4 md:grid-cols-2">
                    <TextField field="email" label="E-Mail" value=i.email.clone() errors maxlength=254/>
                    <TextField field="webseite" label="Webseite" value=i.webseite.clone() errors maxlength=200/>
                    <TextField field="telefon" label="Telefon" kind="tel" value=i.telefon.clone() errors maxlength=50/>
                    <TextField field="mobiltelefon" label="Mobiltelefon" kind="tel" value=i.mobiltelefon.clone() errors maxlength=50/>
                </div>

                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Notiz"</legend>
                    <textarea name="input[notiz]" class="textarea h-24 w-full">{i.notiz.clone()}</textarea>
                </fieldset>

                <p class="text-sm text-base-content/60">"* Pflichtfeld"</p>

                <div class="modal-action mt-0">
                    // AK-14: Abbrechen speichert nichts
                    <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
                    <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                        {move || if pending.get() { "Speichern …" } else { "Speichern" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
