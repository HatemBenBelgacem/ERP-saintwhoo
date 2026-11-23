//! Rust-Seite der Kartenbrücke (ANF-027): steuert `map_bridge.js` (MapLibre + Three.js).
//!
//! `MapView` gibt es in beiden Übersetzungen, damit die Ansicht ohne `cfg` auskommt; auf dem
//! Server hat sie keine Wirkung (Effekte laufen dort ohnehin nicht).

use serde::Serialize;

use crate::backend::site_plans::model::{
    AddressHit, Corner, MeasureKind, SitePlan, format_area, format_distance, measure,
};

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

/// Laufende Messung (wird nicht gespeichert).
#[derive(Serialize)]
struct SceneMeasure {
    points: Vec<[f64; 2]>,
    /// Fläche: Linienzug geschlossen und gefüllt.
    closed: bool,
    labels: Vec<SceneLabel>,
}

impl SceneMeasure {
    fn new(kind: MeasureKind, points: &[Corner]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }
        let result = measure(kind, points);
        let closed = kind == MeasureKind::Area && points.len() >= 3;
        // Abschnittslängen in der Mitte jedes Abschnitts
        let mut labels: Vec<SceneLabel> = result
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
            .collect();
        // Gesamtwert: Fläche im Schwerpunkt der Punkte, Strecke am letzten Punkt
        if let Some(area) = result.area_m2 {
            let n = points.len() as f64;
            labels.push(SceneLabel {
                x: points.iter().map(|p| p.x).sum::<f64>() / n,
                z: points.iter().map(|p| p.z).sum::<f64>() / n,
                text: format_area(area),
                main: true,
            });
        } else if kind == MeasureKind::Distance && points.len() >= 3 {
            let last = points[points.len() - 1];
            labels.push(SceneLabel {
                x: last.x,
                z: last.z,
                text: format!("Total {}", format_distance(result.length_m)),
                main: true,
            });
        }
        Some(Self {
            points: points.iter().map(|c| [c.x, c.z]).collect(),
            closed,
            labels,
        })
    }
}

/// Inhalt der 3D-Szene.
#[derive(Serialize)]
struct Scene {
    elements: Vec<SceneElement>,
    corners: Vec<[f64; 2]>,
    measure: Option<SceneMeasure>,
}

impl Scene {
    fn from_plan(plan: &SitePlan, measurement: Option<(MeasureKind, &[Corner])>) -> Self {
        Self {
            measure: measurement.and_then(|(kind, points)| SceneMeasure::new(kind, points)),
            elements: plan
                .elements
                .iter()
                .map(|e| SceneElement {
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
            on_click: &Closure<dyn FnMut(f64, f64)>,
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
        pub _on_click: Closure<dyn FnMut(f64, f64)>,
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
    /// `on_click(x, z)` meldet Klicks in Metern relativ zum Ursprung, `on_error` Ladefehler.
    #[cfg(feature = "hydrate")]
    pub fn mount(
        container: &leptos::web_sys::HtmlElement,
        origin: (f64, f64),
        on_click: impl FnMut(f64, f64) + 'static,
        on_error: impl FnMut(String) + 'static,
    ) -> Self {
        use wasm_bindgen::prelude::*;
        let on_click = Closure::wrap(Box::new(on_click) as Box<dyn FnMut(f64, f64)>);
        let on_error = Closure::wrap(Box::new(on_error) as Box<dyn FnMut(String)>);
        let map =
            js::SitePlanMap::new(container.as_ref(), origin.0, origin.1, &on_click, &on_error);
        Self {
            handle: js::Handle {
                map,
                _on_click: on_click,
                _on_error: on_error,
            },
        }
    }

    /// Auf dem Server gibt es keine Karte.
    #[cfg(not(feature = "hydrate"))]
    pub fn mount(
        _container: &leptos::web_sys::HtmlElement,
        _origin: (f64, f64),
        _on_click: impl FnMut(f64, f64) + 'static,
        _on_error: impl FnMut(String) + 'static,
    ) -> Self {
        Self {}
    }

    /// Zeichnet Elemente, Rasenfläche und die laufende Messung neu.
    pub fn sync(&self, plan: &SitePlan, measurement: Option<(MeasureKind, &[Corner])>) {
        let scene = Scene::from_plan(plan, measurement);
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
