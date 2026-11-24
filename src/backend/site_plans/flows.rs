//! Besucherströme im Geländeplan (ANF-028): Besucherwege, Zugänge und Zonen mit Fassungsvermögen.
//! Gemeinsam für Server und Browser; Koordinaten wie im Plan in Metern relativ zum Standort.

use geo::{Area, Validation};
use serde::{Deserialize, Serialize};

use super::model::{Corner, MAX_DISTANCE_M, area_polygon, format_m, in_range};
use crate::forms::FieldErrors;

/// Höchstzahl der Wege, Zugänge und Zonen je Plan sowie der Punkte je Weg bzw. Zone.
pub const MAX_FLOW_ITEMS: usize = 50;
pub const MAX_FLOW_POINTS: usize = 100;
/// Höchstlänge einer Bezeichnung.
pub const MAX_FLOW_LABEL_LEN: usize = 100;
/// Obergrenze für Personen pro Stunde bzw. erwartete Personen.
pub const MAX_PERSONS: u32 = 1_000_000;
/// Obergrenze für den Durchlass eines Zugangs (Personen pro Minute).
pub const MAX_PERSONS_PER_MINUTE: u32 = 100_000;
/// Erlaubte Wegbreite in Metern.
pub const ROUTE_WIDTH_M: (f64, f64) = (0.5, 100.0);
/// Erlaubte Personendichte (Personen pro m²) und Vorgabe für neue Zonen.
pub const DENSITY_RANGE: (f64, f64) = (0.1, 10.0);
pub const DEFAULT_DENSITY: f64 = 2.0;
/// Ampel der Zonen: bis hierher grün, darüber bis 100 % gelb, darüber rot.
pub const ZONE_TIGHT_FROM: f64 = 0.8;

/// Richtung eines Besucherwegs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteDirection {
    /// Nur vom ersten zum letzten Punkt.
    OneWay,
    Both,
}

impl RouteDirection {
    pub const ALL: [RouteDirection; 2] = [RouteDirection::OneWay, RouteDirection::Both];

    /// Wert in der Datenbank (Spalte `richtung`).
    pub fn key(self) -> &'static str {
        match self {
            RouteDirection::OneWay => "einbahn",
            RouteDirection::Both => "beide",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|d| d.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            RouteDirection::OneWay => "Einbahn",
            RouteDirection::Both => "Beide Richtungen",
        }
    }
}

/// Ein Besucherweg: Linienzug mit Richtung und erwarteter Menge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VisitorRoute {
    /// Bezeichnung; leer = „Weg <Nummer>“.
    pub label: String,
    pub direction: RouteDirection,
    /// Erwartete Personen pro Stunde (0 = unbekannt).
    pub persons_per_hour: u32,
    /// Wegbreite in Metern (optional).
    pub width_m: Option<f64>,
    pub points: Vec<Corner>,
}

impl VisitorRoute {
    pub fn new(point: Corner) -> Self {
        Self {
            label: String::new(),
            direction: RouteDirection::OneWay,
            persons_per_hour: 0,
            width_m: None,
            points: vec![point],
        }
    }

    pub fn display_name(&self, index: usize) -> String {
        name_or(&self.label, "Weg", index)
    }

    /// Länge des Wegs in Metern.
    pub fn length_m(&self) -> f64 {
        self.points
            .windows(2)
            .map(|w| (w[1].x - w[0].x).hypot(w[1].z - w[0].z))
            .sum()
    }

    /// Menge als Text, z. B. „1’200 P/h“; „–“ ohne Angabe.
    pub fn flow_text(&self) -> String {
        if self.persons_per_hour == 0 {
            "–".into()
        } else {
            format!("{} P/h", format_count(self.persons_per_hour))
        }
    }
}

/// Art eines Zugangs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessKind {
    Entrance,
    Exit,
    EmergencyExit,
}

impl AccessKind {
    pub const ALL: [AccessKind; 3] = [
        AccessKind::Entrance,
        AccessKind::Exit,
        AccessKind::EmergencyExit,
    ];

    /// Wert in der Datenbank (Spalte `art`); gleichzeitig Teil des Farbnamens `--gelaende-<Wert>`.
    pub fn key(self) -> &'static str {
        match self {
            AccessKind::Entrance => "eingang",
            AccessKind::Exit => "ausgang",
            AccessKind::EmergencyExit => "notausgang",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.key() == key)
    }

    pub fn label(self) -> &'static str {
        match self {
            AccessKind::Entrance => "Eingang",
            AccessKind::Exit => "Ausgang",
            AccessKind::EmergencyExit => "Notausgang",
        }
    }

    /// Kurzzeichen auf der Karte.
    pub fn short(self) -> &'static str {
        match self {
            AccessKind::Entrance => "E",
            AccessKind::Exit => "A",
            AccessKind::EmergencyExit => "N",
        }
    }

    /// Farbmarke in der Liste (Farbwerte in `styles/app.css`).
    pub fn swatch_class(self) -> &'static str {
        match self {
            AccessKind::Entrance => "inline-block size-3 shrink-0 rounded-full bg-site-entrance",
            AccessKind::Exit => "inline-block size-3 shrink-0 rounded-full bg-site-exit",
            AccessKind::EmergencyExit => {
                "inline-block size-3 shrink-0 rounded-full bg-site-emergency-exit"
            }
        }
    }
}

/// Ein Zugang (Eingang, Ausgang, Notausgang).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Access {
    pub kind: AccessKind,
    /// Bezeichnung; leer = Art mit Nummer, z. B. „Eingang 1“.
    pub label: String,
    pub x: f64,
    pub z: f64,
    /// Durchlass in Personen pro Minute (optional).
    pub persons_per_minute: Option<u32>,
}

impl Access {
    pub fn display_name(&self, index: usize) -> String {
        name_or(&self.label, self.kind.label(), index)
    }
}

/// Ampel einer Zone.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum ZoneStatus {
    /// Ohne erwartete Personen oder ohne gültige Fläche.
    Unknown,
    Ok,
    Tight,
    Over,
}

impl ZoneStatus {
    /// Schlüssel für die Karte (`map_bridge.js`).
    pub fn key(self) -> &'static str {
        match self {
            ZoneStatus::Unknown => "offen",
            ZoneStatus::Ok => "ok",
            ZoneStatus::Tight => "knapp",
            ZoneStatus::Over => "voll",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ZoneStatus::Unknown => "Offen",
            ZoneStatus::Ok => "Ausreichend",
            ZoneStatus::Tight => "Knapp",
            ZoneStatus::Over => "Überfüllt",
        }
    }

    /// Bezeichnung als Auslastung eines Wegs bzw. einer Engstelle.
    pub fn route_label(self) -> &'static str {
        match self {
            ZoneStatus::Unknown => "Offen",
            ZoneStatus::Ok => "Ausreichend",
            ZoneStatus::Tight => "Knapp",
            ZoneStatus::Over => "Zu eng",
        }
    }

    /// Plakette in der Liste (Signalfarben für den Status).
    pub fn badge_class(self) -> &'static str {
        match self {
            ZoneStatus::Unknown => "badge badge-ghost badge-sm",
            ZoneStatus::Ok => "badge badge-success badge-sm",
            ZoneStatus::Tight => "badge badge-warning badge-sm",
            ZoneStatus::Over => "badge badge-error badge-sm",
        }
    }
}

/// Eine Zone, in der sich Besucher aufhalten.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zone {
    /// Bezeichnung; leer = „Zone <Nummer>“.
    pub label: String,
    /// Erwartete Personen in der Zone (0 = unbekannt).
    pub expected_persons: u32,
    /// Personendichte in Personen pro m².
    pub density: f64,
    pub points: Vec<Corner>,
}

impl Zone {
    pub fn new(point: Corner) -> Self {
        Self {
            label: String::new(),
            expected_persons: 0,
            density: DEFAULT_DENSITY,
            points: vec![point],
        }
    }

    pub fn display_name(&self, index: usize) -> String {
        name_or(&self.label, "Zone", index)
    }

    /// `true`, wenn sich der Umriss selbst überschneidet.
    pub fn crossed(&self) -> bool {
        area_polygon(&self.points).is_some_and(|p| !p.is_valid())
    }

    /// Fläche in m² – nur bei mindestens drei Punkten und ohne Überschneidung.
    pub fn area_m2(&self) -> Option<f64> {
        area_polygon(&self.points)
            .filter(|p| p.is_valid())
            .map(|p| p.unsigned_area())
    }

    /// Fassungsvermögen: Fläche × Dichte, abgerundet.
    pub fn capacity(&self) -> Option<u32> {
        self.area_m2()
            .map(|a| (a * self.density).floor().clamp(0.0, u32::MAX as f64) as u32)
    }

    /// Auslastung (erwartete Personen / Fassungsvermögen), z. B. 0.75 = 75 %.
    pub fn utilization(&self) -> Option<f64> {
        match (self.expected_persons, self.capacity()) {
            (0, _) | (_, None) => None,
            (_, Some(0)) => Some(f64::INFINITY),
            (n, Some(c)) => Some(n as f64 / c as f64),
        }
    }

    pub fn status(&self) -> ZoneStatus {
        match self.utilization() {
            None => ZoneStatus::Unknown,
            Some(u) if u <= ZONE_TIGHT_FROM => ZoneStatus::Ok,
            Some(u) if u <= 1.0 => ZoneStatus::Tight,
            Some(_) => ZoneStatus::Over,
        }
    }

    /// Belegung als Text, z. B. „850 / 1’200 P (71 %)“; ohne erwartete Personen nur das Fassungsvermögen.
    pub fn occupancy_text(&self) -> String {
        match (self.capacity(), self.utilization()) {
            (None, _) => "–".into(),
            (Some(c), None) => format!("max. {} P", format_count(c)),
            (Some(c), Some(u)) => format!(
                "{} / {} P ({})",
                format_count(self.expected_persons),
                format_count(c),
                format_percent(u)
            ),
        }
    }
}

/// Bezeichnung oder Ersatzname „<Ersatz> <Nummer>“ (Nummer ab 1).
fn name_or(label: &str, fallback: &str, index: usize) -> String {
    let label = label.trim();
    if label.is_empty() {
        format!("{fallback} {}", index + 1)
    } else {
        label.to_string()
    }
}

/// Ganze Zahl mit Tausendertrennzeichen, z. B. „1’200“.
pub fn format_count(n: u32) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push('’');
        }
        out.push(ch);
    }
    out
}

/// Anteil als ganze Prozent, z. B. „71 %“.
pub fn format_percent(u: f64) -> String {
    if u.is_finite() {
        format!("{} %", (u * 100.0).round() as i64)
    } else {
        "∞ %".into()
    }
}

/// Zahl aus einer Eingabe (Dezimalkomma erlaubt); leer oder ungültig = `None`.
pub fn parse_decimal(s: &str) -> Option<f64> {
    s.trim()
        .replace(['’', '\''], "")
        .replace(',', ".")
        .parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
}

/// Ganze Zahl ≥ 0 aus einer Eingabe (Tausendertrennzeichen erlaubt); leer oder ungültig = `None`.
pub fn parse_count(s: &str) -> Option<u32> {
    s.trim().replace(['’', '\'', ' '], "").parse::<u32>().ok()
}

/// Prüfregeln der Besucherströme (Schlüssel der Meldung: `besucherstroeme`).
pub fn validate(
    routes: &[VisitorRoute],
    accesses: &[Access],
    zones: &[Zone],
    errors: &mut FieldErrors,
) {
    if let Some(message) = first_error(routes, accesses, zones) {
        errors.insert("besucherstroeme".into(), message);
    }
}

fn first_error(routes: &[VisitorRoute], accesses: &[Access], zones: &[Zone]) -> Option<String> {
    let km = MAX_DISTANCE_M / 1000.0;
    let points_ok = |pts: &[Corner]| pts.iter().all(|c| in_range(c.x) && in_range(c.z));
    let label_ok = |l: &str| l.chars().count() <= MAX_FLOW_LABEL_LEN;

    for (what, count) in [
        ("Besucherwege", routes.len()),
        ("Zugänge", accesses.len()),
        ("Zonen", zones.len()),
    ] {
        if count > MAX_FLOW_ITEMS {
            return Some(format!(
                "Ein Plan darf höchstens {MAX_FLOW_ITEMS} {what} enthalten."
            ));
        }
    }

    for (i, r) in routes.iter().enumerate() {
        let name = r.display_name(i);
        if !label_ok(&r.label) {
            return Some(format!(
                "Weg {}: Die Bezeichnung darf höchstens {MAX_FLOW_LABEL_LEN} Zeichen lang sein.",
                i + 1
            ));
        }
        if r.points.len() < 2 {
            return Some(format!(
                "„{name}“ braucht mindestens 2 Punkte. Punkte ergänzen oder den Weg löschen."
            ));
        }
        if r.points.len() > MAX_FLOW_POINTS {
            return Some(format!(
                "„{name}“ darf höchstens {MAX_FLOW_POINTS} Punkte haben."
            ));
        }
        if !points_ok(&r.points) {
            return Some(format!(
                "„{name}“: Punkte dürfen höchstens {km} km vom Standort entfernt sein."
            ));
        }
        if r.persons_per_hour > MAX_PERSONS {
            return Some(format!(
                "„{name}“: höchstens {} Personen pro Stunde.",
                format_count(MAX_PERSONS)
            ));
        }
        if let Some(w) = r.width_m
            && !(w.is_finite() && (ROUTE_WIDTH_M.0..=ROUTE_WIDTH_M.1).contains(&w))
        {
            return Some(format!(
                "„{name}“: Die Wegbreite muss zwischen {} und {} m liegen.",
                format_m(ROUTE_WIDTH_M.0),
                format_m(ROUTE_WIDTH_M.1)
            ));
        }
    }

    for (i, a) in accesses.iter().enumerate() {
        let name = a.display_name(i);
        if !label_ok(&a.label) {
            return Some(format!(
                "Zugang {}: Die Bezeichnung darf höchstens {MAX_FLOW_LABEL_LEN} Zeichen lang sein.",
                i + 1
            ));
        }
        if !(in_range(a.x) && in_range(a.z)) {
            return Some(format!(
                "„{name}“ darf höchstens {km} km vom Standort entfernt sein."
            ));
        }
        if a.persons_per_minute
            .is_some_and(|n| n > MAX_PERSONS_PER_MINUTE)
        {
            return Some(format!(
                "„{name}“: höchstens {} Personen pro Minute.",
                format_count(MAX_PERSONS_PER_MINUTE)
            ));
        }
    }

    for (i, z) in zones.iter().enumerate() {
        let name = z.display_name(i);
        if !label_ok(&z.label) {
            return Some(format!(
                "Zone {}: Die Bezeichnung darf höchstens {MAX_FLOW_LABEL_LEN} Zeichen lang sein.",
                i + 1
            ));
        }
        if z.points.len() < 3 {
            return Some(format!(
                "„{name}“ braucht mindestens 3 Eckpunkte. Punkte ergänzen oder die Zone löschen."
            ));
        }
        if z.points.len() > MAX_FLOW_POINTS {
            return Some(format!(
                "„{name}“ darf höchstens {MAX_FLOW_POINTS} Eckpunkte haben."
            ));
        }
        if !points_ok(&z.points) {
            return Some(format!(
                "„{name}“: Eckpunkte dürfen höchstens {km} km vom Standort entfernt sein."
            ));
        }
        if z.crossed() {
            return Some(format!(
                "„{name}“: Der Umriss überschneidet sich. Eckpunkte rund um die Zone setzen."
            ));
        }
        if z.expected_persons > MAX_PERSONS {
            return Some(format!(
                "„{name}“: höchstens {} erwartete Personen.",
                format_count(MAX_PERSONS)
            ));
        }
        if !(z.density.is_finite() && (DENSITY_RANGE.0..=DENSITY_RANGE.1).contains(&z.density)) {
            return Some(format!(
                "„{name}“: Die Personendichte muss zwischen {} und {} Personen pro m² liegen.",
                format_m(DENSITY_RANGE.0),
                format_m(DENSITY_RANGE.1)
            ));
        }
    }
    None
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

    fn zone(expected: u32) -> Zone {
        Zone {
            expected_persons: expected,
            points: square(10.0), // 100 m² × 2 P/m² = 200 P
            ..Zone::new(Corner { x: 0.0, z: 0.0 })
        }
    }

    #[test]
    fn zone_capacity_and_traffic_light() {
        assert_eq!(zone(0).capacity(), Some(200));
        assert_eq!(zone(0).status(), ZoneStatus::Unknown);
        assert_eq!(zone(0).occupancy_text(), "max. 200 P");
        assert_eq!(zone(160).status(), ZoneStatus::Ok); // 80 %
        assert_eq!(zone(161).status(), ZoneStatus::Tight);
        assert_eq!(zone(200).status(), ZoneStatus::Tight); // 100 %
        assert_eq!(zone(201).status(), ZoneStatus::Over);
        assert_eq!(zone(150).occupancy_text(), "150 / 200 P (75 %)");

        // unter drei Punkten bzw. mit Überschneidung: keine Fläche, keine Ampel
        let open = Zone {
            expected_persons: 10,
            ..Zone::new(Corner { x: 0.0, z: 0.0 })
        };
        assert_eq!(open.capacity(), None);
        assert_eq!(open.status(), ZoneStatus::Unknown);
        let bowtie = Zone {
            points: vec![
                Corner { x: 0.0, z: 0.0 },
                Corner { x: 10.0, z: 10.0 },
                Corner { x: 10.0, z: 0.0 },
                Corner { x: 0.0, z: 10.0 },
            ],
            ..zone(10)
        };
        assert!(bowtie.crossed());
        assert_eq!(bowtie.status(), ZoneStatus::Unknown);
    }

    #[test]
    fn route_length_and_names() {
        let mut r = VisitorRoute::new(Corner { x: 0.0, z: 0.0 });
        r.points.push(Corner { x: 30.0, z: 40.0 });
        assert_eq!(r.length_m(), 50.0);
        assert_eq!(r.display_name(0), "Weg 1");
        assert_eq!(r.flow_text(), "–");
        r.persons_per_hour = 1200;
        assert_eq!(r.flow_text(), "1’200 P/h");
        let a = Access {
            kind: AccessKind::EmergencyExit,
            label: String::new(),
            x: 0.0,
            z: 0.0,
            persons_per_minute: None,
        };
        assert_eq!(a.display_name(2), "Notausgang 3");
    }

    #[test]
    fn parse_inputs() {
        assert_eq!(parse_decimal(" 2,5 "), Some(2.5));
        assert_eq!(parse_decimal(""), None);
        assert_eq!(parse_count("1’200"), Some(1200));
        assert_eq!(parse_count("-3"), None);
        assert_eq!(format_count(1_234_567), "1’234’567");
        assert_eq!(format_percent(0.714), "71 %");
    }

    #[test]
    fn validation() {
        let mut route = VisitorRoute::new(Corner { x: 0.0, z: 0.0 });
        let mut errors = FieldErrors::new();
        validate(std::slice::from_ref(&route), &[], &[], &mut errors);
        assert!(errors["besucherstroeme"].contains("mindestens 2 Punkte"));

        route.points.push(Corner { x: 10.0, z: 0.0 });
        route.width_m = Some(0.1);
        errors.clear();
        validate(std::slice::from_ref(&route), &[], &[], &mut errors);
        assert!(errors["besucherstroeme"].contains("Wegbreite"));

        route.width_m = Some(3.0);
        let mut z = zone(100);
        z.density = 20.0;
        errors.clear();
        validate(
            std::slice::from_ref(&route),
            &[],
            std::slice::from_ref(&z),
            &mut errors,
        );
        assert!(errors["besucherstroeme"].contains("Personendichte"));

        z.density = 2.0;
        let access = Access {
            kind: AccessKind::Entrance,
            label: "Haupteingang".into(),
            x: 9_000.0,
            z: 0.0,
            persons_per_minute: Some(50),
        };
        errors.clear();
        validate(
            &[route.clone()],
            std::slice::from_ref(&access),
            &[z.clone()],
            &mut errors,
        );
        assert!(errors["besucherstroeme"].contains("km"));

        let access = Access { x: 5.0, ..access };
        errors.clear();
        validate(&[route], &[access], &[z], &mut errors);
        assert!(errors.is_empty());

        for k in AccessKind::ALL {
            assert_eq!(AccessKind::from_key(k.key()), Some(k));
        }
        for d in RouteDirection::ALL {
            assert_eq!(RouteDirection::from_key(d.key()), Some(d));
        }
    }
}
