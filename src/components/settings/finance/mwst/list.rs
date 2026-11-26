//! Einstellungsseite „MWST“: Einstellungen, Umsatzgrenze und MWST-Codes (ANF-012 AK-1 bis AK-11).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use super::form::{CodeModal, SettingsModal, StandardModal};
use crate::backend::company::model::vat_number;
use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::vat::model::{
    LimitStage, Umsatz, VatCode, VatOverview, VatSettings, accounting_label, kind_label,
    limit_stage, method_label, percent_label, period_label,
};
use crate::backend::finance::vat::server_fns::get_vat_overview;
use crate::backend::purchase_orders::model::format_amount;
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Einstellungsseite „MWST“ unter `/einstellungen/finanzen/mwst`. Eingaben im Modal-Fenster
/// (`?neu=1`, `?bearbeiten=<kennung>`, `?einstellungen=1`, `?standard=1`).
#[component]
pub fn VatPage() -> impl IntoView {
    let query = use_query_map();
    let param = move |name: &str| query.read().get(name);
    let base = use_base_href();
    let data = Resource::new(data_version, |_| get_vat_overview());

    view! {
        <Title text="MWST – Einstellungen – ERP-saintwhoo"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"MWST"</h1>
                <a href=move || with_params(&base.get(), &[("einstellungen", "1".into())]) class="btn">
                    <IconView icon=Icon::Pencil/>
                    "Einstellungen"
                </a>
            </div>

            {move || param("gespeichert").map(|v| {
                let text = match v.as_str() {
                    "mwst-einstellungen" => "MWST-Einstellungen gespeichert.".to_string(),
                    "standard" => "Schweizer Standard-Codes übernommen.".to_string(),
                    code => format!("MWST-Code {code} gespeichert."),
                };
                view! { <div role="alert" class="alert alert-success">{text}</div> }
            })}
            {move || param("geloescht").map(|_| view! {
                <div role="alert" class="alert alert-success">"MWST-Code wurde gelöscht."</div>
            })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(overview) => view! { <VatContent overview base/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?einstellungen=1, ?neu=1, ?bearbeiten=<kennung>, ?standard=1
        {move || param("einstellungen").map(|_| view! { <SettingsModal/> })}
        {move || param("neu").map(|_| view! { <CodeModal id=None/> })}
        {move || {
            param("bearbeiten")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <CodeModal id=Some(id)/> })
        }}
        {move || param("standard").map(|_| view! { <StandardModal/> })}
    }
}

/// Inhalt der Seite nach dem Laden: Einstellungen, Umsatzgrenze und Codes.
#[component]
fn VatContent(overview: VatOverview, base: Memo<String>) -> impl IntoView {
    let VatOverview {
        settings,
        codes,
        uid,
        umsatz,
    } = overview;
    let pflichtig = settings.mwst_pflichtig;
    let limit = settings.umsatzgrenze;
    // Codes nur bei eingeschalteter Pflicht (AK-8); der Abschnitt wird vorab gebaut, damit die Daten nur einmal genutzt werden.
    let codes_section = if pflichtig {
        view! { <CodesSection codes base/> }.into_any()
    } else {
        view! {
            <div role="status" class="alert alert-info alert-soft">
                <span>
                    "Die MWST-Pflicht ist ausgeschaltet. Die MWST-Codes werden erst angezeigt, wenn die Pflicht "
                    "eingeschaltet ist. Dazu unter „Einstellungen“ ein Beginndatum setzen."
                </span>
            </div>
        }
        .into_any()
    };

    view! {
        <SettingsCard settings uid base/>
        <LimitCard limit umsatz/>
        {codes_section}
    }
}

/// Pflicht, Beginn, MWST-Nummer und Abrechnung (AK-6, AK-8).
#[component]
fn SettingsCard(settings: VatSettings, uid: Option<String>, base: Memo<String>) -> impl IntoView {
    let status = match settings.mwst_pflichtig_ab.as_deref() {
        Some(ab) if settings.mwst_pflichtig => view! {
            <span class="badge badge-success badge-soft">{format!("Pflicht ab {}", german_date(ab))}</span>
        }
        .into_any(),
        _ => view! { <span class="badge badge-ghost">"Ausgeschaltet"</span> }.into_any(),
    };
    let number = if !settings.mwst_pflichtig {
        "Erst bei eingeschalteter Pflicht".to_string()
    } else {
        match uid {
            Some(uid) => vat_number(&uid),
            None => "Keine UID erfasst (Einstellungen → Firma)".to_string(),
        }
    };

    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <h2 class="card-title text-base">"MWST-Pflicht"</h2>
                    {status}
                </div>
                <dl class="grid gap-x-6 gap-y-2 text-sm sm:grid-cols-2 lg:grid-cols-4">
                    <div>
                        <dt class="text-base-content/60">"MWST-Nummer"</dt>
                        <dd class="font-medium">{number}</dd>
                    </div>
                    <div>
                        <dt class="text-base-content/60">"Abrechnungsmethode"</dt>
                        <dd class="font-medium">{method_label(&settings.methode)}</dd>
                    </div>
                    <div>
                        <dt class="text-base-content/60">"Abrechnungsart"</dt>
                        <dd class="font-medium">{accounting_label(&settings.abrechnungsart)}</dd>
                    </div>
                    <div>
                        <dt class="text-base-content/60">"Abrechnungsperiode"</dt>
                        <dd class="font-medium">{period_label(&settings.periodizitaet)}</dd>
                    </div>
                </dl>
                <div>
                    <a href=move || with_params(&base.get(), &[("einstellungen", "1".into())]) class="btn btn-sm">
                        <IconView icon=Icon::Pencil/>
                        "Einstellungen bearbeiten"
                    </a>
                </div>
            </div>
        </div>
    }
}

/// Umsatzgrenze (AK-11): Umsatz aus dem Buchungsjournal, Stufen ab 80 % (Hinweis) und 100 % (Warnung).
#[component]
fn LimitCard(limit: i64, umsatz: Umsatz) -> impl IntoView {
    let massgebend = umsatz.massgebend();
    let stufe = limit_stage(massgebend, limit);
    let prozent = |betrag: i64| -> i64 {
        if limit > 0 {
            i64::try_from(i128::from(betrag) * 100 / i128::from(limit)).unwrap_or(0)
        } else {
            0
        }
    };
    let zeile = |text: &'static str, betrag: i64| {
        view! {
            <li class="flex justify-between gap-4">
                <span>{text}</span>
                <span class="tabular-nums">
                    "CHF " {format_amount(betrag)} " (" {prozent(betrag)} " %)"
                </span>
            </li>
        }
    };
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-2">
                <h2 class="card-title text-base">"Umsatzgrenze"</h2>
                <p class="text-sm">
                    "Grenze: CHF " <span class="font-medium tabular-nums">{format_amount(limit)}</span>
                    ". Ab 80 % der Grenze erscheint ein Hinweis, ab 100 % eine Warnung."
                </p>
                <ul class="text-sm space-y-1">
                    {zeile("Gebuchte Erlöse im laufenden Geschäftsjahr", umsatz.geschaeftsjahr)}
                    {zeile("Gebuchte Erlöse in den letzten 12 Monaten", umsatz.letzte_12_monate)}
                </ul>
                <p class="text-xs text-base-content/70">
                    "Massgebend ist der höhere der beiden Werte (Kontenklasse 3 aus dem Buchungsjournal)."
                </p>
                {match stufe {
                    LimitStage::Erreicht => view! {
                        <div role="alert" class="alert alert-error">
                            <span>
                                "Umsatzgrenze erreicht – MWST-Pflicht prüfen und bei der ESTV anmelden. "
                                "Die Grenze lässt sich in den MWST-Einstellungen oben auf dieser Seite ändern."
                            </span>
                        </div>
                    }
                    .into_any(),
                    LimitStage::Hinweis => view! {
                        <div role="status" class="alert alert-warning alert-soft">
                            <span>"Ab 80 % der Umsatzgrenze: Umsatz im Blick behalten."</span>
                        </div>
                    }
                    .into_any(),
                    LimitStage::Unauffaellig => ().into_any(),
                }}
            </div>
        </div>
    }
}

/// Liste der MWST-Codes oder der Hinweis auf die Standard-Codes (AK-1, AK-2).
#[component]
fn CodesSection(codes: Vec<VatCode>, base: Memo<String>) -> impl IntoView {
    let empty = codes.is_empty();
    view! {
        <div class="flex flex-wrap items-center justify-between gap-2">
            <h2 class="text-lg font-semibold">"MWST-Codes"</h2>
            {(!empty).then(|| view! {
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neuer MWST-Code"
                </a>
            })}
        </div>
        {if empty {
            view! {
                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h3 class="card-title text-base">"Noch keine MWST-Codes"</h3>
                        <p class="text-base-content/70">
                            "Übernehmen Sie die gängigen Schweizer Codes als Ausgangspunkt oder legen Sie eigene an."
                        </p>
                        <div class="card-actions">
                            <a href=move || with_params(&base.get(), &[("standard", "1".into())]) class="btn btn-primary">
                                "Schweizer Standard-Codes übernehmen"
                            </a>
                        </div>
                    </div>
                </div>
            }
            .into_any()
        } else {
            view! { <CodeTable codes base/> }.into_any()
        }}
    }
}

/// Tabelle der Codes. Klick auf eine Zeile öffnet das Bearbeiten (AK-1).
#[component]
fn CodeTable(codes: Vec<VatCode>, base: Memo<String>) -> impl IntoView {
    let rows = codes
        .into_iter()
        .map(|c| {
            let href = with_params(&base.get_untracked(), &[("bearbeiten", c.id.to_string())]);
            let bis = c
                .gueltig_bis
                .as_deref()
                .map_or("–".to_string(), german_date);
            let status = if c.aktiv {
                view! { <span class="badge badge-sm badge-success badge-soft">"Aktiv"</span> }.into_any()
            } else {
                view! { <span class="badge badge-sm badge-ghost">"Inaktiv"</span> }.into_any()
            };
            view! {
                <tr class="hover:bg-base-200" class:opacity-60=!c.aktiv>
                    <td class="whitespace-nowrap font-medium">
                        <a href=href class="link link-hover">{c.code.clone()}</a>
                    </td>
                    <td>{c.bezeichnung.clone()}</td>
                    <td>{kind_label(&c.art)}</td>
                    <td class="whitespace-nowrap text-right tabular-nums">{percent_label(c.satz)}</td>
                    <td class="whitespace-nowrap tabular-nums">{german_date(&c.gueltig_ab)}</td>
                    <td class="whitespace-nowrap tabular-nums">{bis}</td>
                    <td class="whitespace-nowrap">{format!("{} {}", c.konto_nummer, c.konto_bezeichnung)}</td>
                    <td class="tabular-nums">{c.formularziffer.clone().unwrap_or_else(|| "–".into())}</td>
                    <td>
                        <div class="flex flex-wrap gap-1">
                            {status}
                            {c.verwendet.then(|| view! { <span class="badge badge-sm badge-ghost">"verwendet"</span> })}
                        </div>
                    </td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table">
                <thead>
                    <tr>
                        <th>"Code"</th>
                        <th>"Bezeichnung"</th>
                        <th>"Art"</th>
                        <th class="text-right">"Satz"</th>
                        <th>"Gültig ab"</th>
                        <th>"Gültig bis"</th>
                        <th>"MWST-Konto"</th>
                        <th>"Formularziffer"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
}
