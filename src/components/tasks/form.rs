//! Modal-Fenster für Aufgaben ohne Projekt: Anlegen und Bearbeiten (ANF-006 AK-10 bis AK-12, AK-15).
//! Ein Projekt lässt sich hier nicht wählen – Projektaufgaben entstehen nur im Projekt (AK-13).

use leptos::prelude::*;

use crate::backend::tasks::model::{FieldErrors, PickOption, StandaloneTask, StandaloneTaskInput};
use crate::backend::tasks::server_fns::{CreateTask, UpdateTask};
use crate::backend::users::server_fns::search_active_users;
use crate::forms::{
    Modal, SearchPicker, TextField, field_errors, is_saved, refresh_when, server_error,
    use_base_href,
};

/// „Neue Aufgabe“ (AK-10, AK-11): „Zuständig“ ist mit dem gewählten Benutzer vorbelegt.
/// Nach dem Speichern zurück zur Liste.
#[component]
pub fn TaskCreateModal(user: Option<PickOption>) -> impl IntoView {
    let action = ServerAction::<CreateTask>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));

    view! {
        <Modal title="Neue Aufgabe ohne Projekt".to_string() close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="zurueck" value=move || close.get()/>
                <FormBody
                    initial=StandaloneTaskInput::default()
                    assignee=user
                    errors
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

/// „Aufgabe bearbeiten“ (AK-15).
#[component]
pub fn TaskEditModal(task: StandaloneTask) -> impl IntoView {
    let action = ServerAction::<UpdateTask>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let assignee = task.assignee().pick_option();

    view! {
        <Modal title=format!("„{}“ bearbeiten", task.name) close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=task.id/>
                <FormBody
                    initial=task.to_input()
                    assignee=Some(assignee)
                    errors
                    server_error=Signal::derive(move || server_error(action.value().get()))
                    pending=action.pending().into()
                    cancel_href=close.into()
                />
            </ActionForm>
        </Modal>
    }
}

#[component]
fn FormBody(
    initial: StandaloneTaskInput,
    assignee: Option<PickOption>,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
    cancel_href: Signal<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4">
            {move || server_error.get().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
            {move || (!errors.read().is_empty()).then(|| view! {
                <div role="alert" class="alert alert-warning">"Bitte die markierten Felder korrigieren."</div>
            })}
            <TextField field="name" label="Titel" value=initial.name.clone() errors required=true maxlength=200/>
            <SearchPicker
                field="benutzer_id"
                label="Zuständig"
                placeholder="Aktiven Benutzer suchen (Name, Benutzername) …"
                initial=assignee
                errors
                search=search_active_users
                required=true
            />
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Beschreibung"</legend>
                <textarea name="input[beschreibung]" class="textarea h-28 w-full">{initial.beschreibung.clone()}</textarea>
            </fieldset>
            <p class="text-sm text-base-content/60">
                "* Pflichtfeld · Aufgaben ohne Projekt gehören keinem Projekt an. Projektaufgaben werden im Projekt angelegt."
            </p>
            <div class="modal-action mt-0">
                <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
                <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                    {move || if pending.get() { "Speichern …" } else { "Speichern" }}
                </button>
            </div>
        </div>
    }
}
