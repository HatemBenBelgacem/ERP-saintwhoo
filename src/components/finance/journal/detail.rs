//! Detailansicht einer Buchung unter `/finanzen/journal/<kennung>` (ANF-017 AK-13): alle Zeilen,
//! Herkunft, erfassender Benutzer, Zeitpunkte und Stornoverweis. Aktionen öffnen die Modal-Fenster der Liste.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::journal::model::{
    BookingDetail, amount_cell, document_url, herkunft_label, status_badge, status_label,
    storno_hint,
};
use crate::backend::finance::journal::server_fns::{JOURNAL_URL, get_booking};
use crate::backend::purchase_orders::model::format_amount;
use crate::components::finance::FinanceTabs;
use crate::forms::{data_version, error_message, with_params};
use crate::icons::{Icon, IconView};

#[component]
pub fn BookingDetailPage() -> impl IntoView {
    let params = use_params_map();
    let id = Memo::new(move |_| {
        params
            .read()
            .get("id")
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or(0)
    });
    let data = Resource::new(
        move || (id.get(), data_version()),
        |(id, _)| get_booking(id),
    );

    view! {
        <Title text="Buchung – Finanzen – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Buchung"</h1>
                <a href=JOURNAL_URL class="btn btn-ghost btn-sm">
                    <IconView icon=Icon::Back/>
                    "Zurück zum Journal"
                </a>
            </div>
            <FinanceTabs active="journal"/>

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(booking)) => view! { <BookingCard booking/> }.into_any(),
                        Ok(None) => view! {
                            <div role="alert" class="alert alert-error">"Buchung nicht gefunden."</div>
                        }
                        .into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

#[component]
fn BookingCard(booking: BookingDetail) -> impl IntoView {
    let head = booking.head.clone();
    let id = head.id;
    let hinweis = storno_hint(&head);
    let nummer = head
        .journalnummer
        .map_or("Entwurf".to_string(), |n| format!("Journalnummer {n}"));
    let zeilen = booking
        .zeilen
        .iter()
        .map(|z| {
            view! {
                <tr class:opacity-60=z.ist_mwst_zeile>
                    <td>{z.seite()}</td>
                    <td class="whitespace-nowrap">{format!("{} · {}", z.konto_nummer, z.konto_bezeichnung)}</td>
                    <td class="text-right tabular-nums">{amount_cell(z.soll)}</td>
                    <td class="text-right tabular-nums">{amount_cell(z.haben)}</td>
                    <td>
                        {if z.ist_mwst_zeile {
                            "Steuerzeile".to_string()
                        } else {
                            match (&z.mwst_code, z.mwst_basis) {
                                (Some(code), Some(basis)) => format!("{code} (Basis {})", format_amount(basis)),
                                _ => String::new(),
                            }
                        }}
                    </td>
                    <td>
                        {z.projekt_id.zip(z.projekt_nummer.clone()).map(|(pid, nummer)| view! {
                            <a href=format!("/projekte/{pid}") class="link link-hover">{nummer}</a>
                        })}
                    </td>
                </tr>
            }
        })
        .collect_view();
    let betrag = format_amount(booking.summe);

    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-4">
                <div class="flex flex-wrap items-center gap-2">
                    <span class="text-lg font-semibold">{nummer}</span>
                    <span class=status_badge(&head.status)>{status_label(&head.status)}</span>
                    <span class="badge badge-ghost">{herkunft_label(&head.herkunft)}</span>
                </div>
                <dl class="grid grid-cols-[auto_1fr] gap-x-6 gap-y-1 text-sm sm:grid-cols-[auto_1fr_auto_1fr]">
                    <dt class="text-base-content/70">"Datum"</dt>
                    <dd>{german_date(&head.datum)}</dd>
                    <dt class="text-base-content/70">"Beleg"</dt>
                    <dd>{head.belegnummer.clone()}</dd>
                    <dt class="text-base-content/70">"Text"</dt>
                    <dd class="sm:col-span-3">{head.text.clone()}</dd>
                    <dt class="text-base-content/70">"Erfasst von"</dt>
                    <dd>{head.erfasst_von.clone().unwrap_or_else(|| "–".into())}</dd>
                    <dt class="text-base-content/70">"Erstellt am"</dt>
                    <dd>{head.erstellt_am.clone()}</dd>
                    <dt class="text-base-content/70">"Gebucht am"</dt>
                    <dd>{head.gebucht_am.clone().unwrap_or_else(|| "–".into())}</dd>
                </dl>

                {booking.storno_von.clone().map(|r| view! {
                    <p class="alert alert-info alert-soft text-sm">
                        "Storno der Buchung "
                        <a href=format!("/finanzen/journal/{}", r.id) class="link link-hover">{r.label.clone()}</a>
                    </p>
                })}
                // AK-12: Verweis zum Herkunftsbeleg (Ausgangs- bzw. Eingangsrechnung, ANF-018, ANF-019)
                {booking.herkunftsbeleg.clone().map(|r| view! {
                    <p class="alert alert-info alert-soft text-sm">
                        "Automatische Buchung aus "
                        <a href=document_url(&head.herkunft, r.id) class="link link-hover">{r.label.clone()}</a>
                        ". Storniert wird sie über den Beleg."
                    </p>
                })}
                {booking.storniert_durch.clone().map(|r| view! {
                    <p class="alert alert-warning alert-soft text-sm">
                        "Storniert durch "
                        <a href=format!("/finanzen/journal/{}", r.id) class="link link-hover">{r.label.clone()}</a>
                    </p>
                })}

                <div class="overflow-x-auto">
                    <table class="table table-sm">
                        <thead>
                            <tr>
                                <th>"Seite"</th>
                                <th>"Konto"</th>
                                <th class="text-right">"Soll"</th>
                                <th class="text-right">"Haben"</th>
                                <th>"MWST"</th>
                                <th>"Projekt"</th>
                            </tr>
                        </thead>
                        <tbody>{zeilen}</tbody>
                        <tfoot>
                            <tr class="font-semibold">
                                <td colspan="2">"Summe"</td>
                                <td class="text-right tabular-nums">{betrag.clone()}</td>
                                <td class="text-right tabular-nums">{betrag}</td>
                                <td colspan="2"></td>
                            </tr>
                        </tfoot>
                    </table>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                    {head.is_editable().then(|| view! {
                        <a href=with_params(JOURNAL_URL, &[("bearbeiten", id.to_string())]) class="btn btn-sm">
                            <IconView icon=Icon::Pencil/>
                            "Bearbeiten und buchen"
                        </a>
                        <a href=with_params(JOURNAL_URL, &[("loeschen", id.to_string())]) class="btn btn-sm btn-outline btn-error">
                            <IconView icon=Icon::Trash/>
                            "Entwurf löschen"
                        </a>
                    })}
                    {head.can_storno().then(|| view! {
                        <a href=with_params(JOURNAL_URL, &[("storno", id.to_string())]) class="btn btn-sm btn-warning">
                            "Stornieren"
                        </a>
                    })}
                    <a href=with_params(JOURNAL_URL, &[("kopieren", id.to_string())]) class="btn btn-sm btn-ghost">
                        "Kopieren"
                    </a>
                    {hinweis.map(|text| view! { <span class="text-sm text-base-content/70">{text}</span> })}
                </div>
            </div>
        </div>
    }
}
