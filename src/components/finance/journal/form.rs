//! Modal-Fenster „Neue Buchung“ (ANF-017 AK-3 bis AK-6, AK-10, AK-14): Kopf, Zeilen mit Konto, Betrag,
//! MWST-Code und Projekt, Vorschau der MWST-Zeilen und Ausgleich. Speichern als Entwurf oder buchen.
//!
//! Das Formular ist kein HTML-Formular mit Absendeknopf: Die Eingaben laufen über Signale und werden
//! beim Klick als Server-Aktion übergeben. So bleibt die Liste der Zeilen dynamisch.

use leptos::prelude::*;
use leptos_meta::Title;

use crate::backend::finance::journal::model::{
    AMOUNT_KINDS, BELEG_MAX, BookingDetail, BookingForm, BookingInput, FieldErrors, LineInput,
    SIDES, SaveOutcome, TEXT_MAX, check, expand, key_of, option_label, totals,
};
use crate::backend::finance::journal::server_fns::{SaveBooking, get_booking, get_booking_form};
use crate::backend::finance::vat::model::percent_label;
use crate::backend::purchase_orders::model::{format_amount, format_fixed};
use crate::forms::{Modal, error_message, server_error, use_base_href};

/// Was das Formular bewirkt: neue Buchung, Entwurf bearbeiten oder Vorlage einer Buchung kopieren (AK-14).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FormMode {
    Neu,
    Bearbeiten(i64),
    Kopie(i64),
}

/// Modal-Fenster der Buchung. Lädt Vorgaben und ggf. die Vorlage selbst.
#[component]
pub fn BookingFormModal(mode: FormMode) -> impl IntoView {
    let close = use_base_href();
    let title = match mode {
        FormMode::Neu => "Neue Buchung",
        FormMode::Bearbeiten(_) => "Entwurf bearbeiten",
        FormMode::Kopie(_) => "Buchung kopieren",
    };
    let data = Resource::new(move || mode, load);
    view! {
        <Title text="Buchung – Finanzen – uniqwork"/>
        <Modal title=title.to_string() close_href=close.into()>
            <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                {move || Suspend::new(async move {
                    match data.await {
                        Ok((form, source)) => view! {
                            <BookingFormBody form source mode close/>
                        }
                        .into_any(),
                        Err(e) => view! {
                            <div role="alert" class="alert alert-error">{error_message(&e)}</div>
                        }
                        .into_any(),
                    }
                })}
            </Suspense>
        </Modal>
    }
}

async fn load(mode: FormMode) -> Result<(BookingForm, Option<BookingDetail>), ServerFnError> {
    let form = get_booking_form().await?;
    let source = match mode {
        FormMode::Neu => None,
        FormMode::Bearbeiten(id) | FormMode::Kopie(id) => match get_booking(id).await? {
            Some(b) => Some(b),
            None => return Err(ServerFnError::new("Die Buchung gibt es nicht mehr.")),
        },
    };
    Ok((form, source))
}

/// Eine Zeile im Formular; jedes Feld ist ein Signal.
#[derive(Clone, Copy, PartialEq)]
struct Row {
    key: usize,
    seite: RwSignal<String>,
    konto: RwSignal<String>,
    betrag: RwSignal<String>,
    mwst: RwSignal<String>,
    projekt: RwSignal<String>,
}

impl Row {
    fn new(key: usize, line: LineInput) -> Self {
        Self {
            key,
            seite: RwSignal::new(if line.seite.is_empty() {
                "soll".into()
            } else {
                line.seite
            }),
            konto: RwSignal::new(line.konto),
            betrag: RwSignal::new(line.betrag),
            mwst: RwSignal::new(line.mwst),
            projekt: RwSignal::new(line.projekt),
        }
    }

    fn input(&self) -> LineInput {
        LineInput {
            seite: self.seite.get(),
            konto: self.konto.get(),
            betrag: self.betrag.get(),
            mwst: self.mwst.get(),
            projekt: self.projekt.get(),
        }
    }
}

/// Zeilen der Vorlage: Beträge netto (Basis) mit MWST-Code; die Steuerzeile entsteht beim Speichern neu.
fn lines_from(detail: &BookingDetail) -> (Vec<LineInput>, &'static str) {
    let mut art = "brutto";
    let lines = detail
        .zeilen
        .iter()
        .filter(|z| !z.ist_mwst_zeile)
        .map(|z| {
            if z.mwst_code_id.is_some() {
                art = "netto";
            }
            LineInput {
                seite: z.seite().to_string(),
                konto: option_label(&z.konto_nummer, &z.konto_bezeichnung),
                betrag: format_fixed(z.betrag(), 2),
                mwst: z.mwst_code_id.map(|id| id.to_string()).unwrap_or_default(),
                projekt: match (&z.projekt_nummer, &z.projekt_name) {
                    (Some(n), Some(name)) => option_label(n, name),
                    _ => String::new(),
                },
            }
        })
        .collect();
    (lines, art)
}

/// Vorschau einer gültigen Eingabe: Summen und MWST-Zeilen (AK-5, AK-6).
#[derive(Clone, PartialEq)]
struct Preview {
    soll: i64,
    haben: i64,
    steuer: Vec<(String, i64, bool)>,
}

#[component]
fn BookingFormBody(
    form: BookingForm,
    source: Option<BookingDetail>,
    mode: FormMode,
    close: Memo<String>,
) -> impl IntoView {
    let editing = match mode {
        FormMode::Bearbeiten(id) => id,
        _ => 0,
    };
    let heute = form.heute.clone();
    let (datum0, beleg0, text0, art0, lines0) = match &source {
        Some(b) => {
            let (lines, art) = lines_from(b);
            let datum = if matches!(mode, FormMode::Bearbeiten(_)) {
                b.head.datum.clone()
            } else {
                heute.clone()
            };
            let beleg = if matches!(mode, FormMode::Bearbeiten(_)) {
                b.head.belegnummer.clone()
            } else {
                form.belegnummer.clone()
            };
            (datum, beleg, b.head.text.clone(), art, lines)
        }
        None => (
            heute.clone(),
            form.belegnummer.clone(),
            String::new(),
            "brutto",
            vec![
                LineInput {
                    seite: "soll".into(),
                    ..Default::default()
                },
                LineInput {
                    seite: "haben".into(),
                    ..Default::default()
                },
            ],
        ),
    };

    let datum = RwSignal::new(datum0);
    let belegnummer = RwSignal::new(beleg0);
    let text = RwSignal::new(text0);
    let betragsart = RwSignal::new(art0.to_string());
    let count = lines0.len();
    let rows = RwSignal::new(
        lines0
            .into_iter()
            .enumerate()
            .map(|(i, line)| Row::new(i, line))
            .collect::<Vec<_>>(),
    );
    let next_key = RwSignal::new(count);
    let form = StoredValue::new(form);
    let errors = RwSignal::new(FieldErrors::new());
    let save = ServerAction::<SaveBooking>::new();
    let server_msg = Signal::derive(move || server_error(save.value().get()));

    // Ungültige Eingaben aus der Prüfung auf dem Server anzeigen (AK-6, AK-7)
    Effect::new(move |_| {
        if let Some(Ok(SaveOutcome::Invalid(e))) = save.value().get() {
            errors.set(e);
        }
    });

    let preview = Memo::new(move |_| {
        let input = BookingInput {
            datum: datum.get(),
            belegnummer: belegnummer.get(),
            text: text.get(),
            betragsart: betragsart.get(),
            zeilen: rows.get().iter().map(Row::input).collect(),
            buchen: false,
        };
        form.with_value(|f| {
            check(&input, f).ok().map(|values| {
                let lines = expand(&values, &f.codes);
                let (soll, haben) = totals(&lines);
                let steuer = lines
                    .iter()
                    .filter(|l| l.ist_mwst_zeile)
                    .map(|l| (konto_text(f, l.konto_id), l.betrag, l.soll))
                    .collect();
                Preview {
                    soll,
                    haben,
                    steuer,
                }
            })
        })
    });

    let submit = move |buchen: bool| {
        errors.set(FieldErrors::new());
        save.dispatch(SaveBooking {
            id: editing,
            input: BookingInput {
                datum: datum.get(),
                belegnummer: belegnummer.get(),
                text: text.get(),
                betragsart: betragsart.get(),
                zeilen: rows.get().iter().map(Row::input).collect(),
                buchen,
            },
            zurueck: close.get(),
        });
    };
    let pending = save.pending();
    let konten = form.with_value(|f| f.konten.clone());
    let projekte = form.with_value(|f| f.projekte.clone());

    let add_row = move |_| {
        let key = next_key.get();
        next_key.set(key + 1);
        rows.update(|r| {
            r.push(Row::new(
                key,
                LineInput {
                    seite: "haben".into(),
                    ..Default::default()
                },
            ))
        });
    };

    let field_error = move |name: &'static str| {
        move || {
            errors
                .read()
                .get(name)
                .cloned()
                .map(|m| view! { <p class="label text-error">{m}</p> })
        }
    };
    // Fehler einzelner Zeilen (`zeile2_konto`) als Liste, weil die Zeilen nach dem Speichern umnummeriert sind
    let line_errors = move || {
        errors
            .read()
            .iter()
            .filter(|(k, _)| k.starts_with("zeile"))
            .map(|(k, m)| (k.clone(), m.clone()))
            .collect::<Vec<_>>()
    };

    view! {
        <div class="flex flex-col gap-4">
            {move || server_msg.get().map(|m| view! { <div role="alert" class="alert alert-error">{m}</div> })}

            <div class="grid gap-x-4 gap-y-3 md:grid-cols-3">
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Datum *"</legend>
                    <input
                        type="date"
                        class="input w-full"
                        class:input-error=move || errors.read().contains_key("datum")
                        prop:value=move || datum.get()
                        on:input=move |ev| datum.set(event_target_value(&ev))
                    />
                    {field_error("datum")}
                    <p class="label">"Vorgabe: heute. Das Datum muss in einem offenen Geschäftsjahr und einer offenen Periode liegen."</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Belegnummer *"</legend>
                    <input
                        type="text"
                        class="input w-full"
                        maxlength=BELEG_MAX
                        class:input-error=move || errors.read().contains_key("belegnummer")
                        prop:value=move || belegnummer.get()
                        on:input=move |ev| belegnummer.set(event_target_value(&ev))
                    />
                    {field_error("belegnummer")}
                    <p class="label">"Vorschlag: nächste freie Nummer M-JJJJ-NNNN, frei änderbar."</p>
                </fieldset>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Beträge sind"</legend>
                    <select
                        class="select w-full"
                        aria-label="Beträge netto oder brutto"
                        prop:value=move || betragsart.get()
                        on:change=move |ev| betragsart.set(event_target_value(&ev))
                    >
                        {AMOUNT_KINDS
                            .iter()
                            .map(|(k, l)| view! { <option value=*k>{*l}</option> })
                            .collect_view()}
                    </select>
                    <p class="label">"Gilt für alle Zeilen mit MWST-Code. Vorgabe: brutto."</p>
                </fieldset>
            </div>

            <fieldset class="fieldset">
                <legend class="fieldset-legend">"Text *"</legend>
                <input
                    type="text"
                    class="input w-full"
                    maxlength=TEXT_MAX
                    class:input-error=move || errors.read().contains_key("text")
                    prop:value=move || text.get()
                    on:input=move |ev| text.set(event_target_value(&ev))
                />
                {field_error("text")}
            </fieldset>

            // Auswahllisten für Konto und Projekt: Eingabe filtert nach Nummer und Bezeichnung (AK-4)
            <datalist id="konten-auswahl">
                {konten
                    .iter()
                    .map(|k| view! { <option value=option_label(&k.nummer, &k.bezeichnung)></option> })
                    .collect_view()}
            </datalist>
            <datalist id="projekt-auswahl">
                {projekte
                    .iter()
                    .map(|p| view! { <option value=option_label(&p.nummer, &p.name)></option> })
                    .collect_view()}
            </datalist>

            <div class="space-y-2">
                <div class="flex flex-wrap items-center justify-between gap-2">
                    <h3 class="font-semibold">"Zeilen"</h3>
                    <button type="button" class="btn btn-sm" on:click=add_row>"Weitere Zeile"</button>
                </div>
                <p class="text-sm text-base-content/70">
                    "Konto mit Nummer oder Bezeichnung wählen. Ein Konto mit Vorgabe-MWST schlägt den Code vor."
                </p>
                {move || line_errors().into_iter().map(|(k, m)| {
                    let nr = k.trim_start_matches("zeile").split('_').next().unwrap_or("").to_string();
                    view! { <p role="alert" class="alert alert-error alert-soft py-1 text-sm">{format!("Zeile {nr}: {m}")}</p> }
                }).collect_view()}
                <For each=move || rows.get() key=|row| row.key let:row>
                    <LineFields row rows form/>
                </For>
                {field_error("zeilen")}
            </div>

            // Vorschau: Summen, Differenz und die MWST-Zeilen vor dem Speichern (AK-5, AK-6)
            <div class="rounded-box bg-base-200 p-3 text-sm space-y-2">
                {move || match preview.get() {
                    None => view! {
                        <p class="text-base-content/70">"Die Summen erscheinen, sobald alle Zeilen vollständig sind."</p>
                    }
                    .into_any(),
                    Some(p) => {
                        let diff = p.soll - p.haben;
                        let diff_class = if diff == 0 { "text-success" } else { "text-error" };
                        view! {
                            <div class="flex flex-wrap gap-x-6 gap-y-1 tabular-nums">
                                <span>"Summe Soll " {format_amount(p.soll)}</span>
                                <span>"Summe Haben " {format_amount(p.haben)}</span>
                                <span class=diff_class>"Differenz " {format_amount(diff)}</span>
                            </div>
                            {(!p.steuer.is_empty()).then(|| view! {
                                <ul class="list-disc pl-5">
                                    {p.steuer.iter().map(|(konto, betrag, soll)| view! {
                                        <li>
                                            "MWST-Zeile auf " {konto.clone()} ": " {format_amount(*betrag)}
                                            {if *soll { " im Soll" } else { " im Haben" }}
                                        </li>
                                    }).collect_view()}
                                </ul>
                            })}
                        }
                        .into_any()
                    }
                }}
                {move || errors.read().get("ausgleich").cloned().map(|m| view! {
                    <p role="alert" class="alert alert-error py-1">{m}</p>
                })}
            </div>

            <div class="flex flex-wrap items-center justify-end gap-2">
                <a href=move || close.get() class="btn btn-ghost">"Abbrechen"</a>
                <button
                    type="button"
                    class="btn"
                    disabled=move || pending.get()
                    on:click=move |_| submit(false)
                >
                    "Als Entwurf speichern"
                </button>
                <button
                    type="button"
                    class="btn btn-primary"
                    disabled=move || pending.get()
                    on:click=move |_| submit(true)
                >
                    "Buchen"
                </button>
            </div>
        </div>
    }
}

/// Eine Zeile: Seite, Konto, Betrag, MWST-Code, Projekt und Entfernen.
#[component]
fn LineFields(row: Row, rows: RwSignal<Vec<Row>>, form: StoredValue<BookingForm>) -> impl IntoView {
    let key = row.key;
    // Vorgabe-MWST des Kontos übernehmen, solange kein Code gewählt ist (AK-4)
    let konto_gewaehlt = move |_| {
        let nummer = key_of(&row.konto.get_untracked()).to_string();
        let vorgabe = form.with_value(|f| {
            f.konten
                .iter()
                .find(|k| k.nummer == nummer)
                .and_then(|k| k.vorgabe_mwst)
        });
        if let Some(id) = vorgabe
            && row.mwst.get_untracked().is_empty()
        {
            row.mwst.set(id.to_string());
        }
    };
    let codes = form.with_value(|f| f.codes.clone());
    let show_codes = form.with_value(|f| f.mwst_pflicht_ab.is_some());

    view! {
        <div class="flex flex-wrap items-start gap-2 rounded-box border border-base-300 p-2 sm:flex-nowrap">
            <select
                class="select w-full sm:w-28"
                aria-label="Seite"
                prop:value=move || row.seite.get()
                on:change=move |ev| row.seite.set(event_target_value(&ev))
            >
                {SIDES
                    .iter()
                    .map(|(k, l)| view! { <option value=*k>{*l}</option> })
                    .collect_view()}
            </select>
            <input
                type="text"
                list="konten-auswahl"
                autocomplete="off"
                class="input min-w-0 flex-1 basis-56"
                aria-label="Konto"
                placeholder="Konto: Nummer oder Bezeichnung"
                prop:value=move || row.konto.get()
                on:input=move |ev| row.konto.set(event_target_value(&ev))
                on:change=konto_gewaehlt
            />
            <input
                type="text"
                inputmode="decimal"
                class="input w-full text-right tabular-nums sm:w-32"
                aria-label="Betrag"
                placeholder="0.00"
                prop:value=move || row.betrag.get()
                on:input=move |ev| row.betrag.set(event_target_value(&ev))
            />
            {show_codes.then(|| view! {
                <select
                    class="select w-full sm:w-44"
                    aria-label="MWST-Code"
                    prop:value=move || row.mwst.get()
                    on:change=move |ev| row.mwst.set(event_target_value(&ev))
                >
                    <option value="">"ohne MWST"</option>
                    {codes
                        .iter()
                        .map(|c| view! {
                            <option value=c.id.to_string()>
                                {format!("{} · {}", c.code, percent_label(c.satz))}
                            </option>
                        })
                        .collect_view()}
                </select>
            })}
            <input
                type="text"
                list="projekt-auswahl"
                autocomplete="off"
                class="input w-full sm:w-44"
                aria-label="Projekt"
                placeholder="Projekt (optional)"
                prop:value=move || row.projekt.get()
                on:input=move |ev| row.projekt.set(event_target_value(&ev))
            />
            <button
                type="button"
                class="btn btn-ghost btn-square btn-sm"
                aria-label="Zeile entfernen"
                title="Zeile entfernen"
                on:click=move |_| rows.update(|r| r.retain(|x| x.key != key))
            >
                "✕"
            </button>
        </div>
    }
}

/// Anzeigename eines Kontos (Nummer und Bezeichnung) für die Vorschau.
fn konto_text(form: &BookingForm, konto_id: i64) -> String {
    form.konten
        .iter()
        .find(|k| k.id == konto_id)
        .map(|k| option_label(&k.nummer, &k.bezeichnung))
        .unwrap_or_else(|| format!("Konto {konto_id}"))
}
