//! SQL-Zugriffe für den Geländeplan (ANF-027, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.

use sqlx::{PgConnection, PgPool};

use super::model::{Corner, ElementKind, SiteElement, SitePlan};

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
    let head: Option<(i64, f64, f64, Option<String>, String)> = sqlx::query_as(
        "SELECT id, ursprung_laenge, ursprung_breite, standort_adresse, \
                to_char(geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') \
         FROM gelaendeplan WHERE projekt_id = $1",
    )
    .bind(projekt_id)
    .fetch_optional(pool)
    .await?;
    let Some((id, origin_lng, origin_lat, address, changed)) = head else {
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

    Ok(Some((
        SitePlan {
            origin_lng,
            origin_lat,
            elements,
            corners: corners.into_iter().map(|(x, z)| Corner { x, z }).collect(),
            address,
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
        "INSERT INTO gelaendeplan (projekt_id, ursprung_laenge, ursprung_breite, standort_adresse) \
         VALUES ($1, $2, $3, $4) \
         ON CONFLICT (projekt_id) DO UPDATE \
         SET ursprung_laenge = EXCLUDED.ursprung_laenge, ursprung_breite = EXCLUDED.ursprung_breite, \
             standort_adresse = EXCLUDED.standort_adresse, geaendert_am = now() \
         RETURNING id",
    )
    .bind(projekt_id)
    .bind(plan.origin_lng)
    .bind(plan.origin_lat)
    .bind(plan.address.as_deref())
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
    Ok(())
}

/// Plan des Projekts löschen; liefert, ob es einen gab.
pub async fn delete(pool: &PgPool, projekt_id: i64) -> Result<bool, sqlx::Error> {
    let r = sqlx::query("DELETE FROM gelaendeplan WHERE projekt_id = $1")
        .bind(projekt_id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() > 0)
}
