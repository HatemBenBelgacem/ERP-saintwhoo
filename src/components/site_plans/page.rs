//! Seite „Geländeplan“ eines Projekts (ANF-027): 3D-Karte mit Werkzeugen zum Platzieren von
//! Bühnen, Verpflegungsständen und Festzelten sowie zum Abstecken der Rasenfläche.
//!
//! Der Plan wird im Browser bearbeitet und mit „Speichern“ als Ganzes an den Server geschickt.
//! Den Standort ändert man im Modal-Fenster `?standort=1`.

use leptos::html::Div;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

use super::bridge::MapView;
use crate::backend::site_plans::model::{
    Corner, ElementKind, OriginInput, ROTATION_STEP_DEG, SiteElement, SitePlan, SitePlanPage,
    format_area, format_m,
};
use crate::backend::site_plans::server_fns::{DeleteSitePlan, SaveSitePlan, get_site_plan};
use crate::forms::{
    Breadcrumbs, FieldErrors, Modal, SaveOutcome, error_message, use_base_href, with_params,
};
use crate::icons::{Icon, IconView};

/// Gewähltes Werkzeug: Element platzieren oder Rasenfläche abstecken.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tool {
    Place(ElementKind),
    Area,
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
}

impl Editor {
    fn dirty(self) -> bool {
        self.plan.with(|p| self.saved.with(|s| p != s))
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
        tool: RwSignal::new(Tool::Place(ElementKind::FoodStall)),
        notice: RwSignal::new(None),
        map: StoredValue::new_local(None),
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
            move |x, z| place(ed, x, z),
            move |msg| map_error.set(Some(msg)),
        );
        ed.plan.with_untracked(|p| view.sync(p));
        ed.map.set_value(Some(view));
    });
    // Jede Änderung am Plan sofort in 3D zeigen
    Effect::new(move |_| {
        ed.plan.with(|p| {
            ed.map.with_value(|m| {
                if let Some(m) = m {
                    m.sync(p);
                }
            })
        });
    });
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

/// Klick auf die Karte: Eckpunkt setzen oder Element platzieren (nur auf der Rasenfläche).
fn place(ed: Editor, x: f64, z: f64) {
    match ed.tool.get_untracked() {
        Tool::Area => {
            ed.plan.update(|p| p.corners.push(Corner { x, z }));
            ed.notice.set(None);
        }
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
                    "Element wählen und auf die Karte klicken, um es dort zu platzieren."
                </p>
                <div class="flex flex-col gap-2">
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

/// Rasenfläche: Eckpunkte, Fläche in m² und Hinweis auf Elemente ausserhalb.
#[component]
fn AreaCard() -> impl IntoView {
    let ed = expect_context::<Editor>();
    let corners = move || ed.plan.with(|p| p.corners.len());
    let area = move || ed.plan.with(|p| p.area_m2());
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
                                        <li class="flex items-center justify-between gap-2 rounded-box bg-base-200 p-2 text-sm">
                                            <div class="flex min-w-0 items-center gap-2">
                                                <span class=e.kind.swatch_class()></span>
                                                <div class="min-w-0">
                                                    <div class="flex items-center gap-2 font-semibold">
                                                        {name.clone()}
                                                        {(!fits).then(|| view! { <span class="badge badge-warning badge-sm">"ausserhalb"</span> })}
                                                    </div>
                                                    <div class="text-xs text-base-content/60">
                                                        {format!(
                                                            "{} × {} m · X {} m · Z {} m · {}°",
                                                            format_m(e.width_m),
                                                            format_m(e.depth_m),
                                                            format_m(e.x),
                                                            format_m(e.z),
                                                            format_m(e.rotation_deg),
                                                        )}
                                                    </div>
                                                </div>
                                            </div>
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
    let (lng, lat) = ed.plan.with_untracked(|p| (p.origin_lng, p.origin_lat));
    let input = RwSignal::new(OriginInput {
        laenge: format!("{lng:.6}"),
        breite: format!("{lat:.6}"),
    });
    let errors = RwSignal::new(FieldErrors::new());
    let take_center = move |_| {
        if let Some((lng, lat)) = ed.map.with_value(|m| m.as_ref().and_then(MapView::center)) {
            input.set(OriginInput {
                laenge: format!("{lng:.6}"),
                breite: format!("{lat:.6}"),
            });
        }
    };
    let apply = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        match input.get_untracked().parse() {
            Ok((lng, lat)) => {
                ed.plan.update(|p| {
                    p.origin_lng = lng;
                    p.origin_lat = lat;
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
                    "wenn er geändert wird. Tipp: Karte zum Gelände ziehen und „Kartenmitte übernehmen“ wählen."
                </p>
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
