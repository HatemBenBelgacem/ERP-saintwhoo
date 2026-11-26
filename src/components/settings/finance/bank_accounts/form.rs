//! Modal-Fenster der Bankkonten: anlegen, bearbeiten, löschen (ANF-015 AK-2, AK-5, AK-6).

use leptos::prelude::*;

use crate::backend::finance::accounts::model::Account;
use crate::backend::finance::accounts::server_fns::list_accounts;
use crate::backend::finance::bank_account::model::{
    BANK_MAX, BIC_MAX, BankAccount, BankAccountInput, FieldErrors, KINDS, NAME_MAX,
};
use crate::backend::finance::bank_account::server_fns::{
    CreateBankAccount, DeleteBankAccount, UpdateBankAccount, list_bank_accounts,
};
use crate::forms::{
    ErrorText, Modal, OptionField, SaveButtons, SelectField, ServerAlert, TextField, data_version,
    error_message, field_errors, is_saved, refresh_when, server_error, use_base_href,
};
use crate::icons::{Icon, IconView};

/// Daten für das Formular: Konten im Kontenplan (aktive Aktivkonten) und beim Bearbeiten das Bankkonto.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
struct BankFormData {
    konten: Vec<Account>,
    bankkonto: Option<BankAccount>,
}

async fn load_form(id: Option<i64>) -> Result<BankFormData, ServerFnError> {
    let konten = list_accounts(String::new(), "aktiv".into(), false).await?;
    let bankkonto = match id {
        Some(id) => list_bank_accounts().await?.into_iter().find(|b| b.id == id),
        None => None,
    };
    Ok(BankFormData { konten, bankkonto })
}

/// „Neues Bankkonto“ (`id = None`) bzw. „Bankkonto bearbeiten“ (`id = Some`).
#[component]
pub fn BankAccountModal(id: Option<i64>) -> impl IntoView {
    let close = use_base_href();
    let title = if id.is_some() {
        "Bankkonto bearbeiten"
    } else {
        "Neues Bankkonto"
    };
    let data = Resource::new(move || (id, data_version()), |(id, _)| load_form(id));
    view! {
        <Modal title=title.to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(form) => match (id, form.bankkonto.clone()) {
                            (Some(_), None) => view! {
                                <p>"Dieses Bankkonto gibt es nicht oder es wurde gelöscht."</p>
                            }
                            .into_any(),
                            (Some(_), Some(bank)) => view! {
                                <BankEditForm bank form/>
                            }
                            .into_any(),
                            (None, _) => view! { <BankCreateForm form/> }.into_any(),
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
fn BankCreateForm(form: BankFormData) -> impl IntoView {
    let action = ServerAction::<CreateBankAccount>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    view! {
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerAlert message=Signal::derive(move || server_error(action.value().get()))/>
                <BankFields initial=BankAccountInput::new_default() konten=form.konten errors/>
                <SaveButtons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}

#[component]
fn BankEditForm(bank: BankAccount, form: BankFormData) -> impl IntoView {
    let action = ServerAction::<UpdateBankAccount>::new();
    refresh_when(move || is_saved(action.value().get()));
    let delete = ServerAction::<DeleteBankAccount>::new();
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
    let id = bank.id;
    let name = bank.bezeichnung.clone();
    let standard = bank.standard;
    let initial = bank.to_input();

    view! {
        <ActionForm action=action>
            <input type="hidden" name="id" value=id/>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerAlert message=Signal::derive(move || server_error(action.value().get()))/>
                {standard.then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Dies ist das Standard-Bankkonto. Zum Abwählen oder Deaktivieren zuerst ein anderes Konto als Standard festlegen."
                    </div>
                })}
                <BankFields initial konten=form.konten errors/>
                <div class="modal-action mt-0">
                    <button type="button" class="btn btn-outline btn-error mr-auto" on:click=move |_| confirm.set(true)>
                        <IconView icon=Icon::Trash/>
                        "Löschen"
                    </button>
                    <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                    <button type="submit" class="btn btn-primary" disabled=move || action.pending().get()>
                        "Speichern"
                    </button>
                </div>
            </div>
        </ActionForm>
        <Show when=move || confirm.get()>
            <div role="alertdialog" class="mt-4 space-y-3 rounded-box border border-error/40 p-4">
                <p>"Soll das Bankkonto " {name.clone()} " wirklich gelöscht werden?"</p>
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

/// Felder eines Bankkontos: Bezeichnung, Art, Bank, Konto im Kontenplan, IBAN, QR-IBAN, BIC, Standard, Aktiv.
#[component]
fn BankFields(
    initial: BankAccountInput,
    konten: Vec<Account>,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    let standard = !initial.standard.is_empty();
    let aktiv = !initial.aktiv.is_empty();
    let konto_options: Vec<(String, String)> =
        std::iter::once((String::new(), "– bitte wählen –".to_string()))
            .chain(konten.iter().map(|a| (a.id.to_string(), a.label())))
            .collect();

    view! {
        <div class="grid gap-x-4 md:grid-cols-2">
            <TextField field="bezeichnung" label="Bezeichnung" value=initial.bezeichnung.clone() errors required=true maxlength=NAME_MAX/>
            <SelectField field="art" label="Art" options=KINDS value=initial.art.clone() errors/>
            <TextField field="bank" label="Bank / Institut" value=initial.bank.clone() errors maxlength=BANK_MAX/>
            <OptionField field="konto_id" label="Konto im Kontenplan *" options=konto_options value=initial.konto_id.clone() errors/>
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            <TextField field="iban" label="IBAN (Bank und Post)" value=initial.iban.clone() errors maxlength=50/>
            <TextField field="qr_iban" label="QR-IBAN (optional)" value=initial.qr_iban.clone() errors maxlength=50/>
            <TextField field="bic" label="BIC (optional)" value=initial.bic.clone() errors maxlength=BIC_MAX/>
        </div>
        <div class="flex flex-wrap gap-x-8 gap-y-2">
            <fieldset class="fieldset">
                <label class="label cursor-pointer gap-2">
                    <input type="checkbox" name="input[standard]" value="1" class="checkbox checkbox-primary" checked=standard/>
                    "Standard-Bankkonto (wird in Zahlungen und auf Rechnungen vorgeschlagen)"
                </label>
                <ErrorText field="standard" errors/>
            </fieldset>
            <fieldset class="fieldset">
                <label class="label cursor-pointer gap-2">
                    <input type="checkbox" name="input[aktiv]" value="1" class="checkbox checkbox-primary" checked=aktiv/>
                    "Aktiv"
                </label>
                <ErrorText field="aktiv" errors/>
            </fieldset>
        </div>
    }
}
