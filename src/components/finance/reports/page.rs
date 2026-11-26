//! Auswertungen unter `/finanzen/auswertungen` (ANF-021): Auswahl, Geschäftsjahr und Zeitraum (AK-1),
//! Kontoauszug (AK-2), Saldenliste (AK-3), Bilanz (AK-4), Erfolgsrechnung (AK-5), Vorjahresvergleich (AK-6),
//! Projektauswertung (AK-7), Drucken und CSV (AK-8). Alle Angaben stehen in der Webadresse.

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::{components::Form, hooks::use_query_map, params::ParamsMap};

use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::finance::fiscal_year::server_fns::get_fiscal_years;
use crate::backend::finance::journal::model::amount_cell;
use crate::backend::finance::reports::model::{
    ARTEN, BalanceSheet, Group, IncomeStatement, Period, ProjectReport, ReportKind, ReportPage,
    ReportQuery, Statement, TrialBalance, art_label,
};
use crate::backend::finance::reports::server_fns::{ReportOptions, get_report, get_report_options};
use crate::backend::purchase_orders::model::format_amount;
use crate::components::finance::FinanceTabs;
use crate::forms::{error_message, with_params};
use crate::icons::{Icon, IconView};

const PAGE_URL: &str = "/finanzen/auswertungen";
const CSV_URL: &str = "/finanzen/auswertungen/csv";

/// Auswahl aus der Webadresse.
pub fn query_from(q: &ParamsMap) -> ReportQuery {
    let num = |name: &str| q.get(name).and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
    ReportQuery {
        art: q.get("art").unwrap_or_default(),
        geschaeftsjahr_id: num("geschaeftsjahr_id"),
        von: q.get("von").unwrap_or_default(),
        bis: q.get("bis").unwrap_or_default(),
        konto_id: num("konto_id"),
        projekt_id: num("projekt_id"),
        vorjahr: q.get("vorjahr").unwrap_or_default(),
    }
}

/// Parameter der Auswahl (leere Werte weggelassen).
fn query_params(q: &ReportQuery) -> Vec<(&'static str, String)> {
    let mut p = vec![("art", q.art.clone())];
    if q.geschaeftsjahr_id > 0 {
        p.push(("geschaeftsjahr_id", q.geschaeftsjahr_id.to_string()));
    }
    if !q.von.is_empty() {
        p.push(("von", q.von.clone()));
    }
    if !q.bis.is_empty() {
        p.push(("bis", q.bis.clone()));
    }
    if q.konto_id > 0 {
        p.push(("konto_id", q.konto_id.to_string()));
    }
    if q.projekt_id > 0 {
        p.push(("projekt_id", q.projekt_id.to_string()));
    }
    if !q.vorjahr.is_empty() {
        p.push(("vorjahr", "1".to_string()));
    }
    p
}

/// Webadresse für einen Kontoauszug des Kontos im selben Zeitraum (AK-2).
fn konto_href(period: &Period, konto_id: i64) -> String {
    with_params(
        PAGE_URL,
        &[
            ("art", "kontoauszug".to_string()),
            ("geschaeftsjahr_id", period.geschaeftsjahr_id.to_string()),
            ("von", period.von.clone()),
            ("bis", period.bis.clone()),
            ("konto_id", konto_id.to_string()),
        ],
    )
}

#[component]
pub fn ReportsPage() -> impl IntoView {
    let query = use_query_map();
    let current = Memo::new(move |_| query_from(&query.read()));
    let report = Resource::new(move || current.get(), get_report);
    let options = Resource::new(|| (), |_| get_report_options());
    let years = Resource::new(|| (), |_| get_fiscal_years());
    let csv_href = Memo::new(move |_| with_params(CSV_URL, &query_params(&current.get())));

    view! {
        <Title text="Auswertungen – Finanzen – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2 print:hidden">
                <h1 class="text-2xl font-semibold">"Auswertungen"</h1>
                <div class="flex flex-wrap gap-2">
                    <a href=move || csv_href.get() class="btn btn-sm" download>
                        <IconView icon=Icon::Ledger/>
                        "CSV herunterladen"
                    </a>
                    <button type="button" class="btn btn-sm" onclick="window.print()">
                        "Drucken"
                    </button>
                </div>
            </div>
            <div class="print:hidden">
                <FinanceTabs active="auswertungen"/>
            </div>

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    let jahre = years.await;
                    let opts = options.await;
                    match (jahre, opts) {
                        (Ok(overview), Ok(opts)) => {
                            let jahre: Vec<(i64, String)> = overview
                                .jahre
                                .iter()
                                .map(|j| (j.id, j.bezeichnung.clone()))
                                .collect();
                            view! { <SelectionForm q=current.get_untracked() jahre opts/> }.into_any()
                        }
                        (Err(e), _) | (_, Err(e)) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match report.await {
                        Ok(page) => view! { <ReportView page/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error print:hidden">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>
    }
}

/// Auswahl der Auswertung, des Geschäftsjahrs und des Zeitraums (AK-1). Felder je nach Art.
#[component]
fn SelectionForm(q: ReportQuery, jahre: Vec<(i64, String)>, opts: ReportOptions) -> impl IntoView {
    let art = q.art.clone();
    let zeigt_konto = art == "kontoauszug";
    let zeigt_projekt = art == "projekte";
    let zeigt_vorjahr = art == "bilanz" || art == "erfolgsrechnung";
    let gj = q.geschaeftsjahr_id;
    let konto_id = q.konto_id;
    let projekt_id = q.projekt_id;
    view! {
        <Form method="GET" action=PAGE_URL>
            <div class="flex flex-wrap items-end gap-2 print:hidden">
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-base-content/70">"Auswertung"</span>
                    <select name="art" class="select w-full sm:w-52" onchange="this.form.requestSubmit()">
                        {ARTEN
                            .iter()
                            .map(|(k, l)| view! { <option value=*k selected={art == *k}>{*l}</option> })
                            .collect_view()}
                    </select>
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-base-content/70">"Geschäftsjahr"</span>
                    <select name="geschaeftsjahr_id" class="select w-full sm:w-44" onchange="this.form.requestSubmit()">
                        {jahre
                            .into_iter()
                            .map(|(id, name)| view! { <option value=id.to_string() selected={id == gj}>{name}</option> })
                            .collect_view()}
                    </select>
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-base-content/70">"Von"</span>
                    <input type="date" name="von" class="input w-full sm:w-40" value=q.von.clone()/>
                </label>
                <label class="flex flex-col gap-1">
                    <span class="text-xs text-base-content/70">"Bis (Stichtag)"</span>
                    <input type="date" name="bis" class="input w-full sm:w-40" value=q.bis.clone()/>
                </label>
                {zeigt_konto.then(|| view! {
                    <label class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/70">"Konto"</span>
                        <select name="konto_id" class="select w-full sm:w-72" onchange="this.form.requestSubmit()">
                            <option value="0" selected={konto_id == 0}>"– Konto wählen –"</option>
                            {opts
                                .konten
                                .iter()
                                .map(|k| view! {
                                    <option value=k.id.to_string() selected={k.id == konto_id}>
                                        {format!("{} · {}", k.nummer, k.bezeichnung)}
                                    </option>
                                })
                                .collect_view()}
                        </select>
                    </label>
                })}
                {zeigt_projekt.then(|| view! {
                    <label class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/70">"Projekt"</span>
                        <select name="projekt_id" class="select w-full sm:w-72" onchange="this.form.requestSubmit()">
                            <option value="0" selected={projekt_id == 0}>"Alle Projekte"</option>
                            {opts
                                .projekte
                                .iter()
                                .map(|p| view! {
                                    <option value=p.id.to_string() selected={p.id == projekt_id}>
                                        {format!("{} · {}", p.nummer, p.name)}
                                    </option>
                                })
                                .collect_view()}
                        </select>
                    </label>
                })}
                {zeigt_vorjahr.then(|| view! {
                    <label class="label cursor-pointer gap-2 pb-2">
                        <input
                            type="checkbox"
                            name="vorjahr"
                            value="1"
                            class="checkbox checkbox-primary checkbox-sm"
                            checked={!q.vorjahr.is_empty()}
                            onchange="this.form.requestSubmit()"
                        />
                        "Vorjahr vergleichen"
                    </label>
                })}
                <button type="submit" class="btn">"Anzeigen"</button>
            </div>
        </Form>
    }
}

/// Kopf der Auswertung mit Geschäftsjahr und Zeitraum, dann die Auswertung selbst.
#[component]
fn ReportView(page: ReportPage) -> impl IntoView {
    let titel = art_label(&page.art);
    let period = page.period.clone();
    let zeitraum = format!(
        "Geschäftsjahr {} · {} bis {}",
        period.geschaeftsjahr,
        german_date(&period.von),
        german_date(&period.bis)
    );
    let vergleich = period
        .vorjahr
        .as_ref()
        .map(|(von, bis)| format!("Vorjahr: {} bis {}", german_date(von), german_date(bis)));
    let body = match page.kind {
        ReportKind::Leer => view! {
            <div class="card bg-base-100 shadow print:hidden">
                <div class="card-body items-center text-center text-base-content/70">
                    "Wählen Sie ein Konto, um den Kontoauszug zu sehen."
                </div>
            </div>
        }
        .into_any(),
        ReportKind::Kontoauszug(st) => {
            view! { <StatementView st period=period.clone()/> }.into_any()
        }
        ReportKind::Saldenliste(tb) => view! { <TrialView tb period=period.clone()/> }.into_any(),
        ReportKind::Bilanz(b) => {
            view! { <BalanceView b vergleich=period.vorjahr.is_some()/> }.into_any()
        }
        ReportKind::Erfolgsrechnung(er) => {
            view! { <IncomeView er vergleich=period.vorjahr.is_some()/> }.into_any()
        }
        ReportKind::Projekte(p) => view! { <ProjectView p/> }.into_any(),
    };
    view! {
        <section class="space-y-3">
            <div class="flex flex-wrap items-baseline justify-between gap-2">
                <h2 class="text-xl font-semibold">{titel}</h2>
                <p class="text-sm text-base-content/70">
                    {zeitraum}
                    {vergleich.map(|v| view! { <span>" · " {v}</span> })}
                </p>
            </div>
            {body}
        </section>
    }
}

/// Kontoauszug mit Anfangssaldo, laufendem Saldo und Endsaldo (AK-2).
#[component]
fn StatementView(st: Statement, period: Period) -> impl IntoView {
    let anfang = st.anfangssaldo;
    let zeilen = st
        .zeilen
        .iter()
        .map(|z| {
            view! {
                <tr>
                    <td class="whitespace-nowrap">{german_date(&z.datum)}</td>
                    <td class="tabular-nums">{z.journalnummer.map_or("–".to_string(), |n| n.to_string())}</td>
                    <td class="whitespace-nowrap">
                        <a href=format!("/finanzen/journal/{}", z.buchung_id) class="link link-hover">
                            {z.belegnummer.clone()}
                        </a>
                    </td>
                    <td class="max-w-xs truncate">{z.text.clone()}</td>
                    <td class="whitespace-nowrap">{z.gegenkonto.clone()}</td>
                    <td class="text-right tabular-nums">{amount_cell(z.soll)}</td>
                    <td class="text-right tabular-nums">{amount_cell(z.haben)}</td>
                    <td class="text-right tabular-nums">{saldo_text(z.saldo)}</td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <p class="text-sm">
            <span class="font-medium">{st.konto_nummer.clone()}</span>
            " · "
            {st.konto_bezeichnung.clone()}
            " · "
            <a href=konto_href(&period, st.konto_id) class="link link-hover print:hidden">"Kontoauszug"</a>
        </p>
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th>"Datum"</th>
                        <th>"Journal"</th>
                        <th>"Beleg"</th>
                        <th>"Text"</th>
                        <th>"Gegenkonto"</th>
                        <th class="text-right">"Soll"</th>
                        <th class="text-right">"Haben"</th>
                        <th class="text-right">"Saldo"</th>
                    </tr>
                </thead>
                <tbody>
                    <tr class="bg-base-200/60 font-medium">
                        <td colspan="7">"Anfangssaldo"</td>
                        <td class="text-right tabular-nums">{saldo_text(anfang)}</td>
                    </tr>
                    {zeilen}
                </tbody>
                <tfoot>
                    <tr class="font-semibold">
                        <td colspan="5">"Summe und Endsaldo"</td>
                        <td class="text-right tabular-nums">{format_amount(st.soll)}</td>
                        <td class="text-right tabular-nums">{format_amount(st.haben)}</td>
                        <td class="text-right tabular-nums">{saldo_text(st.endsaldo)}</td>
                    </tr>
                </tfoot>
            </table>
        </div>
    }
}

/// Saldo mit Seite: Soll-Saldo „S“, Haben-Saldo „H“.
fn saldo_text(saldo: i64) -> String {
    match saldo.cmp(&0) {
        std::cmp::Ordering::Greater => format!("{} S", format_amount(saldo)),
        std::cmp::Ordering::Less => format!("{} H", format_amount(-saldo)),
        std::cmp::Ordering::Equal => format_amount(0),
    }
}

/// Saldenliste nach Kontenklasse mit Zwischensummen und Kontrolle Soll = Haben (AK-3).
#[component]
fn TrialView(tb: TrialBalance, period: Period) -> impl IntoView {
    let kontrolle = if tb.ausgeglichen {
        view! { <div role="status" class="alert alert-success alert-soft print:hidden">"Soll und Haben stimmen überein."</div> }.into_any()
    } else {
        view! { <div role="alert" class="alert alert-error print:hidden">"Soll und Haben sind nicht gleich. Bitte die Buchungen prüfen."</div> }.into_any()
    };
    let klassen = tb
        .klassen
        .iter()
        .map(|k| {
            let zeilen = k
                .zeilen
                .iter()
                .map(|z| {
                    view! {
                        <tr>
                            <td class="whitespace-nowrap tabular-nums">
                                <a href=konto_href(&period, z.konto_id) class="link link-hover">{z.nummer.clone()}</a>
                            </td>
                            <td>{z.bezeichnung.clone()}</td>
                            <td class="text-right tabular-nums">{saldo_text(z.anfangssaldo)}</td>
                            <td class="text-right tabular-nums">{format_amount(z.soll)}</td>
                            <td class="text-right tabular-nums">{format_amount(z.haben)}</td>
                            <td class="text-right tabular-nums">{saldo_text(z.endsaldo)}</td>
                        </tr>
                    }
                })
                .collect_view();
            view! {
                <tr class="bg-base-200 font-semibold">
                    <th colspan="2">{k.titel.clone()}</th>
                    <th class="text-right tabular-nums">{saldo_text(k.anfangssaldo)}</th>
                    <th class="text-right tabular-nums">{format_amount(k.soll)}</th>
                    <th class="text-right tabular-nums">{format_amount(k.haben)}</th>
                    <th class="text-right tabular-nums">{saldo_text(k.endsaldo)}</th>
                </tr>
                {zeilen}
            }
        })
        .collect_view();
    view! {
        {kontrolle}
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th>"Konto"</th>
                        <th>"Bezeichnung"</th>
                        <th class="text-right">"Anfangssaldo"</th>
                        <th class="text-right">"Soll"</th>
                        <th class="text-right">"Haben"</th>
                        <th class="text-right">"Endsaldo"</th>
                    </tr>
                </thead>
                <tbody>{klassen}</tbody>
                <tfoot>
                    <tr class="font-semibold">
                        <td colspan="2">"Summe"</td>
                        <td class="text-right tabular-nums">{saldo_text(tb.anfangssaldo)}</td>
                        <td class="text-right tabular-nums">{format_amount(tb.soll)}</td>
                        <td class="text-right tabular-nums">{format_amount(tb.haben)}</td>
                        <td class="text-right tabular-nums">{saldo_text(tb.endsaldo)}</td>
                    </tr>
                </tfoot>
            </table>
        </div>
    }
}

/// Abweichung zum Vorjahr als Text (Vorzeichen zeigt die Richtung).
fn abweichung(betrag: i64, vorjahr: Option<i64>) -> Option<String> {
    vorjahr.map(|v| format_amount(betrag - v))
}

/// Bilanz mit Aktiven links und Passiven rechts (AK-4), optional mit Vorjahr (AK-6).
#[component]
fn BalanceView(b: BalanceSheet, vergleich: bool) -> impl IntoView {
    let aktiven = seite_view(&b.aktiven, vergleich);
    let passiven = seite_view(&b.passiven, vergleich);
    let ausgeglichen = b.summe_aktiven == b.summe_passiven;
    let stichtag = german_date(&b.stichtag);
    view! {
        <p class="text-sm text-base-content/70">"Stichtag " {stichtag}</p>
        <div class="grid gap-4 md:grid-cols-2">
            <div class="overflow-x-auto rounded-box bg-base-100 shadow">
                <h3 class="px-4 pt-3 font-semibold">"Aktiven"</h3>
                {aktiven}
                <SummeZeile titel="Summe Aktiven" betrag=b.summe_aktiven vorjahr=b.vorjahr_aktiven/>
            </div>
            <div class="overflow-x-auto rounded-box bg-base-100 shadow">
                <h3 class="px-4 pt-3 font-semibold">"Passiven"</h3>
                {passiven}
                <SummeZeile titel="Summe Passiven" betrag=b.summe_passiven vorjahr=b.vorjahr_passiven/>
            </div>
        </div>
        {if ausgeglichen {
            view! { <div role="status" class="alert alert-success alert-soft print:hidden">"Aktiven und Passiven stimmen überein."</div> }.into_any()
        } else {
            view! { <div role="alert" class="alert alert-error print:hidden">"Aktiven und Passiven stimmen nicht überein. Bitte die Buchungen prüfen."</div> }.into_any()
        }}
    }
}

/// Gruppen einer Seite der Bilanz: Kontengruppen mit Summen, Zeilen mit Vorjahr.
fn seite_view(gruppen: &[Group], vergleich: bool) -> AnyView {
    gruppen
        .iter()
        .map(|g| {
            let zeilen = g
                .zeilen
                .iter()
                .map(|r| {
                    view! {
                        <tr>
                            <td class="whitespace-nowrap tabular-nums">{r.nummer.clone()}</td>
                            <td>{r.bezeichnung.clone()}</td>
                            <td class="text-right tabular-nums">{format_amount(r.betrag)}</td>
                            {vergleich.then(|| view! {
                                <td class="text-right tabular-nums">{r.vorjahr.map(format_amount).unwrap_or_default()}</td>
                                <td class="text-right tabular-nums">{abweichung(r.betrag, r.vorjahr).unwrap_or_default()}</td>
                            })}
                        </tr>
                    }
                })
                .collect_view();
            view! {
                <table class="table table-sm">
                    <tbody>
                        <tr class="bg-base-200 font-semibold">
                            <th colspan="2">{g.titel.clone()}</th>
                            <th class="text-right tabular-nums">{format_amount(g.summe)}</th>
                            {vergleich.then(|| view! {
                                <th class="text-right tabular-nums">{g.vorjahr.map(format_amount).unwrap_or_default()}</th>
                                <th class="text-right tabular-nums">{abweichung(g.summe, g.vorjahr).unwrap_or_default()}</th>
                            })}
                        </tr>
                        {zeilen}
                    </tbody>
                </table>
            }
        })
        .collect_view()
        .into_any()
}

/// Zeile mit Summe einer Seite (Bilanz) mit Vorjahr.
#[component]
fn SummeZeile(titel: &'static str, betrag: i64, vorjahr: Option<i64>) -> impl IntoView {
    view! {
        <div class="flex justify-between gap-4 border-t border-base-300 px-4 py-2 font-semibold tabular-nums">
            <span>{titel}</span>
            <span>
                {format_amount(betrag)}
                {vorjahr.map(|v| view! { <span class="ml-4 text-base-content/70">{format_amount(v)}</span> })}
            </span>
        </div>
    }
}

/// Erfolgsrechnung nach Kontenklassen mit Zwischenergebnissen (AK-5); Ertrag positiv, Aufwand negativ.
#[component]
fn IncomeView(er: IncomeStatement, vergleich: bool) -> impl IntoView {
    let gruppen = er
        .gruppen
        .iter()
        .filter(|g| !g.zeilen.is_empty())
        .map(|g| {
            let zeilen = g
                .zeilen
                .iter()
                .map(|r| {
                    view! {
                        <tr>
                            <td class="whitespace-nowrap tabular-nums">{r.nummer.clone()}</td>
                            <td>{r.bezeichnung.clone()}</td>
                            <td class="text-right tabular-nums">{format_amount(r.betrag)}</td>
                            {vergleich.then(|| view! {
                                <td class="text-right tabular-nums">{r.vorjahr.map(format_amount).unwrap_or_default()}</td>
                                <td class="text-right tabular-nums">{abweichung(r.betrag, r.vorjahr).unwrap_or_default()}</td>
                            })}
                        </tr>
                    }
                })
                .collect_view();
            view! {
                <tr class="bg-base-200 font-semibold">
                    <th colspan="2">{g.titel.clone()}</th>
                    <th class="text-right tabular-nums">{format_amount(g.summe)}</th>
                    {vergleich.then(|| view! {
                        <th class="text-right tabular-nums">{g.vorjahr.map(format_amount).unwrap_or_default()}</th>
                        <th class="text-right tabular-nums">{abweichung(g.summe, g.vorjahr).unwrap_or_default()}</th>
                    })}
                </tr>
                {zeilen}
            }
        })
        .collect_view();
    let zwischen = er
        .zwischen
        .iter()
        .map(|z| {
            view! {
                <tr class="font-semibold">
                    <td colspan="2">{z.titel.clone()}</td>
                    <td class="text-right tabular-nums">{format_amount(z.betrag)}</td>
                    {vergleich.then(|| view! {
                        <td class="text-right tabular-nums">{z.vorjahr.map(format_amount).unwrap_or_default()}</td>
                        <td class="text-right tabular-nums">{abweichung(z.betrag, z.vorjahr).unwrap_or_default()}</td>
                    })}
                </tr>
            }
        })
        .collect_view();
    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th>"Konto"</th>
                        <th>"Bezeichnung"</th>
                        <th class="text-right">"Geschäftsjahr"</th>
                        {vergleich.then(|| view! {
                            <th class="text-right">"Vorjahr"</th>
                            <th class="text-right">"Abweichung"</th>
                        })}
                    </tr>
                </thead>
                <tbody>{gruppen}</tbody>
                <tbody class="border-t-2 border-base-300">{zwischen}</tbody>
            </table>
        </div>
    }
}

/// Projektauswertung: Erlöse, Aufwand, Ergebnis und offene Einkaufsaufträge je Projekt (AK-7).
#[component]
fn ProjectView(p: ProjectReport) -> impl IntoView {
    if p.zeilen.is_empty() {
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body items-center text-center text-base-content/70">
                    "Keine Projektbewegungen in diesem Zeitraum."
                </div>
            </div>
        }
        .into_any();
    }
    let zeilen = p
        .zeilen
        .iter()
        .map(|r| {
            let ergebnis = r.ergebnis();
            view! {
                <tr>
                    <td class="whitespace-nowrap">
                        <a href=format!("/projekte/{}", r.projekt_id) class="link link-hover">{r.nummer.clone()}</a>
                    </td>
                    <td>{r.name.clone()}</td>
                    <td class="text-right tabular-nums">{format_amount(r.erloes)}</td>
                    <td class="text-right tabular-nums">{format_amount(r.aufwand)}</td>
                    <td class="text-right tabular-nums">{format_amount(ergebnis)}</td>
                    <td class="text-right tabular-nums">{format_amount(r.offen)}</td>
                </tr>
            }
        })
        .collect_view();
    view! {
        <div class="overflow-x-auto rounded-box bg-base-100 shadow">
            <table class="table table-sm">
                <thead>
                    <tr>
                        <th>"Projekt"</th>
                        <th>"Name"</th>
                        <th class="text-right">"Erlöse"</th>
                        <th class="text-right">"Aufwand"</th>
                        <th class="text-right">"Ergebnis"</th>
                        <th class="text-right">"Offene Aufträge"</th>
                    </tr>
                </thead>
                <tbody>{zeilen}</tbody>
                <tfoot>
                    <tr class="font-semibold">
                        <td colspan="2">"Summe"</td>
                        <td class="text-right tabular-nums">{format_amount(p.erloes)}</td>
                        <td class="text-right tabular-nums">{format_amount(p.aufwand)}</td>
                        <td class="text-right tabular-nums">{format_amount(p.erloes - p.aufwand)}</td>
                        <td class="text-right tabular-nums">{format_amount(p.offen)}</td>
                    </tr>
                </tfoot>
            </table>
        </div>
    }
    .into_any()
}

/// Ergebnis der Auswahl für Tests: Kontenschlüssel aus einem Auswahltext.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::finance::journal::model::key_of;

    #[test]
    fn saldo_shows_side() {
        assert_eq!(saldo_text(1_234_50), "1'234.50 S");
        assert_eq!(saldo_text(-50_00), "50.00 H");
        assert_eq!(saldo_text(0), "0.00");
        assert_eq!(key_of("1020 · Bank"), "1020");
    }

    #[test]
    fn deviation_needs_comparison() {
        assert_eq!(abweichung(100_00, None), None);
        assert_eq!(abweichung(100_00, Some(40_00)).as_deref(), Some("60.00"));
    }
}
