//! Engstellen der Besucherwege (ANF-028 Stufe 3), gemeinsam für Server und Browser.
//!
//! Aus den erwarteten Personen pro Stunde und dem Planungswert „Durchfluss je Meter Wegbreite“ ergibt
//! sich die **nötige Breite** eines Wegs. Verglichen wird sie
//! - mit der eingetragenen Wegbreite (Auslastung des ganzen Wegs) und
//! - mit der **freien Breite** entlang des Wegs: Bühnen, Stände und Zelte links und rechts der Wegmitte
//!   engen ihn ein; steht ein Element auf der Wegmitte, ist der Weg versperrt.
//!
//! Die Ampel folgt denselben Grenzen wie bei den Zonen: bis 80 % ausreichend, bis 100 % knapp,
//! darüber zu eng. Das ist eine Planungshilfe, keine Bemessung von Fluchtwegen.

use geo::{Distance, Euclidean, Line};

use super::flows::{VisitorRoute, ZONE_TIGHT_FROM, ZoneStatus, format_percent};
use super::model::{Corner, SitePlan, format_m};

/// Vorgabe und erlaubter Bereich des Durchflusses je Meter Wegbreite (Personen pro Stunde).
/// 2’000 P/(m·h) ≈ 33 Personen pro Minute und Meter: bequemes Gehen, kein Gedränge.
pub const DEFAULT_FLOW_PER_METER: f64 = 2_000.0;
pub const FLOW_PER_METER_RANGE: (f64, f64) = (100.0, 10_000.0);

/// Nötige Breite eines Wegs in Metern; `None` ohne erwartete Personen.
pub fn required_width_m(route: &VisitorRoute, flow_per_meter: f64) -> Option<f64> {
    (route.persons_per_hour > 0 && flow_per_meter > 0.0)
        .then(|| route.persons_per_hour as f64 / flow_per_meter)
}

/// Auslastung des Wegs über seine eingetragene Breite (nötige / vorhandene Breite).
pub fn route_utilization(route: &VisitorRoute, flow_per_meter: f64) -> Option<f64> {
    let required = required_width_m(route, flow_per_meter)?;
    route.width_m.filter(|w| *w > 0.0).map(|w| required / w)
}

/// Ampel aus einer Auslastung (gleiche Grenzen wie bei den Zonen).
pub fn status_of(utilization: Option<f64>) -> ZoneStatus {
    match utilization {
        None => ZoneStatus::Unknown,
        Some(u) if u <= ZONE_TIGHT_FROM => ZoneStatus::Ok,
        Some(u) if u <= 1.0 => ZoneStatus::Tight,
        Some(_) => ZoneStatus::Over,
    }
}

/// Eine Engstelle auf einem Besucherweg.
#[derive(Clone, Debug, PartialEq)]
pub struct Bottleneck {
    /// Index des Wegs in `plan.routes`.
    pub route: usize,
    /// Stelle auf der Wegmitte (für die Beschriftung auf der Karte).
    pub at: Corner,
    /// Freie Breite in Metern; `None` = Weg versperrt (Element steht auf der Wegmitte).
    pub free_width_m: Option<f64>,
    pub required_width_m: f64,
    /// Elemente, die den Weg einengen bzw. versperren (Index in `plan.elements`).
    pub elements: Vec<usize>,
    /// `Tight` (knapp) oder `Over` (zu eng bzw. versperrt).
    pub status: ZoneStatus,
}

impl Bottleneck {
    /// Kurztext für die Karte, z. B. „Engstelle 2,1 m (nötig 3 m)“.
    pub fn short_text(&self) -> String {
        match self.free_width_m {
            None => "Weg versperrt".into(),
            Some(free) => format!(
                "Engstelle {} m (nötig {} m)",
                format_m(free),
                format_m(self.required_width_m)
            ),
        }
    }

    /// Auslastung der freien Breite, z. B. „140 %“.
    pub fn utilization_text(&self) -> String {
        match self.free_width_m {
            None => "versperrt".into(),
            Some(free) => format_percent(self.required_width_m / free),
        }
    }
}

/// Alle Engstellen des Plans, je Weg von Anfang bis Ende.
pub fn find(plan: &SitePlan) -> Vec<Bottleneck> {
    let footprints: Vec<_> = plan.elements.iter().map(|e| e.footprint()).collect();
    let mut found = Vec::new();
    for (route_index, route) in plan.routes.iter().enumerate() {
        let Some(required) = required_width_m(route, plan.flow_per_meter) else {
            continue;
        };
        let half_width = route.width_m.map_or(f64::INFINITY, |w| w / 2.0);
        for seg in route.points.windows(2) {
            let (a, b) = (seg[0], seg[1]);
            let (dx, dz) = (b.x - a.x, b.z - a.z);
            let len2 = dx * dx + dz * dz;
            if len2 == 0.0 {
                continue;
            }
            let line = Line::new((a.x, a.z), (b.x, b.z));
            // Nächstes Element links und rechts der Wegmitte; Elemente auf der Wegmitte versperren
            let mut left: Option<(f64, usize)> = None;
            let mut right: Option<(f64, usize)> = None;
            let mut blocking = Vec::new();
            let mut blocking_at = None;
            for (j, (element, footprint)) in plan.elements.iter().zip(&footprints).enumerate() {
                // Nur Elemente neben diesem Abschnitt (Lot des Mittelpunkts fällt auf den Abschnitt)
                let t = ((element.x - a.x) * dx + (element.z - a.z) * dz) / len2;
                let d: f64 = Euclidean.distance(&line, footprint);
                if d <= 1e-9 {
                    blocking.push(j);
                    blocking_at.get_or_insert(foot(a, dx, dz, t));
                    continue;
                }
                if !(0.0..=1.0).contains(&t) || d >= half_width {
                    continue;
                }
                let side = &mut if dx * (element.z - a.z) - dz * (element.x - a.x) > 0.0 {
                    &mut right
                } else {
                    &mut left
                };
                if side.is_none_or(|(best, _)| d < best) {
                    **side = Some((d, j));
                }
            }

            if !blocking.is_empty() {
                found.push(Bottleneck {
                    route: route_index,
                    at: blocking_at.unwrap_or(a),
                    free_width_m: None,
                    required_width_m: required,
                    elements: blocking,
                    status: ZoneStatus::Over,
                });
                continue;
            }
            // Freie Breite: je Seite der Abstand zum nächsten Element, höchstens die halbe Wegbreite
            let hl = left.map_or(half_width, |(d, _)| d.min(half_width));
            let hr = right.map_or(half_width, |(d, _)| d.min(half_width));
            let free = hl + hr;
            if !free.is_finite() || (left.is_none() && right.is_none()) {
                continue;
            }
            let status = status_of(Some(required / free));
            if !matches!(status, ZoneStatus::Tight | ZoneStatus::Over) {
                continue;
            }
            let elements: Vec<usize> = left.iter().chain(right.iter()).map(|(_, j)| *j).collect();
            // Stelle: Lot vom Mittelpunkt des nächsten Elements auf den Abschnitt
            let nearest = elements.iter().map(|j| &plan.elements[*j]).min_by(|x, y| {
                let dx_ = Euclidean.distance(&line, &x.footprint());
                let dy_ = Euclidean.distance(&line, &y.footprint());
                dx_.total_cmp(&dy_)
            });
            let at = nearest.map_or(a, |e| {
                let t = (((e.x - a.x) * dx + (e.z - a.z) * dz) / len2).clamp(0.0, 1.0);
                foot(a, dx, dz, t)
            });
            found.push(Bottleneck {
                route: route_index,
                at,
                free_width_m: Some(free),
                required_width_m: required,
                elements,
                status,
            });
        }
    }
    found
}

/// Punkt auf dem Abschnitt bei Anteil `t` (0 = Anfang, 1 = Ende).
fn foot(a: Corner, dx: f64, dz: f64, t: f64) -> Corner {
    let t = t.clamp(0.0, 1.0);
    Corner {
        x: a.x + dx * t,
        z: a.z + dz * t,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::site_plans::model::{ElementKind, SiteElement};

    /// Weg von West nach Ost auf z = 0, 50 m lang, mit 4’000 P/h → nötig 2 m.
    fn plan_with_route(width_m: Option<f64>) -> SitePlan {
        let mut route = VisitorRoute::new(Corner { x: 0.0, z: 0.0 });
        route.points.push(Corner { x: 50.0, z: 0.0 });
        route.persons_per_hour = 4_000;
        route.width_m = width_m;
        SitePlan {
            routes: vec![route],
            ..SitePlan::default()
        }
    }

    /// Verpflegungsstand (4 × 2,5 m) mit Mittelpunkt (x, z): reicht 1,25 m nach Norden und Süden.
    fn stall(x: f64, z: f64) -> SiteElement {
        SiteElement::new(ElementKind::FoodStall, x, z)
    }

    #[test]
    fn required_width_and_route_traffic_light() {
        let plan = plan_with_route(Some(3.0));
        let r = &plan.routes[0];
        assert_eq!(required_width_m(r, plan.flow_per_meter), Some(2.0));
        assert_eq!(
            status_of(route_utilization(r, plan.flow_per_meter)),
            ZoneStatus::Ok
        ); // 67 %
        let narrow = plan_with_route(Some(1.5));
        assert_eq!(
            status_of(route_utilization(&narrow.routes[0], 2_000.0)),
            ZoneStatus::Over
        ); // 133 %
        assert_eq!(status_of(route_utilization(r, 0.0)), ZoneStatus::Unknown);
        // Engpass nur über die Wegbreite ist keine Engstelle durch Elemente
        assert!(find(&narrow).is_empty());
    }

    #[test]
    fn gap_between_two_stalls() {
        // Stände bei z = −2,1 und z = +2,1 → je 0,85 m von der Wegmitte → frei 1,7 m < 2 m
        let mut plan = plan_with_route(None);
        plan.elements = vec![stall(25.0, -2.1), stall(25.0, 2.1)];
        let found = find(&plan);
        assert_eq!(found.len(), 1);
        let b = &found[0];
        assert_eq!(b.status, ZoneStatus::Over);
        assert!((b.free_width_m.unwrap() - 1.7).abs() < 1e-9);
        assert_eq!(b.elements.len(), 2);
        assert!((b.at.x - 25.0).abs() < 1e-9 && b.at.z.abs() < 1e-9);
        assert_eq!(b.short_text(), "Engstelle 1,7 m (nötig 2 m)");

        // weiter auseinander: frei 3 m → ausreichend, keine Engstelle
        plan.elements = vec![stall(25.0, -2.75), stall(25.0, 2.75)];
        assert!(find(&plan).is_empty());
    }

    #[test]
    fn one_sided_obstacle_counts_only_with_route_width() {
        // nur ein Stand neben dem Weg, ohne Wegbreite: die andere Seite ist frei
        let mut plan = plan_with_route(None);
        plan.elements = vec![stall(25.0, 1.5)];
        assert!(find(&plan).is_empty());
        // Wegbreite 2,4 m: Stand 0,25 m neben der Wegmitte → frei 0,25 + 1,2 = 1,45 m
        plan.routes[0].width_m = Some(2.4);
        let found = find(&plan);
        assert_eq!(found.len(), 1);
        assert!((found[0].free_width_m.unwrap() - 1.45).abs() < 1e-9);
    }

    #[test]
    fn element_on_route_blocks_it() {
        let mut plan = plan_with_route(None);
        plan.elements = vec![stall(30.0, 0.0)];
        let found = find(&plan);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].free_width_m, None);
        assert_eq!(found[0].short_text(), "Weg versperrt");
        assert_eq!(found[0].elements, vec![0]);
        // ohne erwartete Personen wird nichts geprüft
        plan.routes[0].persons_per_hour = 0;
        assert!(find(&plan).is_empty());
    }
}
