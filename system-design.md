# Systementwurf

Aufbau des Quellcodes von uniqwork: welche Schicht wo liegt, wer wen aufrufen darf
und wie ein neues Modul angelegt wird. Ergänzt [CLAUDE.md](CLAUDE.md) (Arbeitsregeln) und
[index.md](index.md) (Anforderungen).

## Überblick

uniqwork ist **ein** Rust-Paket, das zweimal übersetzt wird:

| Übersetzung | Schalter  | Ergebnis                                    | Enthält                                     |
|-------------|-----------|---------------------------------------------|---------------------------------------------|
| Server      | `ssr`     | Serverprogramm (Axum, `src/main.rs`)        | alles, inkl. `repository` und `db`          |
| Browser     | `hydrate` | WebAssembly (`hydrate()` in `src/lib.rs`)   | alles **ausser** `repository` und `db`      |

Der Browser ruft Server-Funktionen über HTTP auf; Leptos erzeugt dafür aus jeder `#[server]`-Funktion
automatisch den Aufruf im Browser und den Endpunkt auf dem Server.

```
 Browser (WebAssembly)                    Server (Axum)
┌──────────────────────┐               ┌──────────────────────────────────────────────┐
│ components/<modul>/  │  HTTP-Aufruf  │ backend/<modul>/server_fns  → repository → DB│
│   list / detail /    │ ────────────► │   prüft, steuert Transaktion,  nur SQL       │
│   form …             │ ◄──────────── │   meldet Fehler, leitet um                   │
└─────────┬────────────┘   Antwort     └──────────────┬───────────────────────────────┘
          │                                           │
          └──────────► backend/<modul>/model ◄────────┘
                       (gemeinsame Datentypen und Prüfregeln)
```

## Ordnerstruktur

```
src/
  main.rs                     Serverstart: .env, Verbindungspool, Axum-Routen (nur ssr)
  lib.rs                      Module + Einstieg hydrate() für den Browser
  app.rs                      HTML-Rahmen, Komponente `App`, alle Routen
  db.rs                       Verbindungspools (ERP und Portal), Migrationen, LIKE-Muster (nur ssr)
  layout.rs                   Grundgerüst: Kopfzeile + Seitenleiste
  navigation.rs               Navigationseinträge (Haupt- und Einstellungsnavigation)
  forms.rs                    Gemeinsame Formular-Bausteine (Modal, TextField, SearchPicker …)
  icons.rs                    Eingebettete SVG-Symbole

  backend/
    <modulname>/
      mod.rs                  bindet model, server_fns, repository ein
      model.rs                Datentypen, Eingaben, Prüfregeln, Berechnungen (Server + Browser)
      server_fns.rs           #[server]-Funktionen: Schnittstelle für die Oberfläche
      repository.rs           SQL-Zugriffe mit sqlx (nur ssr)

  components/
    <modulname>/
      mod.rs                  bindet die Komponenten ein, gibt die Seiten nach aussen frei
      <name_der_komponente>.rs   eine Ansicht bzw. Komponentengruppe, z. B. list.rs, detail.rs, form.rs
```

### Aktuelle Module

| Modul (Rust)       | Anforderung | `backend/`                                   | `components/`                               |
|--------------------|-------------|----------------------------------------------|---------------------------------------------|
| `addresses`        | ANF-002     | model, server_fns, repository                | list, detail, form                          |
| `projects`         | ANF-003     | model, server_fns, repository                | list, detail, form                          |
| `purchase_orders`  | ANF-004     | model, server_fns, repository                | list, detail, form                          |
| `auth`             | ANF-007, ANF-026 | model, server_fns, repository, session, middleware, password (Portal-Datenbank) | login, register    |
| `portal`           | ANF-026     | model, server_fns, repository, provision (Datenbanken anlegen) | page (`/portal`)          |
| `tasks`            | ANF-006     | model, server_fns, repository                | list, detail, form                          |
| `users`            | ANF-005     | model, server_fns, repository (ohne Passwörter) | list, detail, form                       |
| `system`           | ANF-001     | server_fns, repository (Datenbank-Status)    | –                                           |
| `home`             | ANF-001     | – (nutzt `system`)                           | home_page                                   |
| `finance::accounts` | ANF-011    | model, server_fns, repository (unter `backend/finance/`) | `settings/finance/accounts/`: list, form |
| `site_plans`       | ANF-027, ANF-028 | model (Prüfregeln mit `geo`), flows (Besucherströme), bottlenecks (Engstellen), server_fns, repository | page, flows (Besucherströme), bridge (Rust ↔ `map_bridge.js`: MapLibre + Three.js) |
| `finance::vat`     | ANF-012     | model, server_fns, repository (unter `backend/finance/`) | `settings/finance/mwst/`: list, form |
| `finance::fiscal_year` | ANF-013 | model, server_fns, repository (unter `backend/finance/`) | `settings/finance/geschaeftsjahre/`: list, form |
| `finance::journal` | ANF-017, ANF-010 AK-7/AK-8 | model (Prüfregeln, MWST-Aufteilung), server_fns, repository (unter `backend/finance/`) | `finance/journal/`: list, form, detail, storno |
| `finance::reports` | ANF-021 | model (Rechenregeln, CSV), service (Zusammensetzen), server_fns, repository, csv (Route `/finanzen/auswertungen/csv`) | `finance/reports/`: page |
| `finance::date`    | ANF-012, ANF-013, ANF-017 | model-artige Datei ohne Datenbank (Datumsrechnung, `backend/finance/date.rs`) | – |
| `company`          | ANF-025     | model, server_fns, repository, logo (Auslieferung `/firma/logo`) | `settings/company/`: page, form |
| `settings`         | –           | –                                            | general, company, finance (Stammdaten der Finanzen) |

Ein Modul braucht nicht jede Datei: ohne eigene Datentypen entfällt `model.rs`, ohne Daten entfällt `backend/<modul>`.

## Schichten

### `backend/<modul>/model.rs` – Datenmodell

- Wird für **Server und Browser** übersetzt; darf daher weder sqlx-Abfragen noch `crate::db` enthalten.
- Enthält:
  - Datentypen für Anzeige und Übertragung (`Serialize`, `Deserialize`; mit `#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]`, wenn sie direkt aus SQL gelesen werden),
  - Eingabetypen (`…Input`) mit `normalized()` und `validate()`,
  - reine Berechnungen (z. B. Summen, MWST, Fortschritt) und Modultests dazu.
- Prüfregeln stehen **nur hier**. Der Browser nutzt sie für sofortige Rückmeldung, der Server führt sie
  in `server_fns` immer erneut aus.

### `backend/<modul>/server_fns.rs` – Server-Funktionen

- Die **einzige Schnittstelle** zwischen Oberfläche und Server. Jede öffentliche Funktion ist `#[server]`.
- Aufgaben, in dieser Reihenfolge:
  1. Eingaben normalisieren und prüfen (`model`); bei Fehlern `SaveOutcome::Invalid(…)` zurückgeben.
  2. Verbindungspool holen: `crate::db::pool()?`.
  3. Bei mehreren zusammengehörigen Schritten Transaktion öffnen (`pool.begin()`), Datensatz sperren,
     Statusregeln prüfen (z. B. „nur im Entwurf“).
  4. `repository`-Funktionen aufrufen.
  5. Datenbankfehler in verständliche deutsche Meldungen übersetzen
     (z. B. Fremdschlüssel `23503` → „wird noch in … verwendet“).
  6. Transaktion abschliessen (`tx.commit()`), danach ggf. weiterleiten (`leptos_axum::redirect`).
- Enthält **kein SQL**. Hilfsfunktionen, die nur der Server braucht, stehen hinter `#[cfg(feature = "ssr")]`.
- Das Repository wird mit `#[cfg(feature = "ssr")] use super::repository;` eingebunden
  (im Browser fällt der Rumpf der Server-Funktionen weg).

### `backend/<modul>/repository.rs` – Datenbankzugriff

- Nur auf dem Server übersetzt (`#[cfg(feature = "ssr")] pub mod repository;` in `mod.rs`).
- Enthält **ausschliesslich SQL**: feste SQL-Texte, Spaltenlisten, Sortier-Zuordnungen, Abbildung
  von Zeilen auf Modelltypen.
- Signaturen:
  - Lesen/Schreiben ohne Transaktion: `async fn …(pool: &PgPool, …) -> Result<T, sqlx::Error>`
  - Innerhalb einer Transaktion: `async fn …(conn: &mut PgConnection, …) -> Result<T, sqlx::Error>`
    (der Aufrufer übergibt `&mut tx`).
- Gibt immer `sqlx::Error` zurück – keine `ServerFnError`, keine Meldungstexte, keine Weiterleitungen,
  keine Geschäftsregeln. „Nicht gefunden“ wird als `Option`/Anzahl betroffener Zeilen gemeldet.
- Eingaben immer als Parameter (`$1`, `.bind(…)`). Zusammengesetzte SQL-Texte nur mit `AssertSqlSafe(…)`
  und nur aus festen Texten oder Werten einer festen Liste (z. B. `order_by()` je Modul, mit Modultest).

### `components/<modul>/<name_der_komponente>.rs` – Oberfläche

- Leptos-Komponenten (`#[component]`), eine Datei je Ansicht bzw. zusammengehöriger Gruppe:
  - `list.rs` – Liste mit Suche, Filter, Sortierung, Seiten (Zustand in der Webadresse),
  - `detail.rs` – Detailansicht, Löschdialog,
  - `form.rs` – Modal-Fenster für Anlegen und Bearbeiten.
- Holen Daten **nur** über `crate::backend::<modul>::server_fns` (als `Resource` bzw. `ServerAction`)
  und nutzen Typen aus `crate::backend::<modul>::model`. Kein Zugriff auf `repository` oder `db`.
- `mod.rs` gibt nur die Seiten frei, die `app.rs` für die Routen braucht (z. B. `AddressListPage`).
- Modulübergreifende Nutzung ist erlaubt, immer über die öffentliche Schnittstelle:
  - Daten: `crate::backend::addresses::server_fns::search_addresses`,
  - Komponenten: `crate::components::purchase_orders::ProjectOrders`.
- Gemeinsame Bausteine (Modal, Formularfelder, Symbole, Grundgerüst) liegen in `src/forms.rs`,
  `src/icons.rs`, `src/layout.rs` und werden nicht pro Modul nachgebaut.

## Zugangsschutz (ANF-007, ANF-026)

```
Anfrage ──► auth::middleware (Axum-Schicht in main.rs, Zustand: PortalPool)
              │  Cookie „erp_sitzung“ → SHA-256 → portal_sitzung (Portal-Datenbank) → CurrentUser in die Anfrage
              ├─ feste Dateien (/pkg/…, *.css, *.ico …) ............ immer durchlassen
              ├─ /anmelden, /registrieren, /oeffentlich/… ........ ohne Anmeldung erlaubt
              ├─ /api/… ohne Sitzung .............................. 401 „Nicht angemeldet.“
              ├─ Seite ohne Sitzung ............................... 303 → /anmelden?zurueck=…
              └─ mit Sitzung, Mandant nicht bereit/definiert:
                   /portal, /api/portal/… ......................... erlaubt
                   übrige /api/… .................................. 403 „Kein Mandant gewählt.“
                   übrige Seiten .................................. 303 → /portal
                                                                    (Datenbank bereit, Mandant nicht definiert:
                                                                     /portal?mandant=<Kennung> → Modal-Fenster)
```

- Mit Zugang zum ERP legt die Middleware den Pool der Mandanten-Datenbank (`db::TenantPool`) in die Anfrage und
  ergänzt `CurrentUser::benutzer_id` aus der Tabelle `benutzer` des Mandanten. `crate::db::pool()` liest ihn dort.
- Server-Funktionen lesen den Zugang mit `auth::session::current_user()` (aus `http::request::Parts`).
  `CurrentUser` enthält Zugang (`portal_id`, E-Mail, Name), den Mandanten der Sitzung und – ab dem Erstellen der
  Datenbank – den Benutzer in der Mandanten-Datenbank (`benutzer_id`).
- Ein Mandant zählt nur, wenn er dem Zugang gehört (`mandant.besitzer_id`); ein untergeschobener Mandant wird ignoriert.
- Seitenwechsel im Browser laufen nicht über den Server; deshalb prüft das Grundgerüst (`layout.rs`) bei jedem
  Wechsel `get_current_user()` und leitet bei abgelaufener Sitzung zur Anmeldeseite, ohne Mandant zu `/portal`.

## Auslieferung mit Docker auf Railway (ANF-008)

```
git push main ──► Railway (railway.toml) ──► docker build (Dockerfile)
                                                │
   css       node: npm ci, npm run css:build ──► styles/output.css
   chef      rust + wasm32-Ziel + cargo-chef + cargo-leptos
   planer    cargo chef prepare ──► recipe.json (nur Abhängigkeiten, ohne eigenen Code)
   bau       cargo chef cook (ssr, release)              ┐ zwischengespeichert, solange
             cargo chef cook (hydrate, wasm-release)     ┘ Cargo.toml/Cargo.lock gleich bleiben
             cargo leptos build --release ──► target/release/uniqwork + target/site
   laufzeit  debian slim, Benutzer „erp“ (nicht root):
             /app/uniqwork, /app/site, /app/Cargo.toml
                                                │
Container ──► hört auf 0.0.0.0:$PORT ──► DATABASE_URL ──► PostgreSQL-Dienst (Railway, Europa)
```

- Migrationen sind über `sqlx::migrate!` im Serverprogramm enthalten und laufen beim Start; fehlt die
  Datenbank, legt der Server sie an (`src/db.rs`).
- **Datenbanken (ANF-026)** – alle im selben PostgreSQL-Dienst:

  ```
  DATABASE_URL (Benutzer mit CREATEDB/CREATEROLE, z. B. postgres)
    ├─ uniqwork_portal ...... Portal: Zugänge, Sitzungen, Mandanten (migrations_portal/), Pool PortalPool
    ├─ uq_m_000001 .......... Mandant 1: ERP (migrations/), Eigentümer = Benutzer aus DATABASE_URL
    ├─ uq_m_000002 .......... Mandant 2 …
    └─ Rolle uniqwork_erp ... laufender ERP-Betrieb: nur CONNECT auf uq_m_…, Daten lesen/schreiben, keine DDL
  ```

  - Adresse der Portal-Datenbank aus `PORTAL_DATABASE_URL`; fehlt sie, `DATABASE_URL` mit dem Namen `uniqwork_portal`.
    Ihre Zugangsdaten dienen auch zum Anlegen, Migrieren und Löschen der Mandanten-Datenbanken.
  - **Serverstart** (`db::connect`, `portal::provision::startup`): Portal-Datenbank anlegen und migrieren; Rolle
    `uniqwork_erp` anlegen bzw. mit neuem Zufallspasswort versehen (nie gespeichert, nie protokolliert); abgebrochene
    Erstellungen (`wird_erstellt`) löschen und auf `fehler` setzen; Migrationen und Rechte in allen bereiten
    Mandanten-Datenbanken nachführen. Andere Datenbanken legt der Server beim Start nicht an.
  - **Datenbank erstellen** (`portal::server_fns::create_database`, eigene Tokio-Aufgabe): Zeile des Administrators
    sperren, Regel „eine pro Administrator“ prüfen, Mandant `wird_erstellt` → `CREATE DATABASE`, `CONNECT` nur für
    Eigentümer und `uniqwork_erp`, Migrationen, Rechte, erster Benutzer (Adresse „Person“, ohne Passwort, mit
    `portal_benutzer_id`) → `bereit`. Bei einem Fehler wird die Datenbank gelöscht und der Mandant auf `fehler` gesetzt.
  - **Pools je Mandant** (`db::TenantPools`): beim ersten Zugriff ohne Verbindung angelegt, höchstens 3 Verbindungen,
    unbenutzte schliessen nach 5 Minuten.
  - `CREATE DATABASE` und `GRANT` erlauben keine Parameter: Datenbanknamen kommen nur nach `is_valid_database_name`
    (`uq_m_` + Ziffern, von der Portal-Datenbank erzeugt) in den SQL-Text.
- Umgebungsvariablen im Image: `LEPTOS_ENV=PROD`, `LEPTOS_SITE_ROOT=site`, `LEPTOS_SITE_ADDR=0.0.0.0:8080`,
  `RUST_LOG=info`. In Railway zu setzen: `DATABASE_URL`, `SITZUNG_COOKIE_SECURE=1`
  (`PORTAL_DATABASE_URL` nur, wenn die Portal-Datenbank woanders liegen soll).
- Im Docker-Bau setzt `RUSTFLAGS="--cfg erase_components"` die Leptos-Option für Linux (`.cargo/config.toml`
  gilt nur für Apple-Rechner). Ohne sie bricht der Release-Bau des Servers mit „queries overflow the depth limit“ ab.
- Die `cargo chef cook`-Aufrufe müssen dieselben Funktionsschalter, Profile und Zielordner verwenden wie
  `cargo leptos build` – sonst übersetzt der letzte Schritt alle Bibliotheken neu (kein Fehler, nur langsam).
- Railway-Region der Anwendung steht in `railway.toml`; die Region des PostgreSQL-Dienstes wird in der
  Railway-Oberfläche gewählt (ebenfalls Europa).

## Abhängigkeitsregeln

| Von ↓ darf nutzen →         | `model` | `server_fns` | `repository` | `db` | `components` |
|-----------------------------|:-------:|:------------:|:------------:|:----:|:------------:|
| `components/…`              | ja      | ja           | **nein**     | **nein** | ja       |
| `backend/…/server_fns`      | ja      | –            | ja           | ja   | **nein**     |
| `backend/…/repository`      | ja      | **nein**     | –            | nein¹ | **nein**    |
| `backend/…/model`           | ja      | **nein**     | **nein**     | **nein** | **nein** |

¹ Das Repository bekommt den Pool bzw. die Verbindung als Parameter und holt ihn nicht selbst.

## Ablauf am Beispiel „Adresse speichern“

1. `components/addresses/form.rs` schickt das Formular über `ServerAction<UpdateAddress>`.
2. `backend/addresses/server_fns.rs::update_address` normalisiert und prüft die Eingabe mit
   `AddressInput::validate()` aus `model.rs`; bei Fehlern zurück mit `SaveOutcome::Invalid`.
3. Sie ruft `repository::update(&pool, id, &input)` auf.
4. `backend/addresses/repository.rs::update` führt das `UPDATE adresse …` aus und liefert die Anzahl
   geänderter Zeilen.
5. Bei 0 Zeilen meldet die Server-Funktion „Adresse nicht gefunden.“, sonst leitet sie auf
   `/adressen/<id>?gespeichert=1` weiter; das Modal-Fenster schliesst sich, die Daten werden neu geladen.

## Neues Modul anlegen

1. Anforderung erfassen und freigeben (`anforderungen/ANF-NNN-….md`, `index.md`).
2. Migration anlegen: `sqlx migrate add -r <name>` (Tabellen deutsch, Einzahl).
3. `src/backend/<modul>/` anlegen:
   - `mod.rs`:
     ```rust
     pub mod model;
     #[cfg(feature = "ssr")]
     pub mod repository;
     pub mod server_fns;
     ```
   - `model.rs`, `repository.rs`, `server_fns.rs` nach den Regeln oben.
   - Modul in `src/backend/mod.rs` eintragen.
4. `src/components/<modul>/` anlegen (`mod.rs`, `list.rs`, `detail.rs`, `form.rs` …),
   Modul in `src/components/mod.rs` eintragen.
5. Routen in `src/app.rs` innerhalb der `ParentRoute` ergänzen, Navigationseintrag in `src/navigation.rs`.
6. Prüfen: `cargo fmt`, beide `cargo clippy`-Aufrufe, `cargo test --features ssr`.
