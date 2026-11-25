//! Modal-Fenster der Geschäftsjahre: neues Geschäftsjahr, Periode sperren/entsperren, löschen
//! (ANF-013 AK-2, AK-5, AK-7).

use leptos::prelude::*;

use crate::backend::finance::fiscal_year::model::{FiscalYear, Period, period_status_label};
use crate::backend::finance::fiscal_year::server_fns::{
    CreateFiscalYear, DeleteFiscalYear, SetPeriodLocked, get_fiscal_years,
};
use crate::forms::{
    Modal, TextField, data_version, error_message, field_errors, is_saved, refresh_when,
    server_error, use_base_href,
};

/// Neues Geschäftsjahr (`?neu=1`, AK-2). Der Vorschlag kommt vom Server.
#[component]
pub fn NewYearModal() -> impl IntoView {
    let close = use_base_href();
    let data = Resource::new(data_version, |_| get_fiscal_years());
    view! {
        <Modal title="Neues Geschäftsjahr".to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(overview) => view! {
                            <NewYearForm beginn=overview.vorschlag_beginn ende=overview.vorschlag_ende/>
                        }
                        .into_any(),
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
fn NewYearForm(beginn: String, ende: String) -> impl IntoView {
    let action = ServerAction::<CreateFiscalYear>::new();
    refresh_when(move || is_saved(action.value().get()));
    let close = use_base_href();
    let errors = Signal::derive(move || field_errors(action.value().get()));
    view! {
        <ActionForm action=action>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                <ServerMessage message=Signal::derive(move || server_error(action.value().get()))/>
                <p class="text-sm text-base-content/70">
                    "Beginn am 1. und Ende am letzten Tag eines Monats, Dauer 1 bis 23 Monate. "
                    "Das Geschäftsjahr schliesst lückenlos an das letzte an. Die Monate werden automatisch als Perioden angelegt."
                </p>
                <div class="grid gap-x-4 md:grid-cols-2">
                    <TextField field="beginn" label="Beginn" value=beginn errors required=true kind="date"/>
                    <TextField field="ende" label="Ende" value=ende errors required=true kind="date"/>
                </div>
                <FormButtons cancel_href=close.into() pending=action.pending().into() label="Anlegen"/>
            </div>
        </ActionForm>
    }
}

/// Periode sperren oder entsperren (`?periode=<kennung>`, AK-5). Bestätigung im Fenster.
#[component]
pub fn PeriodLockModal(id: i64) -> impl IntoView {
    let close = use_base_href();
    let data = Resource::new(data_version, |_| get_fiscal_years());
    view! {
        <Modal title="Periode sperren oder entsperren".to_string() close_href=close.into()>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(overview) => match find_period(&overview.jahre, id) {
                            Some(period) => view! { <PeriodLockForm period/> }.into_any(),
                            None => view! { <p>"Diese Periode gibt es nicht."</p> }.into_any(),
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

fn find_period(years: &[FiscalYear], id: i64) -> Option<Period> {
    years
        .iter()
        .flat_map(|y| y.perioden.iter())
        .find(|p| p.id == id)
        .cloned()
}

#[component]
fn PeriodLockForm(period: Period) -> impl IntoView {
    let action = ServerAction::<SetPeriodLocked>::new();
    refresh_when(move || matches!(action.value().get(), Some(Ok(()))));
    let close = use_base_href();
    let locked = period.status == "gesperrt";
    // Der neue Zustand: entsperren, wenn gesperrt; sonst sperren
    let gesperrt_nach = if locked { "false" } else { "true" };
    let label = period.label();
    let text = if locked {
        format!(
            "Die Periode {label} wird wieder geöffnet. Danach kann in diesem Monat gebucht werden."
        )
    } else {
        format!(
            "Die Periode {label} wird gesperrt. Danach kann in diesem Monat nicht mehr gebucht werden."
        )
    };
    let error = move || {
        action
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    view! {
        <ActionForm action=action>
            <input type="hidden" name="periode_id" value=period.id/>
            <input type="hidden" name="gesperrt" value=gesperrt_nach/>
            <input type="hidden" name="zurueck" value=move || close.get()/>
            <div class="flex flex-col gap-4">
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <p>{text}</p>
                <p class="text-sm text-base-content/70">
                    "Aktueller Status: " {period_status_label(&period.status)}
                </p>
                <div class="modal-action mt-0">
                    <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                    <button type="submit" class="btn btn-primary" disabled=move || action.pending().get()>
                        {if locked { "Entsperren" } else { "Sperren" }}
                    </button>
                </div>
            </div>
        </ActionForm>
    }
}

/// Geschäftsjahr löschen (`?loeschen=<kennung>`, AK-7). Nur das neueste ist löschbar.
#[component]
pub fn DeleteYearModal(id: i64) -> impl IntoView {
    let action = ServerAction::<DeleteFiscalYear>::new();
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
        <Modal title="Geschäftsjahr löschen".to_string() close_href=close.into()>
            <ActionForm action=action>
                <input type="hidden" name="id" value=id/>
                <input type="hidden" name="zurueck" value=move || close.get()/>
                <div class="flex flex-col gap-4">
                    {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                    <p>
                        "Das Geschäftsjahr mit allen Perioden wird gelöscht. Das ist nur möglich, "
                        "wenn es keine Buchungen gibt."
                    </p>
                    <div class="modal-action mt-0">
                        <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                        <button type="submit" class="btn btn-error" disabled=move || action.pending().get()>
                            "Endgültig löschen"
                        </button>
                    </div>
                </div>
            </ActionForm>
        </Modal>
    }
}

/// Abbrechen und Absenden am Ende eines Formulars.
#[component]
fn FormButtons(
    cancel_href: Signal<String>,
    pending: Signal<bool>,
    label: &'static str,
) -> impl IntoView {
    view! {
        <div class="modal-action mt-0">
            <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
            <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                {label}
            </button>
        </div>
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
