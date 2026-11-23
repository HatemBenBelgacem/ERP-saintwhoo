//! Seite „Geländeplan“ eines Projekts (ANF-027): 3D-Karte mit Werkzeugen zum Platzieren von
//! Bühnen, Verpflegungsständen und Festzelten sowie zum Abstecken der Rasenfläche.
//!
//! Der Plan wird im Browser bearbeitet und mit „Speichern“ als Ganzes an den Server geschickt.
//! Den Standort ändert man im Modal-Fenster `?standort=1` (mit Adresssuche). Die Messwerkzeuge
//! (Strecke, Fläche) zeichnen nur auf der Karte und werden nicht gespeichert.

use leptos::html::Div;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

use super::bridge::{MapEvent, MapView, ViewState, search_address};
use crate::backend::site_plans::model::{
    AddressHit, Corner, ElementKind, MeasureKind, OriginInput, ROTATION_STEP_DEG, SiteElement,
    SitePlan, SitePlanPage, format_area, format_distance, format_m, measure,
};
use crate::backend::site_plans::server_fns::{DeleteSitePlan, SaveSitePlan, get_site_plan};
use crate::forms::{
    Breadcrumbs, FieldErrors, Modal, SaveOutcome, error_message, use_base_href, with_params,
};
use crate::icons::{Icon, IconView};

/// Gewähltes Werkzeug: auswählen und verschieben, Element platzieren, Rasenfläche abstecken oder messen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Select,
    Place(ElementKind),
    Area,
    Measure(MeasureKind),
}

/// Zustand des Editors, für die Teil-Komponenten bereitgestellt.
#[derive(Clone, Copy)]
struct Editor {
    plan: RwSignal<SitePlan>,
    /// Zuletzt gespeicherter Stand (für „Ungespeicherte Änderungen“ und „Verwerfen“).
    saved: RwSignal<SitePlan>,
    /// Gibt es den Plan schon in der Datenbank?
    stored: RwSignal<bool>,
    tool: RwSignal<Tool>,
    /// Hinweis zur letzten Aktion (z. B. Element passt nicht auf die Fläche).
    notice: RwSignal<Option<String>>,
    map: StoredValue<Option<MapView>, LocalStorage>,
    /// Punkte der laufenden Messung (nicht Teil des Plans).
    measure_points: RwSignal<Vec<Corner>>,
    /// Art der laufenden Messung (bleibt beim Wechsel zu einem anderen Werkzeug sichtbar).
    measure_kind: RwSignal<MeasureKind>,
    /// Postanschrift des Kunden – Vorschlag für die Adresssuche.
    customer_address: StoredValue<Option<String>>,
    /// Ausgewähltes Element (Index in `plan.elements`).
    selected: RwSignal<Option<usize>>,
}

impl Editor {
    fn dirty(self) -> bool {
        self.plan.with(|p| self.saved.with(|s| p != s))
    }

    /// Plan und Messung an die Karte geben (liest die Signale beobachtend).
    fn sync_map(self) {
        let kind = self.measure_kind.get();
        let selected = self.selected.get();
        // Ziehen geht beim Auswählen und beim Platzieren, nicht beim Abstecken und Messen
        let interactive = matches!(self.tool.get(), Tool::Select | Tool::Place(_));
        self.plan.with(|p| {
            self.measure_points.with(|pts| {
                let view = ViewState {
                    measurement: Some((kind, pts.as_slice())),
                    selected: selected.filter(|i| *i < p.elements.len()),
                    interactive,
                };
                self.map.with_value(|m| {
                    if let Some(m) = m {
                        m.sync(p, &view);
                    }
                })
            })
        });
    }

    /// Verschiebt das Element `i` um (dx, dz) Meter – nicht über den Rand der Rasenfläche hinaus,
    /// wenn es vorher darauf stand. Liefert `false`, wenn die Verschiebung abgewiesen wurde.
    fn nudge(self, i: usize, dx: f64, dz: f64) -> bool {
        let Some((moved, fitted)) = self.plan.with_untracked(|p| {
            p.elements.get(i).map(|e| {
                let mut m = e.clone();
                m.x += dx;
                m.z += dz;
                (m, p.fits(e))
            })
        }) else {
            return false;
        };
        if fitted && !self.plan.with_untracked(|p| p.fits(&moved)) {
            self.notice.set(Some(
                "Verschoben würde das Element über die Rasenfläche hinausragen.".into(),
            ));
            return false;
        }
        self.notice.set(None);
        self.plan.update(|p| p.elements[i] = moved);
        true
    }
}

#[component]
pub fn SitePlanPage() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    // Nur von der Kennung abhängig: Fenster-Parameter laden den Plan nicht neu
    let page = Resource::new(id, |id| async move {
        match id {
            Some(id) => get_site_plan(id).await,
            None => Ok(None),
        }
    });
    view! {
        <Transition fallback=|| view! { <span class="loading loading-spinner"></span> }>
            {move || Suspend::new(async move {
                match page.await {
                    Ok(Some(p)) => view! { <SitePlanEditor page=p/> }.into_any(),
                    Ok(None) => view! { <crate::components::projects::NotFound level=crate::backend::projects::model::Level::Project/> }.into_any(),
                    Err(e) => view! { <div role="alert" class="alert alert-error">{error_message(&e)}</div> }.into_any(),
                }
            })}
        </Transition>
    }
}

#[component]
fn SitePlanEditor(page: SitePlanPage) -> impl IntoView {
    let initial = page.plan.clone().unwrap_or_default();
    let ed = Editor {
        plan: RwSignal::new(initial.clone()),
        saved: RwSignal::new(initial),
        stored: RwSignal::new(page.plan.is_some()),
        tool: RwSignal::new(Tool::Select),
        notice: RwSignal::new(None),
        map: StoredValue::new_local(None),
        measure_points: RwSignal::new(Vec::new()),
        measure_kind: RwSignal::new(MeasureKind::Distance),
        selected: RwSignal::new(None),
        customer_address: StoredValue::new(page.kunde_adresse.clone()),
    };
    provide_context(ed);
    let map_error = RwSignal::new(None::<String>);
    let container = NodeRef::<Div>::new();
    let projekt_id = page.projekt_id;

    // Karte erstellen, sobald das Element im Dokument steht (nur im Browser)
    Effect::new(move |_| {
        let Some(el) = container.get() else { return };
        if ed.map.with_value(Option::is_some) {
            return;
        }
        let origin = ed.plan.with_untracked(|p| (p.origin_lng, p.origin_lat));
        let view = MapView::mount(
            &el,
            origin,
            move |event| handle_event(ed, event),
            move |msg| map_error.set(Some(msg)),
        );
        ed.map.set_value(Some(view));
        untrack(|| ed.sync_map());
    });
    // Jede Änderung am Plan oder an der Messung sofort in 3D zeigen
    Effect::new(move |_| ed.sync_map());
    on_cleanup(move || {
        // Karte entfernen (Drop von MapView ruft destroy() auf)
        let _ = ed.map.try_update_value(|m| *m = None);
    });

    // Beim Verlassen der Seite mit ungespeicherten Änderungen nachfragen (Neuladen, Schliessen)
    let unload = window_event_listener(leptos::ev::beforeunload, move |ev| {
        if ed.dirty() {
            ev.prevent_default();
        }
    });
    on_cleanup(move || unload.remove());

    // Tastatur: Pfeiltasten verschieben das ausgewählte Element um 0,5 m (mit Umschalt um 5 m),
    // Escape hebt die Auswahl auf. Nicht in Eingabefeldern und nicht bei offenem Modal-Fenster.
    let keys = window_event_listener(leptos::ev::keydown, move |ev| {
        let Some(i) = ed.selected.get_untracked() else {
            return;
        };
        let in_field = ev
            .target()
            .and_then(|t| {
                leptos::wasm_bindgen::JsCast::dyn_into::<leptos::web_sys::Element>(t).ok()
            })
            .is_some_and(|el| matches!(el.tag_name().as_str(), "INPUT" | "TEXTAREA" | "SELECT"));
        let modal_open = leptos::prelude::document()
            .query_selector(".modal-open")
            .ok()
            .flatten()
            .is_some();
        if in_field || modal_open {
            return;
        }
        let step = if ev.shift_key() { 5.0 } else { 0.5 };
        let (dx, dz) = match ev.key().as_str() {
            "ArrowLeft" => (-step, 0.0),
            "ArrowRight" => (step, 0.0),
            "ArrowUp" => (0.0, -step),
            "ArrowDown" => (0.0, step),
            "Escape" => {
                ed.selected.set(None);
                return;
            }
            _ => return,
        };
        ev.prevent_default();
        ed.nudge(i, dx, dz);
    });
    on_cleanup(move || keys.remove());

    let crumbs = vec![
        ("Projekte".to_string(), Some("/projekte".to_string())),
        (
            page.projekt_nummer.clone(),
            Some(format!("/projekte/{projekt_id}")),
        ),
        ("Geländeplan".to_string(), None),
    ];
    let delete_open = RwSignal::new(false);

    view! {
        <Title text=format!("Geländeplan {} – uniqwork", page.projekt_nummer)/>
        <div class="space-y-4">
            <Breadcrumbs items=crumbs/>
            <Header projekt_id title=page.projekt_name.clone() changed=page.geaendert_am.clone() delete_open/>
            <OriginModal/>
            {move || map_error.get().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
            <div class="grid gap-4 lg:grid-cols-[22rem_minmax(0,1fr)]">
                <div class="order-2 space-y-4 lg:order-1">
                    <ToolCard/>
                    <MeasureCard/>
                    <AreaCard/>
                    <ElementList/>
                </div>
                // Karte mit 3D-Ebene: rechts, auf dem Mobiltelefon oben
                <div class="order-1 lg:order-2">
                    <div class="card overflow-hidden bg-base-100 shadow">
                        <div
                            node_ref=container
                            class="h-[60vh] w-full bg-base-300 lg:h-[calc(100dvh-14rem)] lg:min-h-[32rem]"
                            role="application"
                            aria-label="3D-Karte des Geländes"
                        ></div>
                    </div>
                    <p class="mt-2 text-sm text-base-content/60">
                        "Rechte Maustaste oder Strg + Ziehen kippt und dreht die Karte, auf dem Mobiltelefon zwei Finger."
                    </p>
                </div>
            </div>
        </div>
        <DeleteDialog projekt_id open=delete_open/>
    }
}

/// Bedienung auf der Karte: auswählen, verschieben oder Klick an eine freie Stelle.
fn handle_event(ed: Editor, event: MapEvent) {
    match event {
        MapEvent::Click { x, z } => place(ed, x, z),
        MapEvent::Pick { index } => {
            ed.selected.set(Some(index));
            ed.notice.set(None);
        }
        // Während des Ziehens folgt das Element dem Zeiger; geprüft wird beim Loslassen
        MapEvent::Drag { index, x, z } => ed.plan.update(|p| {
            if let Some(e) = p.elements.get_mut(index) {
                e.x = x;
                e.z = z;
            }
        }),
        MapEvent::Drop {
            index,
            start_x,
            start_z,
        } => {
            let verdict = ed.plan.with_untracked(|p| {
                p.elements.get(index).map(|e| {
                    let mut before = e.clone();
                    before.x = start_x;
                    before.z = start_z;
                    // Abgewiesen, wenn es vorher auf der Fläche stand und jetzt hinausragt
                    p.fits(&before) && !p.fits(e)
                })
            });
            if verdict == Some(true) {
                ed.plan.update(|p| {
                    p.elements[index].x = start_x;
                    p.elements[index].z = start_z;
                });
                ed.notice.set(Some(
                    "Das Element wurde zurückgesetzt: An der neuen Stelle stünde es nicht ganz auf der Rasenfläche."
                        .into(),
                ));
            }
        }
    }
}

/// Klick auf eine freie Stelle: Auswahl aufheben, Eckpunkt oder Messpunkt setzen oder Element
/// platzieren (nur auf der Rasenfläche).
fn place(ed: Editor, x: f64, z: f64) {
    match ed.tool.get_untracked() {
        Tool::Select => ed.selected.set(None),
        Tool::Area => {
            ed.plan.update(|p| p.corners.push(Corner { x, z }));
            ed.notice.set(None);
        }
        Tool::Measure(_) => ed.measure_points.update(|pts| pts.push(Corner { x, z })),
        Tool::Place(kind) => {
            let element = SiteElement::new(kind, x, z);
            if ed.plan.with_untracked(|p| p.fits(&element)) {
                ed.plan.update(|p| p.elements.push(element));
                ed.notice.set(None);
            } else {
                ed.notice.set(Some(format!(
                    "„{}“ wurde nicht platziert, weil das Element nicht ganz auf der Rasenfläche stünde.",
                    kind.label()
                )));
            }
        }
    }
}

/// Titel, Speichern, Verwerfen, Standort und Löschen.
#[component]
fn Header(
    projekt_id: i64,
    title: String,
    changed: Option<String>,
    delete_open: RwSignal<bool>,
) -> impl IntoView {
    let ed = expect_context::<Editor>();
    let save = ServerAction::<SaveSitePlan>::new();
    let base = use_base_href();
    let changed = RwSignal::new(changed);
    // Gesendeter Stand – gilt nach erfolgreichem Speichern als gespeichert
    let sent = StoredValue::new(None::<SitePlan>);

    // Nach dem Speichern gilt der gesendete Stand als gespeichert
    Effect::new(move |_| {
        if let Some(Ok(SaveOutcome::Saved(_))) = save.value().get() {
            if let Some(plan) = sent.get_value() {
                ed.saved.set(plan);
            }
            ed.stored.set(true);
            changed.set(None);
        }
    });
    let errors = move || match save.value().get() {
        Some(Ok(SaveOutcome::Invalid(errors))) => Some(errors.into_values().collect::<Vec<_>>()),
        Some(Err(e)) => Some(vec![error_message(&e)]),
        _ => None,
    };
    let just_saved =
        move || matches!(save.value().get(), Some(Ok(SaveOutcome::Saved(_)))) && !ed.dirty();

    view! {
        <div class="flex flex-wrap items-start justify-between gap-2">
            <div>
                <h1 class="text-2xl font-semibold">"Geländeplan"</h1>
                <p class="flex flex-wrap items-center gap-2 text-base-content/70">
                    {title}
                    {move || ed.dirty().then(|| view! { <span class="badge badge-warning">"Ungespeicherte Änderungen"</span> })}
                    {move || (!ed.stored.get()).then(|| view! { <span class="badge badge-ghost">"Noch nicht gespeichert"</span> })}
                    {move || changed.get().map(|c| view! { <span class="text-sm">"Geändert am " {c}</span> })}
                </p>
                <p class="mt-1 flex items-center gap-1 text-sm text-base-content/70">
                    <IconView icon=Icon::Map/>
                    {move || ed.plan.with(|p| p.address.clone()).unwrap_or_else(|| "Keine Standort-Adresse hinterlegt".into())}
                </p>
            </div>
            <div class="flex flex-wrap gap-2">
                <button
                    type="button"
                    class="btn btn-primary"
                    disabled=move || save.pending().get() || (!ed.dirty() && ed.stored.get())
                    on:click=move |_| {
                        let plan = ed.plan.get_untracked();
                        sent.set_value(Some(plan.clone()));
                        save.dispatch(SaveSitePlan { projekt_id, plan });
                    }
                >
                    {move || save.pending().get().then(|| view! { <span class="loading loading-spinner loading-sm"></span> })}
                    "Speichern"
                </button>
                <button
                    type="button"
                    class="btn"
                    disabled=move || !ed.dirty()
                    on:click=move |_| {
                        let saved = ed.saved.get_untracked();
                        let origin = (saved.origin_lng, saved.origin_lat);
                        ed.selected.set(None);
                        ed.plan.set(saved);
                        ed.map.with_value(|m| if let Some(m) = m { m.set_origin(origin.0, origin.1) });
                        ed.notice.set(None);
                    }
                >
                    "Verwerfen"
                </button>
                <a href=move || with_params(&base.get(), &[("standort", "1".into())]) class="btn">
                    <IconView icon=Icon::Map/>
                    "Standort"
                </a>
                <Show when=move || ed.stored.get()>
                    <button type="button" class="btn btn-outline btn-error" on:click=move |_| delete_open.set(true)>
                        <IconView icon=Icon::Trash/>
                        "Löschen"
                    </button>
                </Show>
            </div>
        </div>
        {move || errors().map(|list| view! {
            <div role="alert" class="alert alert-error">
                <ul>{list.into_iter().map(|e| view! { <li>{e}</li> }).collect_view()}</ul>
            </div>
        })}
        <Show when=just_saved>
            <div role="alert" class="alert alert-success">"Gespeichert."</div>
        </Show>
        {move || ed.notice.get().map(|n| view! { <div role="alert" class="alert alert-warning">{n}</div> })}
    }
}

/// Werkzeuge: ein Element wählen und auf die Karte klicken.
#[component]
fn ToolCard() -> impl IntoView {
    let ed = expect_context::<Editor>();
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Werkzeug"</h2>
                <p class="text-sm text-base-content/70">
                    "Element wählen und auf die Karte klicken, um es dort zu platzieren. Platzierte Elemente "
                    "mit der Maus oder dem Finger anfassen und ziehen, um sie zu verschieben."
                </p>
                <div class="flex flex-col gap-2">
                    <button
                        type="button"
                        class="btn justify-start"
                        class:btn-primary=move || ed.tool.get() == Tool::Select
                        class:btn-outline=move || ed.tool.get() != Tool::Select
                        aria-pressed=move || (ed.tool.get() == Tool::Select).to_string()
                        on:click=move |_| ed.tool.set(Tool::Select)
                    >
                        <IconView icon=Icon::Move/>
                        "Auswählen und verschieben"
                    </button>
                    {ElementKind::ALL
                        .into_iter()
                        .map(|kind| {
                            let tool = Tool::Place(kind);
                            view! {
                                <button
                                    type="button"
                                    class="btn justify-start"
                                    class:btn-primary=move || ed.tool.get() == tool
                                    class:btn-outline=move || ed.tool.get() != tool
                                    aria-pressed=move || (ed.tool.get() == tool).to_string()
                                    on:click=move |_| ed.tool.set(tool)
                                >
                                    <span class=kind.swatch_class()></span>
                                    {kind.tool_label()}
                                </button>
                            }
                        })
                        .collect_view()}
                    <button
                        type="button"
                        class="btn justify-start"
                        class:btn-primary=move || ed.tool.get() == Tool::Area
                        class:btn-outline=move || ed.tool.get() != Tool::Area
                        aria-pressed=move || (ed.tool.get() == Tool::Area).to_string()
                        on:click=move |_| ed.tool.set(Tool::Area)
                    >
                        <span class="inline-block size-3 shrink-0 rounded-sm bg-site-area"></span>
                        "Rasenfläche abstecken"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Messwerkzeuge: Strecke und Fläche. Jeder Klick auf die Karte setzt einen Messpunkt; die
/// Längen erscheinen auf der Karte und hier. Messungen werden nicht gespeichert.
#[component]
fn MeasureCard() -> impl IntoView {
    let ed = expect_context::<Editor>();
    let result = move || {
        let kind = ed.measure_kind.get();
        ed.measure_points
            .with(|pts| (kind, pts.len(), measure(kind, pts)))
    };
    let tool_button = move |kind: MeasureKind, label: &'static str| {
        let tool = Tool::Measure(kind);
        view! {
            <button
                type="button"
                class="btn btn-sm flex-1"
                class:btn-primary=move || ed.tool.get() == tool
                class:btn-outline=move || ed.tool.get() != tool
                aria-pressed=move || (ed.tool.get() == tool).to_string()
                on:click=move |_| {
                    ed.tool.set(tool);
                    ed.measure_kind.set(kind);
                }
            >
                {label}
            </button>
        }
    };
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Messen"</h2>
                <div class="flex gap-2">
                    {tool_button(MeasureKind::Distance, "Strecke messen")}
                    {tool_button(MeasureKind::Area, "Fläche messen")}
                </div>
                {move || {
                    let (kind, count, m) = result();
                    if count == 0 {
                        return view! {
                            <p class="text-sm text-base-content/70">
                                "Werkzeug wählen und auf die Karte klicken: jeder Klick setzt einen Messpunkt. "
                                "Messungen werden nicht gespeichert."
                            </p>
                        }.into_any();
                    }
                    let total_label = if kind == MeasureKind::Area { "Umfang" } else { "Strecke" };
                    view! {
                        <dl class="grid grid-cols-2 gap-1 text-sm">
                            <dt class="text-base-content/70">{total_label}</dt>
                            <dd class="text-right font-semibold">{format_distance(m.length_m)}</dd>
                            {(kind == MeasureKind::Area).then(|| view! {
                                <dt class="text-base-content/70">"Fläche"</dt>
                                <dd class="text-right font-semibold">
                                    {m.area_m2.map_or_else(|| "–".to_string(), format_area)}
                                </dd>
                            })}
                            <dt class="text-base-content/70">"Punkte"</dt>
                            <dd class="text-right">{count}</dd>
                        </dl>
                        {(m.segments.len() > 1).then(|| view! {
                            <ol class="list-inside list-decimal text-xs text-base-content/70">
                                {m.segments.iter().map(|s| view! { <li>{format_distance(*s)}</li> }).collect_view()}
                            </ol>
                        })}
                        {m.self_intersecting.then(|| view! {
                            <div role="alert" class="alert alert-warning text-sm">
                                "Der Umriss überschneidet sich – die Fläche stimmt so nicht. Punkte rund um die Fläche setzen."
                            </div>
                        })}
                        {(kind == MeasureKind::Area && count < 3).then(|| view! {
                            <p class="text-sm text-base-content/70">"Für eine Fläche mindestens drei Punkte setzen."</p>
                        })}
                    }.into_any()
                }}
                <div class="flex flex-wrap gap-2">
                    <button
                        type="button"
                        class="btn btn-sm"
                        disabled=move || ed.measure_points.with(Vec::is_empty)
                        on:click=move |_| ed.measure_points.update(|pts| { pts.pop(); })
                    >
                        "Letzten Punkt entfernen"
                    </button>
                    <button
                        type="button"
                        class="btn btn-sm btn-ghost"
                        disabled=move || ed.measure_points.with(Vec::is_empty)
                        on:click=move |_| ed.measure_points.update(Vec::clear)
                    >
                        "Messung löschen"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Rasenfläche: Eckpunkte, Fläche in m² und Hinweis auf Elemente ausserhalb.
#[component]
fn AreaCard() -> impl IntoView {
    let ed = expect_context::<Editor>();
    let corners = move || ed.plan.with(|p| p.corners.len());
    let area = move || ed.plan.with(|p| p.area_m2());
    let crossed = move || ed.plan.with(SitePlan::area_crossed);
    let outside = move || {
        ed.plan
            .with(|p| p.elements.iter().filter(|e| !p.fits(e)).count())
    };
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Rasenfläche"</h2>
                <dl class="grid grid-cols-2 gap-1 text-sm">
                    <dt class="text-base-content/70">"Fläche"</dt>
                    <dd class="text-right font-semibold">
                        {move || if corners() >= 3 { format_area(area()) } else { "–".to_string() }}
                    </dd>
                    <dt class="text-base-content/70">"Eckpunkte"</dt>
                    <dd class="text-right">{corners}</dd>
                </dl>
                {move || match corners() {
                    0 => Some(view! {
                        <p class="text-sm text-base-content/70">
                            "Ohne Rasenfläche lassen sich Elemente überall platzieren. Mit „Rasenfläche abstecken“ "
                            "setzt jeder Klick einen Eckpunkt."
                        </p>
                    }.into_any()),
                    1 | 2 => Some(view! {
                        <p class="text-sm text-base-content/70">"Mindestens drei Eckpunkte setzen."</p>
                    }.into_any()),
                    _ => None,
                }}
                {move || crossed().then(|| view! {
                    <div role="alert" class="alert alert-warning text-sm">
                        "Der Umriss überschneidet sich – Fläche und Lageprüfung stimmen so nicht, und der Plan "
                        "lässt sich nicht speichern. Eckpunkte rund um die Fläche setzen."
                    </div>
                })}
                {move || (outside() > 0).then(|| view! {
                    <div role="alert" class="alert alert-warning text-sm">
                        {match outside() {
                            1 => "1 Element steht nicht ganz auf der Rasenfläche.".to_string(),
                            n => format!("{n} Elemente stehen nicht ganz auf der Rasenfläche."),
                        }}
                        " Verschieben oder löschen, sonst lässt sich der Plan nicht speichern."
                    </div>
                })}
                <div class="flex flex-wrap gap-2">
                    <button
                        type="button"
                        class="btn btn-sm"
                        disabled=move || corners() == 0
                        on:click=move |_| ed.plan.update(|p| { p.corners.pop(); })
                    >
                        "Letzten Eckpunkt entfernen"
                    </button>
                    <button
                        type="button"
                        class="btn btn-sm btn-ghost"
                        disabled=move || corners() == 0
                        on:click=move |_| ed.plan.update(|p| p.corners.clear())
                    >
                        "Fläche entfernen"
                    </button>
                </div>
            </div>
        </div>
    }
}

/// Liste der platzierten Elemente mit Drehen und Löschen.
#[component]
fn ElementList() -> impl IntoView {
    let ed = expect_context::<Editor>();
    let rotate = move |i: usize| {
        let rotated = ed.plan.with_untracked(|p| {
            p.elements
                .get(i)
                .map(|e| (e.rotated(ROTATION_STEP_DEG), p.fits(e)))
        });
        let Some((rotated, fitted)) = rotated else {
            return;
        };
        // Ein Element, das auf der Fläche steht, darf durch Drehen nicht hinausragen
        if fitted && !ed.plan.with_untracked(|p| p.fits(&rotated)) {
            ed.notice.set(Some(
                "Gedreht würde das Element über die Rasenfläche hinausragen.".into(),
            ));
            return;
        }
        ed.notice.set(None);
        ed.plan.update(|p| p.elements[i] = rotated);
    };
    let remove = move |i: usize| {
        ed.notice.set(None);
        // Auswahl folgt dem Element bzw. entfällt mit ihm
        ed.selected.update(|s| {
            *s = match *s {
                Some(j) if j == i => None,
                Some(j) if j > i => Some(j - 1),
                other => other,
            }
        });
        ed.plan.update(|p| {
            if i < p.elements.len() {
                p.elements.remove(i);
            }
        });
    };

    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">
                    "Platzierte Elemente"
                    <span class="badge badge-ghost">{move || ed.plan.with(|p| p.elements.len())}</span>
                </h2>
                {move || {
                    let plan = ed.plan.get();
                    if plan.elements.is_empty() {
                        return view! {
                            <p class="text-sm text-base-content/70">"Noch keine Elemente platziert."</p>
                        }.into_any();
                    }
                    view! {
                        <ul class="flex max-h-[40vh] flex-col gap-2 overflow-y-auto lg:max-h-none">
                            {plan
                                .elements
                                .iter()
                                .enumerate()
                                .map(|(i, e)| {
                                    let fits = plan.fits(e);
                                    let name = format!("{} {}", e.kind.label(), i + 1);
                                    view! {
                                        <li
                                            class="flex items-center justify-between gap-2 rounded-box p-2 text-sm"
                                            class:bg-base-200=move || ed.selected.get() != Some(i)
                                            class=("bg-primary/15", move || ed.selected.get() == Some(i))
                                            class=("ring-1", move || ed.selected.get() == Some(i))
                                            class=("ring-primary", move || ed.selected.get() == Some(i))
                                        >
                                            <button
                                                type="button"
                                                class="flex min-w-0 flex-1 cursor-pointer items-center gap-2 text-left"
                                                title=format!("{name} auswählen")
                                                aria-pressed=move || (ed.selected.get() == Some(i)).to_string()
                                                on:click=move |_| ed.selected.set(Some(i))
                                            >
                                                <span class=e.kind.swatch_class()></span>
                                                <span class="block min-w-0">
                                                    <span class="flex items-center gap-2 font-semibold">
                                                        {name.clone()}
                                                        {(!fits).then(|| view! { <span class="badge badge-warning badge-sm">"ausserhalb"</span> })}
                                                    </span>
                                                    <span class="block text-xs text-base-content/60">
                                                        {format!(
                                                            "{} × {} m · X {} m · Z {} m · {}°",
                                                            format_m(e.width_m),
                                                            format_m(e.depth_m),
                                                            format_m(e.x),
                                                            format_m(e.z),
                                                            format_m(e.rotation_deg),
                                                        )}
                                                    </span>
                                                </span>
                                            </button>
                                            <div class="flex shrink-0 gap-1">
                                                <button
                                                    type="button"
                                                    class="btn btn-ghost btn-xs"
                                                    title=format!("{name} um 45° drehen")
                                                    aria-label=format!("{name} um 45° drehen")
                                                    on:click=move |_| rotate(i)
                                                >"↻"</button>
                                                <button
                                                    type="button"
                                                    class="btn btn-ghost btn-xs text-error"
                                                    title=format!("{name} löschen")
                                                    aria-label=format!("{name} löschen")
                                                    on:click=move |_| remove(i)
                                                >"✕"</button>
                                            </div>
                                        </li>
                                    }
                                })
                                .collect_view()}
                        </ul>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

/// Modal-Fenster „Standort“ (`?standort=1`): Längen- und Breitengrad des Ursprungs.
/// Der ganze Plan wandert mit dem Standort mit.
#[component]
fn OriginModal() -> impl IntoView {
    let query = use_query_map();
    let open = move || query.read().get("standort").is_some();
    view! { <Show when=open>{|| view! { <OriginForm/> }}</Show> }
}

#[component]
fn OriginForm() -> impl IntoView {
    let ed = expect_context::<Editor>();
    let base = use_base_href();
    let navigate = use_navigate();
    let (lng, lat, address) = ed
        .plan
        .with_untracked(|p| (p.origin_lng, p.origin_lat, p.address.clone()));
    let input = RwSignal::new(OriginInput {
        laenge: format!("{lng:.6}"),
        breite: format!("{lat:.6}"),
        adresse: address.clone().unwrap_or_default(),
    });
    let errors = RwSignal::new(FieldErrors::new());
    let take_center = move |_| {
        if let Some((lng, lat)) = ed.map.with_value(|m| m.as_ref().and_then(MapView::center)) {
            input.update(|i| {
                i.laenge = format!("{lng:.6}");
                i.breite = format!("{lat:.6}");
            });
        }
    };

    // Adresssuche: Vorschlag ist die hinterlegte Adresse, sonst die des Kunden
    let query = RwSignal::new(
        address
            .or_else(|| ed.customer_address.get_value())
            .unwrap_or_default(),
    );
    let hits = RwSignal::new(None::<Result<Vec<AddressHit>, String>>);
    let searching = RwSignal::new(false);
    let picked = RwSignal::new(None::<usize>);
    let search = move || {
        let q = query.get_untracked().trim().to_string();
        if q.len() < 3 || searching.get_untracked() {
            hits.set(Some(Err("Bitte mindestens drei Zeichen eingeben.".into())));
            return;
        }
        searching.set(true);
        picked.set(None);
        leptos::task::spawn_local(async move {
            let result = search_address(q).await;
            searching.set(false);
            hits.set(Some(result));
        });
    };
    let pick = move |i: usize, hit: AddressHit| {
        picked.set(Some(i));
        input.set(OriginInput {
            laenge: format!("{:.6}", hit.lng),
            breite: format!("{:.6}", hit.lat),
            adresse: hit.label,
        });
        errors.set(FieldErrors::new());
    };

    let apply = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        match input.get_untracked().parse() {
            Ok((lng, lat, address)) => {
                ed.plan.update(|p| {
                    p.origin_lng = lng;
                    p.origin_lat = lat;
                    p.address = address;
                });
                ed.map.with_value(|m| {
                    if let Some(m) = m {
                        m.set_origin(lng, lat);
                    }
                });
                navigate(&base.get_untracked(), Default::default());
            }
            Err(e) => errors.set(e),
        }
    };
    let field = move |key: &'static str, label: &'static str| {
        let has_error = move || errors.read().contains_key(key);
        view! {
            <fieldset class="fieldset">
                <legend class="fieldset-legend">{label} " *"</legend>
                <input
                    type="text"
                    inputmode="decimal"
                    class="input w-full"
                    class:input-error=has_error
                    aria-invalid=move || has_error().then_some("true")
                    prop:value=move || input.with(|i| if key == "laenge" { i.laenge.clone() } else { i.breite.clone() })
                    on:input=move |ev| {
                        let v = event_target_value(&ev);
                        input.update(|i| if key == "laenge" { i.laenge = v } else { i.breite = v });
                    }
                />
                {move || errors.read().get(key).cloned().map(|e| view! { <p class="label text-error">{e}</p> })}
            </fieldset>
        }
    };

    view! {
        <Modal title="Standort des Geländes".to_string() close_href=Signal::from(base)>
            <form on:submit=apply class="space-y-4">
                <p class="text-sm text-base-content/70">
                    "Der Standort ist der Bezugspunkt des Plans. Elemente und Rasenfläche wandern mit, "
                    "wenn er geändert wird. Adresse suchen und einen Treffer wählen – oder die Karte zum "
                    "Gelände ziehen und „Kartenmitte übernehmen“ wählen."
                </p>
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Adresse suchen"</legend>
                    <div class="join w-full">
                        <label class="input join-item w-full">
                            <IconView icon=Icon::Search/>
                            <input
                                type="search"
                                placeholder="z. B. Kreuzbleiche, St. Gallen"
                                aria-label="Adresse suchen"
                                maxlength=200
                                prop:value=move || query.get()
                                on:input=move |ev| query.set(event_target_value(&ev))
                                // Enter sucht, statt das Formular abzuschicken
                                on:keydown=move |ev| if ev.key() == "Enter" {
                                    ev.prevent_default();
                                    search();
                                }
                            />
                        </label>
                        <button type="button" class="btn join-item" disabled=move || searching.get() on:click=move |_| search()>
                            {move || searching.get().then(|| view! { <span class="loading loading-spinner loading-sm"></span> })}
                            "Suchen"
                        </button>
                    </div>
                </fieldset>
                {move || hits.get().map(|r| match r {
                    Err(e) => view! { <div role="alert" class="alert alert-error text-sm">{e}</div> }.into_any(),
                    Ok(list) if list.is_empty() => view! {
                        <p class="text-sm text-base-content/70">"Nichts gefunden. Andere Schreibweise versuchen oder Ort ergänzen."</p>
                    }.into_any(),
                    Ok(list) => view! {
                        <div>
                            <ul class="menu w-full rounded-box border border-base-300 bg-base-100 p-1" aria-label="Treffer der Adresssuche">
                                {list.into_iter().enumerate().map(|(i, hit)| {
                                    let label = hit.label.clone();
                                    view! {
                                        <li>
                                            <button
                                                type="button"
                                                class:menu-active=move || picked.get() == Some(i)
                                                on:click=move |_| pick(i, hit.clone())
                                            >
                                                {label}
                                            </button>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                            <p class="mt-1 text-xs text-base-content/60">"Suche: Nominatim, Daten © OpenStreetMap-Mitwirkende"</p>
                        </div>
                    }.into_any(),
                })}
                <fieldset class="fieldset">
                    <legend class="fieldset-legend">"Adresse / Bezeichnung des Standorts"</legend>
                    <input
                        type="text"
                        class="input w-full"
                        class:input-error=move || errors.read().contains_key("adresse")
                        maxlength=300
                        placeholder="z. B. Festwiese Kreuzbleiche"
                        prop:value=move || input.with(|i| i.adresse.clone())
                        on:input=move |ev| {
                            let v = event_target_value(&ev);
                            input.update(|i| i.adresse = v);
                        }
                    />
                    {move || errors.read().get("adresse").cloned().map(|e| view! { <p class="label text-error">{e}</p> })}
                </fieldset>
                <div class="grid gap-4 sm:grid-cols-2">
                    {field("breite", "Breitengrad (Nord)")}
                    {field("laenge", "Längengrad (Ost)")}
                </div>
                <div class="modal-action flex-wrap">
                    <button type="button" class="btn" on:click=take_center>"Kartenmitte übernehmen"</button>
                    <a href=move || base.get() class="btn btn-ghost">"Abbrechen"</a>
                    <button type="submit" class="btn btn-primary">"Übernehmen"</button>
                </div>
            </form>
        </Modal>
    }
}

/// Rückfrage vor dem Löschen des ganzen Plans.
#[component]
fn DeleteDialog(projekt_id: i64, open: RwSignal<bool>) -> impl IntoView {
    let delete = ServerAction::<DeleteSitePlan>::new();
    let error = move || {
        delete
            .value()
            .get()
            .and_then(|r| r.err())
            .map(|e| error_message(&e))
    };
    view! {
        <div class="modal max-sm:modal-bottom" class:modal-open=move || open.get() role="dialog" aria-modal="true">
            <div class="modal-box">
                <h3 class="text-lg font-bold">"Geländeplan löschen?"</h3>
                <p class="py-4">
                    "Soll der Geländeplan mit allen Elementen und der Rasenfläche wirklich gelöscht werden? "
                    "Das kann nicht rückgängig gemacht werden."
                </p>
                {move || error().map(|e| view! { <div role="alert" class="alert alert-error">{e}</div> })}
                <div class="modal-action">
                    <button type="button" class="btn" on:click=move |_| open.set(false)>"Abbrechen"</button>
                    <ActionForm action=delete>
                        <input type="hidden" name="projekt_id" value=projekt_id/>
                        <button type="submit" class="btn btn-error" disabled=move || delete.pending().get()>
                            "Endgültig löschen"
                        </button>
                    </ActionForm>
                </div>
            </div>
            <div class="modal-backdrop" on:click=move |_| open.set(false)></div>
        </div>
    }
}
