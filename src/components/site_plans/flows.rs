//! Besucherströme im Geländeplan (ANF-028): Werkzeuge und Listen für Besucherwege, Zugänge und Zonen.
//!
//! Wie bei den Messungen setzt jeder Klick auf die Karte einen Punkt des aktiven Wegs bzw. der aktiven
//! Zone; Zugänge entstehen mit einem Klick. Alle Angaben gehören zum Plan und werden mit ihm gespeichert.

use leptos::prelude::*;

use super::page::{Editor, Tool};
use crate::backend::site_plans::flows::{
    Access, AccessKind, DEFAULT_DENSITY, MAX_FLOW_ITEMS, MAX_FLOW_LABEL_LEN, RouteDirection,
    VisitorRoute, Zone, format_count, parse_count, parse_decimal,
};
use crate::backend::site_plans::model::{Corner, format_area, format_distance, format_m};

/// Punkt an den aktiven Besucherweg anhängen oder einen neuen Weg beginnen.
pub(super) fn add_route_point(ed: Editor, point: Corner) {
    let active = ed
        .active_route
        .get_untracked()
        .filter(|i| ed.plan.with_untracked(|p| *i < p.routes.len()));
    if let Some(i) = active {
        ed.plan.update(|p| p.routes[i].points.push(point));
        return;
    }
    if !room_for(
        ed,
        ed.plan.with_untracked(|p| p.routes.len()),
        "Besucherwege",
    ) {
        return;
    }
    let mut index = 0;
    ed.plan.update(|p| {
        p.routes.push(VisitorRoute::new(point));
        index = p.routes.len() - 1;
    });
    ed.active_route.set(Some(index));
}

/// Zugang der gewählten Art an die Stelle setzen.
pub(super) fn add_access(ed: Editor, kind: AccessKind, x: f64, z: f64) {
    if !room_for(ed, ed.plan.with_untracked(|p| p.accesses.len()), "Zugänge") {
        return;
    }
    ed.plan.update(|p| {
        p.accesses.push(Access {
            kind,
            label: String::new(),
            x,
            z,
            persons_per_minute: None,
        })
    });
}

/// Eckpunkt an die aktive Zone anhängen oder eine neue Zone beginnen.
pub(super) fn add_zone_point(ed: Editor, point: Corner) {
    let active = ed
        .active_zone
        .get_untracked()
        .filter(|i| ed.plan.with_untracked(|p| *i < p.zones.len()));
    if let Some(i) = active {
        ed.plan.update(|p| p.zones[i].points.push(point));
        return;
    }
    if !room_for(ed, ed.plan.with_untracked(|p| p.zones.len()), "Zonen") {
        return;
    }
    let mut index = 0;
    ed.plan.update(|p| {
        p.zones.push(Zone::new(point));
        index = p.zones.len() - 1;
    });
    ed.active_zone.set(Some(index));
}

/// Höchstzahl prüfen und sonst einen Hinweis zeigen.
fn room_for(ed: Editor, count: usize, what: &str) -> bool {
    if count >= MAX_FLOW_ITEMS {
        ed.notice.set(Some(format!(
            "Ein Plan darf höchstens {MAX_FLOW_ITEMS} {what} enthalten."
        )));
        return false;
    }
    ed.notice.set(None);
    true
}

/// Index nach dem Löschen von Eintrag `removed` nachführen.
fn shift_after_remove(active: Option<usize>, removed: usize) -> Option<usize> {
    match active {
        Some(j) if j == removed => None,
        Some(j) if j > removed => Some(j - 1),
        other => other,
    }
}

/// Bereich „Besucherströme“ in der Seitenleiste.
#[component]
pub(super) fn FlowsCard() -> impl IntoView {
    let ed = expect_context::<Editor>();
    let routes = move || ed.plan.with(|p| p.routes.len());
    let accesses = move || ed.plan.with(|p| p.accesses.len());
    let zones = move || ed.plan.with(|p| p.zones.len());
    let has_routes = move || routes() > 0;
    let has_accesses = move || accesses() > 0;
    let has_zones = move || zones() > 0;

    // Werkzeug-Schaltfläche; `reset` hebt den aktiven Weg bzw. die aktive Zone auf (neu beginnen)
    let tool_button = move |tool: Tool, label: &'static str, reset: fn(Editor)| {
        let on = move || {
            ed.tool.get() == tool
                && match tool {
                    Tool::Route => ed.active_route.get().is_none(),
                    Tool::Zone => ed.active_zone.get().is_none(),
                    _ => true,
                }
        };
        view! {
            <button
                type="button"
                class="btn btn-sm"
                class:btn-primary=on
                class:btn-outline=move || !on()
                aria-pressed=move || on().to_string()
                on:click=move |_| {
                    ed.tool.set(tool);
                    reset(ed);
                }
            >
                {label}
            </button>
        }
    };
    let hint = move || match ed.tool.get() {
        Tool::Route => Some(if ed.active_route.get().is_some() {
            "Jeder Klick auf die Karte setzt einen weiteren Punkt des markierten Wegs.".to_string()
        } else {
            "Auf die Karte klicken, um einen neuen Besucherweg zu beginnen (Richtung: vom ersten zum letzten Punkt).".to_string()
        }),
        Tool::Access(kind) => Some(format!(
            "Auf die Karte klicken, um einen {} zu setzen.",
            kind.label()
        )),
        Tool::Zone => Some(if ed.active_zone.get().is_some() {
            "Jeder Klick auf die Karte setzt einen weiteren Eckpunkt der markierten Zone."
                .to_string()
        } else {
            "Auf die Karte klicken, um eine neue Zone zu beginnen.".to_string()
        }),
        _ => None,
    };

    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body gap-3">
                <h2 class="card-title text-base">"Besucherströme"</h2>
                <div class="grid grid-cols-2 gap-2">
                    {tool_button(Tool::Route, "Neuer Weg", |ed| ed.active_route.set(None))}
                    {tool_button(Tool::Zone, "Neue Zone", |ed| ed.active_zone.set(None))}
                </div>
                <div class="flex flex-wrap gap-2">
                    {AccessKind::ALL
                        .into_iter()
                        .map(|kind| {
                            let tool = Tool::Access(kind);
                            view! {
                                <button
                                    type="button"
                                    class="btn btn-xs flex-1"
                                    class:btn-primary=move || ed.tool.get() == tool
                                    class:btn-outline=move || ed.tool.get() != tool
                                    aria-pressed=move || (ed.tool.get() == tool).to_string()
                                    on:click=move |_| ed.tool.set(tool)
                                >
                                    <span class=kind.swatch_class()></span>
                                    {kind.label()}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
                {move || hint().map(|h| view! { <p class="text-sm text-base-content/70">{h}</p> })}
                <Show when=move || routes() + accesses() + zones() == 0 && hint().is_none()>
                    <p class="text-sm text-base-content/70">
                        "Wege der Besucher mit Richtung und Menge, Ein- und Ausgänge sowie Zonen mit "
                        "Fassungsvermögen einzeichnen. Die Ampel zeigt, ob der Platz in einer Zone reicht."
                    </p>
                </Show>

                <Show when=has_routes>
                    <h3 class="text-sm font-semibold">"Wege " <span class="badge badge-ghost badge-sm">{routes}</span></h3>
                    <ul class="flex flex-col gap-2">
                        <For each=move || 0..routes() key=|i| *i let:i>
                            <RouteRow i/>
                        </For>
                    </ul>
                </Show>
                <Show when=has_accesses>
                    <h3 class="text-sm font-semibold">"Zugänge " <span class="badge badge-ghost badge-sm">{accesses}</span></h3>
                    <ul class="flex flex-col gap-2">
                        <For each=move || 0..accesses() key=|i| *i let:i>
                            <AccessRow i/>
                        </For>
                    </ul>
                </Show>
                <Show when=has_zones>
                    <h3 class="text-sm font-semibold">"Zonen " <span class="badge badge-ghost badge-sm">{zones}</span></h3>
                    <ul class="flex flex-col gap-2">
                        <For each=move || 0..zones() key=|i| *i let:i>
                            <ZoneRow i/>
                        </For>
                    </ul>
                </Show>
            </div>
        </div>
    }
}

/// Klassen einer Listenzeile; die aktive ist in leichtem Indigo markiert.
#[component]
fn Row(active: Signal<bool>, children: Children) -> impl IntoView {
    view! {
        <li
            class="flex flex-col gap-2 rounded-box p-2 text-sm"
            class:bg-base-200=move || !active.get()
            class=("bg-primary/15", move || active.get())
            class=("ring-1", move || active.get())
            class=("ring-primary", move || active.get())
        >
            {children()}
        </li>
    }
}

/// Textfeld für eine Bezeichnung (Fokus bleibt beim Tippen, weil die Zeile bestehen bleibt).
fn label_input(
    value: impl Fn() -> String + Send + Sync + 'static,
    placeholder: String,
    aria: String,
    set: impl Fn(String) + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <input
            type="text"
            class="input input-sm min-w-0 flex-1"
            maxlength=MAX_FLOW_LABEL_LEN
            placeholder=placeholder
            aria-label=aria
            prop:value=value
            on:input=move |ev| set(event_target_value(&ev))
        />
    }
}

/// Zahlenfeld mit Beschriftung; übernommen wird beim Verlassen des Felds. Ungültige Eingaben
/// werden am Feld gemeldet und nicht übernommen.
fn number_field(
    label: &'static str,
    value: impl Fn() -> String + Send + Sync + 'static,
    placeholder: &'static str,
    apply: impl Fn(&str) -> bool + Send + Sync + 'static,
) -> impl IntoView {
    let invalid = RwSignal::new(false);
    view! {
        <label class="flex flex-col gap-1 text-xs text-base-content/70">
            {label}
            <input
                type="text"
                inputmode="decimal"
                class="input input-sm w-full"
                class:input-error=move || invalid.get()
                aria-invalid=move || invalid.get().then_some("true")
                placeholder=placeholder
                prop:value=value
                on:change=move |ev| invalid.set(!apply(&event_target_value(&ev)))
            />
            {move || invalid.get().then(|| view! { <span class="text-error">"Ungültige Zahl"</span> })}
        </label>
    }
}

/// Schaltflächen „Weiter zeichnen“ bzw. „Letzten Punkt entfernen“ und „Fertig“.
fn draw_buttons(
    ed: Editor,
    active: Signal<bool>,
    can_pop: Signal<bool>,
    resume: impl Fn() + Send + Sync + Copy + 'static,
    pop: impl Fn() + Send + Sync + Copy + 'static,
    finish: impl Fn() + Send + Sync + Copy + 'static,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap gap-2">
            <Show
                when=move || active.get()
                fallback=move || view! {
                    <button type="button" class="btn btn-xs" on:click=move |_| resume()>"Weiter zeichnen"</button>
                }
            >
                <button type="button" class="btn btn-xs" disabled=move || !can_pop.get() on:click=move |_| pop()>
                    "Letzten Punkt entfernen"
                </button>
                <button
                    type="button"
                    class="btn btn-xs btn-primary"
                    on:click=move |_| {
                        finish();
                        ed.tool.set(Tool::Select);
                    }
                >"Fertig"</button>
            </Show>
        </div>
    }
}

/// Ein Besucherweg: Bezeichnung, Richtung, Personen pro Stunde, Breite, Länge.
#[component]
fn RouteRow(i: usize) -> impl IntoView {
    let ed = expect_context::<Editor>();
    let get = move || ed.plan.with(|p| p.routes.get(i).cloned());
    let update = move |f: &dyn Fn(&mut VisitorRoute)| {
        ed.plan.update(|p| {
            if let Some(r) = p.routes.get_mut(i) {
                f(r);
            }
        })
    };
    let name = move || get().map(|r| r.display_name(i)).unwrap_or_default();
    let active = Signal::derive(move || ed.active_route.get() == Some(i));
    let remove = move |_| {
        ed.active_route.update(|a| *a = shift_after_remove(*a, i));
        ed.plan.update(|p| {
            if i < p.routes.len() {
                p.routes.remove(i);
            }
        });
    };
    let direction = move || get().map_or(RouteDirection::OneWay, |r| r.direction);

    view! {
        <Row active>
            <div class="flex items-center gap-2">
                <span class="inline-block h-1.5 w-4 shrink-0 rounded-full bg-site-route"></span>
                {label_input(
                    move || get().map(|r| r.label).unwrap_or_default(),
                    format!("Weg {}", i + 1),
                    format!("Bezeichnung von Weg {}", i + 1),
                    move |v| update(&|r| r.label = v.clone()),
                )}
                <button
                    type="button"
                    class="btn btn-ghost btn-xs text-error"
                    title=move || format!("{} löschen", name())
                    aria-label=move || format!("{} löschen", name())
                    on:click=remove
                >"✕"</button>
            </div>
            <div class="grid grid-cols-2 gap-2">
                <label class="flex flex-col gap-1 text-xs text-base-content/70">
                    "Richtung"
                    <select
                        class="select select-sm w-full"
                        on:change=move |ev| {
                            if let Some(d) = RouteDirection::from_key(&event_target_value(&ev)) {
                                update(&|r| r.direction = d);
                            }
                        }
                    >
                        {RouteDirection::ALL
                            .into_iter()
                            .map(|d| view! { <option value=d.key() selected=move || direction() == d>{d.label()}</option> })
                            .collect_view()}
                    </select>
                </label>
                {number_field(
                    "Personen pro Stunde",
                    move || get().map(|r| if r.persons_per_hour == 0 { String::new() } else { format_count(r.persons_per_hour) }).unwrap_or_default(),
                    "z. B. 1’200",
                    move |s| {
                        let v = if s.trim().is_empty() { Some(0) } else { parse_count(s) };
                        v.inspect(|n| update(&|r| r.persons_per_hour = *n)).is_some()
                    },
                )}
                {number_field(
                    "Breite (m)",
                    move || get().and_then(|r| r.width_m).map(format_m).unwrap_or_default(),
                    "optional",
                    move |s| {
                        if s.trim().is_empty() {
                            update(&|r| r.width_m = None);
                            return true;
                        }
                        parse_decimal(s).inspect(|w| update(&|r| r.width_m = Some(*w))).is_some()
                    },
                )}
                <div class="flex flex-col justify-end text-xs text-base-content/70">
                    "Länge"
                    <span class="font-semibold text-base-content">
                        {move || get().map(|r| format_distance(r.length_m())).unwrap_or_default()}
                    </span>
                </div>
            </div>
            {move || get().filter(|r| r.points.len() < 2).map(|_| view! {
                <p class="text-xs text-base-content/70">"Mindestens zwei Punkte setzen."</p>
            })}
            {draw_buttons(
                ed,
                active,
                Signal::derive(move || get().is_some_and(|r| !r.points.is_empty())),
                move || {
                    ed.tool.set(Tool::Route);
                    ed.active_route.set(Some(i));
                },
                move || update(&|r| { r.points.pop(); }),
                move || ed.active_route.set(None),
            )}
        </Row>
    }
}

/// Ein Zugang: Art, Bezeichnung, Durchlass.
#[component]
fn AccessRow(i: usize) -> impl IntoView {
    let ed = expect_context::<Editor>();
    let get = move || ed.plan.with(|p| p.accesses.get(i).cloned());
    let update = move |f: &dyn Fn(&mut Access)| {
        ed.plan.update(|p| {
            if let Some(a) = p.accesses.get_mut(i) {
                f(a);
            }
        })
    };
    let kind = move || get().map_or(AccessKind::Entrance, |a| a.kind);
    let name = move || get().map(|a| a.display_name(i)).unwrap_or_default();
    let remove = move |_| {
        ed.plan.update(|p| {
            if i < p.accesses.len() {
                p.accesses.remove(i);
            }
        })
    };

    view! {
        <Row active=Signal::stored(false)>
            <div class="flex items-center gap-2">
                <span class=move || kind().swatch_class()></span>
                {label_input(
                    move || get().map(|a| a.label).unwrap_or_default(),
                    format!("{} {}", kind().label(), i + 1),
                    format!("Bezeichnung von Zugang {}", i + 1),
                    move |v| update(&|a| a.label = v.clone()),
                )}
                <button
                    type="button"
                    class="btn btn-ghost btn-xs text-error"
                    title=move || format!("{} löschen", name())
                    aria-label=move || format!("{} löschen", name())
                    on:click=remove
                >"✕"</button>
            </div>
            <div class="grid grid-cols-2 gap-2">
                <label class="flex flex-col gap-1 text-xs text-base-content/70">
                    "Art"
                    <select
                        class="select select-sm w-full"
                        on:change=move |ev| {
                            if let Some(k) = AccessKind::from_key(&event_target_value(&ev)) {
                                update(&|a| a.kind = k);
                            }
                        }
                    >
                        {AccessKind::ALL
                            .into_iter()
                            .map(|k| view! { <option value=k.key() selected=move || kind() == k>{k.label()}</option> })
                            .collect_view()}
                    </select>
                </label>
                {number_field(
                    "Durchlass (P/min)",
                    move || get().and_then(|a| a.persons_per_minute).map(format_count).unwrap_or_default(),
                    "optional",
                    move |s| {
                        if s.trim().is_empty() {
                            update(&|a| a.persons_per_minute = None);
                            return true;
                        }
                        parse_count(s).inspect(|n| update(&|a| a.persons_per_minute = Some(*n))).is_some()
                    },
                )}
            </div>
        </Row>
    }
}

/// Eine Zone: Bezeichnung, erwartete Personen, Dichte, Fläche, Fassungsvermögen und Ampel.
#[component]
fn ZoneRow(i: usize) -> impl IntoView {
    let ed = expect_context::<Editor>();
    let get = move || ed.plan.with(|p| p.zones.get(i).cloned());
    let update = move |f: &dyn Fn(&mut Zone)| {
        ed.plan.update(|p| {
            if let Some(z) = p.zones.get_mut(i) {
                f(z);
            }
        })
    };
    let name = move || get().map(|z| z.display_name(i)).unwrap_or_default();
    let active = Signal::derive(move || ed.active_zone.get() == Some(i));
    let remove = move |_| {
        ed.active_zone.update(|a| *a = shift_after_remove(*a, i));
        ed.plan.update(|p| {
            if i < p.zones.len() {
                p.zones.remove(i);
            }
        });
    };

    view! {
        <Row active>
            <div class="flex items-center gap-2">
                <span class="inline-block size-3 shrink-0 rounded-sm border border-base-content/40"></span>
                {label_input(
                    move || get().map(|z| z.label).unwrap_or_default(),
                    format!("Zone {}", i + 1),
                    format!("Bezeichnung von Zone {}", i + 1),
                    move |v| update(&|z| z.label = v.clone()),
                )}
                <button
                    type="button"
                    class="btn btn-ghost btn-xs text-error"
                    title=move || format!("{} löschen", name())
                    aria-label=move || format!("{} löschen", name())
                    on:click=remove
                >"✕"</button>
            </div>
            <div class="grid grid-cols-2 gap-2">
                {number_field(
                    "Erwartete Personen",
                    move || get().map(|z| if z.expected_persons == 0 { String::new() } else { format_count(z.expected_persons) }).unwrap_or_default(),
                    "z. B. 800",
                    move |s| {
                        let v = if s.trim().is_empty() { Some(0) } else { parse_count(s) };
                        v.inspect(|n| update(&|z| z.expected_persons = *n)).is_some()
                    },
                )}
                {number_field(
                    "Dichte (P/m²)",
                    move || get().map(|z| format_m(z.density)).unwrap_or_default(),
                    "2",
                    move |s| {
                        let v = if s.trim().is_empty() { Some(DEFAULT_DENSITY) } else { parse_decimal(s) };
                        v.inspect(|d| update(&|z| z.density = *d)).is_some()
                    },
                )}
            </div>
            {move || get().map(|z| {
                let status = z.status();
                view! {
                    <dl class="grid grid-cols-2 gap-1 text-xs text-base-content/70">
                        <dt>"Fläche"</dt>
                        <dd class="text-right">{z.area_m2().map_or_else(|| "–".to_string(), format_area)}</dd>
                        <dt>"Fassungsvermögen"</dt>
                        <dd class="text-right">{z.capacity().map_or_else(|| "–".to_string(), |c| format!("{} P", format_count(c)))}</dd>
                    </dl>
                    <div class="flex items-center justify-between gap-2">
                        <span class=status.badge_class()>{status.label()}</span>
                        <span class="font-semibold">{z.occupancy_text()}</span>
                    </div>
                    {(z.points.len() < 3).then(|| view! {
                        <p class="text-xs text-base-content/70">"Mindestens drei Eckpunkte setzen."</p>
                    })}
                    {z.crossed().then(|| view! {
                        <div role="alert" class="alert alert-warning text-xs">
                            "Der Umriss überschneidet sich. Eckpunkte rund um die Zone setzen."
                        </div>
                    })}
                    {(z.points.len() >= 3 && !z.crossed() && z.expected_persons == 0).then(|| view! {
                        <p class="text-xs text-base-content/70">"Erwartete Personen eintragen, um die Ampel zu sehen."</p>
                    })}
                }
            })}
            {draw_buttons(
                ed,
                active,
                Signal::derive(move || get().is_some_and(|z| !z.points.is_empty())),
                move || {
                    ed.tool.set(Tool::Zone);
                    ed.active_zone.set(Some(i));
                },
                move || update(&|z| { z.points.pop(); }),
                move || ed.active_zone.set(None),
            )}
        </Row>
    }
}
