//! Modal-Fenster „Stornieren“ und „Entwurf löschen“ (ANF-017 AK-10, AK-11). Beide verlangen
//! eine Bestätigung, bevor die Aktion ausgelöst wird.

use leptos::prelude::*;

use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::journal::model::BookingDetail;
use crate::backend::finance::journal::server_fns::{
    DeleteDraft, StornoBooking, get_booking, get_storno_default,
};
use crate::backend::purchase_orders::model::format_amount;
use crate::forms::{Modal, error_message, field_errors, server_error, use_base_href};

async fn load_storno(id: i64) -> Result<(BookingDetail, String), ServerFnError> {
    let Some(booking) = get_booking(id).await? else {
        return Err(ServerFnError::new("Die Buchung gibt es nicht mehr."));
    };
    let datum = get_storno_default(id).await?;
    Ok((booking, datum))
}

/// Storno (AK-11): Datum vorgeben, bestätigen, dann Gegenbuchung erstellen.
#[component]
pub fn StornoModal(id: i64) -> impl IntoView {
    let close = use_base_href();
    let data = Resource::new(move || id, load_storno);
    view! {
        <Modal title="Buchung stornieren".to_string() close_href=close.into()>
            <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok((booking, datum)) => view! {
                            <StornoForm booking datum close/>
                        }
                        .into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Suspense>
        </Modal>
    }
}

#[component]
fn StornoForm(booking: BookingDetail, datum: String, close: Memo<String>) -> impl IntoView {
    let action = ServerAction::<StornoBooking>::new();
    let confirm = RwSignal::new(false);
    let errors = Signal::derive(move || field_errors(action.value().get()));
    let server_msg = Signal::derive(move || server_error(action.value().get()));
    let head = booking.head.clone();
    let id = head.id;
    let betrag = format_amount(booking.summe);
    let nummer = head
        .journalnummer
        .map_or("Entwurf".to_string(), |n| format!("Journalnummer {n}"));
    let stornodatum = datum.clone();
    view! {
        <div class="space-y-4">
            <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1 text-sm">
                <dt class="text-base-content/70">"Buchung"</dt>
                <dd>{nummer} " vom " {german_date(&head.datum)}</dd>
                <dt class="text-base-content/70">"Beleg"</dt>
                <dd>{head.belegnummer.clone()}</dd>
                <dt class="text-base-content/70">"Text"</dt>
                <dd>{head.text.clone()}</dd>
                <dt class="text-base-content/70">"Betrag"</dt>
                <dd class="tabular-nums">{betrag}</dd>
            </dl>

            <ActionForm action=action>
                <input type="hidden" name="id" value=id/>
                <input type="hidden" name="zurueck" value=move || close.get()/>
                <div class="flex flex-col gap-4">
                    {move || server_msg.get().map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
                    <fieldset class="fieldset">
                        <legend class="fieldset-legend">"Datum der Gegenbuchung *"</legend>
                        <input
                            type="date"
                            name="datum"
                            class="input w-full sm:w-64"
                            class:input-error=move || errors.read().contains_key("datum")
                            value=stornodatum
                        />
                        {move || errors.read().get("datum").cloned().map(|m| view! { <p class="label text-error">{m}</p> })}
                        <p class="label">"Vorgabe: das Datum der Buchung, bzw. heute, wenn die Periode gesperrt ist."</p>
                    </fieldset>

                    <Show
                        when=move || confirm.get()
                        fallback=move || view! {
                            <p class="text-sm">
                                "Die Buchung bleibt im Journal sichtbar und wird als storniert markiert. "
                                "Die Gegenbuchung hebt sie mit vertauschtem Soll und Haben auf."
                            </p>
                        }
                    >
                        <div role="alertdialog" class="alert alert-warning alert-soft">
                            <span>"Soll die Buchung wirklich storniert werden? Das lässt sich nicht rückgängig machen."</span>
                        </div>
                    </Show>

                    <div class="flex flex-wrap items-center justify-end gap-2">
                        <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                        <Show
                            when=move || confirm.get()
                            fallback=move || view! {
                                <button type="button" class="btn btn-warning" on:click=move |_| confirm.set(true)>
                                    "Stornieren"
                                </button>
                            }
                        >
                            <button type="submit" class="btn btn-error" disabled=move || action.pending().get()>
                                "Endgültig stornieren"
                            </button>
                        </Show>
                    </div>
                </div>
            </ActionForm>
        </div>
    }
}

/// Entwurf löschen (AK-10). Gebuchte Buchungen lassen sich nicht löschen.
#[component]
pub fn DeleteDraftModal(id: i64) -> impl IntoView {
    let close = use_base_href();
    let data = Resource::new(move || id, |id| async move { get_booking(id).await });
    view! {
        <Modal title="Entwurf löschen".to_string() close_href=close.into()>
            <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(booking)) => view! { <DeleteForm booking close/> }.into_any(),
                        Ok(None) => view! {
                            <p>"Dieser Entwurf gibt es nicht mehr."</p>
                        }
                        .into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Suspense>
        </Modal>
    }
}

#[component]
fn DeleteForm(booking: BookingDetail, close: Memo<String>) -> impl IntoView {
    let action = ServerAction::<DeleteDraft>::new();
    let server_msg = Signal::derive(move || {
        action
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    });
    let id = booking.head.id;
    let vorhanden = booking.head.is_draft() && booking.head.herkunft == "manuell";
    let text = booking.head.text.clone();
    let beleg = booking.head.belegnummer.clone();
    view! {
        <div class="space-y-4">
            {move || server_msg.get().map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}
            {if vorhanden {
                view! {
                    <p>"Der Entwurf „" {text} "“ (Beleg " {beleg} ") wird endgültig gelöscht. Das kann nicht rückgängig gemacht werden."</p>
                }
                .into_any()
            } else {
                view! { <p>"Nur Entwürfe lassen sich löschen. Gebuchte Buchungen werden storniert."</p> }.into_any()
            }}
            <ActionForm action=action>
                <input type="hidden" name="id" value=id/>
                <input type="hidden" name="zurueck" value=move || close.get()/>
                <div class="flex flex-wrap items-center justify-end gap-2">
                    <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                    {vorhanden.then(|| view! {
                        <button type="submit" class="btn btn-error" disabled=move || action.pending().get()>
                            "Endgültig löschen"
                        </button>
                    })}
                </div>
            </ActionForm>
        </div>
    }
}
