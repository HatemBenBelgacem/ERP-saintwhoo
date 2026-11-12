# CLAUDE.md

Projektrahmen und Arbeitsregeln für Claude Code in diesem Projektarchiv.

## Projekt

**ERP-saintwhoo** – ein ERP-System, geschrieben in **Rust** (Edition 2024).

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
                     Module: addresses (ANF-002), projects (ANF-003), purchase_orders (ANF-004), users (ANF-005), tasks (ANF-006), auth (ANF-007),
                     system (Datenbank-Status), home, settings – Details in system-design.md
  db.rs              Verbindungspool + Migrationen (nur ssr)
migrations/          SQL-Migrationen (sqlx), laufen beim Serverstart automatisch
public/              Feste Dateien (Bilder, Seitensymbol), werden unverändert ausgeliefert
styles/app.css       Tailwind-Eingabe (Quelle, wird bearbeitet)
styles/output.css    Erzeugtes CSS (nicht bearbeiten, nicht eingecheckt)
.env                 DATABASE_URL (nicht eingecheckt, Vorlage: .env.example)
.cargo/config.toml   Übersetzer-Option erase_components (siehe Code)
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
cargo clippy --features ssr
cargo clippy --lib --features hydrate --target wasm32-unknown-unknown
cargo test --features ssr

# Auslieferung (optimiert)
npm run css:build && cargo leptos build --release

# Migrationen
sqlx migrate add -r <name>      # neue Migration in migrations/
```

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
- **Glutorange (`--glutorange`, #E25822) ist die Grundfarbe des ERP.** Sie ist im daisyUI-Farbschema als
  `primary` hinterlegt (`styles/app.css`). Hervorhebungen immer über `primary` gestalten:
  `bg-primary`/`text-primary-content`, `btn-primary`, `link-primary`, `checkbox-primary` usw.
  Der aktive Navigationseintrag ist in **leichtem Glutorange** markiert (15 % Grundfarbe als Hintergrund,
  Schrift in dunklem Glutorange) – Flächen für Markierungen so, kräftiges Glutorange nur für Hauptschaltflächen. Die Kopfzeile ist neutral in dunklem Anthrazit (`bg-neutral`).
- Den Farbwert nur in `styles/app.css` (`--glutorange`) ändern, nie als Hex-Wert im Code.
- **Ebenen-Farben der Projekte** (Tokens in `styles/app.css`, Klassen über `Level::stripe_class()`,
  `badge_class()`, `tint_class()`): Projekt = Glutorange, Teilprojekt = Blau, Vorgang = Petrol, Aufgabe = Schiefergrau.
  Nicht für Status verwenden – Status hat eigene Plaketten (`status_badge`).
- Inhalte nutzen die **ganze Bildschirmbreite**: keine `max-w-*`-Begrenzung für Seiten, Karten oder Formulare. Ausnahme: Modal-Fenster (`max-w-4xl`).
- Klassennamen immer vollständig schreiben, nicht zusammensetzen (`"btn-" + farbe` wird nicht erkannt).

### Navigation & Seiten
- **Eingaben immer in Modal-Fenstern**, nie auf eigenen Formularseiten. Ob ein Fenster offen ist, steht als
  Parameter in der Webadresse (`?neu=…`, `?unter=…`, `?bearbeiten=…`, `?id=…`), damit Neuladen und Zurück-Taste
  funktionieren. Bausteine in `src/forms.rs`: `Modal`, `use_base_href()` (Adresse ohne Fenster-Parameter =
  Schliessen-Ziel), `with_params()`, `refresh_when()`/`is_saved()` + `data_version()` (Daten nach dem Speichern neu laden).
- Neue Seite: Komponente in `src/components/<modul>/`, Daten über `src/backend/<modul>/server_fns.rs` (siehe `system-design.md`), Route in `src/app.rs` innerhalb der `ParentRoute` mit dem Grundgerüst (`AppLayout`).
- Navigationseintrag nur in `src/navigation.rs` ergänzen (`MAIN_NAV` bzw. `SETTINGS_NAV`), nicht im Grundgerüst.
- Einstellungsseiten liegen unter `/einstellungen/…` – dann zeigt die Seitenleiste automatisch die Einstellungsnavigation.
- Interne Verweise als normales `<a href>` – Leptos wechselt die Seite dann ohne Neuladen.

### Anmeldung und Zugangsschutz (ANF-007)
- Jede Anfrage läuft durch `src/backend/auth/middleware.rs`: Seiten ohne Sitzung → `/anmelden`,
  Server-Funktionen unter `/api/…` ohne Sitzung → 401. Neue `#[server]`-Funktionen sind damit automatisch geschützt.
- Nur Funktionen, die **ohne** Anmeldung gehen müssen, bekommen `#[server(prefix = "/oeffentlich")]` – sparsam verwenden.
- Den angemeldeten Benutzer in Server-Funktionen mit `crate::backend::auth::session::current_user()` bzw. `require_user()` holen.

### Datenbank
- Schemaänderungen nur über neue Migrationsdateien, bestehende Migrationen nie ändern.
- Den Verbindungspool in Server-Funktionen mit `crate::db::pool()?` holen (stellt `main.rs` über den Kontext bereit).
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
