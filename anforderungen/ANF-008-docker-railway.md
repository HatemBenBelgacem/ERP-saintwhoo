# ANF-008: Auslieferung als Docker-Image für Railway

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-008                                           |
| **Status**    | Entwurf                                           |
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
- [ ] **AK-1:** Gegeben das Projektarchiv, wenn `docker build -t erp-saintwhoo .` ausgeführt wird, dann
  entsteht ohne Fehler ein Image. Es gibt dafür ein `Dockerfile` und eine `.dockerignore` im
  Hauptordner.
- [ ] **AK-2:** Das `Dockerfile` baut in mehreren Stufen (CSS, Rust, Laufzeit). Das Laufzeit-Image enthält
  keine Übersetzer (kein `cargo`, kein `node`) und keinen Quellcode.
- [ ] **AK-3:** Die `.dockerignore` schliesst mindestens `target/`, `node_modules/`, `.env`,
  `styles/output.css` und `.git/` aus. Die Datei `.env` gelangt nie ins Image.
- [ ] **AK-4:** Das Serverprogramm läuft im Container nicht als Benutzer `root`.
- [ ] **AK-5:** Gegeben ein gestartetes Image mit gültiger `DATABASE_URL`, wenn die Startseite geöffnet wird,
  dann erscheint die Anmeldeseite (ANF-007) mit vollständigem Aussehen (CSS geladen) und sie ist im
  Browser interaktiv (WebAssembly geladen).

**Konfiguration über Umgebungsvariablen**
- [ ] **AK-6:** Gegeben die Umgebungsvariable `PORT` (von Railway vorgegeben), dann hört der Server auf
  `0.0.0.0:<PORT>`. Ohne `PORT` gilt weiterhin `site-addr` aus `Cargo.toml` – die lokale Entwicklung
  mit `cargo leptos watch` bleibt unverändert.
- [ ] **AK-7:** Die Verbindung zur Datenbank kommt ausschliesslich aus `DATABASE_URL`. Zugangsdaten stehen
  weder im `Dockerfile` noch im Code.
- [ ] **AK-8:** Im Image ist `LEPTOS_ENV=PROD` gesetzt; `LEPTOS_SITE_ROOT` zeigt auf den mitgelieferten
  Ordner `site`.
- [ ] **AK-9:** Auf Railway ist `SITZUNG_COOKIE_SECURE=1` gesetzt, damit das Sitzungs-Cookie nur über HTTPS
  gesendet wird (ANF-007).

**Betrieb auf Railway**
- [ ] **AK-10:** Gegeben ein Railway-Projekt mit PostgreSQL-Dienst und dem verbundenen GitHub-Projektarchiv,
  wenn auf `main` ein neuer Stand eingeht, dann baut Railway das Image aus dem `Dockerfile` und stellt es
  bereit. Die Einstellungen dafür stehen in `railway.toml` im Projektarchiv.
- [ ] **AK-11:** Gegeben eine leere Datenbank auf Railway, wenn der Container zum ersten Mal startet, dann
  laufen alle Migrationen durch und die Ersteinrichtung (ANF-007) ist erreichbar.
- [ ] **AK-12:** Gegeben ein neuer Stand mit zusätzlichen Migrationen, wenn er bereitgestellt wird, dann
  laufen nur die neuen Migrationen; bestehende Daten bleiben erhalten.
- [ ] **AK-13:** Gegeben eine nicht erreichbare Datenbank beim Start, dann beendet sich der Server mit einer
  verständlichen Fehlermeldung im Protokoll, und Railway startet ihn erneut (Neustart-Regel in
  `railway.toml`).
- [ ] **AK-14:** Die Anwendung ist über die von Railway vergebene Webadresse per HTTPS erreichbar.

**Dokumentation**
- [ ] **AK-15:** `CLAUDE.md` (Abschnitt „Befehle“) und `system-design.md` beschreiben Bau des Images,
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
- Eigene Domain (z. B. `erp.saintwhoo.ch`) – zunächst die Webadresse von Railway
- Automatische Prüfungen vor dem Bereitstellen (GitHub Actions mit `cargo test`, `clippy`)
- Getrennte Umgebungen (Test / Betrieb)
- `docker-compose` für die lokale Entwicklung
- Überwachung und Alarmierung über das Protokoll von Railway hinaus

## 7. Offene Fragen

- [ ] Bereitstellung automatisch bei jedem Stand auf `main` oder nur auf Knopfdruck?
- [ ] Railway-Region (z. B. Europa West) – wegen Datenschutz und Antwortzeit?
- [ ] Darf das Serverprogramm die Datenbank selbst anlegen (heute `create_database` in `src/db.rs`),
  oder wird sie auf Railway immer vorgegeben? Falls das Anlegen fehlschlägt, muss der Start trotzdem gehen.
- [ ] Braucht es einen Gesundheitspfad (z. B. `/oeffentlich/gesund`) für die Prüfung durch Railway, oder
  genügt die Anmeldeseite?
- [ ] Bauzeit: Rust-Bau auf Railway ist langsam – Zwischenspeicher für Abhängigkeiten (z. B. `cargo-chef`)
  gewünscht?

## 8. Änderungshistorie

| Datum      | Autor | Änderung        |
|------------|-------|-----------------|
| 2026-11-17 | Hatem Ben Belgacem | Erstellt |
