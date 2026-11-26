//! Modal-Fenster des Kontenplans: Konto anlegen/bearbeiten/löschen, Vorlage übernehmen,
//! Systemkonten zuordnen (ANF-011 AK-3 bis AK-11).

use leptos::prelude::*;

use crate::backend::finance::accounts::model::{
    Account, AccountInput, ChartSummary, FieldErrors, GROUP_MAX, KINDS, KMU_TEMPLATE, NAME_MAX,
    NUMBER_MAX_LEN, SYSTEM_ROLES, kinds_text, role_label, suggested_kind,
};
use crate::backend::finance::accounts::server_fns::{
    ApplyTemplate, CreateAccount, DeleteAccount, SaveSystemAccounts, UpdateAccount, get_account,
    get_chart_summary, list_accounts,
};
use crate::backend::finance::vat::model::percent_label;
use crate::backend::finance::vat::server_fns::get_vat_overview;
use crate::forms::{
    ErrorText, Modal, TextField, data_version, error_message, field_errors, is_saved, refresh_when,
    server_error, use_base_href,
};
use crate::icons::{Icon, IconView};

/// „Neues Konto“ (`id = None`, AK-4) bzw. „Konto bearbeiten“ (AK-6). Lädt das Konto selbst.
#[component]
pub fn AccountModal(id: Option<i64>) -> impl IntoView {
    let close = use_base_href();
    let Some(id) = id else {
        return view! {
            <Modal title="Neues Konto".to_string() close_href=close.into()>
                <AccountCreateForm/>
            </Modal>
        }
        .into_any();
    };
    let data = Resource::new(move || (id, data_version()), |(id, _)| get_account(id));
    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(account)) => view! {
                        <Modal title=format!("Konto {} bearbeiten", account.nummer) close_href=close.into()>
                            <AccountEditForm account/>
                        </Modal>
                    }
                    .into_any(),
                    Ok(None) => view! {
                        <Modal title="Konto nicht gefunden".to_string() close_href=close.into()>
                            <p>"Dieses Konto gibt es nicht oder es wurde gelöscht."</p>
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
        </Suspense>
    }
    .into_any()
}

#[component]
fn AccountCreateForm() -> impl IntoView {
    let action = ServerAction::<CreateAccount>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    view! {
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <Messages errors server_error=Signal::derive(move || server_error(action.value().get()))/>
                <AccountFields initial=AccountInput::new_default() errors/>
                <Buttons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}

#[component]
fn AccountEditForm(account: Account) -> impl IntoView {
    let action = ServerAction::<UpdateAccount>::new();
    refresh_when(move || is_saved(action.value().get()));
    let delete = ServerAction::<DeleteAccount>::new();
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
    let roles: Vec<String> = account
        .rollen
        .iter()
        .map(|r| role_label(r).to_string())
        .collect();
    let id = account.id;
    let nummer = account.nummer.clone();

    view! {
        <ActionForm action=action>
            <input type="hidden" name="id" value=id/>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <Messages errors server_error=Signal::derive(move || server_error(action.value().get()))/>
                {(!roles.is_empty()).then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Systemkonto: " {roles.join(", ")} ". Es muss aktiv bleiben und eine passende Kontoart haben."
                    </div>
                })}
                <AccountFields initial=account.to_input() errors gebucht=account.gebucht/>
                <Buttons cancel_href=close.into() pending=action.pending().into()>
                    // AK-9: Löschen mit Bestätigung
                    <button type="button" class="btn btn-outline btn-error mr-auto" on:click=move |_| confirm.set(true)>
                        <IconView icon=Icon::Trash/>
                        "Löschen"
                    </button>
                </Buttons>
            </div>
        </ActionForm>
        <Show when=move || confirm.get()>
            <div role="alertdialog" class="mt-4 space-y-3 rounded-box border border-error/40 p-4">
                <p>
                    "Soll das Konto " {nummer.clone()}
                    " wirklich gelöscht werden? Das kann nicht rückgängig gemacht werden."
                </p>
                {move || delete_error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="flex justify-end gap-2">
                    <button type="button" class="btn" on:click=move |_| confirm.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
                        <input type="hidden" name="id" value=id/>
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

/// Felder eines Kontos. Die Kontoart folgt der Nummer, solange der Benutzer sie nicht selbst
/// gewählt hat (AK-5).
#[component]
fn AccountFields(
    initial: AccountInput,
    errors: Signal<FieldErrors>,
    #[prop(optional)] gebucht: bool,
) -> impl IntoView {
    // Vorgabe-MWST-Codes für die Auswahl (ANF-011 AK-7)
    let codes = Resource::new(|| (), |_| get_vat_overview());
    let mwst_vorgabe = initial.mwst_code_id.clone();
    let kind = RwSignal::new(initial.kontoart.clone());
    let kind_touched = RwSignal::new(!initial.kontoart.is_empty());
    let has_error = move |f: &'static str| move || errors.read().contains_key(f);

    view! {
        <div class="grid gap-x-4 md:grid-cols-3">
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Nummer *"</legend>
                <input
                    type="text"
                    name="input[nummer]"
                    value=initial.nummer.clone()
                    maxlength=NUMBER_MAX_LEN
                    readonly=gebucht
                    inputmode="numeric"
                    autocomplete="off"
                    class="input w-full tabular-nums"
                    class:input-error=has_error("nummer")
                    aria-invalid=move || has_error("nummer")().then_some("true")
                    on:input=move |ev| {
                        if !kind_touched.get_untracked()
                            && let Some(k) = suggested_kind(&event_target_value(&ev))
                        {
                            kind.set(k.to_string());
                        }
                    }
                />
                <ErrorText field="nummer" errors/>
                <p class="label">"4–6 Ziffern; die erste Ziffer ist die Kontenklasse."</p>
            </fieldset>
            <div class="md:col-span-2">
                <TextField field="bezeichnung" label="Bezeichnung" value=initial.bezeichnung.clone() errors required=true maxlength=NAME_MAX/>
            </div>
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Kontoart *"</legend>
                {gebucht.then(|| view! { <input type="hidden" name="input[kontoart]" value=initial.kontoart.clone()/> })}
                <select
                    name="input[kontoart]"
                    class="select w-full"
                    disabled=gebucht
                    class:select-error=has_error("kontoart")
                    prop:value=move || kind.get()
                    on:change=move |ev| {
                        kind.set(event_target_value(&ev));
                        kind_touched.set(true);
                    }
                >
                    <option value="" selected=move || kind.read().is_empty()>"– bitte wählen –"</option>
                    {KINDS
                        .iter()
                        .map(|(k, l)| view! { <option value=*k selected=move || kind.get() == *k>{*l}</option> })
                        .collect_view()}
                </select>
                <ErrorText field="kontoart" errors/>
            </fieldset>
            <div class="md:col-span-2">
                <TextField field="kontengruppe" label="Kontengruppe" value=initial.kontengruppe.clone() errors maxlength=GROUP_MAX/>
            </div>
        </div>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Vorgabe-MWST-Code"</legend>
            <Suspense fallback=|| ()>
                {move || {
                    let vorgabe = mwst_vorgabe.clone();
                    Suspend::new(async move {
                    let codes = codes.await.map(|o| o.codes).unwrap_or_default();
                    view! {
                        <select name="input[mwst_code_id]" class="select w-full" aria-label="Vorgabe-MWST-Code">
                            <option value="" selected=vorgabe.is_empty()>"– keiner –"</option>
                            {codes
                                .into_iter()
                                .filter(|c| c.aktiv)
                                .map(|c| {
                                    let sel = vorgabe == c.id.to_string();
                                    view! {
                                        <option value=c.id.to_string() selected=sel>
                                            {format!("{} · {}", c.code, percent_label(c.satz))}
                                        </option>
                                    }
                                })
                                .collect_view()}
                        </select>
                    }
                    })
                }}
            </Suspense>
            <ErrorText field="mwst_code_id" errors/>
            <p class="label">"Beim Erfassen einer Buchung auf diesem Konto wird der Code vorgeschlagen."</p>
        </fieldset>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Bemerkung"</legend>
            <textarea name="input[bemerkung]" rows="2" class="textarea w-full">{initial.bemerkung.clone()}</textarea>
            <ErrorText field="bemerkung" errors/>
        </fieldset>
        // AK-10: deaktivieren statt löschen
        <fieldset class="fieldset">
            <label class="label cursor-pointer gap-2">
                <input type="checkbox" name="input[aktiv]" value="1" class="checkbox checkbox-primary" checked=initial.is_active()/>
                "Aktiv – in Auswahllisten für neue Buchungen wählbar"
            </label>
            <ErrorText field="aktiv" errors/>
        </fieldset>
    }
}

/// Bestätigung „KMU-Kontenrahmen übernehmen“ (AK-3, Webadresse `?vorlage=1`).
#[component]
pub fn TemplateModal() -> impl IntoView {
    let action = ServerAction::<ApplyTemplate>::new();
    refresh_when(move || matches!(action.value().get(), Some(Ok(()))));
    let close = use_base_href();
    let error = move || {
        action
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    view! {
        <Modal title="KMU-Kontenrahmen übernehmen".to_string() close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="zurueck" value=move || close.get()/>
                <div class="flex flex-col gap-4">
                    {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                    <p>
                        "Es werden " {KMU_TEMPLATE.len()} " gängige Konten des Schweizer KMU-Kontenrahmens angelegt "
                        "und die Systemkonten vorbelegt (z. B. Debitoren 1100, Kreditoren 2000, Vorsteuer 1170, "
                        "Umsatzsteuer 2200). Alles lässt sich danach anpassen."
                    </p>
                    <div class="modal-action mt-0">
                        <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                        <button type="submit" class="btn btn-primary" disabled=move || action.pending().get()>
                            "Übernehmen"
                        </button>
                    </div>
                </div>
            </ActionForm>
        </Modal>
    }
}

/// Systemkonten zuordnen (AK-11, Webadresse `?systemkonten=1`). Je Rolle nur aktive Konten
/// der passenden Kontoart.
#[component]
pub fn SystemAccountsModal() -> impl IntoView {
    let close = use_base_href();
    let data = Resource::new(data_version, |_| async move {
        let accounts = list_accounts(String::new(), "alle".into(), false).await?;
        let summary = get_chart_summary().await?;
        Ok::<_, ServerFnError>((accounts, summary))
    });
    view! {
        <Modal title="Systemkonten".to_string() close_href=close.into()>
            <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok((accounts, summary)) => view! { <SystemAccountsForm accounts summary/> }.into_any(),
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })}
            </Suspense>
        </Modal>
    }
}

#[component]
fn SystemAccountsForm(accounts: Vec<Account>, summary: ChartSummary) -> impl IntoView {
    let action = ServerAction::<SaveSystemAccounts>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));

    let fields = SYSTEM_ROLES
        .iter()
        .map(|role| {
            let current = summary
                .systemkonten
                .iter()
                .find(|s| s.rolle == role.key)
                .and_then(|s| s.konto.as_ref())
                .map(|a| a.id);
            let options = accounts
                .iter()
                .filter(|a| role.kinds.contains(&a.kontoart.as_str()))
                .map(|a| view! { <option value=a.id selected=current == Some(a.id)>{a.label()}</option> })
                .collect_view();
            let key = role.key;
            view! {
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">{role.label} {role.required.then_some(" *")}</legend>
                    <select
                        name=format!("konten[{key}]")
                        class="select w-full"
                        class:select-error=move || errors.read().contains_key(key)
                    >
                        <option value="" selected=current.is_none()>"– nicht zugeordnet –"</option>
                        {options}
                    </select>
                    <ErrorText field=key errors/>
                    <p class="label">{kinds_text(role.kinds)}</p>
                </fieldset>
            }
        })
        .collect_view();

    view! {
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <Messages errors server_error=Signal::derive(move || server_error(action.value().get()))/>
                <p class="text-sm text-base-content/70">
                    "Angeboten werden nur aktive Konten der passenden Kontoart. "
                    "Mit * markierte Konten sind Pflicht für automatische Buchungen."
                </p>
                <div class="grid gap-x-4 md:grid-cols-2">{fields}</div>
                <Buttons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
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
fn Buttons(
    cancel_href: Signal<String>,
    pending: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <p class="text-sm text-base-content/60">"* Pflichtfeld"</p>
        <div class="modal-action mt-0 flex-wrap">
            {children.map(|c| c())}
            <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
            <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                {move || if pending.get() { "Speichern …" } else { "Speichern" }}
            </button>
        </div>
    }
}
