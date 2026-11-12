//! Server-Funktionen für Adressen (ANF-002).

use leptos::prelude::*;

use super::model::{Address, AddressInput, AddressPage, SaveOutcome};
use crate::forms::PickOption;

pub use crate::forms::error_message;

#[cfg(feature = "ssr")]
mod sql {
    use super::super::model::AddressInput;

    pub const SELECT_ADDRESS: &str = "\
        SELECT id, nummer, anzeigename, typ, anrede, firmenname, vorname, nachname, zusatz, \
               strasse, postfach, plz, ort, land, email, telefon, mobiltelefon, webseite, notiz, \
               to_char(erstellt_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS erstellt_am, \
               to_char(geaendert_am AT TIME ZONE 'Europe/Zurich', 'DD.MM.YYYY HH24:MI') AS geaendert_am \
        FROM adresse";

    /// Suchbedingung (AK-2); $1 = Suchbegriff, $2 = LIKE-Muster.
    pub const SEARCH_WHERE: &str = "\
        ($1 = '' OR nummer ILIKE $2 OR firmenname ILIKE $2 OR vorname ILIKE $2 OR nachname ILIKE $2 \
         OR ort ILIKE $2 OR plz ILIKE $2 OR email ILIKE $2)";

    /// Sortierausdruck für einen Schlüssel aus der Webadresse (nur feste Werte → kein SQL-Einschleusen).
    pub fn order_by(sort: &str) -> &'static str {
        match sort {
            "nummer" => "nummer",
            "plz" => "plz",
            "ort" => "lower(ort)",
            "land" => "land",
            "email" => "lower(email)",
            "telefon" => "telefon",
            _ => "lower(anzeigename)",
        }
    }

    /// Leerer Text → NULL in der Datenbank.
    pub fn opt(s: &str) -> Option<&str> {
        (!s.is_empty()).then_some(s)
    }

    /// Bindet alle Eingabefelder in fester Reihenfolge ($1 … $16).
    pub fn bind_input<'q>(
        q: sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments>,
        i: &'q AddressInput,
    ) -> sqlx::query::Query<'q, sqlx::Postgres, sqlx::postgres::PgArguments> {
        q.bind(&i.typ)
            .bind(opt(&i.anrede))
            .bind(opt(&i.firmenname))
            .bind(opt(&i.vorname))
            .bind(opt(&i.nachname))
            .bind(opt(&i.zusatz))
            .bind(opt(&i.strasse))
            .bind(opt(&i.postfach))
            .bind(opt(&i.plz))
            .bind(opt(&i.ort))
            .bind(&i.land)
            .bind(opt(&i.email))
            .bind(opt(&i.telefon))
            .bind(opt(&i.mobiltelefon))
            .bind(opt(&i.webseite))
            .bind(opt(&i.notiz))
    }

    pub const COLUMNS: &str = "typ, anrede, firmenname, vorname, nachname, zusatz, strasse, postfach, \
                               plz, ort, land, email, telefon, mobiltelefon, webseite, notiz";

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn unknown_sort_falls_back_to_name() {
            assert_eq!(order_by("; DROP TABLE adresse"), "lower(anzeigename)");
        }
    }
}

/// Adressliste mit Suche, Sortierung und Seiten (AK-1 bis AK-6).
#[server]
pub async fn list_addresses(
    suche: String,
    sortierung: String,
    richtung: String,
    seite: i64,
) -> Result<AddressPage, ServerFnError> {
    use super::model::{AddressRow, PAGE_SIZE};
    use sqlx::AssertSqlSafe;

    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let pattern = crate::db::like_pattern(&suche);

    // Geprüft: SQL besteht nur aus festen Texten, Eingaben laufen als Parameter.
    let total: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
        "SELECT count(*) FROM adresse WHERE {}",
        sql::SEARCH_WHERE
    )))
    .bind(&suche)
    .bind(&pattern)
    .fetch_one(&pool)
    .await?;

    let page_count = ((total + PAGE_SIZE - 1) / PAGE_SIZE).max(1);
    let page = seite.clamp(1, page_count);
    let direction = if richtung == "ab" { "DESC" } else { "ASC" };

    // Geprüft: Sortierung kommt aus einer festen Liste (sql::order_by), Richtung ist ASC/DESC.
    let rows: Vec<AddressRow> = sqlx::query_as(AssertSqlSafe(format!(
        "SELECT id, nummer, anzeigename, plz, ort, land, email, telefon FROM adresse \
         WHERE {} ORDER BY {} {direction} NULLS LAST, id LIMIT $3 OFFSET $4",
        sql::SEARCH_WHERE,
        sql::order_by(&sortierung),
    )))
    .bind(&suche)
    .bind(&pattern)
    .bind(PAGE_SIZE)
    .bind((page - 1) * PAGE_SIZE)
    .fetch_all(&pool)
    .await?;

    Ok(AddressPage { rows, total, page })
}

/// Eine Adresse (AK-7, AK-17). `None`, wenn es sie nicht gibt.
#[server]
pub async fn get_address(id: i64) -> Result<Option<Address>, ServerFnError> {
    use sqlx::AssertSqlSafe;
    let pool = crate::db::pool()?;
    let address = sqlx::query_as(AssertSqlSafe(format!(
        "{} WHERE id = $1",
        sql::SELECT_ADDRESS
    )))
    .bind(id)
    .fetch_optional(&pool)
    .await?;
    Ok(address)
}

/// Adress-Auswahl in anderen Modulen (Kunde im Projekt, Lieferant im Einkauf): nach Name, Nummer oder Ort.
#[server]
pub async fn search_addresses(suche: String) -> Result<Vec<PickOption>, ServerFnError> {
    let pool = crate::db::pool()?;
    let suche = suche.trim().to_string();
    let rows = sqlx::query_as(
        "SELECT id, nummer, anzeigename AS name FROM adresse \
         WHERE $1 = '' OR anzeigename ILIKE $2 OR nummer ILIKE $2 OR ort ILIKE $2 \
         ORDER BY lower(anzeigename), id LIMIT 20",
    )
    .bind(&suche)
    .bind(crate::db::like_pattern(&suche))
    .fetch_all(&pool)
    .await?;
    Ok(rows)
}

/// Neue Adresse anlegen (AK-8 bis AK-11). Bei Erfolg Weiterleitung zur Detailansicht.
#[server]
pub async fn create_address(input: AddressInput) -> Result<SaveOutcome, ServerFnError> {
    use sqlx::AssertSqlSafe;
    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    let pool = crate::db::pool()?;
    let sql = format!(
        "INSERT INTO adresse ({}) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16) RETURNING id",
        sql::COLUMNS
    );
    let row = sql::bind_input(sqlx::query(AssertSqlSafe(sql)), &input)
        .fetch_one(&pool)
        .await?;
    let id: i64 = sqlx::Row::try_get(&row, "id")?;

    leptos_axum::redirect(&format!("/adressen/{id}?gespeichert=1"));
    Ok(SaveOutcome::Saved(id))
}

/// Adresse ändern (AK-12, AK-13). Bei Erfolg Weiterleitung zur Detailansicht.
#[server]
pub async fn update_address(id: i64, input: AddressInput) -> Result<SaveOutcome, ServerFnError> {
    use sqlx::AssertSqlSafe;
    let input = input.normalized();
    let errors = input.validate();
    if !errors.is_empty() {
        return Ok(SaveOutcome::Invalid(errors));
    }

    let pool = crate::db::pool()?;
    let sets = sql::COLUMNS
        .split(',')
        .enumerate()
        .map(|(i, c)| format!("{} = ${}", c.trim(), i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    let query = format!("UPDATE adresse SET {sets}, geaendert_am = now() WHERE id = $17");
    let result = sql::bind_input(sqlx::query(AssertSqlSafe(query)), &input)
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(ServerFnError::new("Adresse nicht gefunden."));
    }

    leptos_axum::redirect(&format!("/adressen/{id}?gespeichert=1"));
    Ok(SaveOutcome::Saved(id))
}

/// Adresse löschen (AK-15, AK-16). Wird sie noch verwendet, wird das Löschen verhindert.
#[server]
pub async fn delete_address(id: i64) -> Result<(), ServerFnError> {
    let pool = crate::db::pool()?;
    let result =
        sqlx::query_scalar::<_, String>("DELETE FROM adresse WHERE id = $1 RETURNING nummer")
            .bind(id)
            .fetch_optional(&pool)
            .await;

    match result {
        Ok(Some(nummer)) => {
            leptos_axum::redirect(&format!("/adressen?geloescht={nummer}"));
            Ok(())
        }
        Ok(None) => Err(ServerFnError::new("Adresse nicht gefunden.")),
        // 23503 = Fremdschlüssel verletzt: ein anderer Datensatz verweist noch auf die Adresse
        Err(sqlx::Error::Database(db)) if db.code().as_deref() == Some("23503") => {
            let where_used = match db.table() {
                Some("projekt") => "in Projekten (als Kunde)".to_string(),
                Some("einkaufsauftrag") => "in Einkaufsaufträgen (als Lieferant)".to_string(),
                Some(table) => format!("in der Tabelle „{table}“"),
                None => "in anderen Datensätzen".to_string(),
            };
            Err(ServerFnError::new(format!(
                "Die Adresse wird noch {where_used} verwendet und kann nicht gelöscht werden."
            )))
        }
        Err(e) => Err(e.into()),
    }
}
