//! Druckansicht einer Rechnung bzw. Gutschrift für A4 (ANF-018 AK-11), ohne Grundgerüst.
//! Absender aus der Firma (ANF-025 AK-9, AK-10), Bankverbindung aus dem Standard-Bankkonto (ANF-015).
//! Die Seite ist bewusst auf A4-Breite begrenzt (Ausnahme von „ganze Bildschirmbreite“).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_params_map;

use crate::backend::finance::bank_account::model::group_iban;
use crate::backend::finance::debtors::model::{DEBTORS_URL, InvoicePrint, kind_label};
use crate::backend::finance::debtors::server_fns::get_invoice_print;
use crate::backend::finance::vat::model::percent_label;
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::{format_amount, format_trimmed, unit_label};
use crate::components::settings::SenderBlock;
use crate::forms::error_message;

#[component]
pub fn InvoicePrintPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let data = Resource::new(id, |id| async move {
        match id {
            Some(id) => get_invoice_print(id).await,
            None => Ok(None),
        }
    });
    view! {
        <div class="min-h-screen bg-base-200 py-6 print:bg-base-100 print:py-0">
            <Transition fallback=|| view! { <span class="loading loading-spinner m-8"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(print)) => view! { <PrintSheet print/> }.into_any(),
                        Ok(None) => view! {
                            <Title text="Rechnung nicht gefunden – uniqwork"/>
                            <div class="card mx-4 bg-base-100 shadow">
                                <div class="card-body">
                                    <h1 class="card-title">"Rechnung nicht gefunden"</h1>
                                    <a href=DEBTORS_URL class="btn w-fit">"Zu den Debitoren"</a>
                                </div>
                            </div>
                        }.into_any(),
                        Err(e) => view! { <div role="alert" class="alert alert-error m-4">{error_message(&e)}</div> }.into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

#[component]
fn PrintSheet(print: InvoicePrint) -> impl IntoView {
    let totals = print.detail.totals();
    let inv = print.detail.invoice.clone();
    let kind = kind_label(&inv.art);
    let title = match &inv.nummer {
        Some(n) => format!("{kind} {n}"),
        None => format!("{kind} (Entwurf)"),
    };
    let currency = inv.waehrung.clone();
    let rows = print
        .detail
        .positions
        .iter()
        .enumerate()
        .map(|(i, p)| {
            view! {
                <tr class="border-b border-base-300 align-top">
                    <td class="py-1 pr-2">{i + 1}</td>
                    <td class="py-1 pr-2">{p.bezeichnung.clone()}</td>
                    <td class="py-1 pr-2 text-right tabular-nums whitespace-nowrap">
                        {format_trimmed(p.menge, 3)} " " {unit_label(&p.einheit).to_string()}
                    </td>
                    <td class="py-1 pr-2 text-right tabular-nums">{format_amount(p.einzelpreis)}</td>
                    <td class="py-1 pr-2 text-right tabular-nums">
                        {(p.rabatt_prozent > 0).then(|| format!("{} %", format_trimmed(p.rabatt_prozent, 2)))}
                    </td>
                    <td class="py-1 pr-2 text-right">{p.mwst_satz.map(percent_label)}</td>
                    <td class="py-1 text-right tabular-nums">{format_amount(p.amount())}</td>
                </tr>
            }
        })
        .collect_view();
    let vat_rows = totals
        .vat
        .iter()
        .map(|v| {
            view! {
                <tr>
                    <td class="py-0.5 pr-4">{format!("MWST {} auf {} {}", percent_label(v.rate), currency, format_amount(v.base))}</td>
                    <td class="py-0.5 text-right tabular-nums">{format_amount(v.vat)}</td>
                </tr>
            }
        })
        .collect_view();
    let draft = inv.is_draft();
    let back = format!("{DEBTORS_URL}/{}", inv.id);

    view! {
        <Title text=format!("{title} – Druck – uniqwork")/>
        // Werkzeugleiste nur am Bildschirm
        <div class="mx-auto mb-4 flex w-full max-w-[210mm] flex-wrap justify-between gap-2 px-4 print:hidden">
            <a href=back class="btn btn-sm">"‹ Zurück zum Beleg"</a>
            <button type="button" class="btn btn-sm btn-primary" onclick="window.print()">"Drucken"</button>
        </div>
        <article class="mx-auto flex min-h-[297mm] w-full max-w-[210mm] flex-col gap-8 bg-base-100 p-[15mm] text-sm text-base-content shadow print:min-h-0 print:p-0 print:shadow-none">
            <SenderBlock company=print.company.clone()/>

            {draft.then(|| view! {
                <div role="status" class="alert alert-warning">
                    <span class="font-semibold uppercase tracking-widest">"Entwurf"</span>
                    "– nicht gebucht, keine gültige Rechnung"
                </div>
            })}

            <div class="grid grid-cols-2 gap-8">
                <address class="not-italic leading-snug">
                    {print.customer.lines().into_iter().map(|l| view! { <p>{l}</p> }).collect_view()}
                </address>
                <dl class="grid grid-cols-[auto_1fr] gap-x-4 gap-y-0.5 self-start">
                    <dt class="text-base-content/70">{format!("{kind}-Nr.")}</dt>
                    <dd>{inv.nummer.clone().unwrap_or_else(|| "–".into())}</dd>
                    <dt class="text-base-content/70">"Datum"</dt>
                    <dd>{format_date(Some(&inv.rechnungsdatum))}</dd>
                    {inv.faelligkeit.clone().map(|f| view! {
                        <dt class="text-base-content/70">"Zahlbar bis"</dt>
                        <dd>{format_date(Some(&f))}</dd>
                    })}
                    <dt class="text-base-content/70">"Kunden-Nr."</dt>
                    <dd>{inv.kunde_nummer.clone()}</dd>
                    {inv.referenz_kunde.clone().map(|r| view! {
                        <dt class="text-base-content/70">"Ihre Referenz"</dt>
                        <dd>{r}</dd>
                    })}
                    {inv.projekt_nummer.clone().map(|n| view! {
                        <dt class="text-base-content/70">"Projekt"</dt>
                        <dd>{format!("{n} {}", inv.projekt_name.clone().unwrap_or_default())}</dd>
                    })}
                    {inv.bezug_nummer.clone().map(|n| view! {
                        <dt class="text-base-content/70">"Zu Rechnung"</dt>
                        <dd>{n}</dd>
                    })}
                    {print.mwst_nummer.clone().map(|m| view! {
                        <dt class="text-base-content/70">"MWST-Nr."</dt>
                        <dd>{m}</dd>
                    })}
                </dl>
            </div>

            <div class="space-y-3">
                <h1 class="text-xl font-semibold">{title}</h1>
                {inv.einleitungstext.clone().map(|t| view! { <p class="whitespace-pre-line">{t}</p> })}
            </div>

            <table class="w-full border-collapse">
                <thead>
                    <tr class="border-b-2 border-base-content text-left">
                        <th class="py-1 pr-2">"Pos."</th>
                        <th class="py-1 pr-2">"Bezeichnung"</th>
                        <th class="py-1 pr-2 text-right">"Menge"</th>
                        <th class="py-1 pr-2 text-right">"Preis"</th>
                        <th class="py-1 pr-2 text-right">"Rabatt"</th>
                        <th class="py-1 pr-2 text-right">"MWST"</th>
                        <th class="py-1 text-right">{format!("Betrag {currency}")}</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>

            <table class="ml-auto min-w-80 break-inside-avoid">
                <tbody>
                    <tr>
                        <td class="py-0.5 pr-4">"Zwischensumme"</td>
                        <td class="py-0.5 text-right tabular-nums">{format_amount(totals.subtotal)}</td>
                    </tr>
                    {vat_rows}
                    {(totals.rounding != 0).then(|| view! {
                        <tr>
                            <td class="py-0.5 pr-4">"Rundung"</td>
                            <td class="py-0.5 text-right tabular-nums">{format_amount(totals.rounding)}</td>
                        </tr>
                    })}
                    <tr class="border-t-2 border-base-content font-semibold">
                        <td class="py-1 pr-4">{format!("Gesamtbetrag {currency}")}</td>
                        <td class="py-1 text-right tabular-nums">{format_amount(totals.total)}</td>
                    </tr>
                </tbody>
            </table>

            <div class="space-y-3 break-inside-avoid">
                {(inv.art == "rechnung").then(|| view! { <p>{inv.zahlungstext.clone()}</p> })}
                {match print.bank.clone() {
                    Some(b) if inv.art == "rechnung" => view! {
                        <div>
                            <p class="font-semibold">"Bankverbindung"</p>
                            <p>{b.bank.clone().unwrap_or(b.bezeichnung.clone())}</p>
                            {b.iban.clone().map(|i| view! { <p>"IBAN " {group_iban(&i)}</p> })}
                            {b.bic.clone().map(|c| view! { <p>"BIC " {c}</p> })}
                        </div>
                    }.into_any(),
                    None if inv.art == "rechnung" => view! {
                        <p class="text-warning print:hidden">"Kein Standard-Bankkonto erfasst (Einstellungen → Finanzen → Bankkonten)."</p>
                    }.into_any(),
                    _ => ().into_any(),
                }}
                {inv.schlusstext.clone().map(|t| view! { <p class="whitespace-pre-line">{t}</p> })}
            </div>

            {print.company.as_ref().and_then(|c| c.fusszeile.clone()).map(|f| view! {
                <footer class="mt-auto border-t border-base-300 pt-2 text-xs text-base-content/70">{f}</footer>
            })}
        </article>
    }
}
