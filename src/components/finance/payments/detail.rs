//! Detailansicht einer Zahlung bzw. Verrechnung mit Ausgleichszeilen und Storno (ANF-020 AK-10), dazu der
//! Abschnitt „Zahlungen“ in den Detailansichten der Rechnungen.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::backend::finance::debtors::model::{money, rate_text};
use crate::backend::finance::payments::model::{
    AllocationView, PAYMENTS_URL, PaymentDetail, item_label, kind_label, status_badge,
};
use crate::backend::finance::payments::server_fns::{
    StornoPayment, get_payment, list_document_payments,
};
use crate::backend::projects::model::format_date;
use crate::backend::purchase_orders::model::format_amount;
use crate::components::finance::FinanceTabs;
use crate::forms::{Breadcrumbs, data_version, error_message};

#[component]
pub fn PaymentDetailPage() -> impl IntoView {
    let params = use_params_map();
    let storno = ServerAction::<StornoPayment>::new();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let data = Resource::new(
        move || (id(), storno.version().get(), data_version()),
        |(id, _, _)| async move {
            match id {
                Some(id) => get_payment(id).await,
                None => Ok(None),
            }
        },
    );
    view! {
        <div class="space-y-4">
            <FinanceTabs active="zahlungen"/>
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(detail)) => view! { <PaymentView detail storno/> }.into_any(),
                        Ok(None) => view! {
                            <Title text="Zahlung nicht gefunden – uniqwork"/>
                            <div class="card bg-base-100 shadow">
                                <div class="card-body">
                                    <h1 class="card-title">"Zahlung nicht gefunden"</h1>
                                    <a href=PAYMENTS_URL class="btn w-fit">"Zu den Zahlungen"</a>
                                </div>
                            </div>
                        }.into_any(),
                        Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

#[component]
fn PaymentView(detail: PaymentDetail, storno: ServerAction<StornoPayment>) -> impl IntoView {
    let query = use_query_map();
    let p = detail.payment.clone();
    let id = p.id;
    let open = RwSignal::new(false);
    let label = format!("{} vom {}", kind_label(&p.art), format_date(Some(&p.datum)));
    let seite = if p.seite == "kreditoren" {
        "Kreditoren"
    } else {
        "Debitoren"
    };
    let error = move || {
        storno
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    Effect::new(move |_| {
        if matches!(storno.value().get(), Some(Ok(()))) {
            open.set(false);
        }
    });
    let betrag_text = money(&p.waehrung, p.betrag);
    let chf_text = (p.waehrung != "CHF").then(|| {
        format!(
            "{} (Kurs {})",
            money("CHF", p.betrag_chf),
            rate_text(p.kurs)
        )
    });
    let vor_text = (p.vorauszahlung > 0).then(|| {
        format!(
            "{} · offen {}",
            money(&p.waehrung, p.vorauszahlung),
            format_amount(p.vorauszahlung_offen)
        )
    });
    let blocked = (p.status == "gebucht" && !p.can_storno())
        .then_some("Die Vorauszahlung ist schon verrechnet; zuerst die Verrechnung stornieren.");
    view! {
        <Title text=format!("{label} – Zahlungen – uniqwork")/>
        <Breadcrumbs items=vec![
            ("Finanzen".to_string(), Some("/finanzen".to_string())),
            ("Zahlungen".to_string(), Some(PAYMENTS_URL.to_string())),
            (label.clone(), None),
        ]/>
        {move || query.read().get("gespeichert").map(|_| view! { <div role="alert" class="alert alert-success">"Gespeichert."</div> })}
        <div class="flex flex-wrap items-start justify-between gap-2">
            <div>
                <h1 class="text-2xl font-semibold">{label.clone()}</h1>
                <p class="flex flex-wrap items-center gap-2 text-base-content/70">
                    {format!("{seite} · {}", p.partner)}
                    <span class=status_badge(&p.status)>{if p.status == "storniert" { "Storniert" } else { "Gebucht" }}</span>
                </p>
            </div>
            {p.can_storno().then(|| view! {
                <button type="button" class="btn btn-outline btn-warning" on:click=move |_| open.set(true)>"Stornieren"</button>
            })}
        </div>
        {blocked.map(|b| view! { <div role="status" class="alert alert-info alert-soft">{b}</div> })}
        <div class="grid gap-4 md:grid-cols-3">
            <div class="card bg-base-100 shadow">
                <dl class="card-body gap-3">
                    <Info label="Partner"><a href=format!("/adressen/{}", p.adresse_id) class="link link-primary">{p.partner.clone()}</a></Info>
                    <Info label="Bankkonto">{p.bankkonto.clone().unwrap_or_else(|| "– (Verrechnung)".into())}</Info>
                    <Info label="Text">{p.text.clone().unwrap_or_else(|| "–".into())}</Info>
                </dl>
            </div>
            <div class="card bg-base-100 shadow">
                <dl class="card-body gap-3">
                    <Info label="Betrag">{betrag_text.clone()}</Info>
                    {chf_text.clone().map(|t| view! { <Info label="In CHF">{t}</Info> })}
                    {vor_text.clone().map(|t| view! { <Info label="Vorauszahlung">{t}</Info> })}
                </dl>
            </div>
            <div class="card bg-base-100 shadow">
                <dl class="card-body gap-3">
                    <Info label="Buchung">
                        {match p.buchung_id {
                            Some(b) => view! { <a href=format!("/finanzen/journal/{b}") class="link link-primary">"Im Journal anzeigen"</a> }.into_any(),
                            None => view! { "– (ohne Buchung)" }.into_any(),
                        }}
                        {p.storno_buchung_id.map(|s| view! { " · " <a href=format!("/finanzen/journal/{s}") class="link link-primary">"Gegenbuchung"</a> })}
                    </Info>
                    <Info label="Erfasst">{format!("{} · {}", p.erfasst_von.clone().unwrap_or_else(|| "–".into()), p.erstellt_am)}</Info>
                </dl>
            </div>
        </div>
        <AllocationCard title="Ausgeglichene Posten" rows=detail.zeilen show_payment=false/>
        {(!detail.verwendungen.is_empty()).then(|| view! {
            <AllocationCard title="Verrechnung der Vorauszahlung" rows=detail.verwendungen show_payment=true/>
        })}

        <div class="modal max-sm:modal-bottom" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Zahlung stornieren?"</h3>
                <p class="py-4">"Die Buchung wird storniert (Gegenbuchung), offene Beträge und Status der Belege werden zurückgesetzt."</p>
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| open.set(false)>"Abbrechen"</button>
                    <button type="button" class="btn btn-warning" disabled=move || storno.pending().get()
                        on:click=move |_| { storno.dispatch(StornoPayment { id }); }>
                        "Stornieren"
                    </button>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| open.set(false)></div>
        </div>
    }
}

#[component]
fn Info(label: &'static str, children: Children) -> impl IntoView {
    view! {
        <div>
            <dt class="text-sm text-base-content/60">{label}</dt>
            <dd>{children()}</dd>
        </div>
    }
}

/// Tabelle der Ausgleichszeilen; `show_payment` nennt statt des Belegs die Zahlung.
#[component]
fn AllocationCard(
    title: &'static str,
    rows: Vec<AllocationView>,
    show_payment: bool,
) -> impl IntoView {
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">{title}</h2>
                {if rows.is_empty() {
                    view! { <p class="text-base-content/60">"Kein Posten zugeordnet (ganzer Betrag als Vorauszahlung)."</p> }.into_any()
                } else {
                    view! { <AllocationTable rows show_payment/> }.into_any()
                }}
            </div>
        </section>
    }
}

#[component]
fn AllocationTable(rows: Vec<AllocationView>, show_payment: bool) -> impl IntoView {
    view! {
        <div class="overflow-x-auto">
            <table class="table">
                <thead>
                    <tr>
                        <th>{if show_payment { "Zahlung" } else { "Beleg" }}</th>
                        <th class="text-right">"Betrag"</th>
                        <th class="text-right">"Skonto"</th>
                        <th>"Differenz"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>
                    {rows.into_iter().map(|r| {
                        let (href, text) = if show_payment {
                            (format!("{PAYMENTS_URL}/{}", r.zahlung_id), format!("{} vom {}", kind_label(&r.zahlung_art), format_date(Some(&r.zahlung_datum))))
                        } else {
                            (r.href(), format!("{} {}", item_label(&r.art), r.nummer))
                        };
                        let storniert = r.zahlung_status == "storniert";
                        view! {
                            <tr class:opacity-60=storniert>
                                <td><a href=href class="link link-hover">{text}</a></td>
                                <td class="text-right tabular-nums">{format_amount(r.betrag)}</td>
                                <td class="text-right tabular-nums">{(r.skonto > 0).then(|| format_amount(r.skonto))}</td>
                                <td>{(r.differenz > 0).then(|| format!("{} auf {}", format_amount(r.differenz), r.differenz_konto.clone().unwrap_or_default()))}</td>
                                <td><span class=status_badge(&r.zahlung_status)>{if storniert { "Storniert" } else { "Gebucht" }}</span></td>
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </table>
        </div>
    }
}

/// Abschnitt „Zahlungen“ einer Rechnung bzw. Gutschrift mit Knopf zum Erfassen (AK-2, AK-11).
/// `seite`: debitoren oder kreditoren; `key`: Schlüssel des Belegs (`r:12`, `g:5`); `offen`: noch offen.
#[component]
pub fn DocumentPayments(
    seite: &'static str,
    beleg_id: i64,
    key: String,
    offen: bool,
) -> impl IntoView {
    let data = Resource::new(
        move || (beleg_id, data_version()),
        move |(id, _)| list_document_payments(seite.to_string(), id),
    );
    let gutschrift = key.starts_with("g:");
    let neu = match (gutschrift, seite) {
        (true, "kreditoren") => "verrechnung-kreditoren",
        (true, _) => "verrechnung-debitoren",
        (false, "kreditoren") => "ausgang",
        (false, _) => "eingang",
    };
    let href = format!("{PAYMENTS_URL}?neu={neu}&beleg={key}");
    view! {
        <section class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <h2 class="card-title text-base">"Zahlungen"</h2>
                    {offen.then(|| view! {
                        <a href=href class="btn btn-sm btn-primary btn-outline">
                            {if gutschrift { "Verrechnen" } else { "Zahlung erfassen" }}
                        </a>
                    })}
                </div>
                <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match data.await {
                            Ok(rows) if rows.is_empty() => view! { <p class="text-base-content/60">"Noch keine Zahlungen."</p> }.into_any(),
                            Ok(rows) => view! { <AllocationTable rows show_payment=true/> }.into_any(),
                            Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                        }
                    })}
                </Transition>
            </div>
        </section>
    }
}
