//! Abschnitt „Anfragen“ in der Detailansicht eines Vorgangs (ANF-029 AK-1, AK-4 bis AK-6).

use leptos::prelude::*;

use crate::backend::inquiries::model::InquiryRow;
use crate::backend::inquiries::server_fns::{CreateInquiry, DeleteInquiry, list_inquiries};
use crate::forms::{data_version, error_message, refresh_when};

/// Anfragen des Vorgangs mit Schaltfläche „Anfrage erstellen“ und Löschen je Zeile.
#[component]
pub fn VorgangInquiries(vorgang_id: i64) -> impl IntoView {
    let create = ServerAction::<CreateInquiry>::new();
    let delete = ServerAction::<DeleteInquiry>::new();
    refresh_when(move || {
        matches!(create.value().get(), Some(Ok(()))) || matches!(delete.value().get(), Some(Ok(())))
    });
    let data = Resource::new(
        move || (vorgang_id, data_version()),
        |(id, _)| list_inquiries(id),
    );
    let error = move || {
        create
            .value()
            .get()
            .and_then(|r| r.err())
            .or_else(|| delete.value().get().and_then(|r| r.err()))
            .map(|e| error_message(&e))
    };

    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <h2 class="card-title text-base">"Anfragen"</h2>
                    <ActionForm action=create>
                        <input type="hidden" name="vorgang_id" value=vorgang_id/>
                        <button type="submit" class="btn btn-primary btn-sm" disabled=move || create.pending().get()>
                            "Anfrage erstellen"
                        </button>
                    </ActionForm>
                </div>
                <p class="text-sm text-base-content/60">
                    "Die Anfrage übernimmt den Titel des Teilprojekts und die Beschreibung dieses Vorgangs."
                </p>
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match data.await {
                            Ok(rows) if rows.is_empty() => view! {
                                <p class="text-base-content/60">"Noch keine Anfragen für diesen Vorgang."</p>
                            }.into_any(),
                            Ok(rows) => view! {
                                <div class="overflow-x-auto">
                                    <table class="table">
                                        <thead>
                                            <tr>
                                                <th>"Nummer"</th>
                                                <th>"Titel"</th>
                                                <th>"Beschreibung"</th>
                                                <th>"Erstellt am"</th>
                                                <th></th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {rows
                                                .into_iter()
                                                .map(|row| view! { <InquiryTableRow row delete/> })
                                                .collect_view()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any(),
                            Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                        }
                    })}
                </Transition>
            </div>
        </section>
    }
}

/// Eine Zeile der Anfrageliste. „Löschen“ fragt erst nach (AK-6).
#[component]
fn InquiryTableRow(row: InquiryRow, delete: ServerAction<DeleteInquiry>) -> impl IntoView {
    let confirm = RwSignal::new(false);
    let description = row
        .beschreibung
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| "–".into());
    let id = row.id;
    view! {
        <tr>
            <td class="whitespace-nowrap font-mono text-sm">{row.nummer}</td>
            <td class="font-medium">{row.titel}</td>
            <td class="whitespace-pre-line text-sm">{description}</td>
            <td class="whitespace-nowrap text-sm">{row.erstellt_am}</td>
            <td class="text-right">
                <Show
                    when=move || confirm.get()
                    fallback=move || view! {
                        <button type="button" class="btn btn-ghost btn-xs" on:click=move |_| confirm.set(true)>
                            "Löschen"
                        </button>
                    }
                >
                    <span class="inline-flex items-center gap-2">
                        "Wirklich löschen?"
                        <button type="button" class="btn btn-ghost btn-xs" on:click=move |_| confirm.set(false)>
                            "Nein"
                        </button>
                        <ActionForm action=delete>
                            <input type="hidden" name="id" value=id/>
                            <button type="submit" class="btn btn-error btn-xs" disabled=move || delete.pending().get()>
                                "Ja, löschen"
                            </button>
                        </ActionForm>
                    </span>
                </Show>
            </td>
        </tr>
    }
}
