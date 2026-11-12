//! Detailansicht einer Adresse mit Löschen (AK-7, AK-15 bis AK-17).

use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_params_map, use_query_map};

use super::api::{DeleteAddress, error_message, get_address};
use super::form::AddressEditModal;
use super::model::{Address, country_name};
use crate::forms::{data_version, use_base_href, with_params};
use crate::icons::{Icon, IconView};

#[component]
pub fn AddressDetailPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let saved = move || query.read().get("gespeichert").is_some();
    let data = Resource::new(
        move || (id(), data_version()),
        |(id, _)| async move {
            match id {
                Some(id) => get_address(id).await,
                None => Ok(None),
            }
        },
    );

    view! {
        <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match data.await {
                    Ok(Some(address)) => view! { <AddressDetail address saved=Signal::derive(saved)/> }.into_any(),
                    Ok(None) => view! { <NotFound/> }.into_any(),
                    Err(e) => view! {
                        <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                    }
                        .into_any(),
                }
            })}
        </Suspense>
    }
}

/// AK-17
#[component]
pub fn NotFound() -> impl IntoView {
    view! {
        <Title text="Adresse nicht gefunden – ERP-saintwhoo"/>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title">"Adresse nicht gefunden"</h1>
                <p class="text-base-content/70">"Diese Adresse gibt es nicht oder sie wurde gelöscht."</p>
                <div class="card-actions">
                    <a href="/adressen" class="btn">"Zur Adressliste"</a>
                </div>
            </div>
        </div>
    }
}

#[component]
fn AddressDetail(address: Address, saved: Signal<bool>) -> impl IntoView {
    let confirm_open = RwSignal::new(false);
    let query = use_query_map();
    let base = use_base_href();
    let edit_open = move || query.read().get("bearbeiten").is_some();
    let edit_address = address.clone();
    let delete = ServerAction::<DeleteAddress>::new();
    let delete_error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };

    let a = address.clone();
    let anrede = match a.anrede.as_deref() {
        Some("herr") => Some("Herr"),
        Some("frau") => Some("Frau"),
        _ => None,
    };
    let person_name = [a.vorname.clone(), a.nachname.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    let contact = if a.typ == "firma" {
        "Kontaktperson"
    } else {
        "Name"
    };
    let place = [a.plz.clone(), a.ort.clone()]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");

    view! {
        <Title text=format!("{} – Adressen – ERP-saintwhoo", address.anzeigename)/>
        <div class="space-y-4">
            <div class="breadcrumbs text-sm">
                <ul>
                    <li><a href="/adressen">"Adressen"</a></li>
                    <li>{address.nummer.clone()}</li>
                </ul>
            </div>

            {move || saved.get().then(|| view! {
                <div role="alert" class="alert alert-success">"Adresse gespeichert."</div>
            })}

            <div class="flex flex-wrap items-start justify-between gap-2">
                <div>
                    <h1 class="text-2xl font-semibold">{address.anzeigename.clone()}</h1>
                    <p class="text-base-content/70">
                        {address.nummer.clone()} " · "
                        {if address.typ == "firma" { "Firma" } else { "Person" }}
                    </p>
                </div>
                <div class="flex gap-2">
                    <a href=move || with_params(&base.get(), &[("bearbeiten", "1".into())]) class="btn btn-primary">
                        <IconView icon=Icon::Pencil/>
                        "Bearbeiten"
                    </a>
                    <button type="button" class="btn btn-outline btn-error" on:click=move |_| confirm_open.set(true)>
                        <IconView icon=Icon::Trash/>
                        "Löschen"
                    </button>
                </div>
            </div>

            <div class="grid gap-4 md:grid-cols-2">
                <Section title="Adresse">
                    <Field label="Firmenname" value=a.firmenname.clone()/>
                    <Field label="Anrede" value=anrede.map(String::from)/>
                    <Field label=contact value=(!person_name.is_empty()).then_some(person_name)/>
                    <Field label="Zusatz" value=a.zusatz.clone()/>
                    <Field label="Strasse" value=a.strasse.clone()/>
                    <Field label="Postfach" value=a.postfach.clone()/>
                    <Field label="PLZ / Ort" value=(!place.is_empty()).then_some(place)/>
                    <Field label="Land" value=Some(format!("{} ({})", country_name(&a.land), a.land))/>
                </Section>
                <Section title="Kontakt">
                    <div>
                        <dt class="text-sm text-base-content/60">"E-Mail"</dt>
                        <dd>
                            {match a.email.clone() {
                                Some(m) => view! { <a href=format!("mailto:{m}") class="link link-primary">{m.clone()}</a> }.into_any(),
                                None => view! { "–" }.into_any(),
                            }}
                        </dd>
                    </div>
                    <Field label="Telefon" value=a.telefon.clone()/>
                    <Field label="Mobiltelefon" value=a.mobiltelefon.clone()/>
                    <Field label="Webseite" value=a.webseite.clone()/>
                </Section>
            </div>

            <Section title="Notiz">
                <p class="whitespace-pre-line">{a.notiz.clone().unwrap_or_else(|| "–".into())}</p>
            </Section>

            <p class="text-sm text-base-content/60">
                "Erstellt am " {a.erstellt_am.clone()} " · Geändert am " {a.geaendert_am.clone()}
            </p>
        </div>

        // AK-12: Bearbeiten im Modal-Fenster (?bearbeiten=1)
        {move || edit_open().then(|| view! { <AddressEditModal address=edit_address.clone()/> })}

        // AK-15: Bestätigungsdialog
        <div class="modal" class:modal-open=move || confirm_open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Adresse löschen?"</h3>
                <p class="py-4">
                    "Soll die Adresse „" {address.anzeigename.clone()} "“ (" {address.nummer.clone()}
                    ") wirklich gelöscht werden? Das kann nicht rückgängig gemacht werden."
                </p>
                {move || delete_error().map(|e| view! {
                    <div role="alert" class="alert alert-error">{e}</div>
                })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| confirm_open.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
                        <input type="hidden" name="id" value=address.id/>
                        <button type="submit" class="btn btn-error" disabled=move || delete.pending().get()>
                            "Endgültig löschen"
                        </button>
                    </ActionForm>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| confirm_open.set(false)></div>
        </div>
    }
}

#[component]
fn Section(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h2 class="card-title text-base">{title}</h2>
                <dl class="space-y-2">{children()}</dl>
            </div>
        </div>
    }
}

#[component]
fn Field(label: &'static str, value: Option<String>) -> impl IntoView {
    view! {
        <div>
            <dt class="text-sm text-base-content/60">{label}</dt>
            <dd>{value.unwrap_or_else(|| "–".into())}</dd>
        </div>
    }
}
