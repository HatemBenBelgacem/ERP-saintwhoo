//! Anmeldeseite (ANF-007 AK-4 bis AK-9, ANF-026). Ohne Grundgerüst.

use leptos::{html, prelude::*};
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use crate::backend::auth::model::{FORM_ERROR, REGISTER_PATH};
use crate::backend::auth::server_fns::Login;
use crate::forms::{field_errors, server_error};

/// Zentrierte Karte ohne Seitenleiste für Anmeldung und Registrierung.
/// Schmale Karte als bewusste Ausnahme von der vollen Breite (wie ein Dialog).
#[component]
pub(super) fn AuthCard(children: Children) -> impl IntoView {
    view! {
        <main class="flex min-h-dvh items-center justify-center bg-base-200 p-4">
            <div class="card w-full max-w-sm bg-base-100 shadow-lg">
                <div class="card-body gap-4">
                    <h1 class="text-center text-2xl font-semibold">"uniqwork"</h1>
                    {children()}
                </div>
            </div>
        </main>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <Title text="Anmelden – uniqwork"/>
        <AuthCard>
            <LoginForm/>
        </AuthCard>
    }
}

/// E-Mail und Passwort (ANF-007 AK-4 bis AK-8, ANF-026 AK-2).
#[component]
fn LoginForm() -> impl IntoView {
    let query = use_query_map();
    let action = ServerAction::<Login>::new();
    let password_ref = NodeRef::<html::Input>::new();
    let message = move || field_errors(action.value().get()).remove(FORM_ERROR);
    let zurueck = move || query.read().get("zurueck").unwrap_or_default();

    // ANF-007 AK-7: Nach einem Fehlversuch das Passwortfeld leeren
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

        // ANF-007 AK-24: per POST, Passwort nie in der Webadresse
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=zurueck/>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"E-Mail"</legend>
                <input
                    type="email"
                    name="email"
                    autocomplete="username"
                    autocapitalize="none"
                    spellcheck="false"
                    maxlength=254
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
        // ANF-026 AK-1
        <p class="text-center text-sm">
            "Noch kein Zugang? "
            <a href=REGISTER_PATH class="link link-primary">"Registrieren"</a>
        </p>
    }
}
