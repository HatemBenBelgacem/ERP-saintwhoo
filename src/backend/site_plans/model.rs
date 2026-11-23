//! Datentypen und Prüfregeln des Geländeplans (ANF-027), gemeinsam für Server und Browser.
//!
//! Koordinaten stehen in Metern relativ zum Ursprung des Plans: `x` nach Osten, `z` nach Süden
//! (wie in der 3D-Szene von Three.js). Wird der Standort (Ursprung) geändert, wandert der ganze Plan mit.
//! Flächen und Lage werden mit dem Paket `geo` berechnet.

use geo::{Area, Contains, Coord, LineString, Point, Polygon, Rotate};
use serde::{Deserialize, Serialize};

use crate::forms::FieldErrors;

/// Vorgabe-Ursprung eines neuen Plans: St. Gallen (Längengrad, Breitengrad).
pub const DEFAULT_ORIGIN: (f64, f64) = (9.3767, 47.4245);
/// Höchstzahl der Elemente bzw. Eckpunkte je Plan.
pub const MAX_ELEMENTS: usize = 500;
pub const MAX_CORNERS: usize = 100;
/// Grösste erlaubte Kantenlänge eines Elements in Metern.
pub const MAX_SIZE_M: f64 = 200.0;
/// Grösster erlaubter Abstand vom Ursprung in Metern (Plan bleibt ein Gelände, kein Land).
pub const MAX_DISTANCE_M: f64 = 5_000.0;
/// Drehschritt der Schaltfläche „Drehen“.
pub const ROTATION_STEP_DEG: f64 = 45.0;

/// Art eines platzierten Elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementKind {
    Stage,
    FoodStall,
    Tent,
}

impl ElementKind {
    pub const ALL: [ElementKind; 3] = [
        ElementKind::FoodStall,
        ElementKind::Stage,
        ElementKind::Tent,
    ];

    /// Wert in der Datenbank (Spalte `art`).
    pub fn key(self) -> &'static str {
        match self {
            ElementKind::Stage => "buehne",
            ElementKind::FoodStall => "verpflegungsstand",
            ElementKind::Tent => "festzelt",
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
        }
    }

    /// Vorgabe-Grösse (Breite, Tiefe) in Metern.
    pub fn default_size(self) -> (f64, f64) {
        match self {
            ElementKind::Stage => (10.0, 8.0),
            ElementKind::FoodStall => (4.0, 2.5),
            ElementKind::Tent => (6.0, 6.0),
        }
    }

    /// Gesamthöhe des 3D-Modells in Metern (Bühne bis Dach, Zelt bis First).
    pub fn height_m(self) -> f64 {
        match self {
            ElementKind::Stage => 4.5,
            ElementKind::FoodStall => 2.8,
            ElementKind::Tent => 3.0,
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
        }
    }
}

/// Zahl in Metern ohne überflüssige Nachkommastellen, mit Dezimalkomma („2,5“, „10“).
pub fn format_m(v: f64) -> String {
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
}

impl SiteElement {
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
}

impl Default for SitePlan {
    fn default() -> Self {
        Self {
            origin_lng: DEFAULT_ORIGIN.0,
            origin_lat: DEFAULT_ORIGIN.1,
            elements: Vec::new(),
            corners: Vec::new(),
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

    /// `true`, wenn das Element ganz auf der Rasenfläche steht – oder keine Fläche abgesteckt ist.
    pub fn fits(&self, element: &SiteElement) -> bool {
        self.area_polygon()
            .is_none_or(|area| area.contains(&element.footprint()))
    }

    /// Prüfregeln (auf dem Server immer ausgeführt). Schlüssel der Meldungen:
    /// `ursprung`, `elemente`, `flaeche`.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
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
                Some(p) if p.unsigned_area() < 1.0 => {
                    errors.insert(
                        "flaeche".into(),
                        "Die Rasenfläche muss mindestens 1 m² gross sein.".into(),
                    );
                }
                Some(_) => {}
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
            } else if !(e.width_m.is_finite()
                && e.depth_m.is_finite()
                && e.width_m > 0.0
                && e.depth_m > 0.0
                && e.width_m <= MAX_SIZE_M
                && e.depth_m <= MAX_SIZE_M)
            {
                Some(format!(
                    "{name}: Breite und Tiefe müssen zwischen 0 und {} m liegen.",
                    format_m(MAX_SIZE_M)
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

fn in_range(v: f64) -> bool {
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
}

impl OriginInput {
    /// Prüft die Eingabe und liefert (Längengrad, Breitengrad) oder Fehler je Feld.
    pub fn parse(&self) -> Result<(f64, f64), FieldErrors> {
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
        match (lng, lat) {
            (Some(lng), Some(lat)) => Ok((lng, lat)),
            _ => Err(errors),
        }
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
        };
        assert_eq!(ok.parse(), Ok((9.3767, 47.4245)));
        let bad = OriginInput {
            laenge: "abc".into(),
            breite: "91".into(),
        };
        let errors = bad.parse().unwrap_err();
        assert!(errors.contains_key("laenge") && errors.contains_key("breite"));
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
    }
}
