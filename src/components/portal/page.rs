//! Portal-Übersicht `/portal` (ANF-026 AK-4, AK-10, AK-19).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use crate::backend::auth::model::{CurrentUser, SessionTenant, TENANT_PARAM, TENANT_READY};
use crate::backend::auth::server_fns::get_current_user;
use crate::backend::company::model::FieldErrors;
use crate::backend::portal::model::STATUS_FAILED;
use crate::backend::portal::server_fns::{CreateDatabase, DefineTenant};
use crate::components::settings::NewAddressFields;
use crate::forms::{Modal, TextField, field_errors, server_error, use_base_href, with_params};
use crate::icons::{Icon, IconView};
use crate::layout::UserMenu;

#[component]
pub fn PortalPage() -> impl IntoView {
    // Nach „Datenbank erstellen“ neu laden (die Weiterleitung bleibt auf derselben Seite)
    let create = ServerAction::<CreateDatabase>::new();
    let me = Resource::new(move || create.version().get(), |_| get_current_user());

    view! {
        <Title text="Portal – uniqwork"/>
        <div class="flex min-h-dvh flex-col bg-base-200">
            // Kopfzeile wie im ERP, aber ohne Seitenleiste
            <header class="flex min-h-16 items-center gap-2 bg-neutral px-4 py-2 text-neutral-content">
                <span class="text-lg font-semibold">"uniqwork"</span>
                <Transition fallback=|| ()>
                    {move || Suspend::new(async move {
                        me.await.ok().flatten().map(|user| view! { <UserMenu user/> })
                    })}
                </Transition>
            </header>
            <main class="flex-1 space-y-4 p-4 sm:p-6">
                <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match me.await {
                            Ok(Some(user)) => view! { <PortalContent user create/> }.into_any(),
                            Ok(None) => view! {
                                <div role="alert" class="alert alert-warning">"Nicht angemeldet."</div>
                            }
                                .into_any(),
                            Err(e) => view! {
                                <div role="alert" class="alert alert-error">{crate::forms::error_message(&e)}</div>
                            }
                                .into_any(),
                        }
                    })}
                </Suspense>
            </main>
        </div>
    }
}

#[component]
fn PortalContent(user: CurrentUser, create: ServerAction<CreateDatabase>) -> impl IntoView {
    view! {
        <h1 class="text-2xl font-semibold">"Willkommen, " {user.name.clone()}</h1>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h2 class="card-title">
                    <IconView icon=Icon::Building/>
                    "Ihre Datenbank"
                </h2>
                {match user.mandant {
                    None => view! { <NoDatabase create/> }.into_any(),
                    Some(tenant) => view! { <TenantInfo tenant create/> }.into_any(),
                }}
                {move || create.value().get().and_then(Result::err).map(|e| view! {
                    <div role="alert" class="alert alert-error">{crate::forms::error_message(&e)}</div>
                })}
            </div>
        </div>
    }
}

/// Noch keine Datenbank (ANF-026 AK-10).
#[component]
fn NoDatabase(create: ServerAction<CreateDatabase>) -> impl IntoView {
    view! {
        <p>"Sie haben noch keine Datenbank. Erstellen Sie jetzt Ihre eigene Datenbank – danach erfassen Sie Ihre Firma."</p>
        <CreateButton create label="Datenbank erstellen"/>
    }
}

/// Hauptschaltfläche „Datenbank erstellen“; gesperrt mit Ladeanzeige, solange erstellt wird (AK-11).
#[component]
fn CreateButton(create: ServerAction<CreateDatabase>, label: &'static str) -> impl IntoView {
    view! {
        <ActionForm action=create>
            <div class="card-actions">
                <button type="submit" class="btn btn-primary" disabled=move || create.pending().get()>
                    {move || create.pending().get().then(|| view! { <span class="loading loading-spinner"></span> })}
                    {move || if create.pending().get() { "Datenbank wird erstellt …" } else { label }}
                </button>
            </div>
        </ActionForm>
    }
}

/// Datenbank vorhanden: Status, Mandant und Verweis ins ERP; nach einem Fehler erneut versuchen (AK-13).
#[component]
fn TenantInfo(tenant: SessionTenant, create: ServerAction<CreateDatabase>) -> impl IntoView {
    let failed = tenant.status == STATUS_FAILED;
    let undefined = tenant.needs_definition();
    let query = use_query_map();
    let base = use_base_href();
    let id = tenant.id;
    // AK-16, AK-19: Modal-Fenster, solange der Mandant nicht definiert ist und die Webadresse es verlangt
    let modal_open = move || {
        undefined
            && query
                .read()
                .get(TENANT_PARAM)
                .and_then(|v| v.parse::<i64>().ok())
                == Some(id)
    };
    let usable = tenant.is_usable();
    let (label, badge) = match tenant.status.as_str() {
        TENANT_READY => ("Bereit", "badge badge-success"),
        "fehler" => ("Fehler", "badge badge-error"),
        _ => ("Wird erstellt", "badge badge-warning"),
    };
    view! {
        <p class="flex flex-wrap items-center gap-2">
            <span class="font-medium">
                {tenant.firmenname.clone().unwrap_or_else(|| "Mandant noch nicht definiert".into())}
            </span>
            <span class=badge>{label}</span>
        </p>
        {usable.then(|| view! {
            <div class="card-actions">
                <a href="/" class="btn btn-primary">"Zum ERP"</a>
            </div>
        })}
        {failed.then(|| view! {
            <p>"Beim Erstellen ist ein Fehler aufgetreten. Es wurden keine Daten angelegt."</p>
            <CreateButton create label="Erneut versuchen"/>
        })}
        // AK-19: Fenster geschlossen → Hinweis und Schaltfläche
        {undefined.then(|| view! {
            <div role="status" class="alert alert-warning">
                "Mandant noch nicht definiert. Erfassen Sie Ihre Firma, um mit dem ERP zu arbeiten."
            </div>
            <div class="card-actions">
                <a href=move || with_params(&base.get(), &[(TENANT_PARAM, id.to_string())]) class="btn btn-primary">
                    "Mandant definieren"
                </a>
            </div>
        })}
        {move || modal_open().then(|| view! { <DefineTenantModal/> })}
    }
}

/// Modal-Fenster „Mandant definieren“ (ANF-026 AK-16 bis AK-18, Webadresse `?mandant=<Kennung>`):
/// Firmenadresse wie ANF-025 und UID. Nach dem Speichern geht es zur Startseite des ERP.
#[component]
fn DefineTenantModal() -> impl IntoView {
    let action = ServerAction::<DefineTenant>::new();
    let close = use_base_href();
    let errors: Signal<FieldErrors> = Signal::derive(move || field_errors(action.value().get()));

    view! {
        <Modal title="Mandant definieren".to_string() close_href=close.into()>
            <ActionForm action=action>
                // Typ der Adresse ist immer „Firma“
                <input type="hidden" name="adresse[typ]" value="firma"/>
                <div class="flex flex-col gap-4">
                    <p class="text-base-content/70">
                        "Erfassen Sie Ihre Firma. Sie ist der Absender auf allen Belegen und erscheint in der Kopfzeile."
                    </p>
                    {move || server_error(action.value().get()).map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                    {move || (!errors.read().is_empty()).then(|| view! {
                        <div role="alert" class="alert alert-warning">"Bitte die markierten Felder korrigieren."</div>
                    })}
                    <NewAddressFields errors/>
                    <div class="grid gap-x-4 md:grid-cols-2">
                        <div>
                            <TextField field="uid" label="UID" value=String::new() errors maxlength=25/>
                            <p class="text-xs text-base-content/60">
                                "Freiwillig. Format CHE-123.456.789; die Prüfziffer wird kontrolliert."
                            </p>
                        </div>
                    </div>
                    <p class="text-sm text-base-content/60">
                        "* Pflichtfeld. Handelsregister, Fusszeile und Logo ergänzen Sie später im ERP unter Einstellungen › Firma."
                    </p>
                    <div class="modal-action mt-0">
                        <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                        <button type="submit" class="btn btn-primary" disabled=move || action.pending().get()>
                            {move || if action.pending().get() { "Speichern …" } else { "Speichern" }}
                        </button>
                    </div>
                </div>
            </ActionForm>
        </Modal>
    }
}
