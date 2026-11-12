//! Server-Funktionen für Projekte (ANF-003).

use leptos::prelude::*;

use super::model::{ElementInput, Level, PickOption, ProjectPage, ProjectTree, SaveOutcome};

#[cfg(feature = "ssr")]
mod sql {
    use super::super::model::{Level, Node, Project, ProjectTree};
    use sqlx::{AssertSqlSafe, PgPool};
    use std::collections::HashMap;

    pub fn table(level: Level) -> &'static str {
        level.key()
    }

    /// Spalte, die auf das übergeordnete Element zeigt.
    pub fn parent_column(level: Level) -> Option<&'static str> {
        match level {
            Level::Project => None,
            Level::Subproject => Some("projekt_id"),
            Level::Process => Some("teilprojekt_id"),
            Level::Task => Some("vorgang_id"),
        }
    }

    const TS: &str = "to_char(erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                      to_char(geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am";

    #[derive(sqlx::FromRow)]
    pub struct NodeRow {
        pub id: i64,
        pub parent_id: i64,
        pub name: String,
        pub beschreibung: Option<String>,
        pub status: String,
        pub start: Option<String>,
        pub ende: Option<String>,
        pub erstellt_am: String,
        pub geaendert_am: String,
    }

    /// Inhaltsspalten: Teilprojekt/Vorgang mit allen Feldern, Aufgabe nur Titel und erledigt/offen.
    const FULL_COLUMNS: &str = "name, beschreibung, status, \
        to_char(start, 'YYYY-MM-DD') AS start, to_char(ende, 'YYYY-MM-DD') AS ende";
    const TASK_COLUMNS: &str =
        "name, NULL::text AS beschreibung, status, NULL::text AS start, NULL::text AS ende";

    /// Alle Elemente einer Ebene in den angegebenen Projekten, sortiert nach Reihenfolge.
    async fn load_level(
        pool: &PgPool,
        level: Level,
        project_ids: &[i64],
    ) -> Result<Vec<NodeRow>, sqlx::Error> {
        let (columns, filter) = match level {
            Level::Subproject => (FULL_COLUMNS, "projekt_id = ANY($1)"),
            Level::Process => (
                FULL_COLUMNS,
                "teilprojekt_id IN (SELECT id FROM teilprojekt WHERE projekt_id = ANY($1))",
            ),
            _ => (
                TASK_COLUMNS,
                "vorgang_id IN (SELECT v.id FROM vorgang v JOIN teilprojekt t ON t.id = v.teilprojekt_id \
                 WHERE t.projekt_id = ANY($1))",
            ),
        };
        // Geprüft: Tabellen-/Spaltennamen und Bedingungen sind feste Texte je Ebene.
        let q = format!(
            "SELECT id, {parent} AS parent_id, {columns}, {TS} \
             FROM {table} WHERE {filter} ORDER BY reihenfolge, id",
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

    pub async fn load_tree(
        pool: &PgPool,
        project_id: i64,
    ) -> Result<Option<ProjectTree>, sqlx::Error> {
        let project: Option<Project> = sqlx::query_as(
            "SELECT p.id, p.nummer, p.name, p.beschreibung, p.status, \
                    to_char(p.start, 'YYYY-MM-DD') AS start, to_char(p.ende, 'YYYY-MM-DD') AS ende, \
                    p.adresse_id, a.nummer AS kunde_nummer, a.anzeigename AS kunde, \
                    to_char(p.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                    to_char(p.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
             FROM projekt p LEFT JOIN adresse a ON a.id = p.adresse_id WHERE p.id = $1",
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
            erstellt_am: r.erstellt_am,
            geaendert_am: r.geaendert_am,
            children,
        }
    }

    pub fn opt(s: &str) -> Option<&str> {
        (!s.is_empty()).then_some(s)
    }
}

/// Projektliste mit Suche, Statusfilter, Sortierung und Seiten (AK-1 bis AK-3).
/// `status`: leer = alle Status.
#[server]
pub async fn list_projects(
    suche: String,
    status: String,
    sortierung: String,
    richtung: String,
    seite: i64,
) -> Result<ProjectPage, ServerFnError> {
    use super::model::{PAGE_SIZE, ProjectRow};
    use sqlx::AssertSqlSafe;

    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);
    const WHERE: &str = "($1 = '' OR p.nummer ILIKE $2 OR p.name ILIKE $2 OR a.anzeigename ILIKE $2) \
                         AND ($3 = '' OR p.status = $3)";

    let total: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM projekt p LEFT JOIN adresse a ON a.id = p.adresse_id WHERE {WHERE}"
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
        "name" => "lower(p.name)",
        "kunde" => "lower(a.anzeigename)",
        "status" => {
            "array_position(ARRAY['geplant','aktiv','pausiert','abgeschlossen','abgebrochen'], p.status::text)"
        }
        "start" => "p.start",
        "ende" => "p.ende",
        _ => "p.id", // Nummer: in Vergabe-Reihenfolge
    };
    let direction = if richtung == "auf" { "ASC" } else { "DESC" };

    let rows: Vec<ProjectRow> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT p.id, p.nummer, p.name, a.anzeigename AS kunde, p.status, \
                to_char(p.start, 'YYYY-MM-DD') AS start, to_char(p.ende, 'YYYY-MM-DD') AS ende \
         FROM projekt p LEFT JOIN adresse a ON a.id = p.adresse_id \
         WHERE {WHERE} ORDER BY {order} {direction} NULLS LAST, p.id DESC LIMIT $4 OFFSET $5"
    )))
    .bind(&suche)
    .bind(&pattern)
    .bind(&status)
    .bind(PAGE_SIZE)
    .bind((page - 1) * PAGE_SIZE)
    .fetch_all(&pool)
    .await?;

    // Struktur aller Projekte der Seite für die Übersicht
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let mut children = sql::load_children(&pool, &ids).await?;
    let mut rows = rows;
    for row in &mut rows {
        row.children = children.remove(&row.id).unwrap_or_default();
    }

    Ok(ProjectPage { rows, total, page })
}

/// Vollständiges Projekt, zu dem das Element gehört (AK-4, AK-6, AK-17). `None`, wenn es das Element nicht gibt.
#[server]
pub async fn get_tree(level: Level, id: i64) -> Result<Option<ProjectTree>, ServerFnError> {
    let pool = crate::db::pool()?;
    let Some(project_id) = sql::project_id_of(&pool, level, id).await? else {
        return Ok(None);
    };
    Ok(sql::load_tree(&pool, project_id).await?)
}

/// Projekt-Auswahl (z. B. im Einkaufsauftrag, ANF-004): nach Nummer oder Name, neueste zuerst.
#[server]
pub async fn search_projects(suche: String) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let rows = sqlx::query_as(
        "SELECT id, nummer, name FROM projekt \
         WHERE $1 = '' OR nummer ILIKE $2 OR name ILIKE $2 \
         ORDER BY id DESC LIMIT 20",
    )
    .bind(&suche)
    .bind(crate::db::like_pattern(&suche))
    .fetch_all(&pool)
    .await?;
    Ok(rows)
}

/// Element anlegen (AK-7 bis AK-10). Projekte erhalten eine Nummer P-JJJJ-NNN,
/// andere Elemente werden ans Ende ihres übergeordneten Elements gestellt.
#[server]
pub async fn create_element(
    level: Level,
    parent_id: Option<i64>,
    input: ElementInput,
    zurueck: Option<String>,
) -> Result<SaveOutcome, ServerFnError> {
    use sql::opt;

    let input = input.normalized(level);
    let errors = input.validate(level);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;

    let result: Result<i64, sqlx::Error> = match level {
        Level::Project => {
            let (year, n): (i32, i32) = sqlx::query_as(
                "INSERT INTO projekt_nummer (jahr, letzte) \
                 VALUES (extract(year FROM now() AT TIME ZONE 'Europe/Zurich')::int, 1) \
                 ON CONFLICT (jahr) DO UPDATE SET letzte = projekt_nummer.letzte + 1 \
                 RETURNING jahr, letzte",
            )
            .fetch_one(&mut *tx)
            .await?;
            sqlx::query_scalar(
                "INSERT INTO projekt (nummer, name, beschreibung, status, start, ende, adresse_id) \
                 VALUES ($1, $2, $3, $4, $5::date, $6::date, $7) RETURNING id",
            )
            .bind(format!("P-{year}-{n:03}"))
            .bind(&input.name)
            .bind(opt(&input.beschreibung))
            .bind(&input.status)
            .bind(opt(&input.start))
            .bind(opt(&input.ende))
            .bind(input.customer_id())
            .fetch_one(&mut *tx)
            .await
        }
        Level::Subproject | Level::Process => {
            let q = if level == Level::Subproject {
                "INSERT INTO teilprojekt (projekt_id, reihenfolge, name, beschreibung, status, start, ende) \
                 VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM teilprojekt WHERE projekt_id = $1), \
                         $2, $3, $4, $5::date, $6::date) RETURNING id"
            } else {
                "INSERT INTO vorgang (teilprojekt_id, reihenfolge, name, beschreibung, status, start, ende) \
                 VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM vorgang WHERE teilprojekt_id = $1), \
                         $2, $3, $4, $5::date, $6::date) RETURNING id"
            };
            sqlx::query_scalar(q)
                .bind(parent_id)
                .bind(&input.name)
                .bind(opt(&input.beschreibung))
                .bind(&input.status)
                .bind(opt(&input.start))
                .bind(opt(&input.ende))
                .fetch_one(&mut *tx)
                .await
        }
        Level::Task => {
            // Aufgabe: nur Titel; Status startet „offen“ (Standardwert der Tabelle)
            sqlx::query_scalar(
                "INSERT INTO aufgabe (vorgang_id, reihenfolge, name) \
                 VALUES ($1, (SELECT coalesce(max(reihenfolge), 0) + 1 FROM aufgabe WHERE vorgang_id = $1), $2) \
                 RETURNING id",
            )
            .bind(parent_id)
            .bind(&input.name)
            .fetch_one(&mut *tx)
            .await
        }
    };

    let id = match result {
        Ok(id) => id,
        Err(e) => return foreign_key_outcome(level, e),
    };
    tx.commit().await?;
    redirect_after_save(level, id, zurueck.as_deref());
    Ok(SaveOutcome::Saved(id))
}

/// Nach dem Speichern zurück zur Seite unter dem Modal-Fenster (`zurueck`), sonst zur Detailansicht.
#[cfg(feature = "ssr")]
fn redirect_after_save(level: Level, id: i64, zurueck: Option<&str>) {
    let target = zurueck
        .and_then(crate::forms::safe_return_url)
        .map_or_else(|| level.url(id), str::to_string);
    leptos_axum::redirect(&crate::forms::with_params(
        &target,
        &[("gespeichert", "1".into())],
    ));
}

/// Fremdschlüssel-Fehler beim Speichern verständlich melden.
#[cfg(feature = "ssr")]
fn foreign_key_outcome(level: Level, e: sqlx::Error) -> Result<SaveOutcome, ServerFnError> {
    match &e {
        sqlx::Error::Database(db) if db.code().as_deref() == Some("23503") => {
            if level == Level::Project {
                let mut errors = super::model::FieldErrors::new();
                errors.insert(
                    "adresse_id".into(),
                    "Diese Adresse gibt es nicht mehr.".into(),
                );
                Ok(SaveOutcome::Invalid(errors))
            } else {
                Err(ServerFnError::new(
                    "Das übergeordnete Element gibt es nicht mehr.",
                ))
            }
        }
        _ => Err(e.into()),
    }
}

/// Element ändern (AK-11).
#[server]
pub async fn update_element(
    level: Level,
    id: i64,
    input: ElementInput,
    zurueck: Option<String>,
) -> Result<SaveOutcome, ServerFnError> {
    use sql::opt;

    let input = input.normalized(level);
    let errors = input.validate(level);
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }
    let pool = crate::db::pool()?;

    let result = match level {
        Level::Project => {
            sqlx::query(
                "UPDATE projekt SET name = $1, beschreibung = $2, status = $3, start = $4::date, ende = $5::date, \
                 adresse_id = $6, geaendert_am = now() WHERE id = $7",
            )
            .bind(&input.name)
            .bind(opt(&input.beschreibung))
            .bind(&input.status)
            .bind(opt(&input.start))
            .bind(opt(&input.ende))
            .bind(input.customer_id())
            .bind(id)
            .execute(&pool)
            .await
        }
        Level::Subproject | Level::Process => {
            let q = if level == Level::Subproject {
                "UPDATE teilprojekt SET name = $1, beschreibung = $2, status = $3, start = $4::date, \
                 ende = $5::date, geaendert_am = now() WHERE id = $6"
            } else {
                "UPDATE vorgang SET name = $1, beschreibung = $2, status = $3, start = $4::date, \
                 ende = $5::date, geaendert_am = now() WHERE id = $6"
            };
            sqlx::query(q)
                .bind(&input.name)
                .bind(opt(&input.beschreibung))
                .bind(&input.status)
                .bind(opt(&input.start))
                .bind(opt(&input.ende))
                .bind(id)
                .execute(&pool)
                .await
        }
        Level::Task => {
            // Aufgabe: nur der Titel ist änderbar; erledigt/offen über toggle_task
            sqlx::query("UPDATE aufgabe SET name = $1, geaendert_am = now() WHERE id = $2")
                .bind(&input.name)
                .bind(id)
                .execute(&pool)
                .await
        }
    };

    match result {
        Ok(r) if r.rows_affected() == 0 => Err(ServerFnError::new(format!(
            "{} nicht gefunden.",
            level.label()
        ))),
        Ok(_) => {
            redirect_after_save(level, id, zurueck.as_deref());
            Ok(SaveOutcome::Saved(id))
        }
        Err(e) => foreign_key_outcome(level, e),
    }
}

/// Element mit allen untergeordneten Elementen löschen (AK-14, AK-15).
/// Ein einziger DELETE-Befehl; die Datenbank löscht über ON DELETE CASCADE alles darunter – ganz oder gar nicht.
#[server]
pub async fn delete_element(level: Level, id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let target = match level {
        Level::Project => {
            let result: Result<Option<String>, sqlx::Error> =
                sqlx::query_scalar("DELETE FROM projekt WHERE id = $1 RETURNING nummer")
                    .bind(id)
                    .fetch_optional(&pool)
                    .await;
            let nummer = match result {
                Ok(n) => n,
                // 23503: noch Einkaufsaufträge zugeordnet (ANF-004 AK-23)
                Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
                    let orders: Vec<String> = sqlx::query_scalar(
                        "SELECT nummer FROM einkaufsauftrag WHERE projekt_id = $1 ORDER BY id",
                    )
                    .bind(id)
                    .fetch_all(&pool)
                    .await?;
                    let list = if orders.is_empty() {
                        "anderen Datensätzen".to_string()
                    } else {
                        format!("Einkaufsaufträgen ({})", orders.join(", "))
                    };
                    return Err(ServerFnError::new(format!(
                        "Das Projekt wird noch in {list} verwendet und kann nicht gelöscht werden."
                    )));
                }
                Err(e) => return Err(e.into()),
            };
            nummer.map(|n| format!("/projekte?geloescht={}", crate::forms::encode(&n)))
        }
        _ => {
            let q = match level {
                Level::Subproject => {
                    "DELETE FROM teilprojekt WHERE id = $1 RETURNING projekt_id, name"
                }
                Level::Process => {
                    "DELETE FROM vorgang WHERE id = $1 RETURNING teilprojekt_id, name"
                }
                _ => "DELETE FROM aufgabe WHERE id = $1 RETURNING vorgang_id, name",
            };
            let row: Option<(i64, String)> =
                sqlx::query_as(q).bind(id).fetch_optional(&pool).await?;
            row.map(|(parent, name)| {
                let parent_level = level.parent().unwrap_or(Level::Project);
                format!(
                    "{}?geloescht={}",
                    parent_level.url(parent),
                    crate::forms::encode(&name)
                )
            })
        }
    };
    match target {
        Some(url) => {
            leptos_axum::redirect(&url);
            Ok(())
        }
        None => Err(ServerFnError::new(format!(
            "{} nicht gefunden.",
            level.label()
        ))),
    }
}

/// Aufgabe direkt im Baum erledigen bzw. wieder öffnen (AK-12).
#[server]
pub async fn toggle_task(id: i64, erledigt: bool) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    sqlx::query(
        "UPDATE aufgabe SET status = CASE WHEN $2 THEN 'erledigt' ELSE 'offen' END, geaendert_am = now() \
         WHERE id = $1",
    )
    .bind(id)
    .bind(erledigt)
    .execute(&pool)
    .await?;
    Ok(())
}

/// Element innerhalb seines übergeordneten Elements eine Stelle nach oben/unten schieben (AK-13).
#[server]
pub async fn move_element(level: Level, id: i64, nach_oben: bool) -> Result<(), ServerFnError> {
    use sqlx::AssertSqlSafe;

    let Some(parent_col) = sql::parent_column(level) else {
        return Err(ServerFnError::new("Projekte haben keine Reihenfolge."));
    };
    let table = sql::table(level);
    let pool = crate::db::pool()?;
    let mut tx = pool.begin().await?;

    // Geprüft: Tabellen- und Spaltennamen kommen aus festen Listen je Ebene.
    let parent: Option<i64> = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT {parent_col} FROM {table} WHERE id = $1 FOR UPDATE"
    )))
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(parent) = parent else {
        return Err(ServerFnError::new(format!(
            "{} nicht gefunden.",
            level.label()
        )));
    };

    // Geschwister lückenlos durchnummerieren, dann mit dem Nachbarn tauschen.
    sqlx::query(AssertSqlSafe(format!(
        "UPDATE {table} t SET reihenfolge = r.rn \
         FROM (SELECT id, row_number() OVER (ORDER BY reihenfolge, id) AS rn FROM {table} WHERE {parent_col} = $1) r \
         WHERE t.id = r.id"
    )))
    .bind(parent)
    .execute(&mut *tx)
    .await?;

    let pos: i32 = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT reihenfolge FROM {table} WHERE id = $1"
    )))
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;
    let other = if nach_oben { pos - 1 } else { pos + 1 };
    let swapped = sqlx::query(AssertSqlSafe(format!(
        "UPDATE {table} SET reihenfolge = $3 WHERE {parent_col} = $1 AND reihenfolge = $2"
    )))
    .bind(parent)
    .bind(other)
    .bind(pos)
    .execute(&mut *tx)
    .await?;
    if swapped.rows_affected() == 1 {
        sqlx::query(AssertSqlSafe(format!(
            "UPDATE {table} SET reihenfolge = $2 WHERE id = $1"
        )))
        .bind(id)
        .bind(other)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(())
}
