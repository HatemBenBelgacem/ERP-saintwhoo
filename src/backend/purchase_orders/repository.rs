//! SQL-Zugriffe für Einkaufsaufträge und ihre Positionen (ANF-004, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Statusprüfungen, keine Weiterleitungen.
//! Fehler werden als `sqlx::Error` zurückgegeben; die Server-Funktionen übersetzen sie.
//! Funktionen mit `&mut PgConnection` laufen in der Transaktion des Aufrufers.

use sqlx::{AssertSqlSafe, PgConnection, PgPool};
use std::collections::HashMap;

use super::model::{OrderDetail, OrderInput, OrderRow, Position, PositionInput, PositionValues};

/// Liefertermin überschritten und Auftrag noch offen (AK-5). Ohne Liefertermin: nicht überfällig.
const OVERDUE: &str = "coalesce(e.liefertermin < (now() AT TIME ZONE 'Europe/Zurich')::date \
     AND e.status IN ('entwurf', 'bestellt', 'teilweise_geliefert'), false)";

/// Positionen mit Werten in kleinsten Einheiten (siehe model.rs).
const POSITION_COLUMNS: &str = "id, bezeichnung, artikelnummer_lieferant, \
     (menge * 1000)::int8 AS menge, einheit, (einzelpreis * 100)::int8 AS einzelpreis, \
     (rabatt_prozent * 100)::int8 AS rabatt_prozent, (mwst_satz * 100)::int8 AS mwst_satz, \
     (gelieferte_menge * 1000)::int8 AS gelieferte_menge";

/// Spalten einer Listenzeile (für Liste und Projekt-Detailansicht).
const ROW_COLUMNS: &str = "e.id, e.nummer, a.anzeigename AS lieferant, e.projekt_id, \
     p.nummer AS projekt_nummer, e.status, \
     to_char(e.bestelldatum, 'YYYY-MM-DD') AS bestelldatum, \
     to_char(e.liefertermin, 'YYYY-MM-DD') AS liefertermin, e.waehrung";

const FROM: &str = "einkaufsauftrag e JOIN adresse a ON a.id = e.adresse_id \
     LEFT JOIN projekt p ON p.id = e.projekt_id";

/// Suchbedingung der Liste; $1 = Suchbegriff, $2 = LIKE-Muster, $3 = „offen“, „alle“ oder ein Status.
const LIST_WHERE: &str = "($1 = '' OR e.nummer ILIKE $2 OR a.anzeigename ILIKE $2 OR p.nummer ILIKE $2 \
                             OR p.name ILIKE $2 OR e.referenz_lieferant ILIKE $2) \
                          AND ($3 = 'alle' \
                             OR ($3 = 'offen' AND e.status IN ('entwurf', 'bestellt', 'teilweise_geliefert')) \
                             OR e.status = $3)";

/// Sortierausdruck für einen Schlüssel aus der Webadresse (nur feste Werte → kein SQL-Einschleusen).
fn order_by(sort: &str) -> &'static str {
    match sort {
        "lieferant" => "lower(a.anzeigename)",
        "projekt" => "p.nummer",
        "bestelldatum" => "e.bestelldatum",
        "liefertermin" => "e.liefertermin",
        "status" => {
            "array_position(ARRAY['entwurf','bestellt','teilweise_geliefert','geliefert','storniert'], e.status::text)"
        }
        // Sortierung nach Betrag inkl. MWST (Anzeige rechnet genau in model.rs)
        "betrag" => {
            "(SELECT coalesce(sum(round(x.menge * x.einzelpreis * (1 - x.rabatt_prozent / 100), 2) \
                                  * (1 + x.mwst_satz / 100)), 0) \
              FROM einkaufsposition x WHERE x.einkaufsauftrag_id = e.id)"
        }
        _ => "e.id", // Nummer: in Vergabe-Reihenfolge
    }
}

/// Leerer Text → NULL in der Datenbank.
fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

#[derive(sqlx::FromRow)]
struct PositionOfOrder {
    einkaufsauftrag_id: i64,
    #[sqlx(flatten)]
    position: Position,
}

// ---------- Lesen ----------

/// Anzahl Einkaufsaufträge, die zu Suche und Statusfilter passen.
pub async fn count_orders(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    status: &str,
) -> Result<i64, sqlx::Error> {
    // Geprüft: nur feste Texte, Eingaben laufen als Parameter.
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM {FROM} WHERE {LIST_WHERE}"
    )))
    .bind(suche)
    .bind(pattern)
    .bind(status)
    .fetch_one(pool)
    .await
}

/// Eine Seite der Liste, Gesamtbeträge bereits berechnet.
#[allow(clippy::too_many_arguments)]
pub async fn list_orders_page(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    status: &str,
    sortierung: &str,
    ascending: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<OrderRow>, sqlx::Error> {
    let direction = if ascending { "ASC" } else { "DESC" };
    // Geprüft: Sortierung aus fester Liste (order_by), Richtung ist ASC/DESC.
    let mut rows: Vec<OrderRow> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {ROW_COLUMNS}, {OVERDUE} AS ueberfaellig FROM {FROM} WHERE {LIST_WHERE} \
         ORDER BY {} {direction} NULLS LAST, e.id DESC LIMIT $4 OFFSET $5",
        order_by(sortierung)
    )))
    .bind(suche)
    .bind(pattern)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    fill_totals(pool, &mut rows).await?;
    Ok(rows)
}

/// Einkaufsaufträge eines Projekts, neueste zuerst, Gesamtbeträge bereits berechnet.
pub async fn list_by_project(pool: &PgPool, project_id: i64) -> Result<Vec<OrderRow>, sqlx::Error> {
    let mut rows: Vec<OrderRow> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {ROW_COLUMNS}, {OVERDUE} AS ueberfaellig FROM {FROM} \
         WHERE e.projekt_id = $1 ORDER BY e.id DESC"
    )))
    .bind(project_id)
    .fetch_all(pool)
    .await?;
    fill_totals(pool, &mut rows).await?;
    Ok(rows)
}

/// Gesamtbeträge der Zeilen aus ihren Positionen berechnen (gleiche Rechnung wie in der Detailansicht).
async fn fill_totals(pool: &PgPool, rows: &mut [OrderRow]) -> Result<(), sqlx::Error> {
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let positions: Vec<PositionOfOrder> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT einkaufsauftrag_id, {POSITION_COLUMNS} FROM einkaufsposition \
         WHERE einkaufsauftrag_id = ANY($1)"
    )))
    .bind(&ids)
    .fetch_all(pool)
    .await?;
    let mut by_order: HashMap<i64, Vec<Position>> = HashMap::new();
    for p in positions {
        by_order
            .entry(p.einkaufsauftrag_id)
            .or_default()
            .push(p.position);
    }
    for row in rows {
        let positions = by_order.remove(&row.id).unwrap_or_default();
        row.total = super::model::totals(&positions, &row.waehrung).total;
    }
    Ok(())
}

/// Positionen eines Auftrags in ihrer Reihenfolge.
pub async fn load_positions(
    conn: &mut PgConnection,
    order_id: i64,
) -> Result<Vec<Position>, sqlx::Error> {
    // Geprüft: nur feste Spaltenliste
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {POSITION_COLUMNS} FROM einkaufsposition \
         WHERE einkaufsauftrag_id = $1 ORDER BY reihenfolge, id"
    )))
    .bind(order_id)
    .fetch_all(conn)
    .await
}

/// Auftrag mit Positionen; `None`, wenn es ihn nicht gibt.
pub async fn load_detail(
    conn: &mut PgConnection,
    id: i64,
) -> Result<Option<OrderDetail>, sqlx::Error> {
    let order = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT e.id, e.nummer, e.adresse_id, a.nummer AS lieferant_nummer, a.anzeigename AS lieferant, \
                e.projekt_id, p.nummer AS projekt_nummer, p.name AS projekt_name, e.status, \
                to_char(e.bestelldatum, 'YYYY-MM-DD') AS bestelldatum, \
                to_char(e.liefertermin, 'YYYY-MM-DD') AS liefertermin, \
                e.referenz_lieferant, e.waehrung, e.lieferadresse, e.bemerkung, \
                {OVERDUE} AS ueberfaellig, \
                to_char(e.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                to_char(e.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
         FROM {FROM} WHERE e.id = $1"
    )))
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some(order) = order else {
        return Ok(None);
    };
    let positions = load_positions(conn, id).await?;
    Ok(Some(OrderDetail { order, positions }))
}

/// Anzahl Positionen eines Auftrags.
pub async fn count_positions(conn: &mut PgConnection, order_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM einkaufsposition WHERE einkaufsauftrag_id = $1")
        .bind(order_id)
        .fetch_one(conn)
        .await
}

// ---------- Sperren ----------

/// Sperrt den Einkaufsauftrag bis zum Ende der Transaktion und liefert seinen Status (`None` = nicht gefunden).
pub async fn lock_order(conn: &mut PgConnection, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT status FROM einkaufsauftrag WHERE id = $1 FOR UPDATE")
        .bind(id)
        .fetch_optional(conn)
        .await
}

/// Einkaufsauftrag einer Position sperren: (Kennung des Auftrags, Status), `None` = Position nicht gefunden.
pub async fn lock_order_of_position(
    conn: &mut PgConnection,
    position_id: i64,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT e.id, e.status FROM einkaufsposition p \
         JOIN einkaufsauftrag e ON e.id = p.einkaufsauftrag_id \
         WHERE p.id = $1 FOR UPDATE OF e",
    )
    .bind(position_id)
    .fetch_optional(conn)
    .await
}

// ---------- Auftrag schreiben ----------

/// Neuen Auftrag im Status „Entwurf“ speichern, Nummer E-JJJJ-NNNN vergeben; liefert die Kennung.
pub async fn insert_order(conn: &mut PgConnection, input: &OrderInput) -> Result<i64, sqlx::Error> {
    let (year, n): (i32, i32) = sqlx::query_as(
        "INSERT INTO einkaufsauftrag_nummer (jahr, letzte) \
         VALUES (extract(year FROM now() AT TIME ZONE 'Europe/Zurich')::int, 1) \
         ON CONFLICT (jahr) DO UPDATE SET letzte = einkaufsauftrag_nummer.letzte + 1 \
         RETURNING jahr, letzte",
    )
    .fetch_one(&mut *conn)
    .await?;
    sqlx::query_scalar(
        "INSERT INTO einkaufsauftrag (nummer, adresse_id, projekt_id, bestelldatum, liefertermin, \
                                      referenz_lieferant, waehrung, lieferadresse, bemerkung) \
         VALUES ($1, $2, $3, $4::date, $5::date, $6, $7, $8, $9) RETURNING id",
    )
    .bind(format!("E-{year}-{n:04}"))
    .bind(input.supplier_id())
    .bind(input.project_id())
    .bind(opt(&input.bestelldatum))
    .bind(opt(&input.liefertermin))
    .bind(opt(&input.referenz_lieferant))
    .bind(&input.waehrung)
    .bind(opt(&input.lieferadresse))
    .bind(opt(&input.bemerkung))
    .fetch_one(&mut *conn)
    .await
}

/// Kopfdaten ändern.
pub async fn update_order(
    conn: &mut PgConnection,
    id: i64,
    input: &OrderInput,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE einkaufsauftrag SET adresse_id = $1, projekt_id = $2, bestelldatum = $3::date, \
                liefertermin = $4::date, referenz_lieferant = $5, waehrung = $6, lieferadresse = $7, \
                bemerkung = $8, geaendert_am = now() \
         WHERE id = $9",
    )
    .bind(input.supplier_id())
    .bind(input.project_id())
    .bind(opt(&input.bestelldatum))
    .bind(opt(&input.liefertermin))
    .bind(opt(&input.referenz_lieferant))
    .bind(&input.waehrung)
    .bind(opt(&input.lieferadresse))
    .bind(opt(&input.bemerkung))
    .bind(id)
    .execute(conn)
    .await?;
    Ok(())
}

/// Auftrag als geändert markieren (auch bei Änderungen an Positionen).
pub async fn touch(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE einkaufsauftrag SET geaendert_am = now() WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Status setzen.
pub async fn set_status(conn: &mut PgConnection, id: i64, status: &str) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE einkaufsauftrag SET status = $1, geaendert_am = now() WHERE id = $2")
        .bind(status)
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Status „bestellt“ setzen, Bestelldatum = heute, falls leer.
pub async fn mark_ordered(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE einkaufsauftrag SET status = 'bestellt', \
                bestelldatum = coalesce(bestelldatum, (now() AT TIME ZONE 'Europe/Zurich')::date), \
                geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .execute(conn)
    .await?;
    Ok(())
}

/// Auftrag mit Positionen löschen; liefert seine Nummer.
pub async fn delete_order(conn: &mut PgConnection, id: i64) -> Result<String, sqlx::Error> {
    sqlx::query_scalar("DELETE FROM einkaufsauftrag WHERE id = $1 RETURNING nummer")
        .bind(id)
        .fetch_one(conn)
        .await
}

// ---------- Positionen schreiben ----------

/// Position am Ende des Auftrags anfügen; liefert die Kennung.
pub async fn insert_position(
    conn: &mut PgConnection,
    order_id: i64,
    input: &PositionInput,
    values: &PositionValues,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO einkaufsposition (einkaufsauftrag_id, reihenfolge, bezeichnung, artikelnummer_lieferant, \
                                       menge, einheit, einzelpreis, rabatt_prozent, mwst_satz) \
         VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM einkaufsposition WHERE einkaufsauftrag_id = $1), \
                 $2, $3, $4::numeric / 1000, $5, $6::numeric / 100, $7::numeric / 100, $8::numeric / 100) \
         RETURNING id",
    )
    .bind(order_id)
    .bind(&input.bezeichnung)
    .bind(opt(&input.artikelnummer_lieferant))
    .bind(values.menge)
    .bind(&input.einheit)
    .bind(values.einzelpreis)
    .bind(values.rabatt_prozent)
    .bind(values.mwst_satz)
    .fetch_one(conn)
    .await
}

/// Position ändern.
pub async fn update_position(
    conn: &mut PgConnection,
    id: i64,
    input: &PositionInput,
    values: &PositionValues,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE einkaufsposition SET bezeichnung = $1, artikelnummer_lieferant = $2, menge = $3::numeric / 1000, \
                einheit = $4, einzelpreis = $5::numeric / 100, rabatt_prozent = $6::numeric / 100, \
                mwst_satz = $7::numeric / 100, geaendert_am = now() \
         WHERE id = $8",
    )
    .bind(&input.bezeichnung)
    .bind(opt(&input.artikelnummer_lieferant))
    .bind(values.menge)
    .bind(&input.einheit)
    .bind(values.einzelpreis)
    .bind(values.rabatt_prozent)
    .bind(values.mwst_satz)
    .bind(id)
    .execute(conn)
    .await?;
    Ok(())
}

/// Position löschen.
pub async fn delete_position(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM einkaufsposition WHERE id = $1")
        .bind(id)
        .execute(conn)
        .await?;
    Ok(())
}

/// Position innerhalb ihres Auftrags eine Stelle nach oben/unten schieben.
pub async fn move_position(
    conn: &mut PgConnection,
    order_id: i64,
    id: i64,
    nach_oben: bool,
) -> Result<(), sqlx::Error> {
    // Positionen lückenlos durchnummerieren, dann mit dem Nachbarn tauschen.
    sqlx::query(
        "UPDATE einkaufsposition t SET reihenfolge = r.rn \
         FROM (SELECT id, row_number() OVER (ORDER BY reihenfolge, id) AS rn \
               FROM einkaufsposition WHERE einkaufsauftrag_id = $1) r \
         WHERE t.id = r.id",
    )
    .bind(order_id)
    .execute(&mut *conn)
    .await?;
    let pos: i32 = sqlx::query_scalar("SELECT reihenfolge FROM einkaufsposition WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *conn)
        .await?;
    let other = if nach_oben { pos - 1 } else { pos + 1 };
    let swapped = sqlx::query(
        "UPDATE einkaufsposition SET reihenfolge = $3 WHERE einkaufsauftrag_id = $1 AND reihenfolge = $2",
    )
    .bind(order_id)
    .bind(other)
    .bind(pos)
    .execute(&mut *conn)
    .await?;
    if swapped.rows_affected() == 1 {
        sqlx::query("UPDATE einkaufsposition SET reihenfolge = $2 WHERE id = $1")
            .bind(id)
            .bind(other)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

/// Gelieferte Menge einer Position setzen (in Tausendsteln).
pub async fn set_delivered(
    conn: &mut PgConnection,
    position_id: i64,
    value: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE einkaufsposition SET gelieferte_menge = $1::numeric / 1000, geaendert_am = now() WHERE id = $2",
    )
    .bind(value)
    .bind(position_id)
    .execute(conn)
    .await?;
    Ok(())
}

/// Alle Positionen vollständig geliefert setzen (Überlieferungen bleiben).
pub async fn deliver_all_positions(
    conn: &mut PgConnection,
    order_id: i64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE einkaufsposition SET gelieferte_menge = greatest(gelieferte_menge, menge), geaendert_am = now() \
         WHERE einkaufsauftrag_id = $1",
    )
    .bind(order_id)
    .execute(conn)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_sort_falls_back_to_number() {
        assert_eq!(order_by("; DROP TABLE einkaufsauftrag"), "e.id");
    }
}
