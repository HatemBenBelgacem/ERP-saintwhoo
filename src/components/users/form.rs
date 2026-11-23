//! Modal-Fenster: Benutzer anlegen und bearbeiten (ANF-005 AK-9 bis AK-17).
//! Ohne Passwort: angemeldet wird nur über das Portal (ANF-026).

use leptos::prelude::*;

use crate::backend::users::model::{FieldErrors, PickOption, STATUSES, User, UserInput};
use crate::backend::users::server_fns::{CreateUser, UpdateUser, search_person_addresses};
use crate::forms::{
    Modal, SearchPicker, SelectField, TextField, field_errors, is_saved, refresh_when,
    server_error, use_base_href,
};

/// Modal-Fenster „Neuer Benutzer“ (AK-9). Nach dem Speichern öffnet sich die Detailansicht (AK-12).
#[component]
pub fn UserCreateModal() -> impl IntoView {
    let action = ServerAction::<CreateUser>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));

    view! {
        <Modal title="Neuer Benutzer".to_string() close_href=close.into()>
            <ActionForm action=action>
                <div class="flex flex-col gap-4">
                    <Messages errors server_error=Signal::derive(move || server_error(action.value().get()))/>
                    <UserFields initial=UserInput::new_default() address=None user_id=None errors/>
                    <p class="text-sm text-base-content/60">
                        "Der Benutzer erhält kein Passwort: Anmelden können sich nur Zugänge des Portals."
                    </p>
                    <Buttons cancel_href=close.into() pending=action.pending().into()/>
                </div>
            </ActionForm>
        </Modal>
    }
}

/// Modal-Fenster „Benutzer bearbeiten“ (AK-14): ohne Passwortfelder.
#[component]
pub fn UserEditModal(user: User) -> impl IntoView {
    let action = ServerAction::<UpdateUser>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));

    view! {
        <Modal title=format!("{} bearbeiten", user.display_name()) close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=user.id/>
                <div class="flex flex-col gap-4">
                    <Messages errors server_error=Signal::derive(move || server_error(action.value().get()))/>
                    <UserFields initial=user.to_input() address=Some(user.address()) user_id=Some(user.id) errors/>
                    <Buttons cancel_href=close.into() pending=action.pending().into()/>
                </div>
            </ActionForm>
        </Modal>
    }
}

#[component]
fn Messages(errors: Signal<FieldErrors>, server_error: Signal<Option<String>>) -> impl IntoView {
    view! {
        {move || server_error.get().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
        {move || (!errors.read().is_empty()).then(|| view! {
            <div role="alert" class="alert alert-warning">"Bitte die markierten Felder korrigieren."</div>
        })}
    }
}

/// Benutzername, Adresse und Status (AK-10, AK-11).
#[component]
fn UserFields(
    initial: UserInput,
    address: Option<PickOption>,
    /// Beim Bearbeiten: eigene Adresse gilt in der Suche als frei
    user_id: Option<i64>,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    view! {
        <div class="grid gap-x-4 md:grid-cols-2">
            <div>
                <TextField
                    field="benutzername"
                    label="Benutzername"
                    value=initial.benutzername.clone()
                    errors
                    required=true
                    maxlength=50
                />
                <p class="text-xs text-base-content/60">
                    "Kleinbuchstaben a–z, Ziffern sowie . - _ (z. B. anna.muster)"
                </p>
            </div>
            <SelectField field="status" label="Status *" options=STATUSES value=initial.status.clone() errors/>
        </div>
        <SearchPicker
            field="adresse_id"
            label="Adresse (Person)"
            placeholder="Person suchen (Name, Nummer, Ort, E-Mail) …"
            initial=address
            errors
            search=move |q| search_person_addresses(q, user_id)
            required=true
        />
        <p class="-mt-3 text-xs text-base-content/60">
            "Angeboten werden nur Personen, die noch keinem Benutzer zugeordnet sind. "
            "Neue Personen zuerst unter " <a href="/adressen" class="link">"Adressen"</a> " erfassen."
        </p>
    }
}

#[component]
fn Buttons(cancel_href: Signal<String>, pending: Signal<bool>) -> impl IntoView {
    view! {
        <p class="text-sm text-base-content/60">"* Pflichtfeld"</p>
        <div class="modal-action mt-0">
            // AK-17: Abbrechen speichert nichts
            <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
            <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                {move || if pending.get() { "Speichern …" } else { "Speichern" }}
            </button>
        </div>
    }
}
