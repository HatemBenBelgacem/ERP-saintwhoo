//! Rust-Seite der Kartenbrücke (ANF-027): steuert `map_bridge.js` (MapLibre + Three.js).
//!
//! `MapView` gibt es in beiden Übersetzungen, damit die Ansicht ohne `cfg` auskommt; auf dem
//! Server hat sie keine Wirkung (Effekte laufen dort ohnehin nicht).

use serde::Serialize;

use crate::backend::site_plans::model::SitePlan;

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

/// Inhalt der 3D-Szene.
#[derive(Serialize)]
struct Scene {
    elements: Vec<SceneElement>,
    corners: Vec<[f64; 2]>,
}

impl Scene {
    fn from_plan(plan: &SitePlan) -> Self {
        Self {
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

    /// Zeichnet Elemente und Rasenfläche des Plans neu.
    pub fn sync(&self, plan: &SitePlan) {
        let scene = Scene::from_plan(plan);
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
