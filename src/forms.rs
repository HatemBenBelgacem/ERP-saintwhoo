//! Gemeinsame Bausteine für Formulare: Fehler, Speicher-Ergebnis, Eingabefelder, Pfadleiste,
//! Modal-Fenster und Webadressen mit Fenster-Parametern.

use std::collections::BTreeMap;

use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_navigate};
use serde::{Deserialize, Serialize};

/// Parameter in der Webadresse, die ein Modal-Fenster öffnen oder eine Meldung zeigen.
/// Beim Schliessen eines Fensters werden sie entfernt.
pub const MODAL_PARAMS: &[&str] = &[
    "neu",
    "unter",
    "bearbeiten",
    "id",
    "gespeichert",
    "geloescht",
    "wareneingang",
    "passwort",
    "vorlage",
    "systemkonten",
    "logo",
    "mandant",
    "standort",
    "einstellungen",
    "standard",
    "periode",
    "loeschen",
    "kurs",
];

/// Zähler, der nach jedem Speichern erhöht wird. Ressourcen, die ihn lesen, laden danach neu
/// (nötig, wenn nach dem Speichern dieselbe Seite wieder erscheint).
#[derive(Clone, Copy)]
pub struct DataVersion(pub RwSignal<u32>);

/// Aktueller Stand des Zählers (in `App` bereitgestellt).
pub fn data_version() -> u32 {
    use_context::<DataVersion>().map_or(0, |v| v.0.get())
}

/// Erhöht den Zähler, sobald `saved` wahr wird (z. B. eine Speicher-Aktion erfolgreich war).
pub fn refresh_when(saved: impl Fn() -> bool + Send + Sync + 'static) {
    let Some(DataVersion(version)) = use_context::<DataVersion>() else {
        return;
    };
    Effect::new(move |_| {
        if saved() {
            version.update(|v| *v += 1);
        }
    });
}

/// `true`, wenn das Ergebnis einer Speicher-Aktion „gespeichert“ ist.
pub fn is_saved(value: Option<Result<SaveOutcome, ServerFnError>>) -> bool {
    matches!(value, Some(Ok(SaveOutcome::Saved(_))))
}

/// Aktuelle Webadresse ohne Fenster-Parameter – Ziel beim Schliessen eines Modal-Fensters.
pub fn use_base_href() -> Memo<String> {
    let location = use_location();
    Memo::new(move |_| {
        let path = location.pathname.get();
        let query = location.query.get();
        let params: Vec<String> = query
            .into_iter()
            .filter(|(k, _)| !MODAL_PARAMS.contains(&k.as_ref()))
            .map(|(k, v)| format!("{k}={}", encode(&v)))
            .collect();
        if params.is_empty() {
            path
        } else {
            format!("{path}?{}", params.join("&"))
        }
    })
}

/// Hängt Parameter an eine Webadresse an.
pub fn with_params(base: &str, params: &[(&str, String)]) -> String {
    let extra: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{k}={}", encode(v)))
        .collect();
    if extra.is_empty() {
        return base.to_string();
    }
    let sep = if base.contains('?') { '&' } else { '?' };
    format!("{base}{sep}{}", extra.join("&"))
}

/// Rücksprung-Adresse aus einem Formular nur annehmen, wenn sie innerhalb der Anwendung bleibt.
pub fn safe_return_url(url: &str) -> Option<&str> {
    (url.starts_with('/') && !url.starts_with("//") && !url.contains('\\') && !url.contains("://"))
        .then_some(url)
}

/// Fehlermeldungen je Formularfeld (Feldname → Meldung).
pub type FieldErrors = BTreeMap<String, String>;

/// Ergebnis von Anlegen/Bearbeiten: gespeichert (Weiterleitung erfolgt) oder Eingabefehler.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum SaveOutcome {
    Saved(i64),
    Invalid(FieldErrors),
}

/// Lesbare Fehlermeldung aus einem Server-Fehler.
pub fn error_message(e: &ServerFnError) -> String {
    match e {
        ServerFnError::ServerError(msg) => msg.clone(),
        other => other.to_string(),
    }
}

/// Eingabefehler aus dem Ergebnis einer Speicher-Aktion.
pub fn field_errors(value: Option<Result<SaveOutcome, ServerFnError>>) -> FieldErrors {
    match value {
        Some(Ok(SaveOutcome::Invalid(errors))) => errors,
        _ => FieldErrors::new(),
    }
}

/// Server-Fehler (kein Eingabefehler) aus dem Ergebnis einer Speicher-Aktion.
pub fn server_error(value: Option<Result<SaveOutcome, ServerFnError>>) -> Option<String> {
    value.and_then(|r| r.err()).map(|e| error_message(&e))
}

/// Prozent-Kodierung für Werte in der Webadresse.
pub fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Textfeld mit Beschriftung und Fehlermeldung. Formularname: `input[<field>]`.
#[component]
pub fn TextField(
    field: &'static str,
    label: &'static str,
    value: String,
    errors: Signal<FieldErrors>,
    #[prop(into, default = Signal::stored(false))] required: Signal<bool>,
    #[prop(default = "text")] kind: &'static str,
    #[prop(default = 200)] maxlength: usize,
    #[prop(optional)] inputmode: Option<&'static str>,
) -> impl IntoView {
    let has_error = move || errors.read().contains_key(field);
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">{label} {move || required.get().then_some(" *")}</legend>
            <input
                type=kind
                name=format!("input[{field}]")
                value=value
                maxlength=maxlength
                inputmode=inputmode
                class="input w-full"
                class:input-error=has_error
                aria-invalid=move || has_error().then_some("true")
            />
            <ErrorText field errors/>
        </fieldset>
    }
}

/// Fehlermeldung unter einem Feld.
#[component]
pub fn ErrorText(field: &'static str, errors: Signal<FieldErrors>) -> impl IntoView {
    move || {
        errors
            .read()
            .get(field)
            .cloned()
            .map(|e| view! { <p class="label text-error">{e}</p> })
    }
}

/// Pfadleiste: (Beschriftung, Verweis). Der letzte Eintrag hat normalerweise keinen Verweis.
#[component]
pub fn Breadcrumbs(items: Vec<(String, Option<String>)>) -> impl IntoView {
    view! {
        <div class="breadcrumbs text-sm">
            <ul>
                {items
                    .into_iter()
                    .map(|(label, href)| match href {
                        Some(h) => view! { <li><a href=h>{label}</a></li> }.into_any(),
                        None => view! { <li>{label}</li> }.into_any(),
                    })
                    .collect_view()}
            </ul>
        </div>
    }
}

/// Auswahlfeld mit fester Liste (Schlüssel, Beschriftung). Formularname: `input[<field>]`.
#[component]
pub fn SelectField(
    field: &'static str,
    label: &'static str,
    options: &'static [(&'static str, &'static str)],
    value: String,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">{label}</legend>
            <select
                name=format!("input[{field}]")
                class="select w-full"
                class:select-error=move || errors.read().contains_key(field)
            >
                {options
                    .iter()
                    .map(|(k, l)| view! { <option value=*k selected=value == *k>{*l}</option> })
                    .collect_view()}
            </select>
            <ErrorText field errors/>
        </fieldset>
    }
}

/// Auswahlfeld mit einer Liste, die erst zur Laufzeit feststeht (z. B. aus der Datenbank).
/// Die Liste enthält Paare (Schlüssel, Beschriftung). Formularname: `input[<field>]`.
#[component]
pub fn OptionField(
    field: &'static str,
    label: &'static str,
    options: Vec<(String, String)>,
    value: String,
    errors: Signal<FieldErrors>,
) -> impl IntoView {
    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">{label}</legend>
            <select
                name=format!("input[{field}]")
                class="select w-full"
                class:select-error=move || errors.read().contains_key(field)
            >
                {options
                    .into_iter()
                    .map(|(key, text)| {
                        let selected = key == value;
                        view! { <option value=key selected=selected>{text}</option> }
                    })
                    .collect_view()}
            </select>
            <ErrorText field errors/>
        </fieldset>
    }
}

/// Meldung über dem Formular, wenn der Server das Speichern abgelehnt hat.
#[component]
pub fn ServerAlert(message: Signal<Option<String>>) -> impl IntoView {
    move || {
        message
            .get()
            .map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })
    }
}

/// Abbrechen und Speichern am Ende eines Formulars in einem Modal-Fenster.
#[component]
pub fn SaveButtons(cancel_href: Signal<String>, pending: Signal<bool>) -> impl IntoView {
    view! {
        <div class="modal-action mt-0">
            <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
            <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                "Speichern"
            </button>
        </div>
    }
}

/// Eintrag in einer Suchauswahl (z. B. Adresse oder Projekt).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PickOption {
    pub id: i64,
    pub nummer: String,
    pub name: String,
}

/// Auswahl mit Suche: zeigt Treffer von `search` an und speichert die Kennung in `input[<field>]`.
#[component]
pub fn SearchPicker<F, Fut>(
    field: &'static str,
    label: &'static str,
    placeholder: &'static str,
    initial: Option<PickOption>,
    errors: Signal<FieldErrors>,
    search: F,
    #[prop(optional)] required: bool,
    /// Wird nach jeder Auswahl bzw. jedem Entfernen aufgerufen (z. B. um die Webadresse zu ändern).
    #[prop(optional)]
    on_change: Option<Callback<Option<PickOption>>>,
) -> impl IntoView
where
    F: Fn(String) -> Fut + Copy + Send + Sync + 'static,
    Fut: std::future::Future<Output = Result<Vec<PickOption>, ServerFnError>> + 'static,
{
    use crate::icons::{Icon, IconView};

    let selected = RwSignal::new(initial);
    let query = RwSignal::new(String::new());
    let open = RwSignal::new(false);
    let results = LocalResource::new(move || search(query.get()));
    let has_error = move || errors.read().contains_key(field);

    let pick = move |c: PickOption| {
        selected.set(Some(c.clone()));
        open.set(false);
        query.set(String::new());
        if let Some(cb) = on_change {
            cb.run(Some(c));
        }
    };

    view! {
        <fieldset class="fieldset">
            <legend class="fieldset-legend">{label} {required.then_some(" *")}</legend>
            <input
                type="hidden"
                name=format!("input[{field}]")
                prop:value=move || selected.get().map(|c| c.id.to_string()).unwrap_or_default()
            />
            {move || match selected.get() {
                Some(c) => view! {
                    <div class="input w-full">
                        <span class="min-w-0 flex-1 truncate">
                            <span class="text-base-content/60">{c.nummer.clone()} " · "</span>
                            {c.name.clone()}
                        </span>
                        <button
                            type="button"
                            class="btn btn-ghost btn-xs"
                            aria-label=format!("{label} entfernen")
                            title=format!("{label} entfernen")
                            on:click=move |_| {
                                selected.set(None);
                                if let Some(cb) = on_change {
                                    cb.run(None);
                                }
                            }
                        >"✕"</button>
                    </div>
                }
                .into_any(),
                None => view! {
                    <div class="relative">
                        <label class="input w-full" class:input-error=has_error>
                            <IconView icon=Icon::Search/>
                            <input
                                type="text"
                                placeholder=placeholder
                                autocomplete="off"
                                aria-label=format!("{label} suchen")
                                prop:value=move || query.get()
                                on:input=move |ev| {
                                    query.set(event_target_value(&ev));
                                    open.set(true);
                                }
                                on:focus=move |_| open.set(true)
                                on:blur=move |_| open.set(false)
                                on:keydown=move |ev| if ev.key() == "Enter" { ev.prevent_default() }
                            />
                        </label>
                        <Show when=move || open.get()>
                            <ul class="menu absolute z-30 mt-1 max-h-64 w-full flex-nowrap overflow-y-auto rounded-box border border-base-300 bg-base-100 shadow-lg">
                                {move || match results.get() {
                                    None => view! { <li class="menu-disabled"><span>"Suche …"</span></li> }.into_any(),
                                    Some(Err(e)) => view! { <li class="text-error"><span>{error_message(&e)}</span></li> }.into_any(),
                                    Some(Ok(list)) if list.is_empty() => view! {
                                        <li class="menu-disabled"><span>"Nichts gefunden"</span></li>
                                    }.into_any(),
                                    Some(Ok(list)) => list
                                        .into_iter()
                                        .map(|c| {
                                            let name = c.name.clone();
                                            let nummer = c.nummer.clone();
                                            view! {
                                                <li>
                                                    // mousedown statt click: kommt vor dem blur des Suchfelds
                                                    <button type="button" on:mousedown=move |ev| {
                                                        ev.prevent_default();
                                                        pick(c.clone());
                                                    }>
                                                        <span class="text-base-content/60">{nummer}</span>
                                                        {name}
                                                    </button>
                                                </li>
                                            }
                                        })
                                        .collect_view()
                                        .into_any(),
                                }}
                            </ul>
                        </Show>
                    </div>
                }
                .into_any(),
            }}
            <ErrorText field errors/>
        </fieldset>
    }
}

/// Modal-Fenster. Schliessen über ✕, Klick daneben oder Escape führt zu `close_href`.
#[component]
pub fn Modal(title: String, close_href: Signal<String>, children: Children) -> impl IntoView {
    let navigate = use_navigate();
    let handle = window_event_listener(leptos::ev::keydown, move |ev| {
        if ev.key() == "Escape" {
            navigate(&close_href.get_untracked(), Default::default());
        }
    });
    on_cleanup(move || handle.remove());
    let label = title.clone();

    view! {
        // Mobiltelefon: Fenster unten (daisyUI `modal-bottom`) und gleitet von unten herein; ab `sm` mittig.
        <div class="modal modal-open max-sm:modal-bottom" role="dialog" aria-modal="true" aria-label=label>
            // dvh statt daisyUI-Vorgabe 100vh: Auf dem iPhone ist 100vh höher als der sichtbare Bereich
            <div class="modal-box max-h-[calc(100dvh-5em)] w-full sm:w-11/12 max-w-4xl motion-safe:max-sm:animate-modal-von-unten">
                <div class="mb-4 flex items-center justify-between gap-2">
                    <h3 class="text-lg font-bold">{title}</h3>
                    <a href=move || close_href.get() class="btn btn-circle btn-ghost btn-sm" aria-label="Schliessen">"✕"</a>
                </div>
                {children()}
            </div>
            <a href=move || close_href.get() class="modal-backdrop" aria-label="Schliessen" tabindex="-1"></a>
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn params_and_return_url() {
        assert_eq!(
            with_params("/projekte", &[("neu", "projekt".into())]),
            "/projekte?neu=projekt"
        );
        assert_eq!(
            with_params("/projekte?seite=2", &[("id", "5".into())]),
            "/projekte?seite=2&id=5"
        );
        assert_eq!(safe_return_url("/projekte/1"), Some("/projekte/1"));
        assert_eq!(safe_return_url("//boese.example.com"), None);
        assert_eq!(safe_return_url("https://boese.example.com"), None);
    }

    #[test]
    fn encode_umlaut_and_space() {
        assert_eq!(encode("Müller & Co"), "M%C3%BCller%20%26%20Co");
    }
}
