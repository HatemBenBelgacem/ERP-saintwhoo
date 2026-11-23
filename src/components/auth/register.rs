//! Registrierung (ANF-026 AK-1 bis AK-5). Ohne Grundgerüst, öffentlich.

use leptos::prelude::*;
use leptos_meta::Title;

use super::login::AuthCard;
use crate::backend::auth::model::{FORM_ERROR, FieldErrors, LOGIN_PATH};
use crate::backend::auth::server_fns::Register;
use crate::backend::users::model::PASSWORD_MIN;
use crate::forms::{ErrorText, TextField, field_errors, server_error};

#[component]
pub fn RegisterPage() -> impl IntoView {
    let action = ServerAction::<Register>::new();
    let errors: Signal<FieldErrors> = Signal::derive(move || field_errors(action.value().get()));
    let form_message = move || errors.read().get(FORM_ERROR).cloned();
    let password_field = move |name: &'static str, label: &'static str| {
        let has_error = move || errors.read().contains_key(name);
        view! {
            <fieldset class="fieldset">
                <legend class="fieldset-legend">{label} " *"</legend>
                <input
                    type="password"
                    name=format!("passwort[{name}]")
                    autocomplete="new-password"
                    maxlength=200
                    class="input w-full"
                    class:input-error=has_error
                />
                <ErrorText field=name errors/>
            </fieldset>
        }
    };

    view! {
        <Title text="Registrieren – uniqwork"/>
        <AuthCard>
            <p class="text-center text-base-content/70">
                "Legen Sie Ihren Zugang an. Sie sind danach Administrator und können Ihre eigene Datenbank erstellen."
            </p>
            {move || form_message().map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
            {move || server_error(action.value().get()).map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
            // Per POST, Passwort nie in der Webadresse
            <ActionForm action=action>
                <div class="grid gap-x-4 sm:grid-cols-2">
                    <TextField field="vorname" label="Vorname" value=String::new() errors required=true maxlength=100/>
                    <TextField field="nachname" label="Nachname" value=String::new() errors required=true maxlength=100/>
                </div>
                <TextField field="email" label="E-Mail" kind="email" value=String::new() errors required=true maxlength=254/>
                {password_field("passwort", "Passwort")}
                {password_field("wiederholung", "Passwort wiederholen")}
                <p class="text-xs text-base-content/60">{format!("Mindestens {PASSWORD_MIN} Zeichen. * Pflichtfeld")}</p>
                <button type="submit" class="btn btn-primary mt-4 w-full" disabled=move || action.pending().get()>
                    {move || if action.pending().get() { "Registrieren …" } else { "Registrieren" }}
                </button>
            </ActionForm>
            <p class="text-center text-sm">
                "Bereits registriert? "
                <a href=LOGIN_PATH class="link link-primary">"Anmelden"</a>
            </p>
        </AuthCard>
    }
}
