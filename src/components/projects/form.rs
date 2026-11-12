//! Formulare zum Anlegen und Bearbeiten auf allen Ebenen (AK-7 bis AK-11, AK-16).

use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::backend::addresses::server_fns::search_addresses;
use crate::backend::projects::model::{ElementInput, FieldErrors, Level, PickOption};
use crate::backend::projects::server_fns::{CreateElement, UpdateElement, get_tree};
use crate::backend::users::server_fns::search_active_users;
use crate::forms::{
    Modal, SearchPicker, SelectField, TextField, error_message, field_errors, is_saved,
    refresh_when, server_error, use_base_href,
};

/// Was das Modal-Fenster tun soll – kommt aus der Webadresse.
#[derive(Clone, Copy, Debug, PartialEq)]
enum FormRequest {
    /// `?neu=projekt` bzw. `?neu=<ebene>&unter=<kennung des übergeordneten Elements>`
    Create {
        level: Level,
        parent_id: Option<i64>,
    },
    /// `?bearbeiten=<ebene>&id=<kennung>`
    Edit { level: Level, id: i64 },
}

fn parse_request(
    neu: Option<String>,
    unter: Option<String>,
    bearbeiten: Option<String>,
    id: Option<String>,
) -> Option<FormRequest> {
    if let Some(level) = neu.as_deref().and_then(Level::from_key) {
        let parent_id = unter.and_then(|u| u.parse().ok());
        if level != Level::Project && parent_id.is_none() {
            return None;
        }
        return Some(FormRequest::Create { level, parent_id });
    }
    let level = bearbeiten.as_deref().and_then(Level::from_key)?;
    Some(FormRequest::Edit {
        level,
        id: id?.parse().ok()?,
    })
}

/// Öffnet je nach Webadresse das passende Modal-Fenster (Anlegen oder Bearbeiten, alle Ebenen).
/// Wird in der Projektliste und in allen Detailansichten eingebunden.
#[component]
pub fn ProjectFormHost() -> impl IntoView {
    let query = use_query_map();
    let request = Memo::new(move |_| {
        let q = query.read();
        parse_request(
            q.get("neu"),
            q.get("unter"),
            q.get("bearbeiten"),
            q.get("id"),
        )
    });
    move || request.get().map(|request| view! { <FormModal request/> })
}

/// Lädt bei Bedarf die Daten (übergeordnetes Element bzw. das Element selbst) und zeigt das Formular.
#[component]
fn FormModal(request: FormRequest) -> impl IntoView {
    let close = use_base_href();

    // Neues Projekt: nichts zu laden; nach dem Speichern öffnet sich die Projektansicht (AK-7).
    if let FormRequest::Create {
        level: Level::Project,
        ..
    } = request
    {
        return view! {
            <Modal title="Neues Projekt".to_string() close_href=close.into()>
                <ElementForm
                    level=Level::Project
                    request
                    initial=ElementInput::new_default(Level::Project)
                    customer=None
                    responsible=None
                    close
                    return_here=false
                />
            </Modal>
        }
        .into_any();
    }

    let (load_level, load_id) = match request {
        FormRequest::Create { level, parent_id } => (
            level.parent().unwrap_or(Level::Project),
            parent_id.unwrap_or_default(),
        ),
        FormRequest::Edit { level, id } => (level, id),
    };
    let data = Resource::new(move || (load_level, load_id), |(l, id)| get_tree(l, id));

    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                let tree = match data.await {
                    Ok(Some(t)) => t,
                    Ok(None) => {
                        return view! {
                            <Modal title=format!("{} nicht gefunden", load_level.label()) close_href=close.into()>
                                <p>"Dieses Element gibt es nicht oder es wurde gelöscht."</p>
                            </Modal>
                        }
                        .into_any();
                    }
                    Err(e) => {
                        return view! {
                            <Modal title="Fehler".to_string() close_href=close.into()>
                                <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                            </Modal>
                        }
                        .into_any();
                    }
                };
                // Titel und Anfangswerte bestimmen
                let found = if load_level == Level::Project {
                    Some((
                        format!("{} {}", tree.project.nummer, tree.project.name),
                        tree.project.to_input(),
                        tree.project.customer(),
                        tree.project.responsible().map(|r| r.pick_option()),
                    ))
                } else {
                    tree.find_path(load_level, load_id).and_then(|p| {
                        p.last().map(|n| {
                            let responsible = n.responsible.as_ref().map(|r| r.pick_option());
                            (n.title(), n.to_input(), None, responsible)
                        })
                    })
                };
                let Some((name, input, customer, responsible)) = found else {
                    return view! {
                        <Modal title=format!("{} nicht gefunden", load_level.label()) close_href=close.into()>
                            <p>"Dieses Element gibt es nicht oder es wurde gelöscht."</p>
                        </Modal>
                    }
                    .into_any();
                };
                let (level, title, initial, customer, responsible) = match request {
                    FormRequest::Create { level, .. } => (
                        level,
                        format!("{} in {name}", level.new_label()),
                        ElementInput::new_default(level),
                        None,
                        None,
                    ),
                    FormRequest::Edit { level, .. } => {
                        (level, format!("{name} bearbeiten"), input, customer, responsible)
                    }
                };
                view! {
                    <Modal title close_href=close.into()>
                        <ElementForm level request initial customer responsible close return_here=true/>
                    </Modal>
                }
                .into_any()
            })}
        </Suspense>
    }
    .into_any()
}

/// Formular mit passender Server-Aktion (Anlegen oder Bearbeiten).
/// `return_here`: nach dem Speichern zurück zur Seite unter dem Fenster (sonst zur Detailansicht).
#[component]
fn ElementForm(
    level: Level,
    request: FormRequest,
    initial: ElementInput,
    customer: Option<PickOption>,
    responsible: Option<PickOption>,
    close: Memo<String>,
    return_here: bool,
) -> impl IntoView {
    let zurueck = move || {
        return_here.then(|| view! { <input type="hidden" name="zurueck" value=close.get()/> })
    };
    match request {
        FormRequest::Create { parent_id, .. } => {
            let action = ServerAction::<CreateElement>::new();
            refresh_when(move || is_saved(action.value().get()));
            view! {
                <ActionForm action=action>
                    <input type="hidden" name="level" value=level.key()/>
                    {parent_id.map(|p| view! { <input type="hidden" name="parent_id" value=p/> })}
                    {zurueck}
                    <FormBody
                        level
                        initial
                        customer
                        responsible
                        cancel_href=close.into()
                        errors=Signal::derive(move || field_errors(action.value().get()))
                        server_error=Signal::derive(move || server_error(action.value().get()))
                        pending=action.pending().into()
                    />
                </ActionForm>
            }
            .into_any()
        }
        FormRequest::Edit { id, .. } => {
            let action = ServerAction::<UpdateElement>::new();
            refresh_when(move || is_saved(action.value().get()));
            view! {
                <ActionForm action=action>
                    <input type="hidden" name="level" value=level.key()/>
                    <input type="hidden" name="id" value=id/>
                    {zurueck}
                    <FormBody
                        level
                        initial
                        customer
                        responsible
                        cancel_href=close.into()
                        errors=Signal::derive(move || field_errors(action.value().get()))
                        server_error=Signal::derive(move || server_error(action.value().get()))
                        pending=action.pending().into()
                    />
                </ActionForm>
            }
            .into_any()
        }
    }
}

#[component]
fn FormBody(
    level: Level,
    initial: ElementInput,
    customer: Option<PickOption>,
    responsible: Option<PickOption>,
    cancel_href: Signal<String>,
    errors: Signal<FieldErrors>,
    server_error: Signal<Option<String>>,
    pending: Signal<bool>,
) -> impl IntoView {
    let i = initial;
    let required = Signal::stored(true);
    view! {
        <div>
            <div class="flex flex-col gap-4">
                {move || server_error.get().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                {move || (!errors.read().is_empty()).then(|| view! {
                    <div role="alert" class="alert alert-warning">"Bitte die markierten Felder korrigieren."</div>
                })}

                <TextField field="name" label=level.name_label() value=i.name.clone() errors required maxlength=200/>

                // Aufgaben bestehen nur aus Titel und Beschreibung – weitere Felder nur für Projekt, Teilprojekt, Vorgang
                {(level != Level::Task).then(|| view! {
                    <div class="grid gap-x-4 md:grid-cols-3">
                        <SelectField field="status" label="Status *" options=level.statuses() value=i.status.clone() errors/>
                        <TextField field="start" label="Start" kind="date" value=i.start.clone() errors/>
                        <TextField field="ende" label=level.end_label() kind="date" value=i.ende.clone() errors/>
                    </div>
                    // Kundenauswahl (AK-16)
                    {(level == Level::Project).then(|| view! {
                        <SearchPicker
                            field="adresse_id"
                            label="Kunde"
                            placeholder="Adresse suchen (Name, Nummer, Ort) …"
                            initial=customer.clone()
                            errors
                            search=search_addresses
                        />
                    })}
                    // Verantwortlicher Benutzer (ANF-005 AK-24); Aufgaben erben ihn von oben
                    <SearchPicker
                        field="benutzer_id"
                        label="Verantwortlich"
                        placeholder="Aktiven Benutzer suchen (Name, Benutzername) …"
                        initial=responsible.clone()
                        errors
                        search=search_active_users
                    />
                    <p class="-mt-3 text-xs text-base-content/60">{level.responsibility_scope()}</p>
                })}
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Beschreibung"</legend>
                    <textarea name="input[beschreibung]" class="textarea h-28 w-full">{i.beschreibung.clone()}</textarea>
                </fieldset>

                <p class="text-sm text-base-content/60">"* Pflichtfeld"</p>
                <div class="modal-action mt-0">
                    <a href=move || cancel_href.get() class="btn btn-ghost">"Abbrechen"</a>
                    <button type="submit" class="btn btn-primary" disabled=move || pending.get()>
                        {move || if pending.get() { "Speichern …" } else { "Speichern" }}
                    </button>
                </div>
            </div>
        </div>
    }
}
