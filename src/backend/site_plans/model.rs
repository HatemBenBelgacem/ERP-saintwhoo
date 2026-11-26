//! Datentypen und Prüfregeln des Geländeplans (ANF-027), gemeinsam für Server und Browser.
//!
//! Koordinaten stehen in Metern relativ zum Ursprung des Plans: `x` nach Osten, `z` nach Süden
//! (wie in der 3D-Szene von Three.js). Wird der Standort (Ursprung) geändert, wandert der ganze Plan mit.
//! Flächen und Lage werden mit dem Paket `geo` berechnet.

use geo::{Area, Contains, Coord, LineString, Point, Polygon, Rotate, Validation};
use serde::{Deserialize, Serialize};

use super::bottlenecks::{DEFAULT_FLOW_PER_METER, FLOW_PER_METER_RANGE};
use super::flows::{self, Access, VisitorRoute, Zone};
use crate::forms::FieldErrors;

/// Vorgabe-Ursprung eines neuen Plans: St. Gallen (Längengrad, Breitengrad).
pub const DEFAULT_ORIGIN: (f64, f64) = (9.3767, 47.4245);
/// Ausdehnung des Luftbilds SWISSIMAGE (Schweiz und Liechtenstein) als (West, Süd, Ost, Nord);
/// gleiche Werte wie `SWISS_BOUNDS` in `map_bridge.js`.
pub const SWISS_BOUNDS: (f64, f64, f64, f64) = (5.9, 45.8, 10.5, 47.9);

/// `true`, wenn für den Punkt (Längengrad, Breitengrad) ein Luftbild vorhanden ist.
pub fn has_aerial_image(lng: f64, lat: f64) -> bool {
    let (west, south, east, north) = SWISS_BOUNDS;
    (west..=east).contains(&lng) && (south..=north).contains(&lat)
}

/// Höchstzahl der Elemente bzw. Eckpunkte je Plan.
pub const MAX_ELEMENTS: usize = 500;
pub const MAX_CORNERS: usize = 100;
/// Höchstzahl der Messungen je Plan und der Punkte je Messung.
pub const MAX_MEASURES: usize = 50;
pub const MAX_MEASURE_POINTS: usize = 100;
/// Höchstlänge der Bezeichnung einer Messung.
pub const MAX_MEASURE_LABEL_LEN: usize = 100;
/// Grösste erlaubte Kantenlänge eines Elements in Metern.
pub const MAX_SIZE_M: f64 = 200.0;
/// Kleinste Kantenlänge (z. B. Fuss einer Gitterabsperrung) und erlaubte Höhe eines Elements in Metern.
pub const MIN_SIZE_M: f64 = 0.1;
pub const HEIGHT_RANGE_M: (f64, f64) = (0.1, 50.0);
/// Grösster erlaubter Abstand vom Ursprung in Metern (Plan bleibt ein Gelände, kein Land).
pub const MAX_DISTANCE_M: f64 = 5_000.0;
/// Höchstlänge der Standort-Adresse bzw. -Bezeichnung.
pub const MAX_ADDRESS_LEN: usize = 300;
/// Drehschritt der Schaltflächen ↺/↻ und der Taste R.
pub const ROTATION_STEP_DEG: f64 = 15.0;

/// Art eines platzierten Elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementKind {
    Stage,
    FoodStall,
    Tent,
    /// Gitterabsperrung (Absperrgitter); mit grösserer Breite eine ganze Absperrlinie.
    Barrier,
    /// Sanität: Sanitätsposten als weisse Kabine mit rotem Kreuz.
    Medical,
    /// Allgemeines Element für verschiedene Zwecke: quadratischer Körper mit Spitzdach.
    General,
}

impl ElementKind {
    pub const ALL: [ElementKind; 6] = [
        ElementKind::FoodStall,
        ElementKind::Stage,
        ElementKind::Tent,
        ElementKind::Barrier,
        ElementKind::Medical,
        ElementKind::General,
    ];

    /// Wert in der Datenbank (Spalte `art`).
    pub fn key(self) -> &'static str {
        match self {
            ElementKind::Stage => "buehne",
            ElementKind::FoodStall => "verpflegungsstand",
            ElementKind::Tent => "festzelt",
            ElementKind::Barrier => "gitterabsperrung",
            ElementKind::Medical => "sanitaet",
            ElementKind::General => "allgemein",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            ElementKind::Stage => "Bühne",
            ElementKind::FoodStall => "Verpflegungsstand",
            ElementKind::Tent => "Festzelt",
            ElementKind::Barrier => "Gitterabsperrung",
            ElementKind::Medical => "Sanität",
            ElementKind::General => "Allgemeines Element",
        }
    }

    /// Vorgabe-Grösse (Breite, Tiefe) in Metern.
    pub fn default_size(self) -> (f64, f64) {
        match self {
            ElementKind::Stage => (10.0, 8.0),
            ElementKind::FoodStall => (4.0, 2.5),
            ElementKind::Tent => (6.0, 6.0),
            // ein Gitterelement: 2,5 m lang, Füsse 0,6 m tief
            ElementKind::Barrier => (2.5, 0.6),
            // Kabine etwa so gross wie ein Container
            ElementKind::Medical => (6.0, 2.5),
            // quadratisch, wie das Haus-Zeichen
            ElementKind::General => (4.0, 4.0),
        }
    }

    /// Vorgabe der Gesamthöhe des 3D-Modells in Metern (Bühne bis Dach, Zelt bis First).
    pub fn height_m(self) -> f64 {
        match self {
            ElementKind::Stage => 4.5,
            ElementKind::FoodStall => 2.8,
            ElementKind::Tent => 3.0,
            ElementKind::Barrier => 1.1,
            ElementKind::Medical => 2.8,
            ElementKind::General => 4.0,
        }
    }

    /// Beschriftung der Werkzeug-Schaltfläche, z. B. „Bühne (10 × 8 m)“.
    pub fn tool_label(self) -> String {
        let (w, d) = self.default_size();
        format!("{} ({} × {} m)", self.label(), format_m(w), format_m(d))
    }

    /// Farbmarke vor dem Eintrag in der Liste (Farbwerte in `styles/app.css`).
    pub fn swatch_class(self) -> &'static str {
        match self {
            ElementKind::Stage => "inline-block size-3 shrink-0 rounded-sm bg-site-stage",
            ElementKind::FoodStall => "inline-block size-3 shrink-0 rounded-sm bg-site-food-stall",
            ElementKind::Tent => "inline-block size-3 shrink-0 rounded-sm bg-site-tent",
            ElementKind::Barrier => "inline-block size-3 shrink-0 rounded-sm bg-site-barrier",
            ElementKind::Medical => "inline-block size-3 shrink-0 rounded-sm bg-site-medical",
            ElementKind::General => "inline-block size-3 shrink-0 rounded-sm bg-site-general",
        }
    }
}

/// Zahl in Metern ohne überflüssige Nachkommastellen, mit Dezimalkomma („2,5“, „10“).
pub fn format_m(v: f64) -> String {
    // -0,0 als 0 anzeigen
    let v = if v.abs() < 0.05 { 0.0 } else { v };
    let s = format!("{v:.1}");
    s.trim_end_matches(".0").replace('.', ",")
}

/// Ein platziertes Element: Mittelpunkt, Drehung und Grundfläche in Metern.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteElement {
    pub kind: ElementKind,
    pub x: f64,
    pub z: f64,
    pub rotation_deg: f64,
    pub width_m: f64,
    pub depth_m: f64,
    /// Eigene Höhe in Metern; `None` = Vorgabe der Art.
    #[serde(default)]
    pub height_m: Option<f64>,
}

impl SiteElement {
    /// Höhe des 3D-Modells: eigene Höhe oder Vorgabe der Art.
    pub fn height(&self) -> f64 {
        self.height_m.unwrap_or(self.kind.height_m())
    }

    /// Masse als Text, z. B. „4 × 2,5 × 2,8 m“ (Breite × Tiefe × Höhe).
    pub fn size_text(&self) -> String {
        format!(
            "{} × {} × {} m",
            format_m(self.width_m),
            format_m(self.depth_m),
            format_m(self.height())
        )
    }

    /// Neues Element mit Vorgabe-Grösse an der Stelle (x, z).
    pub fn new(kind: ElementKind, x: f64, z: f64) -> Self {
        let (width_m, depth_m) = kind.default_size();
        Self {
            kind,
            x,
            z,
            rotation_deg: 0.0,
            width_m,
            depth_m,
            height_m: None,
        }
    }

    /// Dasselbe Element um `step` Grad weitergedreht (0 ≤ Winkel < 360).
    pub fn rotated(&self, step: f64) -> Self {
        Self {
            rotation_deg: (self.rotation_deg + step).rem_euclid(360.0),
            ..self.clone()
        }
    }

    /// Grundfläche als Polygon. Die Drehung entspricht der in Three.js (`rotation.y`):
    /// in der Draufsicht mit x nach rechts und z nach unten gegen den Uhrzeigersinn.
    pub fn footprint(&self) -> Polygon<f64> {
        let (hw, hd) = (self.width_m / 2.0, self.depth_m / 2.0);
        let rect = Polygon::new(
            LineString::from(vec![
                (self.x - hw, self.z - hd),
                (self.x + hw, self.z - hd),
                (self.x + hw, self.z + hd),
                (self.x - hw, self.z + hd),
            ]),
            vec![],
        );
        rect.rotate_around_point(-self.rotation_deg, Point::new(self.x, self.z))
    }
}

/// Ein Eckpunkt der Rasenfläche.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Corner {
    pub x: f64,
    pub z: f64,
}

/// Inhalt eines Geländeplans (wird als Ganzes gespeichert).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SitePlan {
    /// Ursprung als (Längengrad, Breitengrad).
    pub origin_lng: f64,
    pub origin_lat: f64,
    pub elements: Vec<SiteElement>,
    /// Eckpunkte der Rasenfläche; leer = keine Fläche abgesteckt.
    pub corners: Vec<Corner>,
    /// Adresse oder Bezeichnung des Standorts (z. B. aus der Adresssuche); `None` = keine hinterlegt.
    #[serde(default)]
    pub address: Option<String>,
    /// Beschriftete Messungen (Strecken und Flächen).
    #[serde(default)]
    pub measures: Vec<Measure>,
    /// Besucherströme (ANF-028): Wege, Zugänge, Zonen.
    #[serde(default)]
    pub routes: Vec<VisitorRoute>,
    #[serde(default)]
    pub accesses: Vec<Access>,
    #[serde(default)]
    pub zones: Vec<Zone>,
    /// Planungswert für Engstellen (ANF-028 Stufe 3): Personen pro Stunde je Meter Wegbreite.
    #[serde(default = "default_flow_per_meter")]
    pub flow_per_meter: f64,
}

fn default_flow_per_meter() -> f64 {
    DEFAULT_FLOW_PER_METER
}

impl Default for SitePlan {
    fn default() -> Self {
        Self {
            origin_lng: DEFAULT_ORIGIN.0,
            origin_lat: DEFAULT_ORIGIN.1,
            elements: Vec::new(),
            corners: Vec::new(),
            address: None,
            measures: Vec::new(),
            routes: Vec::new(),
            accesses: Vec::new(),
            zones: Vec::new(),
            flow_per_meter: DEFAULT_FLOW_PER_METER,
        }
    }
}

/// Geländeplan-Seite: Projekt (für Kopf und Pfadleiste) und gespeicherter Plan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SitePlanPage {
    pub projekt_id: i64,
    pub projekt_nummer: String,
    pub projekt_name: String,
    /// `None`, solange für das Projekt noch kein Plan gespeichert ist.
    pub plan: Option<SitePlan>,
    /// Postanschrift des Kunden des Projekts – Vorschlag für die Adresssuche.
    pub kunde_adresse: Option<String>,
    pub geaendert_am: Option<String>,
}

impl SitePlan {
    /// Rasenfläche als Polygon, sobald sie mindestens drei Eckpunkte hat.
    pub fn area_polygon(&self) -> Option<Polygon<f64>> {
        area_polygon(&self.corners)
    }

    /// Grösse der Rasenfläche in Quadratmetern (0 ohne Fläche).
    pub fn area_m2(&self) -> f64 {
        self.area_polygon().map_or(0.0, |p| p.unsigned_area())
    }

    /// `true`, wenn sich die Umrisslinie der Rasenfläche selbst überschneidet.
    pub fn area_crossed(&self) -> bool {
        self.area_polygon().is_some_and(|p| !p.is_valid())
    }

    /// `true`, wenn das Element ganz auf der Rasenfläche steht – oder keine Fläche abgesteckt ist.
    pub fn fits(&self, element: &SiteElement) -> bool {
        self.area_polygon()
            .is_none_or(|area| area.contains(&element.footprint()))
    }

    /// Prüfregeln (auf dem Server immer ausgeführt). Schlüssel der Meldungen:
    /// `ursprung`, `elemente`, `flaeche`, `messungen`, `besucherstroeme`, `adresse`.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        flows::validate(&self.routes, &self.accesses, &self.zones, &mut errors);
        let (min_flow, max_flow) = FLOW_PER_METER_RANGE;
        if !(self.flow_per_meter.is_finite()
            && (min_flow..=max_flow).contains(&self.flow_per_meter))
        {
            errors.insert(
                "besucherstroeme".into(),
                format!(
                    "Der Durchfluss je Meter Wegbreite muss zwischen {} und {} Personen pro Stunde liegen.",
                    flows::format_count(min_flow as u32),
                    flows::format_count(max_flow as u32)
                ),
            );
        }
        if self
            .address
            .as_ref()
            .is_some_and(|a| a.chars().count() > MAX_ADDRESS_LEN)
        {
            errors.insert(
                "adresse".into(),
                format!(
                    "Die Adresse des Standorts darf höchstens {MAX_ADDRESS_LEN} Zeichen lang sein."
                ),
            );
        }
        if !(self.origin_lng.is_finite()
            && self.origin_lat.is_finite()
            && (-180.0..=180.0).contains(&self.origin_lng)
            && (-85.0..=85.0).contains(&self.origin_lat))
        {
            errors.insert(
                "ursprung".into(),
                "Der Standort braucht einen Längengrad zwischen -180 und 180 und einen Breitengrad zwischen -85 und 85."
                    .into(),
            );
        }

        if self.corners.len() > MAX_CORNERS {
            errors.insert(
                "flaeche".into(),
                format!("Die Rasenfläche darf höchstens {MAX_CORNERS} Eckpunkte haben."),
            );
        } else if self
            .corners
            .iter()
            .any(|c| !in_range(c.x) || !in_range(c.z))
        {
            errors.insert(
                "flaeche".into(),
                format!(
                    "Eckpunkte der Rasenfläche dürfen höchstens {} km vom Standort entfernt sein.",
                    MAX_DISTANCE_M / 1000.0
                ),
            );
        } else if !self.corners.is_empty() {
            match self.area_polygon() {
                None => {
                    errors.insert(
                        "flaeche".into(),
                        "Die Rasenfläche braucht mindestens drei Eckpunkte.".into(),
                    );
                }
                Some(p) if !p.is_valid() => {
                    errors.insert(
                        "flaeche".into(),
                        "Die Umrisslinie der Rasenfläche überschneidet sich. Eckpunkte in der Reihenfolge rund um die Fläche setzen."
                            .into(),
                    );
                }
                Some(p) if p.unsigned_area() < 1.0 => {
                    errors.insert(
                        "flaeche".into(),
                        "Die Rasenfläche muss mindestens 1 m² gross sein.".into(),
                    );
                }
                Some(_) => {}
            }
        }

        if self.measures.len() > MAX_MEASURES {
            errors.insert(
                "messungen".into(),
                format!("Ein Plan darf höchstens {MAX_MEASURES} Messungen enthalten."),
            );
        } else {
            for (i, m) in self.measures.iter().enumerate() {
                let name = m.display_name(i);
                let message = if m.label.chars().count() > MAX_MEASURE_LABEL_LEN {
                    Some(format!(
                        "Messung {}: Die Bezeichnung darf höchstens {MAX_MEASURE_LABEL_LEN} Zeichen lang sein.",
                        i + 1
                    ))
                } else if m.points.len() > MAX_MEASURE_POINTS {
                    Some(format!(
                        "„{name}“ darf höchstens {MAX_MEASURE_POINTS} Punkte haben."
                    ))
                } else if m.points.len() < m.kind.min_points() {
                    Some(format!(
                        "„{name}“ braucht mindestens {} Punkte. Punkte ergänzen oder die Messung löschen.",
                        m.kind.min_points()
                    ))
                } else if m.points.iter().any(|c| !in_range(c.x) || !in_range(c.z)) {
                    Some(format!(
                        "„{name}“: Punkte dürfen höchstens {} km vom Standort entfernt sein.",
                        MAX_DISTANCE_M / 1000.0
                    ))
                } else {
                    None
                };
                if let Some(message) = message {
                    errors.insert("messungen".into(), message);
                    break;
                }
            }
        }

        if self.elements.len() > MAX_ELEMENTS {
            errors.insert(
                "elemente".into(),
                format!("Ein Plan darf höchstens {MAX_ELEMENTS} Elemente enthalten."),
            );
            return errors;
        }
        for (i, e) in self.elements.iter().enumerate() {
            let name = format!("{} {}", e.kind.label(), i + 1);
            let message = if !in_range(e.x) || !in_range(e.z) {
                Some(format!(
                    "{name} darf höchstens {} km vom Standort entfernt sein.",
                    MAX_DISTANCE_M / 1000.0
                ))
            } else if ![e.width_m, e.depth_m]
                .iter()
                .all(|v| v.is_finite() && (MIN_SIZE_M..=MAX_SIZE_M).contains(v))
            {
                Some(format!(
                    "{name}: Breite und Tiefe müssen zwischen {} und {} m liegen.",
                    format_m(MIN_SIZE_M),
                    format_m(MAX_SIZE_M)
                ))
            } else if e.height_m.is_some_and(|h| {
                !(h.is_finite() && (HEIGHT_RANGE_M.0..=HEIGHT_RANGE_M.1).contains(&h))
            }) {
                Some(format!(
                    "{name}: Die Höhe muss zwischen {} und {} m liegen.",
                    format_m(HEIGHT_RANGE_M.0),
                    format_m(HEIGHT_RANGE_M.1)
                ))
            } else if !(e.rotation_deg.is_finite() && (0.0..360.0).contains(&e.rotation_deg)) {
                Some(format!(
                    "{name}: Die Drehung muss zwischen 0° und 360° liegen."
                ))
            } else if !errors.contains_key("flaeche") && !self.fits(e) {
                Some(format!("{name} steht nicht ganz auf der Rasenfläche."))
            } else {
                None
            };
            if let Some(m) = message {
                errors.insert("elemente".into(), m);
                break;
            }
        }
        errors
    }
}

pub(super) fn in_range(v: f64) -> bool {
    v.is_finite() && v.abs() <= MAX_DISTANCE_M
}

/// Polygon aus den Eckpunkten (mindestens drei, sonst `None`).
pub fn area_polygon(corners: &[Corner]) -> Option<Polygon<f64>> {
    (corners.len() >= 3).then(|| {
        Polygon::new(
            LineString::from(
                corners
                    .iter()
                    .map(|c| Coord { x: c.x, y: c.z })
                    .collect::<Vec<_>>(),
            ),
            vec![],
        )
    })
}

/// Quadratmeter mit Tausendertrennzeichen, z. B. „1’250 m²“.
pub fn format_area(m2: f64) -> String {
    let n = m2.round() as i64;
    let digits = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('’');
        }
        out.push(ch);
    }
    format!("{}{out} m²", if n < 0 { "-" } else { "" })
}

/// Eingabe des Standorts im Modal-Fenster (Text, damit auch Dezimalkommas gehen).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct OriginInput {
    pub laenge: String,
    pub breite: String,
    /// Adresse oder Bezeichnung; leer = keine.
    pub adresse: String,
}

impl OriginInput {
    /// Prüft die Eingabe und liefert (Längengrad, Breitengrad, Adresse) oder Fehler je Feld.
    pub fn parse(&self) -> Result<(f64, f64, Option<String>), FieldErrors> {
        let mut errors = FieldErrors::new();
        let num = |s: &str| s.trim().replace(',', ".").parse::<f64>().ok();
        let lng = num(&self.laenge).filter(|v| (-180.0..=180.0).contains(v));
        let lat = num(&self.breite).filter(|v| (-85.0..=85.0).contains(v));
        if lng.is_none() {
            errors.insert(
                "laenge".into(),
                "Bitte einen Längengrad zwischen -180 und 180 eingeben.".into(),
            );
        }
        if lat.is_none() {
            errors.insert(
                "breite".into(),
                "Bitte einen Breitengrad zwischen -85 und 85 eingeben.".into(),
            );
        }
        let address = self.adresse.trim();
        if address.chars().count() > MAX_ADDRESS_LEN {
            errors.insert(
                "adresse".into(),
                format!("Höchstens {MAX_ADDRESS_LEN} Zeichen."),
            );
        }
        match (lng, lat) {
            (Some(lng), Some(lat)) if errors.is_empty() => {
                Ok((lng, lat, (!address.is_empty()).then(|| address.to_string())))
            }
            _ => Err(errors),
        }
    }
}

/// Treffer der Adresssuche (Ortssuche im Browser, siehe `map_bridge.js`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AddressHit {
    pub label: String,
    pub lng: f64,
    pub lat: f64,
}

/// Art einer Messung: Strecke (Linienzug) oder Fläche (Vieleck).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasureKind {
    Distance,
    Area,
}

impl MeasureKind {
    /// Wert in der Datenbank (Spalte `art`).
    pub fn key(self) -> &'static str {
        match self {
            MeasureKind::Distance => "strecke",
            MeasureKind::Area => "flaeche",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        [MeasureKind::Distance, MeasureKind::Area]
            .into_iter()
            .find(|k| k.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            MeasureKind::Distance => "Strecke",
            MeasureKind::Area => "Fläche",
        }
    }

    /// Mindestzahl der Punkte, damit die Messung ein Ergebnis hat.
    pub fn min_points(self) -> usize {
        match self {
            MeasureKind::Distance => 2,
            MeasureKind::Area => 3,
        }
    }
}

/// Eine beschriftete Messung im Plan.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Measure {
    pub kind: MeasureKind,
    /// Bezeichnung, z. B. „Zufahrt Feuerwehr“; leer = „Messung <Nummer>“.
    pub label: String,
    pub points: Vec<Corner>,
}

impl Measure {
    /// Angezeigter Name: Bezeichnung oder „Messung <Nummer>“ (Nummer ab 1).
    pub fn display_name(&self, index: usize) -> String {
        let label = self.label.trim();
        if label.is_empty() {
            format!("Messung {}", index + 1)
        } else {
            label.to_string()
        }
    }

    /// Ergebnis der Messung.
    pub fn result(&self) -> Measurement {
        measure(self.kind, &self.points)
    }

    /// Hauptwert als Text: Gesamtstrecke bzw. Fläche („–“, solange Punkte fehlen oder der Umriss
    /// sich überschneidet).
    pub fn summary(&self) -> String {
        let m = self.result();
        match self.kind {
            _ if self.points.len() < self.kind.min_points() => "–".into(),
            MeasureKind::Distance => format_distance(m.length_m),
            MeasureKind::Area => m.area_m2.map_or_else(|| "–".into(), format_area),
        }
    }
}

/// Längen der einzelnen Abschnitte eines Linienzugs in Metern.
pub fn segment_lengths(points: &[Corner]) -> Vec<f64> {
    points
        .windows(2)
        .map(|w| (w[1].x - w[0].x).hypot(w[1].z - w[0].z))
        .collect()
}

/// Ergebnis einer Messung: Gesamtlänge (Strecke) bzw. Umfang und Fläche (Fläche).
#[derive(Clone, Debug, PartialEq)]
pub struct Measurement {
    /// Abschnitte (bei der Fläche einschliesslich des schliessenden Abschnitts).
    pub segments: Vec<f64>,
    /// Gesamtlänge bzw. Umfang in Metern.
    pub length_m: f64,
    /// Fläche in m², nur beim Flächenmass ab drei Punkten und ohne Überschneidung.
    pub area_m2: Option<f64>,
    /// Die Umrisslinie überschneidet sich selbst – die Fläche ist dann nicht aussagekräftig.
    pub self_intersecting: bool,
}

/// Misst die Punkte als Strecke oder als geschlossene Fläche (mit `geo`).
pub fn measure(kind: MeasureKind, points: &[Corner]) -> Measurement {
    let mut segments = segment_lengths(points);
    let mut area_m2 = None;
    let mut self_intersecting = false;
    if kind == MeasureKind::Area && points.len() >= 3 {
        let (first, last) = (points[0], points[points.len() - 1]);
        segments.push((first.x - last.x).hypot(first.z - last.z));
        if let Some(p) = area_polygon(points) {
            self_intersecting = !p.is_valid();
            // Bei überschneidendem Umriss ist die Fläche nicht aussagekräftig
            area_m2 = (!self_intersecting).then(|| p.unsigned_area());
        }
    }
    Measurement {
        length_m: segments.iter().sum(),
        segments,
        area_m2,
        self_intersecting,
    }
}

/// Strecke lesbar: bis 1 km in Metern mit einer Nachkommastelle, darüber in Kilometern.
pub fn format_distance(m: f64) -> String {
    if m < 1000.0 {
        format!("{} m", format!("{m:.1}").replace('.', ","))
    } else {
        format!("{} km", format!("{:.2}", m / 1000.0).replace('.', ","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(size: f64) -> Vec<Corner> {
        vec![
            Corner { x: 0.0, z: 0.0 },
            Corner { x: size, z: 0.0 },
            Corner { x: size, z: size },
            Corner { x: 0.0, z: size },
        ]
    }

    #[test]
    fn area_of_square() {
        let plan = SitePlan {
            corners: square(20.0),
            ..SitePlan::default()
        };
        assert!((plan.area_m2() - 400.0).abs() < 1e-9);
        assert_eq!(format_area(plan.area_m2()), "400 m²");
        assert_eq!(format_area(1250.4), "1’250 m²");
        assert_eq!(format_area(1_234_567.0), "1’234’567 m²");
    }

    #[test]
    fn no_area_with_less_than_three_corners() {
        let plan = SitePlan {
            corners: square(20.0)[..2].to_vec(),
            ..SitePlan::default()
        };
        assert_eq!(plan.area_m2(), 0.0);
        // ohne Fläche passt jedes Element
        assert!(plan.fits(&SiteElement::new(ElementKind::Stage, 500.0, 500.0)));
        assert!(plan.validate().contains_key("flaeche"));
    }

    #[test]
    fn element_must_be_inside_area() {
        let plan = SitePlan {
            corners: square(20.0),
            ..SitePlan::default()
        };
        assert!(plan.fits(&SiteElement::new(ElementKind::FoodStall, 10.0, 10.0)));
        // Bühne 10 × 8 m mit Mittelpunkt bei x = 17 ragt über den Rand
        assert!(!plan.fits(&SiteElement::new(ElementKind::Stage, 17.0, 10.0)));
        assert!(!plan.fits(&SiteElement::new(ElementKind::Tent, 30.0, 10.0)));
    }

    #[test]
    fn rotation_changes_footprint() {
        // 10 × 2 m Streifen in einem 12 × 4 m Feld: gerade passt er, um 90° gedreht nicht
        let plan = SitePlan {
            corners: vec![
                Corner { x: 0.0, z: 0.0 },
                Corner { x: 12.0, z: 0.0 },
                Corner { x: 12.0, z: 4.0 },
                Corner { x: 0.0, z: 4.0 },
            ],
            ..SitePlan::default()
        };
        let e = SiteElement {
            width_m: 10.0,
            depth_m: 2.0,
            ..SiteElement::new(ElementKind::FoodStall, 6.0, 2.0)
        };
        assert!(plan.fits(&e));
        assert!(!plan.fits(&e.rotated(90.0)));
        assert_eq!(e.rotated(45.0).rotated(315.0).rotation_deg, 0.0);
    }

    #[test]
    fn validate_rejects_bad_values() {
        let mut plan = SitePlan::default();
        assert!(plan.validate().is_empty());

        plan.origin_lat = 89.0;
        assert!(plan.validate().contains_key("ursprung"));
        plan.origin_lat = 47.0;

        plan.elements.push(SiteElement {
            width_m: 0.0,
            ..SiteElement::new(ElementKind::Tent, 0.0, 0.0)
        });
        assert!(plan.validate().contains_key("elemente"));
        plan.elements[0].width_m = 6.0;
        plan.elements[0].rotation_deg = 360.0;
        assert!(plan.validate().contains_key("elemente"));
        plan.elements[0].rotation_deg = 0.0;
        plan.elements[0].x = f64::NAN;
        assert!(plan.validate().contains_key("elemente"));
        plan.elements[0].x = 10.0;
        plan.elements[0].z = 10.0;
        assert!(plan.validate().is_empty());

        // Element ausserhalb der Fläche
        plan.corners = square(5.0);
        let errors = plan.validate();
        assert_eq!(
            errors.get("elemente").map(String::as_str),
            Some("Festzelt 1 steht nicht ganz auf der Rasenfläche.")
        );
    }

    #[test]
    fn origin_input_accepts_decimal_comma() {
        let ok = OriginInput {
            laenge: " 9,3767 ".into(),
            breite: "47.4245".into(),
            adresse: " Marktgasse 1, 9000 St. Gallen ".into(),
        };
        assert_eq!(
            ok.parse(),
            Ok((
                9.3767,
                47.4245,
                Some("Marktgasse 1, 9000 St. Gallen".into())
            ))
        );
        let empty_address = OriginInput {
            adresse: "  ".into(),
            ..ok.clone()
        };
        assert_eq!(empty_address.parse(), Ok((9.3767, 47.4245, None)));
        let bad = OriginInput {
            laenge: "abc".into(),
            breite: "91".into(),
            adresse: "x".repeat(MAX_ADDRESS_LEN + 1),
        };
        let errors = bad.parse().unwrap_err();
        assert!(errors.contains_key("laenge") && errors.contains_key("breite"));
        assert!(errors.contains_key("adresse"));
    }

    #[test]
    fn address_too_long_is_rejected() {
        let plan = SitePlan {
            address: Some("x".repeat(MAX_ADDRESS_LEN + 1)),
            ..SitePlan::default()
        };
        assert!(plan.validate().contains_key("adresse"));
    }

    #[test]
    fn measure_distance_and_area() {
        let pts = square(10.0);
        let d = measure(MeasureKind::Distance, &pts);
        assert_eq!(d.segments, vec![10.0, 10.0, 10.0]);
        assert_eq!(d.length_m, 30.0);
        assert_eq!(d.area_m2, None);

        let a = measure(MeasureKind::Area, &pts);
        assert_eq!(a.segments.len(), 4);
        assert_eq!(a.length_m, 40.0);
        assert_eq!(a.area_m2, Some(100.0));
        assert!(!a.self_intersecting);

        // „Fliege“: Reihenfolge über Kreuz → Umriss schneidet sich
        let bowtie = [
            Corner { x: 0.0, z: 0.0 },
            Corner { x: 10.0, z: 10.0 },
            Corner { x: 10.0, z: 0.0 },
            Corner { x: 0.0, z: 10.0 },
        ];
        let crossed = measure(MeasureKind::Area, &bowtie);
        assert!(crossed.self_intersecting);
        assert_eq!(crossed.area_m2, None);
        let plan = SitePlan {
            corners: bowtie.to_vec(),
            ..SitePlan::default()
        };
        assert!(plan.validate()["flaeche"].contains("überschneidet"));

        // Diagonale 3-4-5
        let diag = [Corner { x: 0.0, z: 0.0 }, Corner { x: 3.0, z: 4.0 }];
        assert_eq!(measure(MeasureKind::Area, &diag).length_m, 5.0);
        assert!(measure(MeasureKind::Distance, &[]).segments.is_empty());

        assert_eq!(format_distance(12.34), "12,3 m");
        assert_eq!(format_distance(1250.0), "1,25 km");
    }

    #[test]
    fn kind_keys_roundtrip() {
        for k in ElementKind::ALL {
            assert_eq!(ElementKind::from_key(k.key()), Some(k));
        }
        assert_eq!(
            ElementKind::FoodStall.tool_label(),
            "Verpflegungsstand (4 × 2,5 m)"
        );
        assert_eq!(ElementKind::Stage.tool_label(), "Bühne (10 × 8 m)");
        assert_eq!(ElementKind::Medical.tool_label(), "Sanität (6 × 2,5 m)");
        assert_eq!(
            ElementKind::General.tool_label(),
            "Allgemeines Element (4 × 4 m)"
        );
        // das allgemeine Element ist quadratisch
        let (w, d) = ElementKind::General.default_size();
        assert_eq!(w, d);
    }

    #[test]
    fn aerial_image_only_in_switzerland() {
        assert!(has_aerial_image(DEFAULT_ORIGIN.0, DEFAULT_ORIGIN.1)); // St. Gallen
        assert!(has_aerial_image(8.5417, 47.3769)); // Zürich
        assert!(has_aerial_image(9.5209, 47.1410)); // Vaduz
        assert!(!has_aerial_image(13.4050, 52.5200)); // Berlin
    }

    #[test]
    fn measures_are_named_and_validated() {
        let line = Measure {
            kind: MeasureKind::Distance,
            label: "  ".into(),
            points: vec![Corner { x: 0.0, z: 0.0 }, Corner { x: 30.0, z: 40.0 }],
        };
        assert_eq!(line.display_name(0), "Messung 1");
        assert_eq!(line.summary(), "50,0 m");
        let area = Measure {
            kind: MeasureKind::Area,
            label: "Parkplatz".into(),
            points: square(10.0),
        };
        assert_eq!(area.display_name(1), "Parkplatz");
        assert_eq!(area.summary(), "100 m²");

        let mut plan = SitePlan {
            measures: vec![line.clone(), area],
            ..SitePlan::default()
        };
        assert!(plan.validate().is_empty());

        // zu wenige Punkte
        plan.measures[0].points.truncate(1);
        assert!(plan.validate()["messungen"].contains("mindestens 2 Punkte"));
        assert_eq!(plan.measures[0].summary(), "–");
        plan.measures[0] = line;

        plan.measures[1].label = "x".repeat(MAX_MEASURE_LABEL_LEN + 1);
        assert!(plan.validate().contains_key("messungen"));

        for k in [MeasureKind::Distance, MeasureKind::Area] {
            assert_eq!(MeasureKind::from_key(k.key()), Some(k));
        }
    }

    #[test]
    fn barrier_and_custom_size() {
        let mut e = SiteElement::new(ElementKind::Barrier, 0.0, 0.0);
        assert_eq!(
            ElementKind::Barrier.tool_label(),
            "Gitterabsperrung (2,5 × 0,6 m)"
        );
        assert_eq!(e.height(), 1.1);
        assert_eq!(e.size_text(), "2,5 × 0,6 × 1,1 m");
        // eigene Masse: 20 m lange Absperrlinie, 1,2 m hoch
        e.width_m = 20.0;
        e.height_m = Some(1.2);
        assert_eq!(e.size_text(), "20 × 0,6 × 1,2 m");
        let mut plan = SitePlan {
            elements: vec![e],
            ..SitePlan::default()
        };
        assert!(plan.validate().is_empty());
        plan.elements[0].height_m = Some(80.0);
        assert!(plan.validate()["elemente"].contains("Höhe"));
        plan.elements[0].height_m = None;
        plan.elements[0].depth_m = 0.05;
        assert!(plan.validate()["elemente"].contains("Breite und Tiefe"));
    }
}
