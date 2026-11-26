//! Modal-Fenster der Zahlungsbedingungen: anlegen, bearbeiten, löschen (ANF-016 AK-2, AK-3, AK-4, AK-7).

use leptos::prelude::*;

use crate::backend::finance::payment_term::model::{
    FieldErrors, NAME_MAX, PaymentTerm, PaymentTermInput, TEXT_MAX, suggest_text,
};
use crate::backend::finance::payment_term::server_fns::{
    CreatePaymentTerm, DeletePaymentTerm, UpdatePaymentTerm, list_payment_terms,
};
use crate::forms::{
    ErrorText, Modal, SaveButtons, ServerAlert, TextField, data_version, error_message,
    field_errors, is_saved, refresh_when, server_error, use_base_href,
};
use crate::icons::{Icon, IconView};

/// „Neue Zahlungsbedingung“ (`id = None`) bzw. „Zahlungsbedingung bearbeiten“ (`id = Some`).
#[component]
pub fn PaymentTermModal(id: Option<i64>) -> impl IntoView {
    let close = use_base_href();
    let title = if id.is_some() {
        "Zahlungsbedingung bearbeiten"
    } else {
        "Neue Zahlungsbedingung"
    };
    let data = Resource::new(move || (id, data_version()), |(id, _)| load_form(id));
    view! {
        <Modal title=title.to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok((terms, term)) => match (id, term) {
                            (Some(_), None) => view! {
                                <p>"Diese Zahlungsbedingung gibt es nicht oder sie wurde gelöscht."</p>
                            }
                            .into_any(),
                            (Some(_), Some(term)) => view! { <TermEditForm term/> }.into_any(),
                            (None, _) => view! { <TermCreateForm terms/> }.into_any(),
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

/// Lädt die Bedingungen (für die Standard-Regel) und beim Bearbeiten die gewählte Bedingung.
async fn load_form(
    id: Option<i64>,
) -> Result<(Vec<PaymentTerm>, Option<PaymentTerm>), ServerFnError> {
    let terms = list_payment_terms().await?;
    let term = id.and_then(|id| terms.iter().find(|t| t.id == id).cloned());
    Ok((terms, term))
}

#[component]
fn TermCreateForm(terms: Vec<PaymentTerm>) -> impl IntoView {
    let action = ServerAction::<CreatePaymentTerm>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    // Die erste Bedingung wird automatisch Standard (AK-4); das zeigt das Formular.
    let first = terms.is_empty();
    view! {
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerAlert message=Signal::derive(move || server_error(action.value().get()))/>
                {first.then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Die erste Zahlungsbedingung wird automatisch Standard."
                    </div>
                })}
                <TermFields initial=PaymentTermInput::new_default() errors neu=true/>
                <SaveButtons cancel_href=close.into() pending=action.pending().into()/>
            </div>
        </ActionForm>
    }
}

#[component]
fn TermEditForm(term: PaymentTerm) -> impl IntoView {
    let action = ServerAction::<UpdatePaymentTerm>::new();
    refresh_when(move || is_saved(action.value().get()));
    let delete = ServerAction::<DeletePaymentTerm>::new();
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
    let id = term.id;
    let name = term.bezeichnung.clone();
    let standard = term.standard;
    let used = term.verwendet;
    let initial = term.to_input();

    view! {
        <ActionForm action=action>
            <input type="hidden" name="id" value=id/>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerAlert message=Signal::derive(move || server_error(action.value().get()))/>
                {standard.then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Dies ist die Standard-Bedingung. Zum Abwählen oder Deaktivieren zuerst eine andere als Standard festlegen."
                    </div>
                })}
                {used.then(|| view! {
                    <div role="status" class="alert alert-info alert-soft">
                        "Von Adressen als Vorgabe verwendet: Sie kann nicht gelöscht, nur deaktiviert werden."
                    </div>
                })}
                <TermFields initial errors neu=false/>
                <div class="modal-action mt-0">
                    {(!standard && !used).then(|| view! {
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
                <p>"Soll die Zahlungsbedingung " {name.clone()} " wirklich gelöscht werden?"</p>
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

/// Felder einer Zahlungsbedingung. Beim Anlegen folgt der Rechnungstext den Eingaben, bis er von Hand
/// geändert wird (AK-3); beim Bearbeiten bleibt der gespeicherte Text, bis er geändert wird.
#[component]
fn TermFields(initial: PaymentTermInput, errors: Signal<FieldErrors>, neu: bool) -> impl IntoView {
    let frist = RwSignal::new(initial.zahlungsfrist.clone());
    let skontofrist = RwSignal::new(initial.skontofrist.clone());
    let skonto = RwSignal::new(initial.skonto.clone());
    let text = RwSignal::new(initial.rechnungstext.clone());
    let von_hand = RwSignal::new(!neu);
    let aktiv = !initial.aktiv.is_empty();
    let standard = !initial.standard.is_empty();

    // Vorschlag aus den aktuellen Eingaben; ungültige Eingaben lassen den Text unverändert.
    let refresh = move || {
        if von_hand.get_untracked() {
            return;
        }
        let eingabe = PaymentTermInput {
            zahlungsfrist: frist.get_untracked(),
            skontofrist: skontofrist.get_untracked(),
            skonto: skonto.get_untracked(),
            ..Default::default()
        }
        .normalized();
        if let Some((tage, prozent)) = eingabe.terms() {
            text.set(suggest_text(tage, prozent));
        }
    };

    view! {
        <div class="grid gap-x-4 md:grid-cols-3">
            <div class="md:col-span-2">
                <TextField field="bezeichnung" label="Bezeichnung" value=initial.bezeichnung.clone() errors required=true maxlength=NAME_MAX/>
            </div>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Zahlungsfrist in Tagen *"</legend>
                <input
                    type="text"
                    name="input[zahlungsfrist]"
                    inputmode="numeric"
                    maxlength=3
                    class="input w-full tabular-nums"
                    class:input-error=move || errors.read().contains_key("zahlungsfrist")
                    prop:value=move || frist.get()
                    on:input=move |ev| {
                        frist.set(event_target_value(&ev));
                        refresh();
                    }
                />
                <ErrorText field="zahlungsfrist" errors/>
            </fieldset>
        </div>
        <div class="grid gap-x-4 md:grid-cols-3">
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Skontofrist in Tagen"</legend>
                <input
                    type="text"
                    name="input[skontofrist]"
                    inputmode="numeric"
                    maxlength=3
                    class="input w-full tabular-nums"
                    class:input-error=move || errors.read().contains_key("skontofrist")
                    prop:value=move || skontofrist.get()
                    on:input=move |ev| {
                        skontofrist.set(event_target_value(&ev));
                        refresh();
                    }
                />
                <ErrorText field="skontofrist" errors/>
            </fieldset>
            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Skonto in %"</legend>
                <input
                    type="text"
                    name="input[skonto]"
                    inputmode="decimal"
                    maxlength=6
                    class="input w-full tabular-nums"
                    class:input-error=move || errors.read().contains_key("skonto")
                    prop:value=move || skonto.get()
                    on:input=move |ev| {
                        skonto.set(event_target_value(&ev));
                        refresh();
                    }
                />
                <ErrorText field="skonto" errors/>
            </fieldset>
            <p class="self-end pb-2 text-sm text-base-content/60">
                "Skontofrist und Skonto gemeinsam angeben oder beide leer lassen."
            </p>
        </div>
        <fieldset class="fieldset">
            <legend class="fieldset-legend">"Rechnungstext *"</legend>
            <input
                type="text"
                name="input[rechnungstext]"
                maxlength=TEXT_MAX
                class="input w-full"
                class:input-error=move || errors.read().contains_key("rechnungstext")
                prop:value=move || text.get()
                on:input=move |ev| {
                    text.set(event_target_value(&ev));
                    von_hand.set(true);
                }
            />
            <p class="label">
                "Vorschlag aus Frist und Skonto. Er passt sich an, bis Sie den Text von Hand ändern."
            </p>
            <ErrorText field="rechnungstext" errors/>
        </fieldset>
        <div class="flex flex-wrap gap-x-8 gap-y-2">
            <fieldset class="fieldset">
                <label class="label cursor-pointer gap-2">
                    <input type="checkbox" name="input[standard]" value="1" class="checkbox checkbox-primary" checked=standard/>
                    "Standard-Bedingung (wird bei neuen Rechnungen vorgeschlagen)"
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
