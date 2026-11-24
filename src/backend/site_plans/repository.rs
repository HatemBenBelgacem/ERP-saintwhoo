//! SQL-Zugriffe für den Geländeplan (ANF-027, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.

use std::collections::HashMap;

use sqlx::{AssertSqlSafe, PgConnection, PgPool};

use super::flows::{Access, AccessKind, RouteDirection, VisitorRoute, Zone};
use super::model::{Corner, ElementKind, Measure, MeasureKind, SiteElement, SitePlan};

/// Nummer und Name des Projekts; `None`, wenn es das Projekt nicht gibt.
pub async fn project_head(
    pool: &PgPool,
    projekt_id: i64,
) -> Result<Option<(String, String)>, sqlx::Error> {
    sqlx::query_as("SELECT nummer, name FROM projekt WHERE id = $1")
        .bind(projekt_id)
        .fetch_optional(pool)
        .await
}

/// Postanschrift des Kunden des Projekts als eine Zeile („Strasse, PLZ Ort“); `None` ohne Kunde
/// oder ohne Ort. Dient als Vorschlag für die Adresssuche.
pub async fn customer_address(
    pool: &PgPool,
    projekt_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT NULLIF(concat_ws(', ', NULLIF(btrim(a.strasse), ''), \
                    NULLIF(btrim(concat_ws(' ', a.plz, a.ort)), ''), \
                    CASE WHEN a.land <> 'CH' THEN a.land END), '') \
         FROM projekt p JOIN adresse a ON a.id = p.adresse_id \
         WHERE p.id = $1 AND NULLIF(btrim(a.ort), '') IS NOT NULL",
    )
    .bind(projekt_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(|(a,)| a))
}

/// Gespeicherter Plan des Projekts mit Änderungszeitpunkt; `None`, solange keiner gespeichert ist.
pub async fn find(
    pool: &PgPool,
    projekt_id: i64,
) -> Result<Option<(SitePlan, String)>, sqlx::Error> {
    let head: Option<(i64, f64, f64, Option<String>, f64, String)> = sqlx::query_as(
        "SELECT id, ursprung_laenge, ursprung_breite, standort_adresse, durchfluss_pro_meter, \
                to_char(geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') \
         FROM gelaendeplan WHERE projekt_id = $1",
    )
    .bind(projekt_id)
    .fetch_optional(pool)
    .await?;
    let Some((id, origin_lng, origin_lat, address, flow_per_meter, changed)) = head else {
        return Ok(None);
    };

    let rows: Vec<(String, f64, f64, f64, f64, f64)> = sqlx::query_as(
        "SELECT art, x_m, z_m, drehung_grad, breite_m, tiefe_m \
         FROM gelaendeplan_element WHERE gelaendeplan_id = $1 ORDER BY reihenfolge",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;
    let elements = rows
        .into_iter()
        .filter_map(|(art, x, z, rotation_deg, width_m, depth_m)| {
            ElementKind::from_key(&art).map(|kind| SiteElement {
                kind,
                x,
                z,
                rotation_deg,
                width_m,
                depth_m,
            })
        })
        .collect();

    let corners: Vec<(f64, f64)> = sqlx::query_as(
        "SELECT x_m, z_m FROM gelaendeplan_eckpunkt WHERE gelaendeplan_id = $1 ORDER BY reihenfolge",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    let measures = load_measures(pool, id).await?;
    let (routes, accesses, zones) = load_flows(pool, id).await?;

    Ok(Some((
        SitePlan {
            origin_lng,
            origin_lat,
            elements,
            corners: corners.into_iter().map(|(x, z)| Corner { x, z }).collect(),
            address,
            flow_per_meter,
            measures,
            routes,
            accesses,
            zones,
        },
        changed,
    )))
}

/// Plan anlegen oder ersetzen (innerhalb einer Transaktion): Kopf aktualisieren,
/// Elemente und Eckpunkte vollständig neu schreiben.
pub async fn save(
    conn: &mut PgConnection,
    projekt_id: i64,
    plan: &SitePlan,
) -> Result<(), sqlx::Error> {
    let (id,): (i64,) = sqlx::query_as(
        "INSERT INTO gelaendeplan \
             (projekt_id, ursprung_laenge, ursprung_breite, standort_adresse, durchfluss_pro_meter) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (projekt_id) DO UPDATE \
         SET ursprung_laenge = EXCLUDED.ursprung_laenge, ursprung_breite = EXCLUDED.ursprung_breite, \
             standort_adresse = EXCLUDED.standort_adresse, \
             durchfluss_pro_meter = EXCLUDED.durchfluss_pro_meter, geaendert_am = now() \
         RETURNING id",
    )
    .bind(projekt_id)
    .bind(plan.origin_lng)
    .bind(plan.origin_lat)
    .bind(plan.address.as_deref())
    .bind(plan.flow_per_meter)
    .fetch_one(&mut *conn)
    .await?;

    sqlx::query("DELETE FROM gelaendeplan_element WHERE gelaendeplan_id = $1")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    sqlx::query("DELETE FROM gelaendeplan_eckpunkt WHERE gelaendeplan_id = $1")
        .bind(id)
        .execute(&mut *conn)
        .await?;

    // Alle Zeilen in einer Anweisung über Arrays (unnest) – auch bei vielen Elementen schnell
    let e = &plan.elements;
    sqlx::query(
        "INSERT INTO gelaendeplan_element \
             (gelaendeplan_id, reihenfolge, art, x_m, z_m, drehung_grad, breite_m, tiefe_m) \
         SELECT $1, t.nr, t.art, t.x, t.z, t.drehung, t.breite, t.tiefe \
         FROM unnest($2::int[], $3::text[], $4::float8[], $5::float8[], $6::float8[], $7::float8[], $8::float8[]) \
              AS t(nr, art, x, z, drehung, breite, tiefe)",
    )
    .bind(id)
    .bind((1..=e.len() as i32).collect::<Vec<i32>>())
    .bind(e.iter().map(|e| e.kind.key().to_string()).collect::<Vec<_>>())
    .bind(e.iter().map(|e| e.x).collect::<Vec<_>>())
    .bind(e.iter().map(|e| e.z).collect::<Vec<_>>())
    .bind(e.iter().map(|e| e.rotation_deg).collect::<Vec<_>>())
    .bind(e.iter().map(|e| e.width_m).collect::<Vec<_>>())
    .bind(e.iter().map(|e| e.depth_m).collect::<Vec<_>>())
    .execute(&mut *conn)
    .await?;

    let c = &plan.corners;
    sqlx::query(
        "INSERT INTO gelaendeplan_eckpunkt (gelaendeplan_id, reihenfolge, x_m, z_m) \
         SELECT $1, t.nr, t.x, t.z FROM unnest($2::int[], $3::float8[], $4::float8[]) AS t(nr, x, z)",
    )
    .bind(id)
    .bind((1..=c.len() as i32).collect::<Vec<i32>>())
    .bind(c.iter().map(|c| c.x).collect::<Vec<_>>())
    .bind(c.iter().map(|c| c.z).collect::<Vec<_>>())
    .execute(&mut *conn)
    .await?;

    // Messungen: alte löschen (Punkte über ON DELETE CASCADE), dann je Messung Kopf und Punkte
    sqlx::query("DELETE FROM gelaendeplan_messung WHERE gelaendeplan_id = $1")
        .bind(id)
        .execute(&mut *conn)
        .await?;
    for (i, m) in plan.measures.iter().enumerate() {
        let label = m.label.trim();
        let (measure_id,): (i64,) = sqlx::query_as(
            "INSERT INTO gelaendeplan_messung (gelaendeplan_id, reihenfolge, art, bezeichnung) \
             VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(id)
        .bind(i as i32 + 1)
        .bind(m.kind.key())
        .bind((!label.is_empty()).then_some(label))
        .fetch_one(&mut *conn)
        .await?;
        sqlx::query(
            "INSERT INTO gelaendeplan_messpunkt (messung_id, reihenfolge, x_m, z_m) \
             SELECT $1, t.nr, t.x, t.z FROM unnest($2::int[], $3::float8[], $4::float8[]) AS t(nr, x, z)",
        )
        .bind(measure_id)
        .bind((1..=m.points.len() as i32).collect::<Vec<i32>>())
        .bind(m.points.iter().map(|c| c.x).collect::<Vec<_>>())
        .bind(m.points.iter().map(|c| c.z).collect::<Vec<_>>())
        .execute(&mut *conn)
        .await?;
    }

    // Besucherströme (ANF-028): alte löschen (Punkte über ON DELETE CASCADE), neu schreiben
    for table in [
        "gelaendeplan_besucherweg",
        "gelaendeplan_zugang",
        "gelaendeplan_zone",
    ] {
        sqlx::query(AssertSqlSafe(format!(
            "DELETE FROM {table} WHERE gelaendeplan_id = $1"
        )))
        .bind(id)
        .execute(&mut *conn)
        .await?;
    }
    insert_flows(conn, id, plan).await?;
    Ok(())
}

/// Tabellen mit Punkten eines Linienzugs bzw. einer Fläche (ANF-028). Nur feste Namen – deshalb
/// darf das SQL daraus zusammengesetzt werden (`AssertSqlSafe`).
#[derive(Clone, Copy)]
enum PointTable {
    Route,
    Zone,
}

impl PointTable {
    /// (Punkttabelle, Fremdschlüssel, Kopftabelle)
    fn names(self) -> (&'static str, &'static str, &'static str) {
        match self {
            PointTable::Route => (
                "gelaendeplan_besucherweg_punkt",
                "besucherweg_id",
                "gelaendeplan_besucherweg",
            ),
            PointTable::Zone => ("gelaendeplan_zone_punkt", "zone_id", "gelaendeplan_zone"),
        }
    }
}

/// Punkte eines Kopfs in Reihenfolge schreiben.
async fn insert_points(
    conn: &mut PgConnection,
    table: PointTable,
    parent_id: i64,
    points: &[Corner],
) -> Result<(), sqlx::Error> {
    let (points_table, fk, _) = table.names();
    sqlx::query(AssertSqlSafe(format!(
        "INSERT INTO {points_table} ({fk}, reihenfolge, x_m, z_m) \
         SELECT $1, t.nr, t.x, t.z FROM unnest($2::int[], $3::float8[], $4::float8[]) AS t(nr, x, z)"
    )))
    .bind(parent_id)
    .bind((1..=points.len() as i32).collect::<Vec<i32>>())
    .bind(points.iter().map(|c| c.x).collect::<Vec<_>>())
    .bind(points.iter().map(|c| c.z).collect::<Vec<_>>())
    .execute(&mut *conn)
    .await?;
    Ok(())
}

/// Alle Punkte der Köpfe eines Plans, nach Kopf gruppiert und in Reihenfolge.
async fn load_points(
    pool: &PgPool,
    table: PointTable,
    plan_id: i64,
) -> Result<HashMap<i64, Vec<Corner>>, sqlx::Error> {
    let (points_table, fk, parent_table) = table.names();
    let rows: Vec<(i64, f64, f64)> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT p.{fk}, p.x_m, p.z_m FROM {points_table} p \
         JOIN {parent_table} k ON k.id = p.{fk} \
         WHERE k.gelaendeplan_id = $1 ORDER BY p.{fk}, p.reihenfolge"
    )))
    .bind(plan_id)
    .fetch_all(pool)
    .await?;
    let mut map: HashMap<i64, Vec<Corner>> = HashMap::new();
    for (parent, x, z) in rows {
        map.entry(parent).or_default().push(Corner { x, z });
    }
    Ok(map)
}

/// Leerer Text → NULL.
fn opt_label(label: &str) -> Option<&str> {
    let l = label.trim();
    (!l.is_empty()).then_some(l)
}

/// Besucherwege, Zugänge und Zonen eines Plans schreiben (innerhalb der Speicher-Transaktion,
/// nachdem die alten gelöscht sind).
async fn insert_flows(
    conn: &mut PgConnection,
    plan_id: i64,
    plan: &SitePlan,
) -> Result<(), sqlx::Error> {
    for (i, r) in plan.routes.iter().enumerate() {
        let (route_id,): (i64,) = sqlx::query_as(
            "INSERT INTO gelaendeplan_besucherweg \
                 (gelaendeplan_id, reihenfolge, bezeichnung, richtung, personen_pro_stunde, breite_m) \
             VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
        )
        .bind(plan_id)
        .bind(i as i32 + 1)
        .bind(opt_label(&r.label))
        .bind(r.direction.key())
        .bind(r.persons_per_hour as i32)
        .bind(r.width_m)
        .fetch_one(&mut *conn)
        .await?;
        insert_points(conn, PointTable::Route, route_id, &r.points).await?;
    }
    for (i, a) in plan.accesses.iter().enumerate() {
        sqlx::query(
            "INSERT INTO gelaendeplan_zugang \
                 (gelaendeplan_id, reihenfolge, art, bezeichnung, x_m, z_m, personen_pro_minute) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(plan_id)
        .bind(i as i32 + 1)
        .bind(a.kind.key())
        .bind(opt_label(&a.label))
        .bind(a.x)
        .bind(a.z)
        .bind(a.persons_per_minute.map(|n| n as i32))
        .execute(&mut *conn)
        .await?;
    }
    for (i, z) in plan.zones.iter().enumerate() {
        let (zone_id,): (i64,) = sqlx::query_as(
            "INSERT INTO gelaendeplan_zone \
                 (gelaendeplan_id, reihenfolge, bezeichnung, erwartete_personen, dichte) \
             VALUES ($1, $2, $3, $4, $5) RETURNING id",
        )
        .bind(plan_id)
        .bind(i as i32 + 1)
        .bind(opt_label(&z.label))
        .bind(z.expected_persons as i32)
        .bind(z.density)
        .fetch_one(&mut *conn)
        .await?;
        insert_points(conn, PointTable::Zone, zone_id, &z.points).await?;
    }
    Ok(())
}

/// Besucherwege, Zugänge und Zonen eines Plans, je in Reihenfolge.
#[allow(clippy::type_complexity)]
async fn load_flows(
    pool: &PgPool,
    plan_id: i64,
) -> Result<(Vec<VisitorRoute>, Vec<Access>, Vec<Zone>), sqlx::Error> {
    let mut route_points = load_points(pool, PointTable::Route, plan_id).await?;
    let routes: Vec<(i64, Option<String>, String, i32, Option<f64>)> = sqlx::query_as(
        "SELECT id, bezeichnung, richtung, personen_pro_stunde, breite_m FROM gelaendeplan_besucherweg \
         WHERE gelaendeplan_id = $1 ORDER BY reihenfolge",
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await?;
    let routes = routes
        .into_iter()
        .filter_map(|(id, label, direction, persons, width_m)| {
            RouteDirection::from_key(&direction).map(|direction| VisitorRoute {
                label: label.unwrap_or_default(),
                direction,
                persons_per_hour: persons.max(0) as u32,
                width_m,
                points: route_points.remove(&id).unwrap_or_default(),
            })
        })
        .collect();

    let accesses: Vec<(String, Option<String>, f64, f64, Option<i32>)> = sqlx::query_as(
        "SELECT art, bezeichnung, x_m, z_m, personen_pro_minute FROM gelaendeplan_zugang \
         WHERE gelaendeplan_id = $1 ORDER BY reihenfolge",
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await?;
    let accesses = accesses
        .into_iter()
        .filter_map(|(kind, label, x, z, per_minute)| {
            AccessKind::from_key(&kind).map(|kind| Access {
                kind,
                label: label.unwrap_or_default(),
                x,
                z,
                persons_per_minute: per_minute.map(|n| n.max(0) as u32),
            })
        })
        .collect();

    let mut zone_points = load_points(pool, PointTable::Zone, plan_id).await?;
    let zones: Vec<(i64, Option<String>, i32, f64)> = sqlx::query_as(
        "SELECT id, bezeichnung, erwartete_personen, dichte FROM gelaendeplan_zone \
         WHERE gelaendeplan_id = $1 ORDER BY reihenfolge",
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await?;
    let zones = zones
        .into_iter()
        .map(|(id, label, expected, density)| Zone {
            label: label.unwrap_or_default(),
            expected_persons: expected.max(0) as u32,
            density,
            points: zone_points.remove(&id).unwrap_or_default(),
        })
        .collect();

    Ok((routes, accesses, zones))
}

/// Messungen eines Plans mit ihren Punkten, in Reihenfolge.
async fn load_measures(pool: &PgPool, plan_id: i64) -> Result<Vec<Measure>, sqlx::Error> {
    let heads: Vec<(i64, String, Option<String>)> = sqlx::query_as(
        "SELECT id, art, bezeichnung FROM gelaendeplan_messung \
         WHERE gelaendeplan_id = $1 ORDER BY reihenfolge",
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await?;
    let points: Vec<(i64, f64, f64)> = sqlx::query_as(
        "SELECT p.messung_id, p.x_m, p.z_m FROM gelaendeplan_messpunkt p \
         JOIN gelaendeplan_messung m ON m.id = p.messung_id \
         WHERE m.gelaendeplan_id = $1 ORDER BY p.messung_id, p.reihenfolge",
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await?;
    Ok(heads
        .into_iter()
        .filter_map(|(measure_id, art, label)| {
            MeasureKind::from_key(&art).map(|kind| Measure {
                kind,
                label: label.unwrap_or_default(),
                points: points
                    .iter()
                    .filter(|(m, _, _)| *m == measure_id)
                    .map(|(_, x, z)| Corner { x: *x, z: *z })
                    .collect(),
            })
        })
        .collect())
}

/// Plan des Projekts löschen; liefert, ob es einen gab.
pub async fn delete(pool: &PgPool, projekt_id: i64) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM gelaendeplan WHERE projekt_id = $1")
        .bind(projekt_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}
