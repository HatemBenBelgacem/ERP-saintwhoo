//! Einstellungsseite „Geschäftsjahre“: Liste mit aufklappbaren Perioden (ANF-013 AK-1 bis AK-7).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use super::form::{DeleteYearModal, NewYearModal, PeriodLockModal};
use crate::backend::finance::fiscal_year::model::{
    FiscalYear, FiscalYearOverview, german_date, period_status_label, status_badge, status_label,
};
use crate::backend::finance::fiscal_year::server_fns::get_fiscal_years;
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Einstellungsseite „Geschäftsjahre“. Eingaben im Modal-Fenster (`?neu=1`, `?periode=<kennung>`,
/// `?loeschen=<kennung>`).
#[component]
pub fn FiscalYearPage() -> impl IntoView {
    let query = use_query_map();
    let param = move |name: &str| query.read().get(name);
    let base = use_base_href();
    let data = Resource::new(data_version, |_| get_fiscal_years());

    view! {
        <Title text="Geschäftsjahre – Einstellungen – ERP-saintwhoo"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Geschäftsjahre"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neues Geschäftsjahr"
                </a>
            </div>

            {move || param("gespeichert").map(|v| {
                let text = match v.as_str() {
                    "geschaeftsjahr" => "Geschäftsjahr angelegt. Die Perioden sind offen.",
                    "periode-gesperrt" => "Periode gesperrt.",
                    _ => "Periode entsperrt.",
                };
                view! { <div role="alert" class="alert alert-success">{text}</div> }
            })}
            {move || param("geloescht").map(|_| view! {
                <div role="alert" class="alert alert-success">"Geschäftsjahr wurde gelöscht."</div>
            })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(overview) => view! { <YearList overview base/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?neu=1, ?periode=<kennung>, ?loeschen=<kennung>
        {move || param("neu").map(|_| view! { <NewYearModal/> })}
        {move || {
            param("periode")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <PeriodLockModal id/> })
        }}
        {move || {
            param("loeschen")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <DeleteYearModal id/> })
        }}
    }
}

/// Geschäftsjahre, neueste zuoberst. Das neueste ist aufgeklappt und als einziges löschbar (AK-7).
#[component]
fn YearList(overview: FiscalYearOverview, base: Memo<String>) -> impl IntoView {
    if overview.jahre.is_empty() {
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title text-base">"Noch kein Geschäftsjahr"</h2>
                    <p class="text-base-content/70">
                        "Legen Sie das erste Geschäftsjahr an. Der Vorschlag ist das laufende Kalenderjahr."
                    </p>
                    <div class="card-actions">
                        <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                            "Geschäftsjahr anlegen"
                        </a>
                    </div>
                </div>
            </div>
        }
        .into_any();
    }

    let items = overview
        .jahre
        .into_iter()
        .enumerate()
        .map(|(index, year)| view! { <YearItem year newest=index == 0 base/> })
        .collect_view();

    view! { <div class="space-y-3">{items}</div> }.into_any()
}

/// Ein Geschäftsjahr mit seinen Perioden (AK-1, AK-5).
#[component]
fn YearItem(year: FiscalYear, newest: bool, base: Memo<String>) -> impl IntoView {
    let FiscalYear {
        id,
        bezeichnung,
        beginn,
        ende,
        status,
        perioden,
    } = year;
    let range = format!("{} – {}", german_date(&beginn), german_date(&ende));
    let rows = perioden
        .into_iter()
        .map(|p| {
            let locked = p.status == "gesperrt";
            let button = if locked { "Entsperren" } else { "Sperren" };
            let href = with_params(&base.get_untracked(), &[("periode", p.id.to_string())]);
            view! {
                <tr>
                    <td class="whitespace-nowrap font-medium">{p.label()}</td>
                    <td class="whitespace-nowrap tabular-nums">{format!("{} – {}", german_date(&p.beginn), german_date(&p.ende))}</td>
                    <td><span class=status_badge(&p.status)>{period_status_label(&p.status)}</span></td>
                    <td class="text-base-content/70">{p.vermerk.clone().unwrap_or_default()}</td>
                    <td class="text-right">
                        <a href=href class="btn btn-xs">{button}</a>
                    </td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <details class="collapse-arrow collapse bg-base-100 shadow" open=newest>
            <summary class="collapse-title">
                <div class="flex flex-wrap items-center gap-3 pr-8">
                    <span class="font-semibold">{bezeichnung}</span>
                    <span class="text-base-content/70">{range}</span>
                    <span class=status_badge(&status)>{status_label(&status)}</span>
                </div>
            </summary>
            <div class="collapse-content space-y-3">
                <div class="overflow-x-auto">
                    <table class="table table-sm">
                        <thead>
                            <tr>
                                <th>"Monat"</th>
                                <th>"Zeitraum"</th>
                                <th>"Status"</th>
                                <th>"Letzte Änderung"</th>
                                <th></th>
                            </tr>
                        </thead>
                        <tbody>{rows}</tbody>
                    </table>
                </div>
                {newest.then(|| view! {
                    <a
                        href=move || with_params(&base.get(), &[("loeschen", id.to_string())])
                        class="btn btn-sm btn-outline btn-error"
                    >
                        <IconView icon=Icon::Trash/>
                        "Geschäftsjahr löschen"
                    </a>
                })}
            </div>
        </details>
    }
}
