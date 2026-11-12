//! SQL-Zugriffe für Projekte → Teilprojekte → Vorgänge → Aufgaben (ANF-003, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Fehler werden als `sqlx::Error` zurückgegeben; die Server-Funktionen übersetzen sie.

use sqlx::{AssertSqlSafe, PgConnection, PgPool};
use std::collections::HashMap;

use super::model::{ElementInput, Level, Node, Project, ProjectRow, ProjectTree, Responsible};
use crate::forms::PickOption;

fn table(level: Level) -> &'static str {
    level.key()
}

/// Spalte, die auf das übergeordnete Element zeigt.
fn parent_column(level: Level) -> Option<&'static str> {
    match level {
        Level::Project => None,
        Level::Subproject => Some("projekt_id"),
        Level::Process => Some("teilprojekt_id"),
        Level::Task => Some("vorgang_id"),
    }
}

/// Leerer Text → NULL in der Datenbank.
fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

const TS: &str = "to_char(x.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                  to_char(x.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am";

/// Verantwortlicher Benutzer (ANF-005); Tabelle mit Alias `x`, Benutzer `b`, dessen Adresse `ba`.
const RESPONSIBLE_COLUMNS: &str =
    "x.benutzer_id, b.benutzername, ba.anzeigename AS benutzer_name, b.aktiv AS benutzer_aktiv";
const RESPONSIBLE_JOIN: &str =
    "LEFT JOIN benutzer b ON b.id = x.benutzer_id LEFT JOIN adresse ba ON ba.id = b.adresse_id";

#[derive(sqlx::FromRow)]
struct NodeRow {
    id: i64,
    parent_id: i64,
    name: String,
    beschreibung: Option<String>,
    status: String,
    start: Option<String>,
    ende: Option<String>,
    benutzer_id: Option<i64>,
    benutzername: Option<String>,
    benutzer_name: Option<String>,
    benutzer_aktiv: Option<bool>,
    erstellt_am: String,
    geaendert_am: String,
}

/// Inhaltsspalten: Teilprojekt/Vorgang mit allen Feldern und verantwortlichem Benutzer,
/// Aufgabe nur Titel, Beschreibung und erledigt/offen (Tabelle mit Alias `x`).
const FULL_COLUMNS: &str = "x.name, x.beschreibung, x.status, \
    to_char(x.start, 'YYYY-MM-DD') AS start, to_char(x.ende, 'YYYY-MM-DD') AS ende";
const TASK_COLUMNS: &str = "x.name, x.beschreibung, x.status, NULL::text AS start, NULL::text AS ende, \
    NULL::bigint AS benutzer_id, NULL::text AS benutzername, NULL::text AS benutzer_name, \
    NULL::boolean AS benutzer_aktiv";

/// Suchbedingung der Projektliste; $1 = Suchbegriff, $2 = LIKE-Muster, $3 = Status (leer = alle).
const LIST_WHERE: &str = "($1 = '' OR p.nummer ILIKE $2 OR p.name ILIKE $2 OR a.anzeigename ILIKE $2) \
                          AND ($3 = '' OR p.status = $3)";

/// Sortierausdruck für einen Schlüssel aus der Webadresse (nur feste Werte → kein SQL-Einschleusen).
fn order_by(sort: &str) -> &'static str {
    match sort {
        "name" => "lower(p.name)",
        "kunde" => "lower(a.anzeigename)",
        "status" => {
            "array_position(ARRAY['geplant','aktiv','pausiert','abgeschlossen','abgebrochen'], p.status::text)"
        }
        "start" => "p.start",
        "ende" => "p.ende",
        _ => "p.id", // Nummer: in Vergabe-Reihenfolge
    }
}

/// Anzahl Projekte, die zu Suche und Statusfilter passen.
pub async fn count_projects(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    status: &str,
) -> Result<i64, sqlx::Error> {
    // Geprüft: nur feste Texte, Eingaben laufen als Parameter.
    sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM projekt p LEFT JOIN adresse a ON a.id = p.adresse_id WHERE {LIST_WHERE}"
    )))
    .bind(suche)
    .bind(pattern)
    .bind(status)
    .fetch_one(pool)
    .await
}

/// Eine Seite der Projektliste (ohne Struktur, siehe `load_children`).
#[allow(clippy::too_many_arguments)]
pub async fn list_projects_page(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    status: &str,
    sortierung: &str,
    ascending: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<ProjectRow>, sqlx::Error> {
    let direction = if ascending { "ASC" } else { "DESC" };
    // Geprüft: Sortierung aus fester Liste (order_by), Richtung ist ASC/DESC.
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT p.id, p.nummer, p.name, a.anzeigename AS kunde, p.status, \
                to_char(p.start, 'YYYY-MM-DD') AS start, to_char(p.ende, 'YYYY-MM-DD') AS ende, \
                p.benutzer_id, b.benutzername, ba.anzeigename AS benutzer_name, b.aktiv AS benutzer_aktiv \
         FROM projekt p LEFT JOIN adresse a ON a.id = p.adresse_id \
              LEFT JOIN benutzer b ON b.id = p.benutzer_id LEFT JOIN adresse ba ON ba.id = b.adresse_id \
         WHERE {LIST_WHERE} ORDER BY {} {direction} NULLS LAST, p.id DESC LIMIT $4 OFFSET $5",
        order_by(sortierung)
    )))
    .bind(suche)
    .bind(pattern)
    .bind(status)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Alle Elemente einer Ebene in den angegebenen Projekten, sortiert nach Reihenfolge.
async fn load_level(
    pool: &PgPool,
    level: Level,
    project_ids: &[i64],
) -> Result<Vec<NodeRow>, sqlx::Error> {
    let full = format!("{FULL_COLUMNS}, {RESPONSIBLE_COLUMNS}");
    let (columns, join, filter) = match level {
        Level::Subproject => (full.as_str(), RESPONSIBLE_JOIN, "x.projekt_id = ANY($1)"),
        Level::Process => (
            full.as_str(),
            RESPONSIBLE_JOIN,
            "x.teilprojekt_id IN (SELECT id FROM teilprojekt WHERE projekt_id = ANY($1))",
        ),
        _ => (
            TASK_COLUMNS,
            "",
            "x.vorgang_id IN (SELECT v.id FROM vorgang v JOIN teilprojekt t ON t.id = v.teilprojekt_id \
             WHERE t.projekt_id = ANY($1))",
        ),
    };
    // Geprüft: Tabellen-/Spaltennamen, Verknüpfungen und Bedingungen sind feste Texte je Ebene.
    let q = format!(
        "SELECT x.id, x.{parent} AS parent_id, {columns}, {TS} \
         FROM {table} x {join} WHERE {filter} ORDER BY x.reihenfolge, x.id",
        parent = parent_column(level).unwrap_or("id"),
        table = table(level),
    );
    sqlx::query_as(AssertSqlSafe(q))
        .bind(project_ids)
        .fetch_all(pool)
        .await
}

/// Struktur (Teilprojekte mit Vorgängen und Aufgaben) je Projekt – 3 Abfragen für beliebig viele Projekte.
pub async fn load_children(
    pool: &PgPool,
    project_ids: &[i64],
) -> Result<HashMap<i64, Vec<Node>>, sqlx::Error> {
    let subprojects = load_level(pool, Level::Subproject, project_ids).await?;
    let processes = load_level(pool, Level::Process, project_ids).await?;
    let tasks = load_level(pool, Level::Task, project_ids).await?;

    let mut tasks_by_parent: HashMap<i64, Vec<NodeRow>> = HashMap::new();
    for t in tasks {
        tasks_by_parent.entry(t.parent_id).or_default().push(t);
    }
    let mut processes_by_parent: HashMap<i64, Vec<NodeRow>> = HashMap::new();
    for p in processes {
        processes_by_parent.entry(p.parent_id).or_default().push(p);
    }
    let mut subprojects_by_project: HashMap<i64, Vec<NodeRow>> = HashMap::new();
    for sp in subprojects {
        subprojects_by_project
            .entry(sp.parent_id)
            .or_default()
            .push(sp);
    }

    let mut result = HashMap::new();
    for (project_id, sps) in subprojects_by_project {
        let children = sps
            .into_iter()
            .enumerate()
            .map(|(i, sp)| {
                let sp_no = format!("{}", i + 1);
                let procs = processes_by_parent.remove(&sp.id).unwrap_or_default();
                let proc_nodes = procs
                    .into_iter()
                    .enumerate()
                    .map(|(j, pr)| {
                        let pr_no = format!("{sp_no}.{}", j + 1);
                        let ts = tasks_by_parent.remove(&pr.id).unwrap_or_default();
                        let task_nodes = ts
                            .into_iter()
                            .enumerate()
                            .map(|(k, t)| {
                                node(t, Level::Task, format!("{pr_no}.{}", k + 1), vec![])
                            })
                            .collect();
                        node(pr, Level::Process, pr_no, task_nodes)
                    })
                    .collect();
                node(sp, Level::Subproject, sp_no, proc_nodes)
            })
            .collect();
        result.insert(project_id, children);
    }
    Ok(result)
}

fn node(r: NodeRow, level: Level, number: String, children: Vec<Node>) -> Node {
    Node {
        id: r.id,
        level,
        number,
        name: r.name,
        beschreibung: r.beschreibung,
        status: r.status,
        start: r.start,
        ende: r.ende,
        responsible: Responsible::from_columns(
            r.benutzer_id,
            r.benutzername,
            r.benutzer_name,
            r.benutzer_aktiv,
        ),
        erstellt_am: r.erstellt_am,
        geaendert_am: r.geaendert_am,
        children,
    }
}

/// Projekt-Kennung zu einem beliebigen Element.
pub async fn project_id_of(
    pool: &PgPool,
    level: Level,
    id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let q = match level {
        Level::Project => "SELECT id FROM projekt WHERE id = $1",
        Level::Subproject => "SELECT projekt_id FROM teilprojekt WHERE id = $1",
        Level::Process => {
            "SELECT t.projekt_id FROM vorgang v JOIN teilprojekt t ON t.id = v.teilprojekt_id WHERE v.id = $1"
        }
        Level::Task => {
            "SELECT t.projekt_id FROM aufgabe a JOIN vorgang v ON v.id = a.vorgang_id \
             JOIN teilprojekt t ON t.id = v.teilprojekt_id WHERE a.id = $1"
        }
    };
    sqlx::query_scalar(q).bind(id).fetch_optional(pool).await
}

/// Vollständiges Projekt mit Struktur; `None`, wenn es das Projekt nicht gibt.
pub async fn load_tree(pool: &PgPool, project_id: i64) -> Result<Option<ProjectTree>, sqlx::Error> {
    let project: Option<Project> = sqlx::query_as(
        "SELECT p.id, p.nummer, p.name, p.beschreibung, p.status, \
                to_char(p.start, 'YYYY-MM-DD') AS start, to_char(p.ende, 'YYYY-MM-DD') AS ende, \
                p.adresse_id, a.nummer AS kunde_nummer, a.anzeigename AS kunde, \
                p.benutzer_id, b.benutzername, ba.anzeigename AS benutzer_name, b.aktiv AS benutzer_aktiv, \
                to_char(p.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                to_char(p.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
         FROM projekt p LEFT JOIN adresse a ON a.id = p.adresse_id \
              LEFT JOIN benutzer b ON b.id = p.benutzer_id LEFT JOIN adresse ba ON ba.id = b.adresse_id \
         WHERE p.id = $1",
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await?;
    let Some(project) = project else {
        return Ok(None);
    };

    let children = load_children(pool, &[project_id])
        .await?
        .remove(&project_id)
        .unwrap_or_default();

    Ok(Some(ProjectTree { project, children }))
}

/// Höchstens 20 Treffer für die Projekt-Auswahl, neueste zuerst.
pub async fn search_options(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
) -> Result<Vec<PickOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT id, nummer, name FROM projekt \
         WHERE $1 = '' OR nummer ILIKE $2 OR name ILIKE $2 \
         ORDER BY id DESC LIMIT 20",
    )
    .bind(suche)
    .bind(pattern)
    .fetch_all(pool)
    .await
}

/// Element speichern; liefert die Kennung. Projekte erhalten eine Nummer P-JJJJ-NNN,
/// andere Elemente werden ans Ende ihres übergeordneten Elements gestellt.
/// Läuft in der Transaktion des Aufrufers (Nummernvergabe und Einfügen gemeinsam).
pub async fn insert_element(
    conn: &mut PgConnection,
    level: Level,
    parent_id: Option<i64>,
    input: &ElementInput,
) -> Result<i64, sqlx::Error> {
    match level {
        Level::Project => {
            let (year, n): (i32, i32) = sqlx::query_as(
                "INSERT INTO projekt_nummer (jahr, letzte) \
                 VALUES (extract(year FROM now() AT TIME ZONE 'Europe/Zurich')::int, 1) \
                 ON CONFLICT (jahr) DO UPDATE SET letzte = projekt_nummer.letzte + 1 \
                 RETURNING jahr, letzte",
            )
            .fetch_one(&mut *conn)
            .await?;
            sqlx::query_scalar(
                "INSERT INTO projekt (nummer, name, beschreibung, status, start, ende, adresse_id, benutzer_id) \
                 VALUES ($1, $2, $3, $4, $5::date, $6::date, $7, $8) RETURNING id",
            )
            .bind(format!("P-{year}-{n:03}"))
            .bind(&input.name)
            .bind(opt(&input.beschreibung))
            .bind(&input.status)
            .bind(opt(&input.start))
            .bind(opt(&input.ende))
            .bind(input.customer_id())
            .bind(input.responsible_id())
            .fetch_one(&mut *conn)
            .await
        }
        Level::Subproject | Level::Process => {
            let q = if level == Level::Subproject {
                "INSERT INTO teilprojekt (projekt_id, reihenfolge, name, beschreibung, status, start, ende, benutzer_id) \
                 VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM teilprojekt WHERE projekt_id = $1), \
                         $2, $3, $4, $5::date, $6::date, $7) RETURNING id"
            } else {
                "INSERT INTO vorgang (teilprojekt_id, reihenfolge, name, beschreibung, status, start, ende, benutzer_id) \
                 VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM vorgang WHERE teilprojekt_id = $1), \
                         $2, $3, $4, $5::date, $6::date, $7) RETURNING id"
            };
            sqlx::query_scalar(q)
                .bind(parent_id)
                .bind(&input.name)
                .bind(opt(&input.beschreibung))
                .bind(&input.status)
                .bind(opt(&input.start))
                .bind(opt(&input.ende))
                .bind(input.responsible_id())
                .fetch_one(&mut *conn)
                .await
        }
        Level::Task => {
            // Aufgabe: nur Titel und Beschreibung; Status startet „offen“ (Standardwert der Tabelle)
            sqlx::query_scalar(
                "INSERT INTO aufgabe (vorgang_id, reihenfolge, name, beschreibung) \
                 VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM aufgabe WHERE vorgang_id = $1), $2, $3) \
                 RETURNING id",
            )
            .bind(parent_id)
            .bind(&input.name)
            .bind(opt(&input.beschreibung))
            .fetch_one(&mut *conn)
            .await
        }
    }
}

/// Element ändern; liefert die Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update_element(
    pool: &PgPool,
    level: Level,
    id: i64,
    input: &ElementInput,
) -> Result<u64, sqlx::Error> {
    let result = match level {
        Level::Project => {
            sqlx::query(
                "UPDATE projekt SET name = $1, beschreibung = $2, status = $3, start = $4::date, ende = $5::date, \
                 adresse_id = $6, benutzer_id = $7, geaendert_am = now() WHERE id = $8",
            )
            .bind(&input.name)
            .bind(opt(&input.beschreibung))
            .bind(&input.status)
            .bind(opt(&input.start))
            .bind(opt(&input.ende))
            .bind(input.customer_id())
            .bind(input.responsible_id())
            .bind(id)
            .execute(pool)
            .await?
        }
        Level::Subproject | Level::Process => {
            let q = if level == Level::Subproject {
                "UPDATE teilprojekt SET name = $1, beschreibung = $2, status = $3, start = $4::date, \
                 ende = $5::date, benutzer_id = $6, geaendert_am = now() WHERE id = $7"
            } else {
                "UPDATE vorgang SET name = $1, beschreibung = $2, status = $3, start = $4::date, \
                 ende = $5::date, benutzer_id = $6, geaendert_am = now() WHERE id = $7"
            };
            sqlx::query(q)
                .bind(&input.name)
                .bind(opt(&input.beschreibung))
                .bind(&input.status)
                .bind(opt(&input.start))
                .bind(opt(&input.ende))
                .bind(input.responsible_id())
                .bind(id)
                .execute(pool)
                .await?
        }
        Level::Task => {
            // Aufgabe: Titel und Beschreibung änderbar; erledigt/offen über set_task_done
            sqlx::query(
                "UPDATE aufgabe SET name = $1, beschreibung = $2, geaendert_am = now() \
                 WHERE id = $3 AND vorgang_id IS NOT NULL",
            )
            .bind(&input.name)
            .bind(opt(&input.beschreibung))
            .bind(id)
                .execute(pool)
                .await?
        }
    };
    Ok(result.rows_affected())
}

/// Status eines Benutzers: `Some(true)` aktiv, `Some(false)` gesperrt, `None` = gibt es nicht (ANF-005 AK-24).
pub async fn user_active(pool: &PgPool, user_id: i64) -> Result<Option<bool>, sqlx::Error> {
    sqlx::query_scalar("SELECT aktiv FROM benutzer WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

/// Derzeit hinterlegter verantwortlicher Benutzer eines Projekts, Teilprojekts oder Vorgangs.
pub async fn responsible_of(
    pool: &PgPool,
    level: Level,
    id: i64,
) -> Result<Option<i64>, sqlx::Error> {
    let q = match level {
        Level::Project => "SELECT benutzer_id FROM projekt WHERE id = $1",
        Level::Subproject => "SELECT benutzer_id FROM teilprojekt WHERE id = $1",
        Level::Process => "SELECT benutzer_id FROM vorgang WHERE id = $1",
        Level::Task => return Ok(None),
    };
    let current: Option<Option<i64>> = sqlx::query_scalar(q).bind(id).fetch_optional(pool).await?;
    Ok(current.flatten())
}

/// Projekt löschen (Struktur über ON DELETE CASCADE); liefert die Nummer oder `None`.
pub async fn delete_project(pool: &PgPool, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("DELETE FROM projekt WHERE id = $1 RETURNING nummer")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Nummern der Einkaufsaufträge, die einem Projekt zugeordnet sind.
pub async fn order_numbers_of_project(
    pool: &PgPool,
    project_id: i64,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT nummer FROM einkaufsauftrag WHERE projekt_id = $1 ORDER BY id")
        .bind(project_id)
        .fetch_all(pool)
        .await
}

/// Teilprojekt, Vorgang oder Aufgabe löschen (alles darunter über ON DELETE CASCADE).
/// Liefert (Kennung des übergeordneten Elements, Name) oder `None`.
pub async fn delete_child(
    pool: &PgPool,
    level: Level,
    id: i64,
) -> Result<Option<(i64, String)>, sqlx::Error> {
    let q = match level {
        Level::Subproject => "DELETE FROM teilprojekt WHERE id = $1 RETURNING projekt_id, name",
        Level::Process => "DELETE FROM vorgang WHERE id = $1 RETURNING teilprojekt_id, name",
        // Aufgaben ohne Projekt (ANF-006) werden nur in der Navigation „Aufgaben“ gelöscht
        _ => {
            "DELETE FROM aufgabe WHERE id = $1 AND vorgang_id IS NOT NULL RETURNING vorgang_id, name"
        }
    };
    sqlx::query_as(q).bind(id).fetch_optional(pool).await
}

/// Aufgabe erledigt bzw. offen setzen.
pub async fn set_task_done(pool: &PgPool, id: i64, erledigt: bool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE aufgabe SET status = CASE WHEN $2 THEN 'erledigt' ELSE 'offen' END, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(erledigt)
    .execute(pool)
    .await?;
    Ok(())
}

/// Element innerhalb seines übergeordneten Elements eine Stelle nach oben/unten schieben.
/// Liefert `false`, wenn es das Element nicht gibt (oder es ein Projekt ist).
/// Läuft in der Transaktion des Aufrufers.
pub async fn move_element(
    conn: &mut PgConnection,
    level: Level,
    id: i64,
    nach_oben: bool,
) -> Result<bool, sqlx::Error> {
    let Some(parent_col) = parent_column(level) else {
        return Ok(false);
    };
    let table = table(level);

    // Geprüft: Tabellen- und Spaltennamen kommen aus festen Listen je Ebene.
    let parent: Option<i64> = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT {parent_col} FROM {table} WHERE id = $1 AND {parent_col} IS NOT NULL FOR UPDATE"
    )))
    .bind(id)
    .fetch_optional(&mut *conn)
    .await?;
    let Some(parent) = parent else {
        return Ok(false);
    };

    // Geschwister lückenlos durchnummerieren, dann mit dem Nachbarn tauschen.
    sqlx::query(AssertSqlSafe(format!(
        "UPDATE {table} t SET reihenfolge = r.rn \
         FROM (SELECT id, row_number() OVER (ORDER BY reihenfolge, id) AS rn FROM {table} WHERE {parent_col} = $1) r \
         WHERE t.id = r.id"
    )))
    .bind(parent)
    .execute(&mut *conn)
    .await?;

    let pos: i32 = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT reihenfolge FROM {table} WHERE id = $1"
    )))
    .bind(id)
    .fetch_one(&mut *conn)
    .await?;
    let other = if nach_oben { pos - 1 } else { pos + 1 };
    let swapped = sqlx::query(AssertSqlSafe(format!(
        "UPDATE {table} SET reihenfolge = $3 WHERE {parent_col} = $1 AND reihenfolge = $2"
    )))
    .bind(parent)
    .bind(other)
    .bind(pos)
    .execute(&mut *conn)
    .await?;
    if swapped.rows_affected() == 1 {
        sqlx::query(AssertSqlSafe(format!(
            "UPDATE {table} SET reihenfolge = $2 WHERE id = $1"
        )))
        .bind(id)
        .bind(other)
        .execute(&mut *conn)
        .await?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_sort_falls_back_to_number() {
        assert_eq!(order_by("; DROP TABLE projekt"), "p.id");
    }
}
