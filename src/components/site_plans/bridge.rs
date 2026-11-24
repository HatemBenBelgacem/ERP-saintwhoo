//! Rust-Seite der Kartenbrücke (ANF-027): steuert `map_bridge.js` (MapLibre + Three.js).
//!
//! `MapView` gibt es in beiden Übersetzungen, damit die Ansicht ohne `cfg` auskommt; auf dem
//! Server hat sie keine Wirkung (Effekte laufen dort ohnehin nicht).

use serde::{Deserialize, Serialize};

use crate::backend::site_plans::flows::RouteDirection;
use crate::backend::site_plans::model::{
    AddressHit, Corner, Measure, MeasureKind, SitePlan, format_distance,
};

/// Bedienung auf der Karte, gemeldet von `map_bridge.js` (Meter relativ zum Ursprung).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MapEvent {
    /// Klick auf eine freie Stelle (kein Element getroffen).
    Click { x: f64, z: f64 },
    /// Element angeklickt bzw. zum Ziehen angefasst.
    Pick { index: usize },
    /// Element wird gezogen: neuer Mittelpunkt.
    Drag { index: usize, x: f64, z: f64 },
    /// Ziehen beendet; `start_*` ist die Lage vor dem Ziehen.
    Drop {
        index: usize,
        start_x: f64,
        start_z: f64,
    },
    /// Element wird am Drehgriff um die eigene Achse gedreht: neuer Winkel (0 ≤ Grad < 360).
    Rotate { index: usize, deg: f64 },
    /// Drehen beendet; `start_deg` ist der Winkel vor dem Drehen.
    Rotated { index: usize, start_deg: f64 },
    /// Eckpunkt der Rasenfläche wird gezogen: neue Lage.
    Corner { index: usize, x: f64, z: f64 },
}

/// Ein Element, wie es die 3D-Szene braucht.
#[derive(Serialize)]
struct SceneElement {
    /// Art als Schlüssel; die Farbe kommt aus `--gelaende-<kind>` in `styles/app.css`.
    kind: &'static str,
    x: f64,
    z: f64,
    rotation_deg: f64,
    width_m: f64,
    depth_m: f64,
    height_m: f64,
    /// Steht nicht ganz auf der Rasenfläche → Modell halbdurchsichtig.
    outside: bool,
    /// Ausgewählt → hervorgehoben.
    selected: bool,
}

/// Beschriftung auf der Karte (z. B. Länge eines Messabschnitts).
#[derive(Serialize)]
struct SceneLabel {
    x: f64,
    z: f64,
    text: String,
    /// Hervorgehoben (Gesamtwert) oder normal (Abschnitt).
    main: bool,
}

/// Eine Messung, wie sie die Karte zeichnet.
#[derive(Serialize)]
struct SceneMeasure {
    points: Vec<[f64; 2]>,
    /// Fläche: Linienzug geschlossen und gefüllt.
    closed: bool,
    /// Messung, an der gerade gemessen wird (kräftiger gezeichnet, mit Abschnittslängen).
    active: bool,
    labels: Vec<SceneLabel>,
}

impl SceneMeasure {
    fn new(index: usize, m: &Measure, active: bool) -> Option<Self> {
        let points = &m.points;
        let last = *points.last()?;
        let result = m.result();
        let closed = m.kind == MeasureKind::Area && points.len() >= 3;
        // Abschnittslängen in der Mitte jedes Abschnitts – nur bei der aktiven Messung (übersichtlich)
        let mut labels: Vec<SceneLabel> = if active {
            result
                .segments
                .iter()
                .enumerate()
                .map(|(i, len)| {
                    let (a, b) = (points[i], points[(i + 1) % points.len()]);
                    SceneLabel {
                        x: (a.x + b.x) / 2.0,
                        z: (a.z + b.z) / 2.0,
                        text: format_distance(*len),
                        main: false,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };
        // Bezeichnung mit Ergebnis: Fläche im Schwerpunkt der Punkte, Strecke am letzten Punkt
        let (x, z) = if closed {
            let n = points.len() as f64;
            (
                points.iter().map(|p| p.x).sum::<f64>() / n,
                points.iter().map(|p| p.z).sum::<f64>() / n,
            )
        } else {
            (last.x, last.z)
        };
        let summary = m.summary();
        labels.push(SceneLabel {
            x,
            z,
            text: if summary == "–" {
                m.display_name(index)
            } else {
                format!("{}: {summary}", m.display_name(index))
            },
            main: true,
        });
        Some(Self {
            points: points.iter().map(|c| [c.x, c.z]).collect(),
            closed,
            active,
            labels,
        })
    }
}

/// Besucherweg (ANF-028), wie ihn die Karte zeichnet.
#[derive(Serialize)]
struct SceneRoute {
    points: Vec<[f64; 2]>,
    /// Personen pro Stunde – bestimmt die Linienbreite.
    flow: u32,
    /// Pfeile vom ersten zum letzten Punkt.
    one_way: bool,
    active: bool,
    /// Beschriftung „Name: … P/h“ und ihre Lage (Mitte des Wegs).
    label: String,
    label_at: [f64; 2],
}

/// Zugang (ANF-028).
#[derive(Serialize)]
struct SceneAccess {
    /// Art als Schlüssel; die Farbe kommt aus `--gelaende-<kind>`.
    kind: &'static str,
    short: &'static str,
    label: String,
    x: f64,
    z: f64,
}

/// Zone (ANF-028) mit Ampel.
#[derive(Serialize)]
struct SceneZone {
    points: Vec<[f64; 2]>,
    /// `offen`, `ok`, `knapp`, `voll`
    status: &'static str,
    active: bool,
    label: String,
    label_at: [f64; 2],
}

/// Mitte eines Linienzugs entlang seiner Länge.
fn path_middle(points: &[Corner]) -> [f64; 2] {
    let total: f64 = points
        .windows(2)
        .map(|w| (w[1].x - w[0].x).hypot(w[1].z - w[0].z))
        .sum();
    let mut rest = total / 2.0;
    for w in points.windows(2) {
        let len = (w[1].x - w[0].x).hypot(w[1].z - w[0].z);
        if len >= rest && len > 0.0 {
            let t = rest / len;
            return [
                w[0].x + (w[1].x - w[0].x) * t,
                w[0].z + (w[1].z - w[0].z) * t,
            ];
        }
        rest -= len;
    }
    points.first().map_or([0.0, 0.0], |p| [p.x, p.z])
}

/// Schwerpunkt der Punkte (für Beschriftungen von Flächen).
fn centroid(points: &[Corner]) -> [f64; 2] {
    let n = points.len().max(1) as f64;
    [
        points.iter().map(|p| p.x).sum::<f64>() / n,
        points.iter().map(|p| p.z).sum::<f64>() / n,
    ]
}

/// Inhalt der 3D-Szene.
#[derive(Serialize)]
struct Scene {
    /// Elemente lassen sich anfassen, ziehen und drehen (nicht beim Abstecken und Messen).
    drag_elements: bool,
    /// Eckpunkte der Rasenfläche lassen sich ziehen (nicht beim Messen).
    drag_corners: bool,
    elements: Vec<SceneElement>,
    corners: Vec<[f64; 2]>,
    measures: Vec<SceneMeasure>,
    routes: Vec<SceneRoute>,
    accesses: Vec<SceneAccess>,
    zones: Vec<SceneZone>,
}

/// Zustand der Bedienung, der nicht zum Plan gehört.
pub struct ViewState {
    /// Messung, an der gerade gemessen wird (Index in `plan.measures`).
    pub active_measure: Option<usize>,
    /// Besucherweg bzw. Zone, an dem gerade gezeichnet wird (ANF-028).
    pub active_route: Option<usize>,
    pub active_zone: Option<usize>,
    /// Ausgewähltes Element (Index in `plan.elements`).
    pub selected: Option<usize>,
    /// Elemente lassen sich anfassen, ziehen und drehen.
    pub drag_elements: bool,
    /// Eckpunkte der Rasenfläche lassen sich ziehen.
    pub drag_corners: bool,
}

impl Scene {
    fn from_plan(plan: &SitePlan, view: &ViewState) -> Self {
        Self {
            routes: plan
                .routes
                .iter()
                .enumerate()
                .filter(|(_, r)| !r.points.is_empty())
                .map(|(i, r)| SceneRoute {
                    points: r.points.iter().map(|c| [c.x, c.z]).collect(),
                    flow: r.persons_per_hour,
                    one_way: r.direction == RouteDirection::OneWay,
                    active: view.active_route == Some(i),
                    label: if r.persons_per_hour == 0 {
                        r.display_name(i)
                    } else {
                        format!("{}: {}", r.display_name(i), r.flow_text())
                    },
                    label_at: path_middle(&r.points),
                })
                .collect(),
            accesses: plan
                .accesses
                .iter()
                .enumerate()
                .map(|(i, a)| SceneAccess {
                    kind: a.kind.key(),
                    short: a.kind.short(),
                    label: a.display_name(i),
                    x: a.x,
                    z: a.z,
                })
                .collect(),
            zones: plan
                .zones
                .iter()
                .enumerate()
                .filter(|(_, z)| !z.points.is_empty())
                .map(|(i, z)| SceneZone {
                    points: z.points.iter().map(|c| [c.x, c.z]).collect(),
                    status: z.status().key(),
                    active: view.active_zone == Some(i),
                    label: match z.occupancy_text().as_str() {
                        "–" => z.display_name(i),
                        text => format!("{}: {text}", z.display_name(i)),
                    },
                    label_at: centroid(&z.points),
                })
                .collect(),
            drag_elements: view.drag_elements,
            drag_corners: view.drag_corners,
            measures: plan
                .measures
                .iter()
                .enumerate()
                .filter_map(|(i, m)| SceneMeasure::new(i, m, view.active_measure == Some(i)))
                .collect(),
            elements: plan
                .elements
                .iter()
                .enumerate()
                .map(|(i, e)| SceneElement {
                    selected: view.selected == Some(i),
                    kind: e.kind.key(),
                    x: e.x,
                    z: e.z,
                    rotation_deg: e.rotation_deg,
                    width_m: e.width_m,
                    depth_m: e.depth_m,
                    height_m: e.kind.height_m(),
                    outside: !plan.fits(e),
                })
                .collect(),
            corners: plan.corners.iter().map(|c| [c.x, c.z]).collect(),
        }
    }
}

#[cfg(feature = "hydrate")]
mod js {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(module = "/src/components/site_plans/map_bridge.js")]
    extern "C" {
        pub type SitePlanMap;

        #[wasm_bindgen(constructor)]
        pub fn new(
            container: &JsValue,
            origin_lng: f64,
            origin_lat: f64,
            on_event: &Closure<dyn FnMut(JsValue)>,
            on_error: &Closure<dyn FnMut(String)>,
        ) -> SitePlanMap;

        #[wasm_bindgen(method)]
        pub fn sync(this: &SitePlanMap, scene: JsValue);

        #[wasm_bindgen(method, js_name = setOrigin)]
        pub fn set_origin(this: &SitePlanMap, lng: f64, lat: f64);

        #[wasm_bindgen(method, js_name = centerLng)]
        pub fn center_lng(this: &SitePlanMap) -> f64;

        #[wasm_bindgen(method, js_name = centerLat)]
        pub fn center_lat(this: &SitePlanMap) -> f64;

        #[wasm_bindgen(method)]
        pub fn destroy(this: &SitePlanMap);

        /// Ortssuche (Promise mit `[{ label, lng, lat }]`); wirft bei Fehlern.
        #[wasm_bindgen(js_name = searchAddress, catch)]
        pub async fn search_address(query: &str) -> Result<JsValue, JsValue>;
    }

    /// Karte samt Rückruf-Funktionen; die Rückrufe leben so lange wie die Karte.
    pub struct Handle {
        pub map: SitePlanMap,
        pub _on_event: Closure<dyn FnMut(JsValue)>,
        pub _on_error: Closure<dyn FnMut(String)>,
    }
}

/// Verbindung zur 3D-Karte. Beim Verwerfen (`Drop`) wird die Karte entfernt.
pub struct MapView {
    #[cfg(feature = "hydrate")]
    handle: js::Handle,
}

impl MapView {
    /// Erstellt die Karte im Element `container` mit dem Ursprung (Längengrad, Breitengrad).
    /// `on_event` meldet Klicks und das Ziehen von Elementen, `on_error` Ladefehler.
    #[cfg(feature = "hydrate")]
    pub fn mount(
        container: &leptos::web_sys::HtmlElement,
        origin: (f64, f64),
        mut on_event: impl FnMut(MapEvent) + 'static,
        on_error: impl FnMut(String) + 'static,
    ) -> Self {
        use wasm_bindgen::prelude::*;
        let on_event = Closure::wrap(Box::new(move |value: JsValue| {
            if let Ok(event) = serde_wasm_bindgen::from_value::<MapEvent>(value) {
                on_event(event);
            }
        }) as Box<dyn FnMut(JsValue)>);
        let on_error = Closure::wrap(Box::new(on_error) as Box<dyn FnMut(String)>);
        let map =
            js::SitePlanMap::new(container.as_ref(), origin.0, origin.1, &on_event, &on_error);
        Self {
            handle: js::Handle {
                map,
                _on_event: on_event,
                _on_error: on_error,
            },
        }
    }

    /// Auf dem Server gibt es keine Karte.
    #[cfg(not(feature = "hydrate"))]
    pub fn mount(
        _container: &leptos::web_sys::HtmlElement,
        _origin: (f64, f64),
        _on_event: impl FnMut(MapEvent) + 'static,
        _on_error: impl FnMut(String) + 'static,
    ) -> Self {
        Self {}
    }

    /// Zeichnet Elemente, Rasenfläche, Auswahl und die laufende Messung neu.
    pub fn sync(&self, plan: &SitePlan, view: &ViewState) {
        let scene = Scene::from_plan(plan, view);
        #[cfg(feature = "hydrate")]
        if let Ok(value) = serde_wasm_bindgen::to_value(&scene) {
            self.handle.map.sync(value);
        }
        #[cfg(not(feature = "hydrate"))]
        let _ = scene;
    }

    /// Neuer Ursprung; die Karte fliegt dorthin.
    pub fn set_origin(&self, _lng: f64, _lat: f64) {
        #[cfg(feature = "hydrate")]
        self.handle.map.set_origin(_lng, _lat);
    }

    /// Aktuelle Kartenmitte (Längengrad, Breitengrad), sobald die Karte geladen ist.
    pub fn center(&self) -> Option<(f64, f64)> {
        #[cfg(feature = "hydrate")]
        {
            let (lng, lat) = (self.handle.map.center_lng(), self.handle.map.center_lat());
            (lng.is_finite() && lat.is_finite()).then_some((lng, lat))
        }
        #[cfg(not(feature = "hydrate"))]
        None
    }
}

impl Drop for MapView {
    fn drop(&mut self) {
        #[cfg(feature = "hydrate")]
        self.handle.map.destroy();
    }
}

/// Sucht Orte bzw. Adressen (höchstens fünf Treffer). Nur im Browser; der Dienst wird in
/// `map_bridge.js` aufgerufen.
pub async fn search_address(_query: String) -> Result<Vec<AddressHit>, String> {
    #[cfg(feature = "hydrate")]
    {
        let value = js::search_address(&_query).await.map_err(|e| {
            e.as_string()
                .or_else(|| {
                    js_sys::Reflect::get(&e, &"message".into())
                        .ok()
                        .and_then(|m| m.as_string())
                })
                .unwrap_or_else(|| "Die Adresssuche ist fehlgeschlagen.".into())
        })?;
        serde_wasm_bindgen::from_value(value)
            .map_err(|_| "Die Antwort der Adresssuche ist unverständlich.".to_string())
    }
    #[cfg(not(feature = "hydrate"))]
    Err("Die Adresssuche geht nur im Browser.".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_events_from_json() {
        let parse = |s: &str| serde_json::from_str::<MapEvent>(s).unwrap();
        assert_eq!(
            parse(r#"{"type":"click","x":1.5,"z":-2}"#),
            MapEvent::Click { x: 1.5, z: -2.0 }
        );
        assert_eq!(
            parse(r#"{"type":"pick","index":3}"#),
            MapEvent::Pick { index: 3 }
        );
        assert_eq!(
            parse(r#"{"type":"rotate","index":1,"deg":90}"#),
            MapEvent::Rotate {
                index: 1,
                deg: 90.0
            }
        );
        assert_eq!(
            parse(r#"{"type":"rotated","index":1,"start_deg":0}"#),
            MapEvent::Rotated {
                index: 1,
                start_deg: 0.0
            }
        );
        assert_eq!(
            parse(r#"{"type":"corner","index":2,"x":1,"z":2}"#),
            MapEvent::Corner {
                index: 2,
                x: 1.0,
                z: 2.0
            }
        );
        assert_eq!(
            parse(r#"{"type":"drop","index":0,"start_x":4,"start_z":5}"#),
            MapEvent::Drop {
                index: 0,
                start_x: 4.0,
                start_z: 5.0
            }
        );
    }
}
