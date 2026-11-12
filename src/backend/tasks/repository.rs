//! SQL-Zugriffe für die Navigation „Aufgaben“ (ANF-006, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Aufgaben ohne Projekt liegen in `aufgabe` mit `vorgang_id IS NULL` und `benutzer_id` gesetzt.

use sqlx::{AssertSqlSafe, PgPool};

use super::model::{StandaloneTask, StandaloneTaskInput, TaskItem};
use crate::backend::users::repository::NUMBERED;

const TS_CHANGED: &str =
    "to_char(a.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am";

/// Alle Aufgaben, die dem Benutzer ($1) zugeordnet sind (AK-3 bis AK-6):
/// Projektaufgaben, für die er hinterlegt oder geerbt verantwortlich ist (gleiche Regel wie ANF-005),
/// und seine Aufgaben ohne Projekt. Filter: $2 Status (leer = alle), $3 Art („mit“, „ohne“, leer = alle),
/// $4 Suchbegriff, $5 LIKE-Muster.
/// Bei mehreren Ebenen wird als Herkunft die oberste genannt (wie ANF-005 AK-28).
pub async fn list_for_user(
    pool: &PgPool,
    user_id: i64,
    status: &str,
    kind: &str,
    suche: &str,
    pattern: &str,
) -> Result<Vec<TaskItem>, sqlx::Error> {
    // Geprüft: nur feste Texte, alle Eingaben laufen als Parameter.
    sqlx::query_as(AssertSqlSafe(format!(
        "{NUMBERED}, \
         ag AS ( \
             SELECT a.id, a.name, a.beschreibung, a.status, a.geaendert_am, vg.projekt_id, \
                    vg.benutzer_id AS vg_benutzer_id, vg.tp_benutzer_id, \
                    vg.pos || row_number() OVER (PARTITION BY a.vorgang_id ORDER BY a.reihenfolge, a.id) AS pos \
             FROM aufgabe a JOIN vg ON vg.id = a.vorgang_id \
         ) \
         SELECT t.id, t.name, t.beschreibung, t.status, t.projekt_id, t.projekt_nummer, t.projekt_name, \
                t.nummer, t.herkunft, t.geaendert_am \
         FROM ( \
             SELECT a.id, a.name, a.beschreibung, a.status, p.id AS projekt_id, p.nummer AS projekt_nummer, \
                    p.name AS projekt_name, array_to_string(a.pos, '.') AS nummer, \
                    CASE WHEN p.benutzer_id = $1 THEN 'vom Projekt' \
                         WHEN a.tp_benutzer_id = $1 THEN 'vom Teilprojekt ' || a.pos[1] \
                         ELSE 'vom Vorgang ' || array_to_string(a.pos[1:2], '.') END AS herkunft, \
                    {TS_CHANGED}, a.pos AS sort_pos \
             FROM ag a JOIN projekt p ON p.id = a.projekt_id \
             WHERE p.benutzer_id = $1 OR a.tp_benutzer_id = $1 OR a.vg_benutzer_id = $1 \
             UNION ALL \
             SELECT a.id, a.name, a.beschreibung, a.status, NULL::bigint, NULL::text, NULL::text, NULL::text, \
                    'direkt zugewiesen', {TS_CHANGED}, NULL::bigint[] \
             FROM aufgabe a WHERE a.benutzer_id = $1 \
         ) t \
         WHERE ($2 = '' OR t.status = $2) \
           AND ($3 = '' OR ($3 = 'ohne') = (t.projekt_id IS NULL)) \
           AND ($4 = '' OR t.name ILIKE $5 OR t.beschreibung ILIKE $5 \
                OR t.projekt_nummer ILIKE $5 OR t.projekt_name ILIKE $5) \
         ORDER BY (t.projekt_id IS NOT NULL), t.projekt_nummer, t.sort_pos, t.id DESC"
    )))
    .bind(user_id)
    .bind(status)
    .bind(kind)
    .bind(suche)
    .bind(pattern)
    .fetch_all(pool)
    .await
}

/// `Some(true)` = Aufgabe ohne Projekt, `Some(false)` = Projektaufgabe, `None` = gibt es nicht.
pub async fn is_standalone(pool: &PgPool, id: i64) -> Result<Option<bool>, sqlx::Error> {
    sqlx::query_scalar("SELECT vorgang_id IS NULL FROM aufgabe WHERE id = $1")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Aufgabe ohne Projekt mit zuständigem Benutzer; `None`, wenn es sie nicht gibt.
pub async fn find_standalone(
    pool: &PgPool,
    id: i64,
) -> Result<Option<StandaloneTask>, sqlx::Error> {
    sqlx::query_as(
        "SELECT a.id, a.name, a.beschreibung, a.status, a.benutzer_id, b.benutzername, \
                ad.anzeigename AS benutzer_name, b.aktiv AS benutzer_aktiv, \
                to_char(a.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
                to_char(a.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
         FROM aufgabe a JOIN benutzer b ON b.id = a.benutzer_id JOIN adresse ad ON ad.id = b.adresse_id \
         WHERE a.id = $1 AND a.vorgang_id IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Zuständiger Benutzer einer Aufgabe ohne Projekt.
pub async fn assignee_of(pool: &PgPool, id: i64) -> Result<Option<i64>, sqlx::Error> {
    sqlx::query_scalar("SELECT benutzer_id FROM aufgabe WHERE id = $1 AND vorgang_id IS NULL")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Status eines Benutzers: `Some(true)` aktiv, `Some(false)` gesperrt, `None` = gibt es nicht.
pub async fn user_active(pool: &PgPool, user_id: i64) -> Result<Option<bool>, sqlx::Error> {
    sqlx::query_scalar("SELECT aktiv FROM benutzer WHERE id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

/// Leerer Text → NULL in der Datenbank.
fn opt(s: &str) -> Option<&str> {
    (!s.is_empty()).then_some(s)
}

/// Neue Aufgabe ohne Projekt, Status „offen“ (Standardwert der Tabelle); liefert die Kennung.
pub async fn insert_standalone(
    pool: &PgPool,
    input: &StandaloneTaskInput,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO aufgabe (vorgang_id, reihenfolge, name, beschreibung, benutzer_id) \
         VALUES (NULL, 0, $1, $2, $3) RETURNING id",
    )
    .bind(&input.name)
    .bind(opt(&input.beschreibung))
    .bind(input.user_id())
    .fetch_one(pool)
    .await
}

/// Aufgabe ohne Projekt ändern; liefert die Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update_standalone(
    pool: &PgPool,
    id: i64,
    input: &StandaloneTaskInput,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE aufgabe SET name = $1, beschreibung = $2, benutzer_id = $3, geaendert_am = now() \
         WHERE id = $4 AND vorgang_id IS NULL",
    )
    .bind(&input.name)
    .bind(opt(&input.beschreibung))
    .bind(input.user_id())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Aufgabe ohne Projekt löschen; liefert (Titel, zuständiger Benutzer) oder `None`.
pub async fn delete_standalone(
    pool: &PgPool,
    id: i64,
) -> Result<Option<(String, i64)>, sqlx::Error> {
    sqlx::query_as(
        "DELETE FROM aufgabe WHERE id = $1 AND vorgang_id IS NULL RETURNING name, benutzer_id",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Anzahl Aufgaben ohne Projekt, für die der Benutzer zuständig ist (ANF-006 AK-19).
pub async fn count_for_user(pool: &PgPool, user_id: i64) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT count(*) FROM aufgabe WHERE benutzer_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
}
