//! Modal-Fenster der MWST-Seite: Code anlegen/bearbeiten/löschen, Einstellungen, Standard-Codes
//! (ANF-012 AK-2 bis AK-8).

use leptos::prelude::*;

use crate::backend::finance::accounts::model::{Account, ChartSummary};
use crate::backend::finance::accounts::server_fns::{get_chart_summary, list_accounts};
use crate::backend::finance::vat::model::{
    ACCOUNTING, FORM_MAX, FieldErrors, KINDS, METHODS, NAME_MAX, PERIODS, STANDARD_CODES, VatCode,
    VatCodeInput, VatOverview, VatSettingsInput, konto_kind_for, system_role_for,
};
use crate::backend::finance::vat::server_fns::{
    AdoptStandardCodes, CreateVatCode, DeleteVatCode, SaveVatSettings, UpdateVatCode,
    get_vat_overview,
};
use crate::forms::{
    ErrorText, Modal, SelectField, TextField, data_version, error_message, field_errors, is_saved,
    refresh_when, server_error, use_base_href,
};
use crate::icons::{Icon, IconView};

/// Daten für das Code-Formular: Konten, Systemkonten und (beim Bearbeiten) der Code.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct CodeFormData {
    konten: Vec<Account>,
    systemkonten: ChartSummary,
    code: Option<VatCode>,
}

async fn load_code_form(id: Option<i64>) -> Result<CodeFormData, ServerFnError> {
    let konten = list_accounts(String::new(), String::new(), false).await?;
    let systemkonten = get_chart_summary().await?;
    let code = match id {
        Some(id) => get_vat_overview()
            .await?
            .codes
            .into_iter()
            .find(|c| c.id == id),
        None => None,
    };
    Ok(CodeFormData {
        konten,
        systemkonten,
        code,
    })
}

/// „Neuer MWST-Code“ (`id = None`, AK-3) bzw. „MWST-Code bearbeiten“ (AK-5, AK-7).
#[component]
pub fn CodeModal(id: Option<i64>) -> impl IntoView {
    let close = use_base_href();
    let title = if id.is_some() {
        "MWST-Code bearbeiten"
    } else {
        "Neuer MWST-Code"
    };
    let data = Resource::new(move || (id, data_version()), |(id, _)| load_code_form(id));
    view! {
        <Modal title=title.to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(form) => match (id, form.code.clone()) {
                            (Some(_), None) => view! {
                                <p>"Dieser MWST-Code gibt es nicht oder er wurde gelöscht."</p>
                            }
                            .into_any(),
                            (Some(_), Some(code)) => view! {
                                <CodeEditForm code form=form.clone()/>
                            }
                            .into_any(),
                            (None, _) => view! { <CodeCreateForm form/> }.into_any(),
                        },
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

#[component]
fn CodeCreateForm(form: CodeFormData) -> impl IntoView {
    let action = ServerAction::<CreateVatCode>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    view! {
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerMessage message=Signal::derive(move || server_error(action.value().get()))/>
                <CodeFields
                    initial=VatCodeInput::new_default()
                    used=false
                    form
                    errors
                />
                <FormButtons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}

#[component]
fn CodeEditForm(code: VatCode, form: CodeFormData) -> impl IntoView {
    let action = ServerAction::<UpdateVatCode>::new();
    refresh_when(move || is_saved(action.value().get()));
    let delete = ServerAction::<DeleteVatCode>::new();
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
    let id = code.id;
    let used = code.verwendet;
    let name = code.code.clone();

    view! {
        <ActionForm action=action>
            <input type="hidden" name="id" value=id/>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerMessage message=Signal::derive(move || server_error(action.value().get()))/>
                {used.then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Dieser Code wird bereits verwendet: Satz und Art bleiben gesperrt. "
                        "Mit „Gültig bis“ beenden und einen neuen Code anlegen."
                    </div>
                })}
                <CodeFields initial=code.to_input() used form errors/>
                <div class="modal-action mt-0">
                    {(!used).then(|| view! {
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
                <p>"Soll der MWST-Code " {name.clone()} " wirklich gelöscht werden?"</p>
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

/// Felder eines MWST-Codes. Konto-Vorschlag nach Art (AK-4); bei verwendeten Codes sind Art und Satz gesperrt (AK-5).
#[component]
fn CodeFields(
    initial: VatCodeInput,
    used: bool,
    form: CodeFormData,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    let art = RwSignal::new(initial.art.clone());
    let konto = RwSignal::new(initial.konto_id.clone());
    let konto_touched = RwSignal::new(!initial.konto_id.is_empty());
    let aktiv = !initial.aktiv.is_empty();
    let has_error = move |f: &'static str| move || errors.read().contains_key(f);

    // Systemkonto je Art, z. B. Vorsteuer → 1170 (AK-4)
    let defaults: Vec<(String, String)> = KINDS
        .iter()
        .map(|(kind, _)| {
            let role = system_role_for(kind);
            let id = form
                .systemkonten
                .systemkonten
                .iter()
                .find(|s| s.rolle == role)
                .and_then(|s| s.konto.as_ref())
                .map(|a| a.id.to_string())
                .unwrap_or_default();
            ((*kind).to_string(), id)
        })
        .collect();
    let default_konto = move |kind: &str| {
        defaults
            .iter()
            .find(|(k, _)| k == kind)
            .map(|(_, id)| id.clone())
            .unwrap_or_default()
    };
    let konten = form.konten;

    view! {
        <div class="grid gap-x-4 md:grid-cols-3">
            <TextField field="code" label="Code" value=initial.code.clone() errors required=true maxlength=10/>
            <div class="md:col-span-2">
                <TextField field="bezeichnung" label="Bezeichnung" value=initial.bezeichnung.clone() errors required=true maxlength=NAME_MAX/>
            </div>
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Art *"</legend>
                <select
                    name="input[art]"
                    class="select w-full"
                    class:select-error=has_error("art")
                    disabled=used
                    prop:value=move || art.get()
                    on:change=move |ev| {
                        let value = event_target_value(&ev);
                        if !konto_touched.get_untracked() {
                            konto.set(default_konto(&value));
                        }
                        art.set(value);
                    }
                >
                    {KINDS
                        .iter()
                        .map(|(k, l)| view! { <option value=*k selected=move || art.get() == *k>{*l}</option> })
                        .collect_view()}
                </select>
                {used.then(|| view! { <input type="hidden" name="input[art]" value=initial.art.clone()/> })}
                <ErrorText field="art" errors/>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Satz % *"</legend>
                <input
                    type="text"
                    name="input[satz]"
                    value=initial.satz.clone()
                    readonly=used
                    inputmode="decimal"
                    maxlength=6
                    class="input w-full tabular-nums"
                    class:bg-base-200=used
                    class:input-error=has_error("satz")
                />
                <ErrorText field="satz" errors/>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"MWST-Konto *"</legend>
                <select
                    name="input[konto_id]"
                    class="select w-full"
                    class:select-error=has_error("konto_id")
                    prop:value=move || konto.get()
                    on:change=move |ev| {
                        konto.set(event_target_value(&ev));
                        konto_touched.set(true);
                    }
                >
                    <option value="" selected=move || konto.read().is_empty()>"– bitte wählen –"</option>
                    {move || {
                        let kind = konto_kind_for(&art.get());
                        konten
                            .iter()
                            .filter(|a| a.aktiv && a.kontoart == kind)
                            .map(|a| {
                                let value = a.id.to_string();
                                let selected_value = value.clone();
                                view! {
                                    <option value=value selected=move || konto.get() == selected_value>{a.label()}</option>
                                }
                            })
                            .collect_view()
                    }}
                </select>
                <ErrorText field="konto_id" errors/>
            </fieldset>
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            <TextField field="gueltig_ab" label="Gültig ab" value=initial.gueltig_ab.clone() errors required=true kind="date"/>
            <TextField field="gueltig_bis" label="Gültig bis" value=initial.gueltig_bis.clone() errors kind="date"/>
            <TextField field="formularziffer" label="Formularziffer (ESTV)" value=initial.formularziffer.clone() errors maxlength=FORM_MAX/>
        </div>
        <fieldset class="fieldset">
            <label class="label cursor-pointer gap-2">
                <input type="checkbox" name="input[aktiv]" value="1" class="checkbox checkbox-primary" checked=aktiv/>
                "Aktiv – im Einkauf wählbar"
            </label>
            <ErrorText field="aktiv" errors/>
        </fieldset>
    }
}

/// Einstellungen der MWST: Pflicht mit Beginndatum, Umsatzgrenze, Methode, Art, Periodizität (AK-8, AK-11).
#[component]
pub fn SettingsModal() -> impl IntoView {
    let close = use_base_href();
    let data = Resource::new(data_version, |_| get_vat_overview());
    view! {
        <Modal title="MWST-Einstellungen".to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(overview) => view! { <SettingsForm overview/> }.into_any(),
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

#[component]
fn SettingsForm(overview: VatOverview) -> impl IntoView {
    let action = ServerAction::<SaveVatSettings>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let initial: VatSettingsInput = overview.settings.to_input();
    let pflichtig = !initial.pflichtig.is_empty();
    let uid_missing = overview.uid.is_none();
    view! {
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerMessage message=Signal::derive(move || server_error(action.value().get()))/>
                {uid_missing.then(|| view! {
                    <div role="status" class="alert alert-warning alert-soft">
                        "Keine UID erfasst. Die MWST-Pflicht lässt sich erst einschalten, wenn die UID unter "
                        "Einstellungen → Firma steht."
                    </div>
                })}
                <fieldset class="fieldset">
                    <label class="label cursor-pointer gap-2">
                        <input
                            type="checkbox"
                            name="input[pflichtig]"
                            value="1"
                            class="checkbox checkbox-primary"
                            checked=pflichtig
                        />
                        "MWST-pflichtig (ab dem Beginndatum gelten die MWST-Codes)"
                    </label>
                    <ErrorText field="pflichtig" errors/>
                </fieldset>
                <div class="grid gap-x-4 md:grid-cols-2">
                    <TextField field="pflichtig_ab" label="Beginndatum der Pflicht" value=initial.pflichtig_ab.clone() errors kind="date"/>
                    <TextField field="umsatzgrenze" label="Umsatzgrenze in CHF" value=initial.umsatzgrenze.clone() errors inputmode="decimal" maxlength=20/>
                </div>
                <div class="grid gap-x-4 md:grid-cols-3">
                    <SelectField field="methode" label="Abrechnungsmethode" options=METHODS value=initial.methode.clone() errors/>
                    <SelectField field="abrechnungsart" label="Abrechnungsart" options=ACCOUNTING value=initial.abrechnungsart.clone() errors/>
                    <SelectField field="periodizitaet" label="Abrechnungsperiode" options=PERIODS value=initial.periodizitaet.clone() errors/>
                </div>
                <FormButtons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}

/// Bestätigung „Schweizer Standard-Codes übernehmen“ (AK-2, Webadresse `?standard=1`).
#[component]
pub fn StandardModal() -> impl IntoView {
    let action = ServerAction::<AdoptStandardCodes>::new();
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
        <Modal title="Schweizer Standard-Codes übernehmen".to_string() close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="zurueck" value=move || close.get()/>
                <div class="flex flex-col gap-4">
                    {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                    <p>
                        "Es werden " {STANDARD_CODES.len()} " Codes angelegt, gültig ab 1. Januar 2024: "
                        {STANDARD_CODES.iter().map(|c| c.code).collect::<Vec<_>>().join(", ")}
                        ". Die Konten kommen aus dem Kontenplan (Systemkonten Vorsteuer und Umsatzsteuer). "
                        "Die Formularziffern bitte anhand des aktuellen ESTV-Formulars ergänzen."
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

/// Meldung des Servers über dem Formular.
#[component]
fn ServerMessage(message: Signal<Option<String>>) -> impl IntoView {
    move || {
        message
            .get()
            .map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })
    }
}

/// Abbrechen und Speichern am Ende eines Formulars.
#[component]
fn FormButtons(cancel_href: Signal<String>, pending: Signal<bool>) -> impl IntoView {
    view! {
        <div class="modal-action mt-0">
            <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
            <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                "Speichern"
            </button>
        </div>
    }
}
