//! Anmeldeseite mit Ersteinrichtung (ANF-007 AK-4 bis AK-9, AK-21). Ohne Grundgerüst.

use leptos::{html, prelude::*};
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use crate::backend::auth::model::{FORM_ERROR, FieldErrors};
use crate::backend::auth::server_fns::{FirstSetup, Login, setup_required};
use crate::forms::{ErrorText, TextField, field_errors, server_error};

#[component]
pub fn LoginPage() -> impl IntoView {
    let setup = Resource::new(|| (), |_| setup_required());

    view! {
        <Title text="Anmelden – uniqwork"/>
        // Zentrierte Karte ohne Seitenleiste; schmale Karte als bewusste Ausnahme (wie ein Dialog)
        <main class="flex min-h-dvh items-center justify-center bg-base-200 p-4">
            <div class="card w-full max-w-sm bg-base-100 shadow-lg">
                <div class="card-body gap-4">
                    <h1 class="text-center text-2xl font-semibold">"uniqwork"</h1>
                    <Suspense fallback=|| view! { <span class="loading loading-spinner mx-auto"></span> }>
                        {move || Suspend::new(async move {
                            match setup.await {
                                Ok(true) => view! { <FirstSetupForm/> }.into_any(),
                                Ok(false) => view! { <LoginForm/> }.into_any(),
                                Err(e) => view! {
                                    <div role="alert" class="alert alert-error">{crate::forms::error_message(&e)}</div>
                                }
                                    .into_any(),
                            }
                        })}
                    </Suspense>
                </div>
            </div>
        </main>
    }
}

/// Benutzername und Passwort (AK-4 bis AK-8).
#[component]
fn LoginForm() -> impl IntoView {
    let query = use_query_map();
    let action = ServerAction::<Login>::new();
    let password_ref = NodeRef::<html::Input>::new();
    let message = move || field_errors(action.value().get()).remove(FORM_ERROR);
    let zurueck = move || query.read().get("zurueck").unwrap_or_default();

    // AK-7: Nach einem Fehlversuch das Passwortfeld leeren
    Effect::new(move |_| {
        if message().is_some()
            && let Some(input) = password_ref.get()
        {
            input.set_value("");
        }
    });

    view! {
        <p class="text-center text-base-content/70">"Bitte melden Sie sich an."</p>
        {move || query.read().get("abgemeldet").map(|_| view! {
            <div role="status" class="alert alert-success">"Sie wurden abgemeldet."</div>
        })}
        {move || query.read().get("abgelaufen").map(|_| view! {
            <div role="status" class="alert alert-warning">"Ihre Sitzung ist abgelaufen. Bitte erneut anmelden."</div>
        })}
        {move || message().map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
        {move || server_error(action.value().get()).map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}

        // AK-24: per POST, Passwort nie in der Webadresse
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=zurueck/>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Benutzername"</legend>
                <input
                    type="text"
                    name="benutzername"
                    autocomplete="username"
                    autocapitalize="none"
                    spellcheck="false"
                    maxlength=50
                    required
                    autofocus
                    class="input w-full"
                />
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Passwort"</legend>
                <input
                    node_ref=password_ref
                    type="password"
                    name="passwort"
                    autocomplete="current-password"
                    maxlength=200
                    required
                    class="input w-full"
                />
            </fieldset>
            <button type="submit" class="btn btn-primary mt-4 w-full" disabled=move || action.pending().get()>
                {move || if action.pending().get() { "Anmelden …" } else { "Anmelden" }}
            </button>
        </ActionForm>
    }
}

/// Ersteinrichtung, solange es keinen Benutzer gibt (AK-21).
#[component]
fn FirstSetupForm() -> impl IntoView {
    let action = ServerAction::<FirstSetup>::new();
    let errors: Signal<FieldErrors> = Signal::derive(move || field_errors(action.value().get()));
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
        <div role="status" class="alert alert-info">
            "Ersteinrichtung: Es gibt noch keinen Benutzer. Legen Sie jetzt den ersten Benutzer an – "
            "danach sind Sie direkt angemeldet."
        </div>
        {move || server_error(action.value().get()).map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
        <ActionForm action=action>
            <div class="grid gap-x-4 sm:grid-cols-2">
                <TextField field="vorname" label="Vorname" value=String::new() errors required=true maxlength=100/>
                <TextField field="nachname" label="Nachname" value=String::new() errors required=true maxlength=100/>
            </div>
            <TextField field="email" label="E-Mail" kind="email" value=String::new() errors maxlength=254/>
            <TextField field="benutzername" label="Benutzername" value=String::new() errors required=true maxlength=50/>
            {password_field("passwort", "Passwort")}
            {password_field("wiederholung", "Passwort wiederholen")}
            <p class="text-xs text-base-content/60">"Mindestens 10 Zeichen. * Pflichtfeld"</p>
            <button type="submit" class="btn btn-primary mt-4 w-full" disabled=move || action.pending().get()>
                "Einrichten und anmelden"
            </button>
        </ActionForm>
    }
}
