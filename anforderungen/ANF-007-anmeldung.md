# ANF-007: Anmeldeseite

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-007                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Benutzer                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-12                                        |
| **Geändert**  | 2026-11-23                                        |

> **Geändert durch ANF-026 (2026-11-23):** Anmeldung und Sitzungen laufen über die **Portal-Datenbank**.
> Angemeldet wird mit **E-Mail und Passwort** des Portal-Zugangs statt mit dem Benutzernamen; neue Zugänge
> entstehen über die Registrierung (`/registrieren`). Die **Ersteinrichtung (AK-21, AK-22) entfällt**.
> Die übrigen Regeln (Sitzungsdauer, Sperre nach Fehlversuchen, Cookie, Zugangsschutz) gelten unverändert.

## 1. Beschreibung

Das ERP ist künftig nur noch nach einer **Anmeldung** nutzbar. Benutzer melden sich auf einer eigenen
**Anmeldeseite** mit **Benutzername und Passwort** an. Die Benutzer selbst werden in der
Benutzerverwaltung (ANF-005) angelegt; dort wird auch das Passwort gesetzt.

Nach der Anmeldung merkt sich das System den angemeldeten Benutzer in einer **Sitzung**. Solange die
Sitzung gültig ist, kann der Benutzer alle Seiten öffnen. Über **„Abmelden“** in der Kopfzeile beendet er
die Sitzung.

Mit der Anmeldung weiss das System, **wer** gerade arbeitet. Das ersetzt die vorläufige Benutzer-Auswahl
in der Navigation „Aufgaben“ (ANF-006 AK-2): Dort werden standardmässig die eigenen Aufgaben gezeigt.

```
Nicht angemeldet ──► /anmelden ──(Benutzername + Passwort richtig, Benutzer aktiv)──► gewünschte Seite
                         ▲                                                               │
                         └──────────────── Abmelden / Sitzung abgelaufen ◄───────────────┘
```

Die Anmeldeseite wird **ohne** Grundgerüst angezeigt (keine Seitenleiste, keine Navigation): nur eine
zentrierte Karte mit dem Anmeldeformular.

## 2. Benutzergeschichte

> Als **Benutzer** möchte ich **mich mit Benutzername und Passwort anmelden**, damit **nur berechtigte
> Personen mit dem ERP arbeiten und das System weiss, wer ich bin**.

> Als **Verwalter des Systems** möchte ich, **dass gesperrte Benutzer sich nicht mehr anmelden können**,
> damit **ich den Zugang ohne Löschen entziehen kann**.

## 3. Akzeptanzkriterien

**Zugang nur mit Anmeldung**
- [x] **AK-1:** Gegeben ein nicht angemeldeter Besucher, wenn er eine beliebige Seite öffnet (z. B.
  `/projekte/4`), dann wird er auf `/anmelden?zurueck=/projekte/4` umgeleitet. Das gilt auch beim
  Neuladen und bei direkt eingegebenen Webadressen.
- [x] **AK-2:** Gegeben ein nicht angemeldeter Besucher, wenn eine Server-Funktion aufgerufen wird (z. B.
  direkt über die Webadresse `/api/…`), dann wird sie nicht ausgeführt und meldet „Nicht angemeldet“.
  Der Schutz gilt also nicht nur für Seiten, sondern auch für alle Daten.
- [x] **AK-3:** Ausgenommen sind nur die Anmeldeseite selbst, die Anmelde-Funktion und feste Dateien
  (CSS, WebAssembly, Bilder, Seitensymbol).

**Anmeldeseite**
- [x] **AK-4:** Gegeben die Webadresse `/anmelden`, dann erscheint eine zentrierte Karte mit dem Namen
  des ERP, den Feldern **Benutzername** und **Passwort** sowie der Schaltfläche **„Anmelden“**
  (`btn-primary`). Die Seite hat kein Grundgerüst (keine Seitenleiste) und funktioniert auf Mobiltelefon
  und grossem Bildschirm.
- [x] **AK-5:** Gegeben richtige Anmeldedaten eines **aktiven** Benutzers, wenn „Anmelden“ geklickt oder
  Enter gedrückt wird, dann wird eine Sitzung angelegt und der Benutzer auf die Seite aus `zurueck`
  weitergeleitet (ohne `zurueck`: Startseite). Nur Adressen innerhalb der Anwendung werden als Ziel
  angenommen (`safe_return_url`).
- [x] **AK-6:** Beim Benutzernamen spielt die Gross-/Kleinschreibung keine Rolle; Leerzeichen am Anfang und
  Ende werden entfernt. Das Passwort wird unverändert geprüft.
- [x] **AK-7:** Gegeben ein unbekannter Benutzername, ein falsches Passwort **oder** ein gesperrter Benutzer,
  dann wird nicht angemeldet und es erscheint immer **dieselbe** Meldung: „Benutzername oder Passwort
  ist falsch.“ – damit sich nicht herausfinden lässt, welche Benutzernamen es gibt. Der eingegebene
  Benutzername bleibt im Feld stehen, das Passwortfeld wird geleert.
- [x] **AK-8:** Gegeben leere Felder, dann erscheint „Bitte Benutzername und Passwort eingeben.“
- [x] **AK-9:** Gegeben ein bereits angemeldeter Benutzer, wenn er `/anmelden` öffnet, dann wird er direkt
  auf die Startseite (bzw. `zurueck`) weitergeleitet.
- [x] **AK-10:** Gegeben eine erfolgreiche Anmeldung, dann wird „Letzte Anmeldung“ des Benutzers
  (ANF-005, Feld `letzte_anmeldung`) auf den aktuellen Zeitpunkt gesetzt.

**Schutz vor Ausprobieren von Passwörtern**
- [x] **AK-11:** Gegeben 5 fehlgeschlagene Anmeldungen für denselben Benutzernamen innerhalb von 15 Minuten,
  dann werden weitere Versuche für diesen Benutzernamen 15 Minuten lang abgewiesen mit der Meldung
  „Zu viele Fehlversuche. Bitte später erneut versuchen.“ Die Sperre gilt auch, wenn der Benutzername
  nicht existiert (gleiche Antwort, AK-7).
- [x] **AK-12:** Die Prüfung eines Passworts dauert immer etwa gleich lang – auch bei unbekanntem
  Benutzernamen wird ein Hash geprüft –, damit sich gültige Benutzernamen nicht über die Antwortzeit
  erkennen lassen.

**Sitzung**
- [x] **AK-13:** Die Sitzung wird in der Datenbank gespeichert (Tabelle `sitzung`) und im Browser nur über
  ein zufälliges, nicht erratbares Merkmal in einem Cookie wiedererkannt. Das Cookie ist `HttpOnly`,
  `SameSite=Lax`, gilt für den ganzen Pfad `/` und im Betrieb mit HTTPS zusätzlich `Secure`.
  In der Datenbank wird nur ein Hash des Merkmals gespeichert.
- [x] **AK-14:** Eine Sitzung läuft ab nach **12 Stunden ohne Aktivität** oder spätestens nach
  **7 Tagen**. Danach führt der nächste Seitenaufruf auf die Anmeldeseite (mit `zurueck`) und zeigt
  „Ihre Sitzung ist abgelaufen. Bitte erneut anmelden.“
- [x] **AK-15:** Gegeben ein Benutzer, der in der Benutzerverwaltung **gesperrt**, **gelöscht** oder dessen
  **Passwort neu gesetzt** wird, dann werden alle seine Sitzungen sofort ungültig.
- [x] **AK-16:** Bei jeder Anmeldung wird ein **neues** Sitzungsmerkmal erzeugt (keine Übernahme eines
  vorher vorhandenen Cookies).

**Abmelden und Kopfzeile**
- [x] **AK-17:** Gegeben ein angemeldeter Benutzer, dann zeigt die Kopfzeile (ANF-001) rechts seinen Namen
  (Anzeigename der Adresse) und eine Schaltfläche **„Abmelden“**. Auf dem Mobiltelefon darf der Name
  gekürzt bzw. nur als Symbol erscheinen.
- [x] **AK-18:** Gegeben „Abmelden“ wird geklickt, dann wird die Sitzung in der Datenbank gelöscht, das
  Cookie entfernt und die Anmeldeseite mit „Sie wurden abgemeldet.“ angezeigt. Die Zurück-Taste führt
  danach nicht zurück in geschützte Seiten (erneute Umleitung auf die Anmeldung).

**Zusammenspiel mit anderen Anforderungen**
- [x] **AK-19:** Gegeben die Navigation „Aufgaben“ (ANF-006) ohne Parameter `benutzer`, dann werden die
  Aufgaben des **angemeldeten** Benutzers gezeigt; die Auswahl eines anderen Benutzers bleibt möglich.
  Beim Anlegen einer Aufgabe ohne Projekt ist „Zuständig“ mit dem angemeldeten Benutzer vorbelegt.
- [x] **AK-20:** Gegeben der angemeldete Benutzer will sich in der Benutzerverwaltung selbst **sperren** oder
  **löschen**, dann wird das verhindert („Sie können sich nicht selbst sperren bzw. löschen.“), damit man
  sich nicht versehentlich aussperrt.

**Erster Benutzer** *(entfällt, ersetzt durch die Registrierung aus ANF-026)*
- [x] ~~**AK-21:**~~ Gegeben es gibt noch **keinen** Benutzer in der Datenbank, wenn `/anmelden` geöffnet wird,
  dann erscheint statt der Anmeldung die **Ersteinrichtung**: Vorname, Nachname, E-Mail (optional),
  Benutzername und Passwort mit Wiederholung. Beim Speichern werden eine Adresse vom Typ „Person“
  (ANF-002) und ein aktiver Benutzer (ANF-005) angelegt und der Benutzer direkt angemeldet.
  Es gelten die Prüfregeln aus ANF-002 und ANF-005.
- [x] ~~**AK-22:**~~ Sobald mindestens ein Benutzer existiert, ist die Ersteinrichtung nicht mehr erreichbar
  (auch nicht über die Server-Funktion).

**Allgemein**
- [x] **AK-23:** Die Tabelle `sitzung` (und die Fehlversuche) werden per neuer sqlx-Migration angelegt;
  abgelaufene Sitzungen und alte Fehlversuche werden regelmässig (z. B. beim Anmelden) gelöscht.
- [x] **AK-24:** Passwörter und Sitzungsmerkmale erscheinen nie in Protokollen, Fehlermeldungen oder in der
  Webadresse; das Anmeldeformular wird per `POST` gesendet.

## 4. Daten / Felder

**Anmeldeformular:**

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Benutzername | Text (50) | Ja | `autocomplete="username"`, ohne Beachtung der Gross-/Kleinschreibung |
| Passwort | Passwort | Ja | `autocomplete="current-password"` |

Tabelle `sitzung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Merkmal-Hash (`merkmal_hash`) | Text | Ja | SHA-256 des zufälligen Merkmals aus dem Cookie (mind. 32 Byte Zufall); eindeutig |
| Benutzer (`benutzer_id`) | Fremdschlüssel → `benutzer` | Ja | `ON DELETE CASCADE` (Sitzungen verschwinden mit dem Benutzer, AK-15) |
| Erstellt am | Zeitstempel | Ja | automatisch; für die Höchstdauer (7 Tage) |
| Zuletzt aktiv (`zuletzt_aktiv`) | Zeitstempel | Ja | wird bei Aufrufen aktualisiert (höchstens einmal pro Minute); für 12 Stunden ohne Aktivität |

Tabelle `anmeldeversuch` (Fehlversuche, AK-11):

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Benutzername (`benutzername`) | Text (50) | Ja | so wie eingegeben, klein geschrieben (auch unbekannte Namen) |
| Zeitpunkt (`zeitpunkt`) | Zeitstempel | Ja | automatisch |

**Beziehung (Datenbank):**

```
benutzer 1 ──── 0..n sitzung   (sitzung.benutzer_id NOT NULL, ON DELETE CASCADE)
```

**Webadressen:**

| Webadresse | Bedeutung |
|------------|-----------|
| `/anmelden` | Anmeldeseite (bzw. Ersteinrichtung, AK-21) |
| `/anmelden?zurueck=<adresse>` | Nach der Anmeldung dorthin weiterleiten |
| `/anmelden?abgemeldet=1` | Meldung „Sie wurden abgemeldet.“ |
| `/anmelden?abgelaufen=1` | Meldung „Ihre Sitzung ist abgelaufen.“ |

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst, Kopfzeile), ANF-002 (Adressen, für die Ersteinrichtung),
  ANF-005 (Benutzer, Passwort-Hash mit Argon2id, Status aktiv/gesperrt)
- Ändert: ANF-005 (Sitzungen beim Sperren/Löschen/Passwort setzen beenden, sich nicht selbst sperren),
  ANF-006 (Standard = angemeldeter Benutzer)
- Wird benötigt von: Rollen und Berechtigungen (noch zu erfassen)

## 6. Nicht im Umfang

- „Passwort vergessen“ bzw. Zurücksetzen per E-Mail
- Eigenes Passwort ändern durch den Benutzer selbst (vorerst nur über die Benutzerverwaltung)
- „Angemeldet bleiben“ über die 7 Tage hinaus
- Zwei-Faktor-Anmeldung, Anmeldung über externe Dienste (Microsoft, Google), Passkeys
- Rollen und Berechtigungen – nach dieser Anforderung darf jeder angemeldete Benutzer alles
- Übersicht der eigenen Sitzungen bzw. Abmelden auf allen Geräten
- Protokoll aller Anmeldungen

## 7. Offene Fragen

Vorläufige Annahmen in Klammern.

- [ ] Wie lange gilt eine Sitzung? *(vorerst: 12 Stunden ohne Aktivität, höchstens 7 Tage)*
- [ ] Wie viele Fehlversuche bis zur Sperre und wie lange? *(vorerst: 5 Versuche in 15 Minuten → 15 Minuten gesperrt)*
- [ ] Wie wird der erste Benutzer angelegt? *(vorerst: Ersteinrichtung auf der Anmeldeseite, solange es keinen Benutzer gibt, AK-21)*
- [ ] Sollen Benutzer ihr eigenes Passwort ändern können? *(vorerst: nein, eigene Anforderung)*
- [ ] Wird das ERP über HTTPS betrieben (Cookie `Secure`)? *(vorerst: im Betrieb ja, in der Entwicklung über `http://127.0.0.1` ohne `Secure`)*
- [ ] Soll die Anmeldeseite ein Logo bzw. einen Leitspruch zeigen? *(vorerst: nur der Name „uniqwork“)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261112200000_sitzung.sql` | Tabellen `sitzung` (nur Hash des Merkmals, `ON DELETE CASCADE`) und `anmeldeversuch` |
| `src/backend/auth/middleware.rs` | Zugangsschutz für **jede** Anfrage (Seiten und Server-Funktionen), in `src/main.rs` als Axum-Schicht eingebunden |
| `src/backend/auth/session.rs` | Zufälliges Merkmal (32 Byte), SHA-256, Cookie, angemeldeter Benutzer der Anfrage (`current_user()`) |
| `src/backend/auth/server_fns.rs` + `repository.rs` | Anmelden, Abmelden, Ersteinrichtung, aktueller Benutzer; Fehlversuche, Sitzungen aufräumen |
| `src/backend/auth/model.rs` | Regeln und Hilfen (Ziel nach Anmeldung, Cookie lesen, Ersteinrichtung prüfen) + Modultests |
| `src/components/auth/login.rs` | Anmeldeseite ohne Grundgerüst, Ersteinrichtung |
| `src/layout.rs` | Name und „Abmelden“ in der Kopfzeile; abgelaufene Sitzung beim Seitenwechsel → Anmeldeseite |
| `src/backend/users/server_fns.rs` | Sitzungen beim Sperren/Passwort setzen beenden, sich nicht selbst sperren/löschen |
| `src/components/tasks/list.rs` | Standard = angemeldeter Benutzer |

Hinweise:
- **AK-2, AK-3:** Öffentliche Server-Funktionen stehen unter `/oeffentlich/…` (`#[server(prefix = "/oeffentlich")]`),
  alle anderen unter `/api/…` sind ohne Sitzung gesperrt (401). Neue Server-Funktionen sind damit automatisch geschützt.
  Feste Dateien erkennt die Middleware an `/pkg/…` bzw. einer bekannten Dateiendung.
- **AK-13:** `Secure` wird gesetzt, wenn die Umgebungsvariable `SITZUNG_COOKIE_SECURE=1` gesetzt ist (Betrieb mit HTTPS).
- **AK-15:** Setzt ein Benutzer sein **eigenes** Passwort neu, bleibt seine laufende Sitzung bestehen; alle anderen
  Sitzungen werden beendet (sonst würde er sich selbst abmelden).
- **AK-4:** Die Anmeldekarte ist schmal (`max-w-sm`) – bewusste Ausnahme von der Regel „volle Breite“, wie bei Modal-Fenstern.

Geprüft auf einer frischen Testdatenbank über HTTP: Umleitungen ohne Anmeldung (Seiten 303, Server-Funktionen 401),
Ersteinrichtung (ungültig, gültig, zweites Mal gesperrt), Anmeldung (leer, falsch, unbekannt, gesperrt, Gross-/Kleinschreibung,
`zurueck` inkl. fremder Adresse), gleiche Antwortzeit, Sperre nach 5 Fehlversuchen, Cookie-Merkmale, „Letzte Anmeldung“,
Sperren/Passwort beendet Sitzungen, sich nicht selbst sperren/löschen, abgelaufene Sitzung, Abmelden, Kopfzeile,
Aufgaben des angemeldeten Benutzers; 66 Modultests. Bedienung im Browser noch nicht von Hand geprüft.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-12 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-12 | Hatem Ben Belgacem | Umgesetzt |
| 2026-11-23 | Hatem Ben Belgacem | ANF-026: Anmeldung über die Portal-Datenbank mit E-Mail; Ersteinrichtung (AK-21, AK-22) entfällt |
