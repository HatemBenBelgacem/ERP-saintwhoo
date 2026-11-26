//! Einstellungsseite „Bankkonten“ (ANF-015 AK-1 bis AK-6). Eingaben im Modal-Fenster
//! (`?neu=1`, `?bearbeiten=<kennung>`).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use super::form::BankAccountModal;
use crate::backend::finance::bank_account::model::{BankAccount, group_iban, kind_label};
use crate::backend::finance::bank_account::server_fns::list_bank_accounts;
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

/// Einstellungsseite „Bankkonten“ unter `/einstellungen/finanzen/bankkonten`.
#[component]
pub fn BankAccountsPage() -> impl IntoView {
    let query = use_query_map();
    let param = move |name: &str| query.read().get(name);
    let base = use_base_href();
    let data = Resource::new(data_version, |_| list_bank_accounts());

    view! {
        <Title text="Bankkonten – Einstellungen – uniqwork"/>
        <div class="space-y-4">
            <div class="flex flex-wrap items-center justify-between gap-2">
                <h1 class="text-2xl font-semibold">"Bankkonten"</h1>
                <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                    <IconView icon=Icon::Plus/>
                    "Neues Bankkonto"
                </a>
            </div>

            {move || param("gespeichert").map(|_| view! {
                <div role="alert" class="alert alert-success">"Bankkonto gespeichert."</div>
            })}
            {move || param("geloescht").map(|_| view! {
                <div role="alert" class="alert alert-success">"Bankkonto wurde gelöscht."</div>
            })}

            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(list) => view! { <BankTable list base/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?neu=1, ?bearbeiten=<kennung>
        {move || param("neu").map(|_| view! { <BankAccountModal id=None/> })}
        {move || {
            param("bearbeiten")
                .and_then(|v| v.parse::<i64>().ok())
                .map(|id| view! { <BankAccountModal id=Some(id)/> })
        }}
    }
}

/// Tabelle der Bankkonten; ein Klick auf die Bezeichnung öffnet das Bearbeiten (AK-1).
#[component]
fn BankTable(list: Vec<BankAccount>, base: Memo<String>) -> impl IntoView {
    if list.is_empty() {
        return view! {
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title text-base">"Noch keine Bankkonten"</h2>
                    <p class="text-base-content/70">
                        "Erfassen Sie Ihre Bank-, Post- oder Kassenkonten. Sie werden später bei Zahlungen "
                        "und auf Rechnungen verwendet."
                    </p>
                    <div class="card-actions">
                        <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                            "Neues Bankkonto"
                        </a>
                    </div>
                </div>
            </div>
        }
        .into_any();
    }

    let rows = list
        .into_iter()
        .map(|b| {
            let href = with_params(&base.get_untracked(), &[("bearbeiten", b.id.to_string())]);
            let iban = b.iban.as_deref().map_or("–".to_string(), group_iban);
            let konto = format!("{} {}", b.konto_nummer, b.konto_bezeichnung);
            let standard = b.standard.then(|| {
                view! {
                    <span class="badge badge-sm badge-primary badge-soft">"Standard"</span>
                }
            });
            let status = if b.aktiv {
                view! { <span class="badge badge-sm badge-success badge-soft">"Aktiv"</span> }
                    .into_any()
            } else {
                view! { <span class="badge badge-sm badge-ghost">"Inaktiv"</span> }.into_any()
            };
            view! {
                <tr class="hover:bg-base-200" class:opacity-60=!b.aktiv>
                    <td class="font-medium">
                        <a href=href class="link link-hover">{b.bezeichnung.clone()}</a>
                    </td>
                    <td>{kind_label(&b.art)}</td>
                    <td>{b.bank.clone().unwrap_or_else(|| "–".into())}</td>
                    <td class="whitespace-nowrap tabular-nums">{iban}</td>
                    <td>{konto}</td>
                    <td>
                        <div class="flex flex-wrap gap-1">
                            {standard}
                            {status}
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
                        <th>"Bezeichnung"</th>
                        <th>"Art"</th>
                        <th>"Bank"</th>
                        <th>"IBAN"</th>
                        <th>"Konto im Kontenplan"</th>
                        <th>"Status"</th>
                    </tr>
                </thead>
                <tbody>{rows}</tbody>
            </table>
        </div>
    }
    .into_any()
}
