//! Server-Funktionen für Einkaufsaufträge (ANF-004).
//!
//! Jede Änderung sperrt den Einkaufsauftrag (`FOR UPDATE`) und prüft seinen Status auf dem Server,
//! damit z. B. ein bestellter Auftrag auch über direkte Aufrufe nicht mehr geändert werden kann.

use leptos::prelude::*;

use super::model::{OrderDetail, OrderInput, OrderPage, OrderRow, PositionInput, SaveOutcome};

#[cfg(feature = "ssr")]
mod sql {
    use super::super::model::{OrderDetail, OrderRow, Position};
    use leptos::prelude::ServerFnError;
    use sqlx::{PgConnection, PgPool};
    use std::collections::HashMap;

    /// Liefertermin überschritten und Auftrag noch offen (AK-5). Ohne Liefertermin: nicht überfällig.
    pub const OVERDUE: &str = "coalesce(e.liefertermin < (now() AT TIME ZONE 'Europe/Zurich')::date \
         AND e.status IN ('entwurf', 'bestellt', 'teilweise_geliefert'), false)";

    /// Positionen mit Werten in kleinsten Einheiten (siehe model.rs).
    const POSITION_COLUMNS: &str = "id, bezeichnung, artikelnummer_lieferant, \
         (menge * 1000)::int8 AS menge, einheit, (einzelpreis * 100)::int8 AS einzelpreis, \
         (rabatt_prozent * 100)::int8 AS rabatt_prozent, (mwst_satz * 100)::int8 AS mwst_satz, \
         (gelieferte_menge * 1000)::int8 AS gelieferte_menge";

    /// Spalten einer Listenzeile (für Liste und Projekt-Detailansicht).
    pub const ROW_COLUMNS: &str = "e.id, e.nummer, a.anzeigename AS lieferant, e.projekt_id, \
         p.nummer AS projekt_nummer, e.status, \
         to_char(e.bestelldatum, 'YYYY-MM-DD') AS bestelldatum, \
         to_char(e.liefertermin, 'YYYY-MM-DD') AS liefertermin, e.waehrung";

    pub const FROM: &str = "einkaufsauftrag e JOIN adresse a ON a.id = e.adresse_id \
         LEFT JOIN projekt p ON p.id = e.projekt_id";

    #[derive(sqlx::FromRow)]
    struct PositionOfOrder {
        einkaufsauftrag_id: i64,
        #[sqlx(flatten)]
        position: Position,
    }

    pub async fn load_positions(
        conn: &mut PgConnection,
        order_id: i64,
    ) -> Result<Vec<Position>, sqlx::Error> {
        // Geprüft: nur feste Spaltenliste
        sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "SELECT {POSITION_COLUMNS} FROM einkaufsposition \
             WHERE einkaufsauftrag_id = $1 ORDER BY reihenfolge, id"
        )))
        .bind(order_id)
        .fetch_all(conn)
        .await
    }

    /// Gesamtbeträge der Zeilen aus ihren Positionen berechnen (gleiche Rechnung wie in der Detailansicht).
    pub async fn fill_totals(pool: &PgPool, rows: &mut [OrderRow]) -> Result<(), sqlx::Error> {
        let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
        let positions: Vec<PositionOfOrder> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
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
            row.total = super::super::model::totals(&positions, &row.waehrung).total;
        }
        Ok(())
    }

    pub async fn load_detail(
        conn: &mut PgConnection,
        id: i64,
    ) -> Result<Option<OrderDetail>, sqlx::Error> {
        let order = sqlx::query_as(sqlx::AssertSqlSafe(format!(
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

    /// Sperrt den Einkaufsauftrag bis zum Ende der Transaktion und liefert seinen Status.
    pub async fn lock_order(conn: &mut PgConnection, id: i64) -> Result<String, ServerFnError> {
        sqlx::query_scalar("SELECT status FROM einkaufsauftrag WHERE id = $1 FOR UPDATE")
            .bind(id)
            .fetch_optional(conn)
            .await?
            .ok_or_else(not_found)
    }

    /// Einkaufsauftrag einer Position sperren: (Kennung des Auftrags, Status).
    pub async fn lock_order_of_position(
        conn: &mut PgConnection,
        position_id: i64,
    ) -> Result<(i64, String), ServerFnError> {
        sqlx::query_as(
            "SELECT e.id, e.status FROM einkaufsposition p \
             JOIN einkaufsauftrag e ON e.id = p.einkaufsauftrag_id \
             WHERE p.id = $1 FOR UPDATE OF e",
        )
        .bind(position_id)
        .fetch_optional(conn)
        .await?
        .ok_or_else(|| ServerFnError::new("Position nicht gefunden."))
    }

    /// Auftrag als geändert markieren (auch bei Änderungen an Positionen).
    pub async fn touch(conn: &mut PgConnection, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE einkaufsauftrag SET geaendert_am = now() WHERE id = $1")
            .bind(id)
            .execute(conn)
            .await?;
        Ok(())
    }

    pub fn not_found() -> ServerFnError {
        ServerFnError::new("Einkaufsauftrag nicht gefunden.")
    }

    /// Positionen und Lieferant sind nur im Entwurf änderbar (AK-13, AK-15).
    pub fn require_draft(status: &str) -> Result<(), ServerFnError> {
        if status == "entwurf" {
            Ok(())
        } else {
            Err(ServerFnError::new(
                "Der Einkaufsauftrag ist nicht mehr im Entwurf; Positionen können nicht mehr geändert werden.",
            ))
        }
    }

    /// Wareneingang nur bei bestellten, noch nicht vollständig gelieferten Aufträgen (AK-16).
    pub fn require_ordered(status: &str) -> Result<(), ServerFnError> {
        if matches!(status, "bestellt" | "teilweise_geliefert") {
            Ok(())
        } else {
            Err(ServerFnError::new(
                "Liefermengen können nur bei bestellten Aufträgen erfasst werden.",
            ))
        }
    }

    pub fn opt(s: &str) -> Option<&str> {
        (!s.is_empty()).then_some(s)
    }

    /// Fehlercode der Datenbank und Name der verletzten Regel.
    pub fn db_error(e: &sqlx::Error) -> Option<(String, String)> {
        match e {
            sqlx::Error::Database(db) => Some((
                db.code().map(|c| c.to_string()).unwrap_or_default(),
                db.constraint().unwrap_or_default().to_string(),
            )),
            _ => None,
        }
    }
}

/// Liste mit Suche, Statusfilter, Sortierung und Seiten (AK-1 bis AK-5).
/// `status`: „offen“, „alle“ oder ein einzelner Status.
#[server]
pub async fn list_orders(
    suche: String,
    status: String,
    sortierung: String,
    richtung: String,
    seite: i64,
) -> Result<OrderPage, ServerFnError> {
    use super::model::PAGE_SIZE;
    use sqlx::AssertSqlSafe;

    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    const WHERE: &str = "($1 = '' OR e.nummer ILIKE $2 OR a.anzeigename ILIKE $2 OR p.nummer ILIKE $2 \
                            OR p.name ILIKE $2 OR e.referenz_lieferant ILIKE $2) \
                         AND ($3 = 'alle' \
                            OR ($3 = 'offen' AND e.status IN ('entwurf', 'bestellt', 'teilweise_geliefert')) \
                            OR e.status = $3)";

    // Geprüft: nur feste Texte, Eingaben laufen als Parameter.
    let total: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM {} WHERE {WHERE}",
        sql::FROM
    )))
    .bind(&suche)
    .bind(&pattern)
    .bind(&status)
    .fetch_one(&pool)
    .await?;

    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = seite.clamp(1, page_count);
    // Geprüft: nur feste Ausdrücke aus dieser Liste.
    let order = match sortierung.as_str() {
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
    };
    let direction = if richtung == "auf" { "ASC" } else { "DESC" };

    let mut rows: Vec<OrderRow> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {}, {} AS ueberfaellig FROM {} WHERE {WHERE} \
         ORDER BY {order} {direction} NULLS LAST, e.id DESC LIMIT $4 OFFSET $5",
        sql::ROW_COLUMNS,
        sql::OVERDUE,
        sql::FROM
    )))
    .bind(&suche)
    .bind(&pattern)
    .bind(&status)
    .bind(PAGE_SIZE)
    .bind((page - 1) * PAGE_SIZE)
    .fetch_all(&pool)
    .await?;
    sql::fill_totals(&pool, &mut rows).await?;

    Ok(OrderPage { rows, total, page })
}

/// Einkaufsaufträge eines Projekts (AK-22), neueste zuerst.
#[server]
pub async fn list_project_orders(projekt_id: i64) -> Result<Vec<OrderRow>, ServerFnError> {
    use sqlx::AssertSqlSafe;
    let pool = crate::db::pool()?;
    let mut rows: Vec<OrderRow> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT {}, {} AS ueberfaellig FROM {} WHERE e.projekt_id = $1 ORDER BY e.id DESC",
        sql::ROW_COLUMNS,
        sql::OVERDUE,
        sql::FROM
    )))
    .bind(projekt_id)
    .fetch_all(&pool)
    .await?;
    sql::fill_totals(&pool, &mut rows).await?;
    Ok(rows)
}

/// Ein Einkaufsauftrag mit Positionen (AK-6, AK-7, AK-24). `None`, wenn es ihn nicht gibt.
#[server]
pub async fn get_order(id: i64) -> Result<Option<OrderDetail>, ServerFnError> {
    let pool = crate::db::pool()?;
    let mut conn = pool.acquire().await?;
    Ok(sql::load_detail(&mut conn, id).await?)
}

/// Fehler bei Lieferant oder Projekt (gelöscht, während das Formular offen war) als Feldfehler melden.
#[cfg(feature = "ssr")]
fn reference_outcome(e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    match sql::db_error(&e) {
        Some((code, constraint)) if code == "23503" => {
            let mut errors = super::model::FieldErrors::new();
            if constraint.contains("projekt") {
                errors.insert(
                    "projekt_id".into(),
                    "Dieses Projekt gibt es nicht mehr.".into(),
                );
            } else {
                errors.insert(
                    "adresse_id".into(),
                    "Diese Adresse gibt es nicht mehr.".into(),
                );
            }
            Ok(SaveOutcome::Invalid(errors))
        }
        _ => Err(e.into()),
    }
}

#[cfg(feature = "ssr")]
fn redirect_to_order(id: i64) {
    leptos_axum::redirect(&format!("/einkauf/{id}?gespeichert=1"));
}

/// Neuen Einkaufsauftrag im Status „Entwurf“ anlegen, Nummer E-JJJJ-NNNN (AK-8 bis AK-10, AK-12).
#[server]
pub async fn create_order(input: OrderInput) -> Result<SaveOutcome, ServerFnError> {
    use sql::opt;

    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (year, n): (i32, i32) = sqlx::query_as(
        "INSERT INTO einkaufsauftrag_nummer (jahr, letzte) \
         VALUES (extract(year FROM now() AT TIME ZONE 'Europe/Zurich')::int, 1) \
         ON CONFLICT (jahr) DO UPDATE SET letzte = einkaufsauftrag_nummer.letzte + 1 \
         RETURNING jahr, letzte",
    )
    .fetch_one(&mut *tx)
    .await?;
    let result: Result<i64, sqlx::Error> = sqlx::query_scalar(
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
    .fetch_one(&mut *tx)
    .await;
    let id = match result {
        Ok(id) => id,
        Err(e) => return reference_outcome(e),
    };
    tx.commit().await?;
    redirect_to_order(id);
    Ok(SaveOutcome::Saved(id))
}

/// Kopfdaten ändern (AK-13, AK-15). Nach dem Bestellen werden nur Liefertermin, Referenz und Bemerkung übernommen.
#[server]
pub async fn update_order(id: i64, input: OrderInput) -> Result<SaveOutcome, ServerFnError> {
    use sql::opt;

    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    sql::lock_order(&mut tx, id).await?;
    let current = sql::load_detail(&mut tx, id)
        .await?
        .ok_or_else(sql::not_found)?
        .order;

    let input = input.normalized();
    let input = if current.is_draft() {
        input
    } else if current.is_ordered() {
        input.restricted_to_ordered(&current.to_input())
    } else {
        return Err(ServerFnError::new(
            "Der Einkaufsauftrag ist abgeschlossen und kann nicht mehr geändert werden.",
        ));
    };
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    let result = sqlx::query(
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
    .execute(&mut *tx)
    .await;
    if let Err(e) = result {
        return reference_outcome(e);
    }
    tx.commit().await?;
    redirect_to_order(id);
    Ok(SaveOutcome::Saved(id))
}

/// Position am Ende anfügen (AK-11, AK-12). Nur im Entwurf.
#[server]
pub async fn create_position(
    auftrag_id: i64,
    input: PositionInput,
) -> Result<SaveOutcome, ServerFnError> {
    use sql::opt;

    let input = input.normalized();
    let values = match input.validate() {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    sql::require_draft(&sql::lock_order(&mut tx, auftrag_id).await?)?;
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO einkaufsposition (einkaufsauftrag_id, reihenfolge, bezeichnung, artikelnummer_lieferant, \
                                       menge, einheit, einzelpreis, rabatt_prozent, mwst_satz) \
         VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM einkaufsposition WHERE einkaufsauftrag_id = $1), \
                 $2, $3, $4::numeric / 1000, $5, $6::numeric / 100, $7::numeric / 100, $8::numeric / 100) \
         RETURNING id",
    )
    .bind(auftrag_id)
    .bind(&input.bezeichnung)
    .bind(opt(&input.artikelnummer_lieferant))
    .bind(values.menge)
    .bind(&input.einheit)
    .bind(values.einzelpreis)
    .bind(values.rabatt_prozent)
    .bind(values.mwst_satz)
    .fetch_one(&mut *tx)
    .await?;
    sql::touch(&mut tx, auftrag_id).await?;
    tx.commit().await?;
    redirect_to_order(auftrag_id);
    Ok(SaveOutcome::Saved(id))
}

/// Position ändern (AK-13). Nur im Entwurf.
#[server]
pub async fn update_position(id: i64, input: PositionInput) -> Result<SaveOutcome, ServerFnError> {
    use sql::opt;

    let input = input.normalized();
    let values = match input.validate() {
        Ok(v) => v,
        Err(errors) => return Ok(SaveOutcome::Invalid(errors)),
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = sql::lock_order_of_position(&mut tx, id).await?;
    sql::require_draft(&status)?;
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
    .execute(&mut *tx)
    .await?;
    sql::touch(&mut tx, order_id).await?;
    tx.commit().await?;
    redirect_to_order(order_id);
    Ok(SaveOutcome::Saved(id))
}

/// Position löschen (AK-13). Nur im Entwurf.
#[server]
pub async fn delete_position(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = sql::lock_order_of_position(&mut tx, id).await?;
    sql::require_draft(&status)?;
    sqlx::query("DELETE FROM einkaufsposition WHERE id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    sql::touch(&mut tx, order_id).await?;
    tx.commit().await?;
    Ok(())
}

/// Position eine Stelle nach oben/unten schieben (AK-13). Nur im Entwurf.
#[server]
pub async fn move_position(id: i64, nach_oben: bool) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = sql::lock_order_of_position(&mut tx, id).await?;
    sql::require_draft(&status)?;

    // Positionen lückenlos durchnummerieren, dann mit dem Nachbarn tauschen.
    sqlx::query(
        "UPDATE einkaufsposition t SET reihenfolge = r.rn \
         FROM (SELECT id, row_number() OVER (ORDER BY reihenfolge, id) AS rn \
               FROM einkaufsposition WHERE einkaufsauftrag_id = $1) r \
         WHERE t.id = r.id",
    )
    .bind(order_id)
    .execute(&mut *tx)
    .await?;
    let pos: i32 = sqlx::query_scalar("SELECT reihenfolge FROM einkaufsposition WHERE id = $1")
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
    let other = if nach_oben { pos - 1 } else { pos + 1 };
    let swapped = sqlx::query(
        "UPDATE einkaufsposition SET reihenfolge = $3 WHERE einkaufsauftrag_id = $1 AND reihenfolge = $2",
    )
    .bind(order_id)
    .bind(other)
    .bind(pos)
    .execute(&mut *tx)
    .await?;
    if swapped.rows_affected() == 1 {
        sqlx::query("UPDATE einkaufsposition SET reihenfolge = $2 WHERE id = $1")
            .bind(id)
            .bind(other)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(())
}

/// Bestellen: Entwurf → Bestellt, Bestelldatum = heute, falls leer (AK-14).
#[server]
pub async fn place_order(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let status = sql::lock_order(&mut tx, id).await?;
    if status != "entwurf" {
        return Err(ServerFnError::new(
            "Der Einkaufsauftrag wurde bereits bestellt.",
        ));
    }
    let count: i64 =
        sqlx::query_scalar("SELECT count(*) FROM einkaufsposition WHERE einkaufsauftrag_id = $1")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    if count == 0 {
        return Err(ServerFnError::new(
            "Ohne Positionen kann nicht bestellt werden. Bitte zuerst eine Position erfassen.",
        ));
    }
    let result = sqlx::query(
        "UPDATE einkaufsauftrag SET status = 'bestellt', \
                bestelldatum = coalesce(bestelldatum, (now() AT TIME ZONE 'Europe/Zurich')::date), \
                geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await;
    match result {
        Ok(_) => {}
        Err(e) if sql::db_error(&e).is_some_and(|(code, _)| code == "23514") => {
            return Err(ServerFnError::new(
                "Der Liefertermin liegt vor dem Bestelldatum (heute). Bitte zuerst den Liefertermin anpassen.",
            ));
        }
        Err(e) => return Err(e.into()),
    }
    tx.commit().await?;
    Ok(())
}

/// Gelieferte Menge einer Position erfassen und Status neu bestimmen (AK-16 bis AK-18).
#[server]
pub async fn record_delivery(id: i64, geliefert: String) -> Result<SaveOutcome, ServerFnError> {
    use super::model::{delivery_status, parse_delivered};

    let value = match parse_delivered(&geliefert) {
        Ok(v) => v,
        Err(message) => {
            let mut errors = super::model::FieldErrors::new();
            errors.insert("geliefert".into(), message);
            return Ok(SaveOutcome::Invalid(errors));
        }
    };
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let (order_id, status) = sql::lock_order_of_position(&mut tx, id).await?;
    sql::require_ordered(&status)?;
    sqlx::query(
        "UPDATE einkaufsposition SET gelieferte_menge = $1::numeric / 1000, geaendert_am = now() WHERE id = $2",
    )
    .bind(value)
    .bind(id)
    .execute(&mut *tx)
    .await?;
    let positions = sql::load_positions(&mut tx, order_id).await?;
    sqlx::query("UPDATE einkaufsauftrag SET status = $1, geaendert_am = now() WHERE id = $2")
        .bind(delivery_status(&positions))
        .bind(order_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    redirect_to_order(order_id);
    Ok(SaveOutcome::Saved(id))
}

/// Alle Positionen vollständig geliefert: Liefermenge = bestellte Menge (Überlieferungen bleiben), Status „Geliefert“.
#[server]
pub async fn deliver_all(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    sql::require_ordered(&sql::lock_order(&mut tx, id).await?)?;
    sqlx::query(
        "UPDATE einkaufsposition SET gelieferte_menge = greatest(gelieferte_menge, menge), geaendert_am = now() \
         WHERE einkaufsauftrag_id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE einkaufsauftrag SET status = 'geliefert', geaendert_am = now() WHERE id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Stornieren (AK-19): nur im Entwurf oder bestellt, solange nichts geliefert ist.
#[server]
pub async fn cancel_order(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    sql::lock_order(&mut tx, id).await?;
    let detail = sql::load_detail(&mut tx, id)
        .await?
        .ok_or_else(sql::not_found)?;
    if !detail.order.can_cancel(&detail.positions) {
        return Err(ServerFnError::new(
            "Nur Einkaufsaufträge im Entwurf oder bestellte Aufträge ohne Lieferung können storniert werden.",
        ));
    }
    sqlx::query(
        "UPDATE einkaufsauftrag SET status = 'storniert', geaendert_am = now() WHERE id = $1",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Löschen (AK-20): nur im Entwurf; Positionen werden mitgelöscht.
#[server]
pub async fn delete_order(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;
    let status = sql::lock_order(&mut tx, id).await?;
    if status != "entwurf" {
        return Err(ServerFnError::new(
            "Nur Einkaufsaufträge im Entwurf können gelöscht werden. Bitte stattdessen stornieren.",
        ));
    }
    let nummer: String =
        sqlx::query_scalar("DELETE FROM einkaufsauftrag WHERE id = $1 RETURNING nummer")
            .bind(id)
            .fetch_one(&mut *tx)
            .await?;
    tx.commit().await?;
    leptos_axum::redirect(&format!(
        "/einkauf?geloescht={}",
        crate::forms::encode(&nummer)
    ));
    Ok(())
}
