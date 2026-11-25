//! Einstellungsseite „Firma“ (ANF-025 AK-1 bis AK-3) und Absenderblock für Belege (AK-9, AK-10).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;

use super::form::{CompanyModal, LogoModal};
use crate::backend::addresses::model::country_name;
use crate::backend::company::model::{Company, vat_number};
use crate::backend::company::server_fns::{COMPANY_URL, get_company};
use crate::backend::finance::vat::server_fns::get_vat_overview;
use crate::forms::{data_version, error_message, use_base_href, with_params};
use crate::icons::{Icon, IconView};

#[component]
pub fn CompanyPage() -> impl IntoView {
    let query = use_query_map();
    let data = Resource::new(data_version, |_| get_company());
    // AK-6: die MWST-Nummer erscheint erst, wenn die MWST-Pflicht eingeschaltet ist (ANF-012)
    let vat = Resource::new(data_version, |_| get_vat_overview());
    let param = move |name: &str| query.read().get(name);

    view! {
        <Title text="Firma – Einstellungen – uniqwork"/>
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold">"Firma"</h1>
            {move || param("gespeichert").map(|v| {
                let text = match v.as_str() {
                    "logo" => "Logo gespeichert.",
                    "logo-entfernt" => "Logo entfernt.",
                    _ => "Firma gespeichert.",
                };
                view! { <div role="alert" class="alert alert-success">{text}</div> }
            })}
            <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok(Some(company)) => {
                            let pflichtig = vat
                                .await
                                .map(|overview| overview.settings.mwst_pflichtig)
                                .unwrap_or(false);
                            view! { <CompanyView company pflichtig/> }.into_any()
                        }
                        Ok(None) => view! { <NoCompany/> }.into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                            .into_any(),
                    }
                })}
            </Transition>
        </div>

        // Modal-Fenster: ?neu=1, ?bearbeiten=1, ?logo=1
        {move || (param("neu").is_some() || param("bearbeiten").is_some()).then(|| view! { <CompanyModal/> })}
        {move || param("logo").map(|_| view! { <LogoModal/> })}
    }
}

/// Noch keine Firma (AK-4).
#[component]
fn NoCompany() -> impl IntoView {
    let base = use_base_href();
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h2 class="card-title text-base">"Noch keine Firma erfasst"</h2>
                <p class="text-base-content/70">
                    "Die Firma ist der Absender auf Rechnungen, Mahnungen und Auswertungen. "
                    "Name und Anschrift kommen aus einer Adresse vom Typ „Firma“."
                </p>
                <div class="card-actions">
                    <a href=move || with_params(&base.get(), &[("neu", "1".into())]) class="btn btn-primary">
                        <IconView icon=Icon::Plus/>
                        "Firma erfassen"
                    </a>
                </div>
            </div>
        </div>
    }
}

/// Angaben der Firma (AK-2), Logo (AK-7) und Belegvorschau (AK-3).
#[component]
fn CompanyView(company: Company, pflichtig: bool) -> impl IntoView {
    let base = use_base_href();
    let address_href = format!("/adressen/{}", company.adresse_id);
    let place = [company.plz.clone(), company.ort.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let problem = company.address_problem();
    let logo = company.logo_url();
    let has_logo = logo.is_some();

    view! {
        <div class="flex flex-wrap items-start justify-between gap-2">
            <div>
                <h2 class="text-xl font-semibold">{company.name.clone()}</h2>
                <p class="text-base-content/70">"Adresse " {company.adresse_nummer.clone()}</p>
            </div>
            <a href=move || with_params(&base.get(), &[("bearbeiten", "1".into())]) class="btn btn-primary">
                <IconView icon=Icon::Pencil/>
                "Bearbeiten"
            </a>
        </div>

        {problem.map(|m| view! {
            <div role="alert" class="alert alert-warning">
                <span>{m} " " <a href=format!("{address_href}?bearbeiten=1") class="link">"Adresse bearbeiten"</a></span>
            </div>
        })}

        <div class="grid gap-4 lg:grid-cols-2">
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h3 class="card-title text-base">"Absender (aus der Adresse)"</h3>
                    <dl class="space-y-2">
                        <Field label="Firmenname" value=Some(company.name.clone())/>
                        <Field label="Zusatz" value=company.zusatz.clone()/>
                        <Field label="Strasse" value=company.strasse.clone()/>
                        <Field label="Postfach" value=company.postfach.clone()/>
                        <Field label="PLZ / Ort" value=(!place.is_empty()).then_some(place)/>
                        <Field label="Land" value=Some(country_name(&company.land).to_string())/>
                        <Field label="Telefon" value=company.telefon.clone()/>
                        <Field label="E-Mail" value=company.email.clone()/>
                        <Field label="Webseite" value=company.webseite.clone()/>
                    </dl>
                    <div class="card-actions">
                        <a href=format!("{address_href}?bearbeiten=1") class="btn btn-sm">
                            <IconView icon=Icon::Pencil/>
                            "Adresse bearbeiten"
                        </a>
                        <a href=address_href.clone() class="btn btn-ghost btn-sm">"Zur Adresse"</a>
                    </div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h3 class="card-title text-base">"Rechtliche Angaben und Belege"</h3>
                    <dl class="space-y-2">
                        <Field label="UID" value=company.uid.clone()/>
                        // AK-6: nur bei eingeschalteter MWST-Pflicht (ANF-012)
                        {pflichtig.then(|| view! {
                            <Field label="MWST-Nummer" value=company.uid.as_deref().map(vat_number)/>
                        })}
                        <Field label="Handelsregistereintrag" value=company.handelsregister.clone()/>
                        <Field label="Fusszeile für Belege" value=company.fusszeile.clone()/>
                    </dl>
                    <div class="divider my-1"></div>
                    <h3 class="font-semibold">"Logo"</h3>
                    {match logo {
                        Some(src) => view! {
                            <img src=src alt="Logo der Firma" class="h-16 w-auto max-w-64 object-contain object-left"/>
                        }
                        .into_any(),
                        None => view! { <p class="text-base-content/60">"Kein Logo hinterlegt."</p> }.into_any(),
                    }}
                    <div class="card-actions">
                        <a href=move || with_params(&base.get(), &[("logo", "1".into())]) class="btn btn-sm">
                            {if has_logo { "Logo ersetzen oder entfernen" } else { "Logo hochladen" }}
                        </a>
                    </div>
                </div>
            </div>
        </div>

        // AK-3: Belegvorschau
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h3 class="card-title text-base">"Belegvorschau"</h3>
                <p class="text-sm text-base-content/70">"So erscheint der Absender auf Rechnungen und anderen Belegen."</p>
                <div class="rounded-box border border-base-300 bg-base-100 p-6">
                    <SenderBlock company=Some(company.clone())/>
                    <div class="mt-8 text-sm sm:ml-[55%]">
                        <p>"Musterkunde AG"</p>
                        <p>"Beispielweg 5"</p>
                        <p>"3000 Bern"</p>
                    </div>
                    <p class="mt-8 font-semibold">"Rechnung R-2026-0001"</p>
                    {company.fusszeile.clone().map(|f| view! {
                        <p class="mt-8 border-t border-base-300 pt-2 text-xs text-base-content/60">{f}</p>
                    })}
                </div>
            </div>
        </div>
    }
}

/// Absenderblock eines Belegs (AK-3, AK-9): Logo, Anschrift, UID bzw. MWST-Nummer.
/// Ohne erfasste Firma erscheint der Hinweis „Firma nicht erfasst“ (AK-10).
#[component]
pub fn SenderBlock(company: Option<Company>) -> impl IntoView {
    let Some(company) = company else {
        return view! {
            <div role="alert" class="alert alert-warning">
                <span>
                    "Firma nicht erfasst – der Absender fehlt. "
                    <a href=COMPANY_URL class="link">"Firma erfassen"</a>
                </span>
            </div>
        }
        .into_any();
    };
    let lines = company.address_lines();
    let contact = [
        company.telefon.clone(),
        company.email.clone(),
        company.webseite.clone(),
    ]
    .into_iter()
    .flatten()
    .filter(|s| !s.trim().is_empty())
    .collect::<Vec<_>>()
    .join(" · ");
    view! {
        <div class="flex flex-wrap items-start justify-between gap-4">
            <address class="text-sm not-italic">
                {lines
                    .into_iter()
                    .enumerate()
                    .map(|(i, l)| view! { <p class:font-semibold=i == 0>{l}</p> })
                    .collect_view()}
                {(!contact.is_empty()).then(|| view! { <p class="mt-1 text-base-content/70">{contact}</p> })}
                {company.uid.clone().map(|u| view! { <p class="text-base-content/70">"UID " {u}</p> })}
            </address>
            {company.logo_url().map(|src| view! {
                <img src=src alt="Logo" class="h-14 w-auto max-w-56 object-contain object-right"/>
            })}
        </div>
    }
    .into_any()
}

#[component]
fn Field(label: &'static str, value: Option<String>) -> impl IntoView {
    view! {
        <div>
            <dt class="text-sm text-base-content/60">{label}</dt>
            <dd>{value.filter(|v| !v.trim().is_empty()).unwrap_or_else(|| "–".into())}</dd>
        </div>
    }
}
