# ANF-008: Auslieferung als Docker-Image für Railway

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-008                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Technisch                                         |
| **Modul**     | Betrieb                                           |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-17                                        |
| **Geändert**  | 2026-11-17                                        |

## 1. Beschreibung

Das ERP läuft bisher nur auf dem eigenen Rechner (`cargo leptos watch`, `http://127.0.0.1:3002`).
Damit es im Internet erreichbar ist, wird es als **Docker-Image** gebaut und auf **Railway** betrieben.
Die Datenbank ist ein **PostgreSQL-Dienst auf Railway** im selben Railway-Projekt.

Das Image enthält alles, was zum Betrieb nötig ist: das Serverprogramm, den Browserteil (WebAssembly),
das erzeugte CSS und die festen Dateien aus `public/`. Für den Betrieb werden weder Rust noch Node.js
benötigt.

```
GitHub (main) ──► Railway baut das Dockerfile ──► Container: Serverprogramm
                                                       │  hört auf 0.0.0.0:$PORT
                                                       │  DATABASE_URL ──► PostgreSQL (Railway)
                                                       ▼
                                   https://<name>.up.railway.app  (HTTPS durch Railway)
```

Der Bau läuft in Stufen:
1. **CSS-Stufe** (Node.js): `npm ci` und `npm run css:build` erzeugen `styles/output.css`.
2. **Rust-Stufe**: `cargo leptos build --release` übersetzt Serverprogramm und WebAssembly
   (Zielplattform `wasm32-unknown-unknown`, `.cargo/config.toml` mit `erase_components` bleibt wirksam).
3. **Laufzeit-Stufe**: schlankes Image (z. B. Debian slim) nur mit dem Serverprogramm und `target/site`.

Die Migrationen sind über `sqlx::migrate!` bereits im Serverprogramm enthalten und laufen wie bisher
beim Start automatisch.

## 2. Benutzergeschichte

> Als **Entwickler** möchte ich **das ERP mit einem Dockerfile auf Railway bereitstellen**, damit **die
> Benutzer es ohne eigene Installation über eine Webadresse nutzen können**.

> Als **Benutzer** möchte ich **das ERP über eine sichere Webadresse (HTTPS) öffnen**, damit **meine
> Anmeldedaten geschützt übertragen werden**.

## 3. Akzeptanzkriterien

**Docker-Image**
- [x] **AK-1:** Gegeben das Projektarchiv, wenn `docker build -t uniqwork .` ausgeführt wird, dann
  entsteht ohne Fehler ein Image. Es gibt dafür ein `Dockerfile` und eine `.dockerignore` im
  Hauptordner.
- [x] **AK-2:** Das `Dockerfile` baut in mehreren Stufen (CSS, Rust, Laufzeit). Das Laufzeit-Image enthält
  keine Übersetzer (kein `cargo`, kein `node`) und keinen Quellcode.
- [x] **AK-3:** Die `.dockerignore` schliesst mindestens `target/`, `node_modules/`, `.env`,
  `styles/output.css` und `.git/` aus. Die Datei `.env` gelangt nie ins Image.
- [x] **AK-4:** Das Serverprogramm läuft im Container nicht als Benutzer `root`.
- [x] **AK-5:** Gegeben ein gestartetes Image mit gültiger `DATABASE_URL`, wenn die Startseite geöffnet wird,
  dann erscheint die Anmeldeseite (ANF-007) mit vollständigem Aussehen (CSS geladen) und sie ist im
  Browser interaktiv (WebAssembly geladen).

**Konfiguration über Umgebungsvariablen**
- [x] **AK-6:** Gegeben die Umgebungsvariable `PORT` (von Railway vorgegeben), dann hört der Server auf
  `0.0.0.0:<PORT>`. Ohne `PORT` gilt weiterhin `site-addr` aus `Cargo.toml` – die lokale Entwicklung
  mit `cargo leptos watch` bleibt unverändert.
- [x] **AK-7:** Die Verbindung zur Datenbank kommt ausschliesslich aus `DATABASE_URL`. Zugangsdaten stehen
  weder im `Dockerfile` noch im Code.
- [x] **AK-8:** Im Image ist `LEPTOS_ENV=PROD` gesetzt; `LEPTOS_SITE_ROOT` zeigt auf den mitgelieferten
  Ordner `site`.
- [ ] **AK-9:** Auf Railway ist `SITZUNG_COOKIE_SECURE=1` gesetzt, damit das Sitzungs-Cookie nur über HTTPS
  gesendet wird (ANF-007).

**Betrieb auf Railway**
- [ ] **AK-10:** Gegeben ein Railway-Projekt mit PostgreSQL-Dienst und dem verbundenen GitHub-Projektarchiv,
  wenn auf `main` ein neuer Stand eingeht, dann baut Railway **automatisch** (ohne Knopfdruck) das Image aus dem `Dockerfile` und stellt es
  bereit. Die Einstellungen dafür stehen in `railway.toml` im Projektarchiv.
- [x] **AK-11:** Gegeben eine leere Datenbank auf Railway, wenn der Container zum ersten Mal startet, dann
  legt der Server die Datenbank an, falls sie fehlt (wie bisher in `src/db.rs`), alle Migrationen laufen durch und die Ersteinrichtung (ANF-007) ist erreichbar.
- [x] **AK-12:** Gegeben ein neuer Stand mit zusätzlichen Migrationen, wenn er bereitgestellt wird, dann
  laufen nur die neuen Migrationen; bestehende Daten bleiben erhalten.
- [x] **AK-13:** Gegeben eine nicht erreichbare Datenbank beim Start, dann beendet sich der Server mit einer
  verständlichen Fehlermeldung im Protokoll, und Railway startet ihn erneut (Neustart-Regel in
  `railway.toml`).
- [ ] **AK-14:** Die Anwendung ist über die von Railway vergebene Webadresse per HTTPS erreichbar.
- [ ] **AK-16:** Anwendung und PostgreSQL-Dienst laufen in einer **europäischen** Railway-Region
  („EU West“), damit die Daten in Europa bleiben. Die Region ist in `railway.toml` festgelegt.

**Bauzeit**
- [x] **AK-17:** Das `Dockerfile` verwendet `cargo-chef`: Die fremden Bibliotheken werden in einem eigenen
  Bauschritt übersetzt, der nur von `Cargo.toml` und `Cargo.lock` abhängt (`cargo chef prepare` /
  `cargo chef cook`), und zwar für beide Übersetzungen (Server mit `ssr`, WebAssembly mit `hydrate`).
  Gegeben ein neuer Stand, der nur Dateien in `src/` ändert, dann werden die Bibliotheken nicht neu
  übersetzt, sofern Railway den Zwischenspeicher behalten hat. `cargo-chef` ist im Laufzeit-Image nicht
  enthalten.

**Dokumentation**
- [x] **AK-15:** `CLAUDE.md` (Abschnitt „Befehle“) und `system-design.md` beschreiben Bau des Images,
  lokalen Probelauf mit Docker und die nötigen Umgebungsvariablen auf Railway.

## 4. Daten / Felder (optional)

Umgebungsvariablen im Betrieb:

| Variable | Wert auf Railway | Pflicht | Bemerkung |
|----------|------------------|---------|-----------|
| `DATABASE_URL` | Verweis auf den PostgreSQL-Dienst (`${{Postgres.DATABASE_URL}}`) | Ja | Zugangsdaten nur hier |
| `PORT` | von Railway gesetzt | Ja | Server hört auf `0.0.0.0:$PORT` |
| `SITZUNG_COOKIE_SECURE` | `1` | Ja | ANF-007, nur über HTTPS |
| `LEPTOS_ENV` | `PROD` (im Image gesetzt) | Ja | |
| `LEPTOS_SITE_ROOT` | `site` (im Image gesetzt) | Ja | Ordner mit CSS, WebAssembly, `public/` |
| `RUST_LOG` | z. B. `info` | Nein | Ausführlichkeit des Protokolls |

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 bis ANF-007 (bestehende Anwendung), insbesondere ANF-007 (Sitzungs-Cookie, Ersteinrichtung)
- Wird benötigt von: –

## 6. Nicht im Umfang

- Datensicherung der Datenbank (Backups) und Wiederherstellung
- Eigene Domain (z. B. `erp.uniqwork.ch`) – zunächst die Webadresse von Railway
- Automatische Prüfungen vor dem Bereitstellen (GitHub Actions mit `cargo test`, `clippy`)
- Getrennte Umgebungen (Test / Betrieb)
- `docker-compose` für die lokale Entwicklung
- Überwachung und Alarmierung über das Protokoll von Railway hinaus
- Eigener Gesundheitspfad für die Prüfung durch Railway

## 7. Offene Fragen

- [x] Bereitstellung automatisch bei jedem Stand auf `main` oder nur auf Knopfdruck?
  → **Automatisch** bei jedem neuen Stand auf `main` (AK-10).
- [x] Railway-Region? → **Europa** (Railway-Region „EU West“), damit die Daten in Europa bleiben (AK-16).
- [x] Darf das Serverprogramm die Datenbank selbst anlegen? → **Ja**, wie heute in `src/db.rs` (AK-11).
- [x] Braucht es einen Gesundheitspfad für Railway? → **Nein**.
- [x] Bauzeit: Zwischenspeicher für Abhängigkeiten mit `cargo-chef` gewünscht? → **Ja** (AK-17).

## 8. Änderungshistorie

| Datum      | Autor | Änderung        |
|------------|-------|-----------------|
| 2026-11-17 | Hatem Ben Belgacem | Erstellt |
| 2026-11-17 | Hatem Ben Belgacem | Offene Fragen 1–4 beantwortet: automatische Bereitstellung, Region Europa (AK-16), Server legt Datenbank an, kein Gesundheitspfad |
| 2026-11-17 | Hatem Ben Belgacem | Frage 5 beantwortet: `cargo-chef` (AK-17); Status Freigegeben |
| 2026-11-17 | Claude Code | Umgesetzt: `Dockerfile`, `.dockerignore`, `railway.toml`, `PORT` in `src/main.rs`. Lokal mit Docker geprüft (AK-1–8, 11–13, 15, 17). Offen: Einrichtung auf Railway (AK-9, 10, 14, 16) |
