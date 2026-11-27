# CLAUDE.md

Projektrahmen und Arbeitsregeln für Claude Code in diesem Projektarchiv.

## Projekt

**uniqwork** – ein ERP-System, geschrieben in **Rust** (Edition 2024).

| Schicht         | Technik |
|-----------------|---------|
| Gesamtanwendung | **Leptos 0.8** (Darstellung auf dem Server, danach im Browser interaktiv), gebaut mit `cargo-leptos` |
| Server          | **Axum 0.8**, Tokio |
| Datenbank       | **PostgreSQL** über **sqlx 0.9** (Migrationen in `migrations/`) |
| Oberfläche      | **Tailwind CSS v4** + **daisyUI v5** (über npm, nur als Werkzeug zum Erzeugen des CSS) |

Ein Rust-Paket, zwei Übersetzungen über Funktionsschalter (Cargo-Features):
- `ssr` → Serverprogramm (`src/main.rs`) mit Axum, sqlx und Server-Funktionen.
- `hydrate` → WebAssembly für den Browser (`hydrate()` in `src/lib.rs`).
Reiner Servercode (sqlx, Modul `db`) steht immer hinter `#[cfg(feature = "ssr")]` oder in `#[server]`-Funktionen.

## Struktur

```
CLAUDE.md            Diese Datei
system-design.md     Systementwurf: Schichten, Ordnerstruktur, Abhängigkeitsregeln
index.md             Übersicht aller Dokumente und Anforderungen
task.md              Aufgabenliste (In Arbeit / Offen / Erledigt)
anforderungen/       Eine Markdown-Datei pro Anforderung
  _VORLAGE.md        Vorlage für neue Anforderungen
src/
  main.rs            Serverstart: .env, Verbindungspool, Axum-Routen (nur ssr)
  lib.rs             Module + Einstieg hydrate() für den Browser
  app.rs             HTML-Rahmen, Komponente `App`, alle Routen
  layout.rs          Grundgerüst: Kopfzeile + Seitenleiste (ANF-001)
  navigation.rs      Zentrale Navigationseinträge (Haupt- und Einstellungsnavigation)
  icons.rs           Eingebettete SVG-Symbole
  forms.rs           Gemeinsame Formular-Bausteine: TextField, SelectField, SearchPicker, Fehler, Pfadleiste, SaveOutcome
  backend/<modul>/   model.rs (Datentypen + Prüfregeln), server_fns.rs (#[server]), repository.rs (SQL, nur ssr)
  components/<modul>/ Eine Datei pro Komponente bzw. Ansicht (list.rs, detail.rs, form.rs …)
                     Module: addresses (ANF-002), projects (ANF-003), purchase_orders (ANF-004), users (ANF-005), tasks (ANF-006), auth (ANF-007, ANF-026),
                     portal (ANF-026, Seite /portal), site_plans (ANF-027 Geländeplan /projekte/<Kennung>/gelaendeplan, ANF-028 Besucherströme,
                     mit map_bridge.js: MapLibre + Three.js als ES-Module aus dem Netz, nur im Browser),
                     system (Datenbank-Status), home, settings, company (ANF-025, Seite unter settings/company) – Details in system-design.md
  backend/finance/<teil>/  Finanzen (ANF-010), je Komponente ein Untermodul: accounts (ANF-011 Kontenplan),
                     journal (ANF-017), debtors (ANF-018 Debitoren, Druckansicht /finanzen/debitoren/<Kennung>/druck) …
  components/settings/finance/<teil>/  Stammdaten-Seiten der Finanzen unter /einstellungen/finanzen/…
  db.rs              Verbindungspools (ERP und Portal) + Migrationen (nur ssr)
migrations/          SQL-Migrationen der ERP- bzw. Mandanten-Datenbank (sqlx), laufen beim Serverstart automatisch
migrations_portal/   SQL-Migrationen der Portal-Datenbank (ANF-026: Zugänge, Mandanten, Sitzungen)
public/              Feste Dateien (Bilder, Seitensymbol), werden unverändert ausgeliefert
styles/app.css       Tailwind-Eingabe (Quelle, wird bearbeitet)
styles/output.css    Erzeugtes CSS (nicht bearbeiten, nicht eingecheckt)
.env                 DATABASE_URL, PORTAL_DATABASE_URL (nicht eingecheckt, Vorlage: .env.example)
.cargo/config.toml   Übersetzer-Option erase_components (siehe Code)
Dockerfile           Docker-Image für Railway in Stufen, mit cargo-chef (ANF-008)
.dockerignore        Was nicht ins Image kommt (target/, node_modules/, .env …)
railway.toml         Railway-Einstellungen: Bau mit Dockerfile, Neustart-Regel, Region Europa (ANF-008)
```

## Befehle

```bash
# Einmalig
npm install                     # Tailwind + daisyUI
cp .env.example .env            # DATABASE_URL eintragen
npm run css:build               # styles/output.css erzeugen (vor dem ersten cargo leptos nötig)

# Entwicklung (zwei Terminalfenster)
npm run css:watch               # CSS neu bauen bei Änderungen
cargo leptos watch              # Server + Browserteil, lädt bei Änderungen neu: http://127.0.0.1:3002

# Prüfen
cargo fmt
cargo clippy --features ssr            # unter Linux mit RUSTFLAGS="--cfg erase_components" (wie im Dockerfile)
cargo clippy --lib --features hydrate --target wasm32-unknown-unknown
cargo test --features ssr

# Auslieferung (optimiert)
npm run css:build && cargo leptos build --release

# Migrationen
sqlx migrate add -r <name>      # neue Migration in migrations/ (ERP)
sqlx migrate add -r --source migrations_portal <name>   # neue Migration der Portal-Datenbank

# Docker-Image (ANF-008) – lokal bauen und ausprobieren
docker build -t uniqwork .
docker run --rm -p 8080:8080 -e PORT=8080 -e DATABASE_URL=postgres://… uniqwork
```

Betrieb auf Railway (ANF-008): Railway baut bei jedem neuen Stand auf `main` das `Dockerfile` und stellt es
bereit. Umgebungsvariablen in der Railway-Oberfläche: `DATABASE_URL=${{Postgres.DATABASE_URL}}`,
`SITZUNG_COOKIE_SECURE=1`. Der Benutzer in `DATABASE_URL` muss Datenbanken und Rollen anlegen dürfen
(ANF-026; auf Railway der Benutzer `postgres`). Nur **eine** Server-Instanz je PostgreSQL-Dienst: Das Passwort der
Rolle `uniqwork_erp` wird bei jedem Start neu gesetzt. `PORT` setzt Railway selbst; der Server hört dann auf `0.0.0.0:$PORT`
(`listen_addr` in `src/main.rs`), sonst auf `site-addr` aus `Cargo.toml`. Details in `system-design.md`.

## Arbeitsregeln

### Sprache
- **Alle Begriffe werden auf Deutsch geführt** – in Dokumentation, Anforderungen und Oberfläche.
  Keine englischen Fachbegriffe, wenn es ein deutsches Wort gibt (siehe Begriffe unten).
- Ausnahmen: Eigennamen von Werkzeugen (Rust, Leptos, PostgreSQL, Tailwind …), Befehle,
  Dateinamen und wörtlich zitierter Code.
- Datenbank: Tabellen- und Spaltennamen deutsch, klein, ohne Umlaute (`ä→ae`, `ö→oe`, `ü→ue`, `ß→ss`).
  Tabellennamen immer in der **Einzahl**, z. B. `adresse`, `projekt`, `vorgang` (nicht `adressen`).
  Technischer Schlüssel vorläufig `id`, Fremdschlüssel `<tabelle>_id` (z. B. `adresse_id`) – noch offen, ob `kennung`.
- Rust-Code (Bezeichner, Typen, Funktionen) auf Englisch; Kommentare auf Deutsch.

### Begriffe

| Deutsch (verwenden) | Nicht verwenden |
|---------------------|-----------------|
| Anforderung (`ANF-…`) | Requirement (`REQ-…`) |
| Anlegen, Anzeigen, Bearbeiten, Löschen (Grundfunktionen) | CRUD, Create, Read, Update, Delete |
| Benutzergeschichte | User Story |
| Grundgerüst | AppLayout, Layout |
| Kopfzeile | Header |
| Fusszeile | Footer |
| Seitenleiste | Sidebar |
| Pfadleiste | Breadcrumb |
| Kontrollkästchen | Checkbox |
| Webadresse | URL |
| Kennung | Id |
| Farbschema | Theme |
| Mobiltelefon / Mobilgerät | Smartphone, Mobile |
| grosser Bildschirm | Desktop |
| Webseite | Website |
| Symbol | Icon |
| Modultest | Unit-Test |
| Aufgabe | Task |

### Anforderungen
- Neue Anforderung = Kopie von `anforderungen/_VORLAGE.md`.
- Dateiname: `ANF-<NNN>-<kurzer-titel>.md` (dreistellig, fortlaufend, Kleinbuchstaben mit Bindestrichen,
  ohne Umlaute), z. B. `ANF-004-artikelstamm.md`.
- Nummern werden nie wiederverwendet; verworfene Anforderungen bekommen Status „Verworfen“.
- Nach jeder neuen oder geänderten Anforderung die Tabelle in `index.md` aktualisieren
  (Nummer, Titel, Modul, Priorität, Status).

### Aufgaben
- `task.md` ist die Quelle für den aktuellen Stand.
- Vor Beginn einer Aufgabe nach „In Arbeit“ verschieben, danach nach „Erledigt“ abhaken.
- Aufgaben, die eine Anforderung umsetzen, verweisen auf deren Nummer (z. B. `ANF-001`).

### Gestaltung
- Nur Tailwind-Klassen und daisyUI-Komponenten (`btn`, `card`, `table`, …), kein eigenes CSS ohne Grund.
- Tailwind sucht Klassen nur in `src/`. Weitere Ordner in `styles/app.css` per `@source` eintragen.
- **Farbpalette (ANF-009)**, hinterlegt im daisyUI-Farbschema in `styles/app.css`:
  - **Primärfarbe Indigo** (`--primaerfarbe`, `primary`): Hauptschaltflächen, Hauptnavigation, aktive Zustände,
    Fokus – `btn-primary`, `link-primary`, `checkbox-primary`, `bg-primary`/`text-primary-content` usw.
  - **Sekundärfarbe Teal** (`--sekundaerfarbe`, `secondary`): Akzente, sekundäre Filter, dezente Hervorhebungen
    (z. B. aktive Sortierspalte `text-secondary`, Plakette `badge-secondary`).
  - **Neutrale Töne** (`base-100` … `base-300`, `base-content`, `neutral`): Hintergründe, Oberflächen, Rahmen, Text.
    Die Kopfzeile ist dunkles Anthrazit (`bg-neutral`).
  - **Signalfarben** nur für Meldungen und Status: `error` Rot, `warning` Gelb, `success` Grün, `info` Blau.
  Der aktive Navigationseintrag ist in **leichtem Indigo** markiert (15 % Primärfarbe als Hintergrund, Schrift in
  dunklem Indigo) – Flächen für Markierungen so, kräftiges Indigo nur für Hauptschaltflächen.
- Farbwerte nur in `styles/app.css` ändern, nie als Hex-Wert im Code.
- **Ebenen-Farben der Projekte** (Tokens in `styles/app.css`, Klassen über `Level::stripe_class()`,
  `badge_class()`, `tint_class()`): Projekt = Indigo, Teilprojekt = Himmelblau, Vorgang = Petrol, Aufgabe = Schiefergrau.
  Nicht für Status verwenden – Status hat eigene Plaketten (`status_badge`).
- Inhalte nutzen die **ganze Bildschirmbreite**: keine `max-w-*`-Begrenzung für Seiten, Karten oder Formulare. Ausnahme: Modal-Fenster (`max-w-4xl`).
- Klassennamen immer vollständig schreiben, nicht zusammensetzen (`"btn-" + farbe` wird nicht erkannt).

### Navigation & Seiten
- **Eingaben immer in Modal-Fenstern**, nie auf eigenen Formularseiten. Ob ein Fenster offen ist, steht als
  Parameter in der Webadresse (`?neu=…`, `?unter=…`, `?bearbeiten=…`, `?id=…`), damit Neuladen und Zurück-Taste
  funktionieren. Bausteine in `src/forms.rs`: `Modal`, `use_base_href()` (Adresse ohne Fenster-Parameter =
  Schliessen-Ziel), `with_params()`, `refresh_when()`/`is_saved()` + `data_version()` (Daten nach dem Speichern neu laden).
- Neue Seite: Komponente in `src/components/<modul>/`, Daten über `src/backend/<modul>/server_fns.rs` (siehe `system-design.md`), Route in `src/app.rs` innerhalb der `ParentRoute` mit dem Grundgerüst (`AppLayout`).
- Navigationseintrag nur in `src/navigation.rs` ergänzen (`MAIN_NAV`, `SETTINGS_NAV` bzw. `SETTINGS_FINANCE_NAV`
  für den Abschnitt „Finanzen“ der Einstellungen), nicht im Grundgerüst.
- Einstellungsseiten liegen unter `/einstellungen/…` – dann zeigt die Seitenleiste automatisch die Einstellungsnavigation.
- Interne Verweise als normales `<a href>` – Leptos wechselt die Seite dann ohne Neuladen.

### Anmeldung und Zugangsschutz (ANF-007, ANF-026)
- Registrierung (`/registrieren`), Anmeldung (E-Mail + Passwort) und Sitzungen laufen über die **Portal-Datenbank**.
- Jede Anfrage läuft durch `src/backend/auth/middleware.rs`: Seiten ohne Sitzung → `/anmelden`,
  Server-Funktionen unter `/api/…` ohne Sitzung → 401. Mit Sitzung, aber ohne bereiten und definierten Mandanten
  ist nur das Portal erreichbar: Seiten → `/portal` (ist die Datenbank bereit, aber der Mandant nicht definiert:
  `/portal?mandant=<Kennung>` mit dem Modal-Fenster „Mandant definieren“), ERP-Server-Funktionen → 403.
  Neue `#[server]`-Funktionen sind damit automatisch geschützt.
- Nur Funktionen, die **ohne** Anmeldung gehen müssen, bekommen `#[server(prefix = "/oeffentlich")]` – sparsam verwenden.
- Funktionen des Portals, die angemeldet, aber **ohne Mandant** gehen müssen, bekommen `#[server(prefix = "/api/portal")]`.
- Passwörter gibt es nur für Zugänge der Portal-Datenbank (`crate::backend::auth::password`, `PasswordInput` in
  `auth::model`). Benutzer der Mandanten-Datenbank haben kein Passwort; ihr Passwort ändern sie im Portal.
- Den angemeldeten Zugang in Server-Funktionen mit `crate::backend::auth::session::current_user()` bzw. `require_user()`
  holen. `CurrentUser::benutzer_id` ist der Benutzer in der Mandanten-Datenbank (z. B. für „meine Aufgaben“).

### Datenbank
- Schemaänderungen nur über neue Migrationsdateien, bestehende Migrationen nie ändern.
- Den Verbindungspool in Server-Funktionen mit `crate::db::pool()?` holen. Er ist der Pool der **Mandanten-Datenbank**
  der laufenden Anfrage (die Middleware legt ihn in die Anfrage); ohne Mandant liefert er „Kein Mandant gewählt.“
- **Datenbanken (ANF-026):** Die Portal-Datenbank (`uniqwork_portal`, Pool über `crate::db::portal_pool()?`)
  enthält nur Zugänge, Sitzungen und das Verzeichnis der Mandanten – nie ERP-Tabellen; Migrationen in
  `migrations_portal/`. Jeder Mandant hat eine eigene Datenbank `uq_m_…` mit den Migrationen aus `migrations/`;
  ERP-Tabellen gehören nur dorthin. Neue Migrationen laufen beim Serverstart in allen Mandanten-Datenbanken.
- Das ERP arbeitet mit der Rolle `uniqwork_erp`: Daten lesen/schreiben ja, Tabellen anlegen/ändern, `TRUNCATE` und
  `CREATE DATABASE` nein. Schemaänderungen deshalb nur über Migrationen, nie zur Laufzeit.
- Eingaben immer als Parameter (`$1`, `.bind(…)`), nie in den SQL-Text einsetzen. Zusammengesetzte SQL-Texte
  verlangt sqlx 0.9 mit `AssertSqlSafe(…)` – nur verwenden, wenn alle Teile feste Texte oder Werte aus einer
  festen Liste sind (Beispiel: Sortierung in `src/backend/addresses/repository.rs`).
- SQL steht nur in `repository.rs`; Server-Funktionen (`server_fns.rs`) prüfen, steuern Transaktionen und melden Fehler.
- Prüfregeln für Eingaben im gemeinsamen Modell (`model.rs`) ablegen und auf dem Server immer ausführen.

### Code
- `.cargo/config.toml` setzt `--cfg erase_components` (Leptos). Ohne diese Option stürzt der Apple-Linker
  bei grossen Ansichten ab. Nicht entfernen; bei Übersetzungsfehlern mit langen Typen zuerst daran denken.
- Rekursive Ansichten (z. B. Strukturbaum) als Funktion mit Rückgabetyp `AnyView` schreiben.
- Vor dem Abschluss: `cargo fmt`, beide `cargo clippy`-Aufrufe und `cargo test --features ssr` müssen sauber durchlaufen.
- Umsetzung nur auf Basis freigegebener Anforderungen; bei Unklarheiten nachfragen.
