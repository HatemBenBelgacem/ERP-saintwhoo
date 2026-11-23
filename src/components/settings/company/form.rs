//! Modal-Fenster der Firma: erfassen (AK-4), bearbeiten (AK-5, AK-6) und Logo (AK-7).

use leptos::prelude::*;
use leptos::web_sys::{FormData, HtmlFormElement};
use server_fn::ServerFn;

use crate::backend::addresses::model::COUNTRIES;
use crate::backend::company::model::{
    Company, CompanyInput, FOOTER_MAX, FieldErrors, REGISTER_MAX, SaveOutcome,
};
use crate::backend::company::server_fns::{
    CreateCompany, RemoveLogo, UpdateCompany, UploadLogo, get_company, search_company_addresses,
    upload_logo,
};
use crate::forms::{
    ErrorText, Modal, SearchPicker, TextField, data_version, error_message, field_errors, is_saved,
    refresh_when, server_error, use_base_href,
};

/// „Firma erfassen“ bzw. „Firma bearbeiten“ – je nachdem, ob schon eine Firma existiert.
#[component]
pub fn CompanyModal() -> impl IntoView {
    let close = use_base_href();
    let data = Resource::new(data_version, |_| get_company());
    view! {
        // Transition statt Suspense: Das Nachladen der Suchtreffer (SearchPicker) darf das
        // Fenster nicht ausblenden, sonst geht der Fokus im Suchfeld verloren.
        <Transition fallback=|| ()>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(company)) => view! {
                        <Modal title="Firma bearbeiten".to_string() close_href=close.into()>
                            <EditForm company/>
                        </Modal>
                    }
                    .into_any(),
                    Ok(None) => view! {
                        <Modal title="Firma erfassen".to_string() close_href=close.into()>
                            <CreateForm/>
                        </Modal>
                    }
                    .into_any(),
                    Err(e) => view! {
                        <Modal title="Fehler".to_string() close_href=close.into()>
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        </Modal>
                    }
                    .into_any(),
                }
            })}
        </Transition>
    }
}

/// AK-4: bestehende Adresse wählen oder neue Adresse im selben Schritt anlegen.
#[component]
fn CreateForm() -> impl IntoView {
    let action = ServerAction::<CreateCompany>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let new_address = RwSignal::new(false);

    view! {
        <ActionForm action=action>
            // Typ der neuen Adresse ist immer „Firma“
            <input type="hidden" name="adresse[typ]" value="firma"/>
            <div class="flex flex-col gap-4">
                <Messages errors server_error=Signal::derive(move || server_error(action.value().get()))/>
                <div role="radiogroup" aria-label="Adresse" class="flex flex-wrap gap-4">
                    <label class="label cursor-pointer gap-2">
                        <input type="radio" name="input[modus]" value="bestehend" class="radio radio-primary radio-sm"
                            checked=true on:change=move |_| new_address.set(false)/>
                        "Bestehende Adresse wählen"
                    </label>
                    <label class="label cursor-pointer gap-2">
                        <input type="radio" name="input[modus]" value="neu" class="radio radio-primary radio-sm"
                            on:change=move |_| new_address.set(true)/>
                        "Neue Adresse anlegen"
                    </label>
                </div>
                <Show
                    when=move || new_address.get()
                    fallback=move || view! { <AddressPicker initial=None errors/> }
                >
                    <NewAddressFields errors/>
                </Show>
                <div class="divider my-0">"Rechtliche Angaben"</div>
                <CompanyFields initial=CompanyInput::default() errors/>
                <Buttons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}

/// AK-5: UID, Handelsregister, Fusszeile und verknüpfte Adresse ändern.
#[component]
fn EditForm(company: Company) -> impl IntoView {
    let action = ServerAction::<UpdateCompany>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let address_href = format!("/adressen/{}?bearbeiten=1", company.adresse_id);

    view! {
        <ActionForm action=action>
            <div class="flex flex-col gap-4">
                <Messages errors server_error=Signal::derive(move || server_error(action.value().get()))/>
                <AddressPicker initial=Some(company.address_option()) errors/>
                <p class="-mt-3 text-xs text-base-content/60">
                    "Name, Anschrift und Kontakt werden in der Adresse gepflegt: "
                    <a href=address_href class="link">"Adresse bearbeiten"</a>
                </p>
                <CompanyFields initial=company.to_input() errors/>
                <Buttons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}

#[component]
fn AddressPicker(
    initial: Option<crate::forms::PickOption>,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    view! {
        <SearchPicker
            field="adresse_id"
            label="Adresse (Firma)"
            placeholder="Firma suchen (Name, Nummer, Ort) …"
            initial
            errors
            search=search_company_addresses
            required=true
        />
    }
}

/// UID, Handelsregister, Fusszeile (AK-6).
#[component]
fn CompanyFields(initial: CompanyInput, errors: Signal<FieldErrors>) -> impl IntoView {
    view! {
        <div class="grid gap-x-4 md:grid-cols-2">
            <div>
                <TextField field="uid" label="UID" value=initial.uid.clone() errors maxlength=25/>
                <p class="text-xs text-base-content/60">"Format CHE-123.456.789; die Prüfziffer wird kontrolliert."</p>
            </div>
            <TextField field="handelsregister" label="Handelsregistereintrag" value=initial.handelsregister.clone() errors maxlength=REGISTER_MAX/>
        </div>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Fusszeile für Belege"</legend>
            <textarea name="input[fusszeile]" rows="2" maxlength=FOOTER_MAX class="textarea w-full"
                class:textarea-error=move || errors.read().contains_key("fusszeile")>{initial.fusszeile.clone()}</textarea>
            <ErrorText field="fusszeile" errors/>
        </fieldset>
    }
}

/// Felder einer neuen Firmenadresse (AK-4); Formularnamen `adresse[…]`.
/// Auch im Modal-Fenster „Mandant definieren“ (ANF-026 AK-17).
#[component]
pub fn NewAddressFields(errors: Signal<FieldErrors>) -> impl IntoView {
    let field = move |name: &'static str, label: &'static str, required: bool, max: usize| {
        let has_error = move || errors.read().contains_key(name);
        view! {
            <fieldset class="fieldset">
                <legend class="fieldset-legend">{label} {required.then_some(" *")}</legend>
                <input type="text" name=format!("adresse[{name}]") maxlength=max class="input w-full"
                    class:input-error=has_error aria-invalid=move || has_error().then_some("true")/>
                <ErrorText field=name errors/>
            </fieldset>
        }
    };
    view! {
        <div class="grid gap-x-4 md:grid-cols-2">
            {field("firmenname", "Firmenname", true, 200)}
            {field("zusatz", "Zusatz", false, 200)}
            {field("strasse", "Strasse", true, 200)}
            {field("postfach", "Postfach", false, 50)}
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            {field("plz", "PLZ", true, 20)}
            {field("ort", "Ort", true, 100)}
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Land *"</legend>
                <select name="adresse[land]" class="select w-full">
                    {COUNTRIES.iter().map(|(c, n)| view! { <option value=*c>{*n}</option> }).collect_view()}
                </select>
                <ErrorText field="land" errors/>
            </fieldset>
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            {field("telefon", "Telefon", false, 50)}
            {field("email", "E-Mail", false, 254)}
            {field("webseite", "Webseite", false, 200)}
        </div>
        <p class="-mt-3 text-xs text-base-content/60">"Die Adresse erscheint danach auch im Adressstamm."</p>
    }
}

/// Logo hochladen, ersetzen oder entfernen (AK-7, Webadresse `?logo=1`).
#[component]
pub fn LogoModal() -> impl IntoView {
    let close = use_base_href();
    let upload = Action::new_local(|data: &FormData| upload_logo(data.clone().into()));
    refresh_when(move || is_saved(upload.value().get()));
    let remove = ServerAction::<RemoveLogo>::new();
    refresh_when(move || matches!(remove.value().get(), Some(Ok(()))));
    let errors = Signal::derive(move || match upload.value().get() {
        Some(Ok(SaveOutcome::Invalid(e))) => e,
        _ => FieldErrors::new(),
    });
    let upload_error = move || {
        upload
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    let remove_error = move || {
        remove
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    let company = Resource::new(data_version, |_| get_company());

    view! {
        <Modal title="Logo".to_string() close_href=close.into()>
            <div class="flex flex-col gap-4">
                {move || upload_error().or_else(remove_error).map(|e| view! {
                    <div role="alert" class="alert alert-error">{e}</div>
                })}
                <Suspense fallback=|| ()>
                    {move || Suspend::new(async move {
                        company.await.ok().flatten().and_then(|c| c.logo_url()).map(|src| view! {
                            <div class="flex flex-wrap items-center gap-4">
                                <img src=src alt="Aktuelles Logo" class="h-16 w-auto max-w-64 object-contain"/>
                                <ActionForm action=remove>
                                    <button type="submit" class="btn btn-outline btn-error btn-sm" disabled=move || remove.pending().get()>
                                        "Logo entfernen"
                                    </button>
                                </ActionForm>
                            </div>
                        })
                    })}
                </Suspense>
                // `action`: funktioniert auch, bevor der Browserteil geladen ist (normales Absenden)
                <form
                    method="post"
                    action=UploadLogo::url()
                    enctype="multipart/form-data"
                    on:submit=move |ev| {
                        ev.prevent_default();
                        let form = event_target::<HtmlFormElement>(&ev);
                        if let Ok(data) = FormData::new_with_form(&form) {
                            upload.dispatch_local(data);
                        }
                    }
                >
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Neues Logo"</legend>
                        <input
                            type="file"
                            name="logo"
                            accept="image/png,image/jpeg,image/svg+xml"
                            class="file-input w-full"
                            class:file-input-error=move || errors.read().contains_key("logo")
                        />
                        <ErrorText field="logo" errors/>
                        <p class="label">"PNG, JPG oder SVG, höchstens 1 MB."</p>
                    </fieldset>
                    <div class="modal-action">
                        <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                        <button type="submit" class="btn btn-primary" disabled=move || upload.pending().get()>
                            {move || if upload.pending().get() { "Hochladen …" } else { "Hochladen" }}
                        </button>
                    </div>
                </form>
            </div>
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

#[component]
fn Buttons(cancel_href: Signal<String>, pending: Signal<bool>) -> impl IntoView {
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
