//! SQL-Zugriffe für Benutzer (ANF-005, nur Server).
//!
//! Hier steht ausschliesslich Datenbankzugriff: keine Prüfregeln, keine Weiterleitungen.
//! Fehler werden als `sqlx::Error` zurückgegeben; die Server-Funktionen übersetzen sie.
//! Der Passwort-Hash wird nur geschrieben, nie gelesen (AK-21).

use sqlx::{AssertSqlSafe, PgPool};

use super::model::{AssignedElement, User, UserInput, UserRow};
use crate::backend::projects::model::Level;
use crate::forms::PickOption;

const SELECT_USER: &str = "\
    SELECT b.id, b.benutzername, b.aktiv, \
           to_char(b.letzte_anmeldung AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS letzte_anmeldung, \
           to_char(b.erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
           to_char(b.geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am, \
           a.id AS adresse_id, a.nummer AS adresse_nummer, a.anzeigename AS name, \
           a.email, a.telefon, a.mobiltelefon \
    FROM benutzer b JOIN adresse a ON a.id = b.adresse_id";

/// Such- und Statusbedingung der Liste (AK-2, AK-3); $1 = Suchbegriff, $2 = LIKE-Muster,
/// $3 = Status (leer = alle, sonst „aktiv“ oder „gesperrt“).
const LIST_WHERE: &str = "\
    ($1 = '' OR b.benutzername ILIKE $2 OR a.anzeigename ILIKE $2 OR a.email ILIKE $2) \
    AND ($3 = '' OR b.aktiv = ($3 = 'aktiv'))";

/// Sortierausdruck für einen Schlüssel aus der Webadresse (nur feste Werte → kein SQL-Einschleusen).
fn order_by(sort: &str) -> &'static str {
    match sort {
        "name" => "lower(a.anzeigename)",
        "email" => "lower(a.email)",
        "status" => "NOT b.aktiv", // aufsteigend: aktive zuerst
        "anmeldung" => "b.letzte_anmeldung",
        _ => "b.benutzername",
    }
}

/// Teilprojekte und Vorgänge aller Projekte, in denen der Benutzer ($1) irgendwo hinterlegt ist,
/// mit ihrer Nummer im Projekt (`pos`, z. B. {2,3} = „2.3“). Gleiche Reihenfolge wie die
/// Strukturansicht (reihenfolge, id); es werden immer ganze Projekte nummeriert.
pub const NUMBERED: &str = "\
    WITH pr AS ( \
        SELECT id FROM projekt WHERE benutzer_id = $1 \
        UNION SELECT projekt_id FROM teilprojekt WHERE benutzer_id = $1 \
        UNION SELECT t.projekt_id FROM vorgang v JOIN teilprojekt t ON t.id = v.teilprojekt_id \
              WHERE v.benutzer_id = $1 \
    ), \
    tp AS ( \
        SELECT t.id, t.projekt_id, t.name, t.benutzer_id, \
               ARRAY[row_number() OVER (PARTITION BY t.projekt_id ORDER BY t.reihenfolge, t.id)] AS pos \
        FROM teilprojekt t WHERE t.projekt_id IN (SELECT id FROM pr) \
    ), \
    vg AS ( \
        SELECT v.id, tp.projekt_id, v.name, v.benutzer_id, tp.benutzer_id AS tp_benutzer_id, \
               tp.pos || row_number() OVER (PARTITION BY v.teilprojekt_id ORDER BY v.reihenfolge, v.id) AS pos \
        FROM vorgang v JOIN tp ON tp.id = v.teilprojekt_id \
    )";

/// Benutzerliste mit Suche, Statusfilter und Sortierung (AK-1 bis AK-4).
pub async fn list(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    status: &str,
    sortierung: &str,
    descending: bool,
) -> Result<Vec<UserRow>, sqlx::Error> {
    let direction = if descending { "DESC" } else { "ASC" };
    // Geprüft: Sortierung aus fester Liste (order_by), Richtung ist ASC/DESC.
    sqlx::query_as(AssertSqlSafe(format!(
        "SELECT b.id, b.benutzername, a.anzeigename AS name, a.email, b.aktiv, \
                to_char(b.letzte_anmeldung AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS letzte_anmeldung \
         FROM benutzer b JOIN adresse a ON a.id = b.adresse_id \
         WHERE {LIST_WHERE} ORDER BY {} {direction} NULLS LAST, b.benutzername",
        order_by(sortierung)
    )))
    .bind(suche)
    .bind(pattern)
    .bind(status)
    .fetch_all(pool)
    .await
}

/// Ein Benutzer; `None`, wenn es ihn nicht gibt.
pub async fn find(pool: &PgPool, id: i64) -> Result<Option<User>, sqlx::Error> {
    sqlx::query_as(AssertSqlSafe(format!("{SELECT_USER} WHERE b.id = $1")))
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Höchstens 20 aktive Benutzer für die Auswahl „Verantwortlich“ (AK-24).
pub async fn search_active(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
) -> Result<Vec<PickOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT b.id, b.benutzername AS nummer, a.anzeigename AS name \
         FROM benutzer b JOIN adresse a ON a.id = b.adresse_id \
         WHERE b.aktiv AND ($1 = '' OR b.benutzername ILIKE $2 OR a.anzeigename ILIKE $2) \
         ORDER BY lower(a.anzeigename), b.id LIMIT 20",
    )
    .bind(suche)
    .bind(pattern)
    .fetch_all(pool)
    .await
}

/// Höchstens 20 Adressen vom Typ „Person“, die noch keinem Benutzer gehören (AK-10).
/// Die Adresse des Benutzers `except_user` (beim Bearbeiten) gilt als frei.
pub async fn search_free_persons(
    pool: &PgPool,
    suche: &str,
    pattern: &str,
    except_user: Option<i64>,
) -> Result<Vec<PickOption>, sqlx::Error> {
    sqlx::query_as(
        "SELECT a.id, a.nummer, a.anzeigename AS name FROM adresse a \
         WHERE a.typ = 'person' \
           AND NOT EXISTS (SELECT 1 FROM benutzer b WHERE b.adresse_id = a.id AND b.id IS DISTINCT FROM $3) \
           AND ($1 = '' OR a.anzeigename ILIKE $2 OR a.nummer ILIKE $2 OR a.ort ILIKE $2 OR a.email ILIKE $2) \
         ORDER BY lower(a.anzeigename), a.id LIMIT 20",
    )
    .bind(suche)
    .bind(pattern)
    .bind(except_user)
    .fetch_all(pool)
    .await
}

/// Typ einer Adresse („firma“/„person“); `None`, wenn es sie nicht gibt.
pub async fn address_type(pool: &PgPool, address_id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT typ FROM adresse WHERE id = $1")
        .bind(address_id)
        .fetch_optional(pool)
        .await
}

/// Benutzername des Benutzers, dem die Adresse gehört – ausser `except_user`.
pub async fn address_owner(
    pool: &PgPool,
    address_id: i64,
    except_user: Option<i64>,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT benutzername FROM benutzer WHERE adresse_id = $1 AND id IS DISTINCT FROM $2",
    )
    .bind(address_id)
    .bind(except_user)
    .fetch_optional(pool)
    .await
}

/// `true`, wenn der Benutzername (ohne Beachtung der Gross-/Kleinschreibung) schon vergeben ist.
pub async fn username_taken(
    pool: &PgPool,
    username: &str,
    except_user: Option<i64>,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM benutzer WHERE lower(benutzername) = lower($1) \
                        AND id IS DISTINCT FROM $2)",
    )
    .bind(username)
    .bind(except_user)
    .fetch_one(pool)
    .await
}

/// Neuen Benutzer speichern; liefert die Kennung.
pub async fn insert(
    pool: &PgPool,
    input: &UserInput,
    password_hash: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO benutzer (benutzername, adresse_id, aktiv, passwort_hash) \
         VALUES ($1, $2, $3, $4) RETURNING id",
    )
    .bind(&input.benutzername)
    .bind(input.address_id())
    .bind(input.is_active())
    .bind(password_hash)
    .fetch_one(pool)
    .await
}

/// Benutzername, Adresse und Status ändern; liefert die Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn update(pool: &PgPool, id: i64, input: &UserInput) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        "UPDATE benutzer SET benutzername = $1, adresse_id = $2, aktiv = $3, geaendert_am = now() \
         WHERE id = $4",
    )
    .bind(&input.benutzername)
    .bind(input.address_id())
    .bind(input.is_active())
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}

/// Neuen Passwort-Hash setzen; liefert die Anzahl geänderter Zeilen (0 = nicht gefunden).
pub async fn set_password(pool: &PgPool, id: i64, password_hash: &str) -> Result<u64, sqlx::Error> {
    let result =
        sqlx::query("UPDATE benutzer SET passwort_hash = $1, geaendert_am = now() WHERE id = $2")
            .bind(password_hash)
            .bind(id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected())
}

/// Benutzer löschen; liefert den Benutzernamen oder `None`, wenn es ihn nicht gab.
/// Ist er noch in Projekten hinterlegt, meldet die Datenbank 23503 (AK-32).
pub async fn delete(pool: &PgPool, id: i64) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("DELETE FROM benutzer WHERE id = $1 RETURNING benutzername")
        .bind(id)
        .fetch_optional(pool)
        .await
}

/// Anzahl Projekte, Teilprojekte und Vorgänge, bei denen der Benutzer hinterlegt ist (AK-32).
pub async fn responsibility_counts(
    pool: &PgPool,
    id: i64,
) -> Result<[(Level, usize); 3], sqlx::Error> {
    let (p, t, v): (i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT count(*) FROM projekt WHERE benutzer_id = $1), \
                (SELECT count(*) FROM teilprojekt WHERE benutzer_id = $1), \
                (SELECT count(*) FROM vorgang WHERE benutzer_id = $1)",
    )
    .bind(id)
    .fetch_one(pool)
    .await?;
    let n = |x: i64| usize::try_from(x).unwrap_or_default();
    Ok([
        (Level::Project, n(p)),
        (Level::Subproject, n(t)),
        (Level::Process, n(v)),
    ])
}

#[derive(sqlx::FromRow)]
struct AssignedRow {
    ebene: String,
    id: i64,
    nummer: String,
    name: String,
    projekt_id: i64,
    projekt_nummer: String,
    projekt_name: String,
}

/// Projekte, Teilprojekte und Vorgänge, bei denen der Benutzer hinterlegt ist (AK-31).
pub async fn assigned_elements(
    pool: &PgPool,
    id: i64,
) -> Result<Vec<AssignedElement>, sqlx::Error> {
    // Geprüft: nur feste Texte, die Kennung läuft als Parameter.
    let rows: Vec<AssignedRow> = sqlx::query_as(AssertSqlSafe(format!(
        "{NUMBERED} \
         SELECT 'projekt' AS ebene, p.id, p.nummer, p.name, p.id AS projekt_id, \
                p.nummer AS projekt_nummer, p.name AS projekt_name, ARRAY[]::bigint[] AS pos \
         FROM projekt p WHERE p.benutzer_id = $1 \
         UNION ALL \
         SELECT 'teilprojekt', tp.id, array_to_string(tp.pos, '.'), tp.name, p.id, p.nummer, p.name, tp.pos \
         FROM tp JOIN projekt p ON p.id = tp.projekt_id WHERE tp.benutzer_id = $1 \
         UNION ALL \
         SELECT 'vorgang', vg.id, array_to_string(vg.pos, '.'), vg.name, p.id, p.nummer, p.name, vg.pos \
         FROM vg JOIN projekt p ON p.id = vg.projekt_id WHERE vg.benutzer_id = $1 \
         ORDER BY projekt_nummer, pos"
    )))
    .bind(id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            Some(AssignedElement {
                level: Level::from_key(&r.ebene)?,
                id: r.id,
                nummer: r.nummer,
                name: r.name,
                projekt_id: r.projekt_id,
                projekt_nummer: r.projekt_nummer,
                projekt_name: r.projekt_name,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_sort_falls_back_to_username() {
        assert_eq!(order_by("; DROP TABLE benutzer"), "b.benutzername");
    }
}

// ---------------------------------------------------------------------------------------------
// Sitzungen der ERP-Datenbank (ANF-007 AK-15). Seit ANF-026 läuft die Anmeldung über die
// Portal-Datenbank; diese Tabelle wird mit dem Entfernen der Passwörter aus der
// Mandanten-Datenbank (ANF-026 Schritt 3) aufgelöst.

/// Alle Sitzungen eines Benutzers beenden (gesperrt, ANF-007 AK-15).
pub async fn end_all_sessions(pool: &PgPool, user_id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sitzung WHERE benutzer_id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Alle Sitzungen eines Benutzers beenden ausser der mit `keep_hash` (Passwort neu gesetzt, ANF-007 AK-15).
pub async fn end_sessions_except(
    pool: &PgPool,
    user_id: i64,
    keep_hash: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sitzung WHERE benutzer_id = $1 AND merkmal_hash IS DISTINCT FROM $2")
        .bind(user_id)
        .bind(keep_hash)
        .execute(pool)
        .await?;
    Ok(())
}
