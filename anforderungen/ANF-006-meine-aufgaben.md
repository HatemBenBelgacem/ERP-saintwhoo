# ANF-006: Aufgaben eines Benutzers und Aufgaben ohne Projekt

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-006                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Aufgaben                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-12                                        |
| **Geändert**  | 2026-11-12                                        |

## 1. Beschreibung

Das ERP erhält in der Hauptnavigation (ANF-001) den neuen Eintrag **„Aufgaben“** unter dem Pfad `/aufgaben`.
Dort sieht ein Benutzer an einer Stelle **alle Aufgaben, die ihm zugeordnet sind** – egal aus welchem
Projekt sie stammen.

Es gibt zwei Arten von Aufgaben:

| Art | Woher | Wem zugeordnet | Wo anlegen |
|-----|-------|----------------|------------|
| **Projektaufgabe** | ANF-003: liegt unter einem Vorgang | Allen Benutzern, die beim Vorgang, Teilprojekt oder Projekt als verantwortlich hinterlegt sind (geerbt, ANF-005 AK-25 bis AK-29) | Nur im Projekt (wie bisher) |
| **Aufgabe ohne Projekt** | Neu: gehört zu keinem Projekt | Genau **einem** Benutzer (Feld „Zuständig“, Pflicht) | **Nur** in der Navigation „Aufgaben“ |

Aufgaben ohne Projekt sind gedacht für Arbeiten, die zu keinem Projekt gehören (z. B. „Offerte nachfassen“,
„Bürobedarf bestellen“). Sie haben dieselben Felder wie Projektaufgaben (Titel, Beschreibung, erledigt/offen)
plus den zuständigen Benutzer.

Solange es keine Anmeldung gibt, wählt man oben auf der Seite, **wessen** Aufgaben angezeigt werden
(Benutzer-Auswahl). Mit der späteren Anmeldung wird automatisch der angemeldete Benutzer gewählt.

Grundfunktionen für **Aufgaben ohne Projekt**:

| Grundfunktion | Bedeutung |
|---------------|-----------|
| **Anlegen**    | Neue Aufgabe ohne Projekt erfassen und einem Benutzer zuweisen |
| **Anzeigen**   | In der Aufgabenliste und in einer Detailansicht |
| **Bearbeiten** | Titel, Beschreibung, Zuständig ändern; erledigt/offen umschalten |
| **Löschen**    | Aufgabe entfernen |

Projektaufgaben werden in der Navigation „Aufgaben“ nur **angezeigt und abgehakt**; Anlegen, Bearbeiten und
Löschen bleiben im Projekt (ANF-003).

## 2. Benutzergeschichte

> Als **Mitarbeiter** möchte ich **alle mir zugeordneten Aufgaben aus allen Projekten an einer Stelle sehen
> und abhaken**, damit **ich nicht jedes Projekt einzeln durchsuchen muss**.

> Als **Mitarbeiter** möchte ich **auch Aufgaben ohne Projekt erfassen**, damit **ich alle meine Arbeiten
> an einem Ort verwalte**.

## 3. Akzeptanzkriterien

**Navigation und Benutzer-Auswahl**
- [x] **AK-1:** Gegeben die Hauptnavigation, dann gibt es den Eintrag „Aufgaben“ (Pfad `/aufgaben`) mit
  eigenem Symbol; er erscheint auch als Karte auf der Startseite (ANF-001 AK-12).
- [x] **AK-2:** Gegeben die Seite „Aufgaben“, dann wählt man oben über ein Suchfeld (`SearchPicker`) einen
  **aktiven** Benutzer. Die Auswahl steht in der Webadresse (`/aufgaben?benutzer=<kennung>`), damit Neuladen
  und Zurück-Taste funktionieren. Ohne Auswahl erscheint der Hinweis „Bitte einen Benutzer wählen.“
  statt einer Liste.

**Liste (Anzeigen)**
- [x] **AK-3:** Gegeben ein gewählter Benutzer, dann zeigt eine Tabelle alle ihm zugeordneten Aufgaben:
  - Projektaufgaben, für die er verantwortlich ist – hinterlegt beim Vorgang oder geerbt vom Teilprojekt
    bzw. Projekt (gleiche Regel wie ANF-005 AK-25 bis AK-29),
  - Aufgaben ohne Projekt, bei denen er als „Zuständig“ eingetragen ist.
- [x] **AK-4:** Spalten: Kontrollkästchen „Erledigt“, Titel, Projekt (Nummer und Name bzw. „ohne Projekt“),
  Nummer im Projekt (z. B. `2.3.1`, bei Aufgaben ohne Projekt leer), Herkunft (z. B. „vom Projekt“,
  „vom Teilprojekt 2“, „vom Vorgang 2.3“ bzw. „direkt zugewiesen“), Geändert am.
  Projektaufgaben und Aufgaben ohne Projekt sind farblich bzw. mit einer Plakette unterscheidbar.
- [x] **AK-5:** Gegeben die Liste, dann kann gefiltert werden nach
  - Status: offen (Standard), erledigt, alle,
  - Art: alle (Standard), mit Projekt, ohne Projekt,
  - Suchbegriff in Titel, Beschreibung, Projektnummer oder Projektname (Gross-/Kleinschreibung egal).

  Filter und Suche stehen in der Webadresse.
- [x] **AK-6:** Sortierung standardmässig: Aufgaben ohne Projekt zuerst (neueste zuerst), danach Projektaufgaben
  nach Projektnummer und Nummer im Projekt.
- [x] **AK-7:** Gegeben keine Treffer, dann erscheint ein Hinweis statt einer leeren Tabelle.
- [x] **AK-8:** Gegeben eine Aufgabe in der Liste, wenn das Kontrollkästchen angeklickt wird, dann wird sie
  sofort erledigt bzw. wieder offen gesetzt (für beide Arten); die Liste wird neu geladen.
- [x] **AK-9:** Gegeben eine Projektaufgabe, wenn ihr Titel angeklickt wird, dann öffnet sich ihre
  Detailansicht im Projekt (`/projekte/aufgabe/<kennung>`). Gegeben eine Aufgabe ohne Projekt, dann öffnet
  sich ihre Detailansicht unter `/aufgaben/<kennung>`.

**Anlegen (nur Aufgaben ohne Projekt)**
- [x] **AK-10:** Gegeben die Seite „Aufgaben“, wenn auf „Neue Aufgabe“ geklickt wird, dann öffnet sich ein
  Formular in einem Modal-Fenster (Webadresse `…?neu=1`) mit den Feldern Titel (Pflicht), Beschreibung
  (optional) und Zuständig (Pflicht, nur aktive Benutzer). „Zuständig“ ist mit dem oben gewählten Benutzer
  vorbelegt.
- [x] **AK-11:** Gegeben ein gültiges Formular, wenn gespeichert wird, dann wird die Aufgabe **ohne Projekt**
  mit Status „offen“ angelegt, das Fenster schliesst sich und die Liste zeigt sie an. Ein Projekt, Teilprojekt
  oder Vorgang lässt sich in diesem Formular nicht wählen.
- [x] **AK-12:** Gegeben ein ungültiges Formular (Titel leer oder länger als 200 Zeichen, Zuständig fehlt,
  ist gesperrt oder existiert nicht), dann wird nichts gespeichert und die Felder werden mit einer
  verständlichen Meldung markiert. Die Prüfung erfolgt auf dem Server (Prüfregeln in `model.rs`).
- [x] **AK-13:** Im Projektbereich (ANF-003) ändert sich nichts: Dort lassen sich weiterhin nur Aufgaben
  unter einem Vorgang anlegen; Aufgaben ohne Projekt erscheinen dort nicht.

**Detailansicht, Bearbeiten und Löschen (nur Aufgaben ohne Projekt)**
- [x] **AK-14:** Gegeben eine Aufgabe ohne Projekt, dann zeigt die Detailansicht `/aufgaben/<kennung>`
  Titel, Beschreibung, Status (umschaltbar), Zuständig (mit Verweis auf den Benutzer, gesperrte markiert),
  Erstellt am und Geändert am. Die Pfadleiste lautet „Aufgaben › <Titel>“.
- [x] **AK-15:** Gegeben die Detailansicht, wenn auf „Bearbeiten“ geklickt wird, dann öffnet sich das Formular
  im Modal-Fenster (`…?bearbeiten=1`). Titel, Beschreibung und Zuständig sind änderbar; es gelten dieselben
  Prüfungen wie beim Anlegen. Ein bereits zuständiger, inzwischen gesperrter Benutzer darf bleiben
  (wie ANF-005 AK-33).
- [x] **AK-16:** Gegeben die Detailansicht, wenn auf „Löschen“ geklickt wird, dann erscheint ein
  Bestätigungsdialog mit dem Titel. Erst nach Bestätigung wird gelöscht und die Liste angezeigt.
- [x] **AK-17:** Gegeben eine Kennung einer Projektaufgabe oder eine unbekannte Kennung unter
  `/aufgaben/<kennung>`, dann erscheint „Aufgabe nicht gefunden“ bzw. bei Projektaufgaben ein Verweis
  auf deren Ansicht im Projekt.

**Zusammenspiel mit der Benutzerverwaltung (ANF-005)**
- [x] **AK-18:** Gegeben die Detailansicht eines Benutzers (ANF-005 AK-31), dann enthält die Liste der offenen
  Aufgaben auch seine Aufgaben ohne Projekt (Herkunft „direkt zugewiesen“), mit Verweis auf die Seite
  „Aufgaben“ dieses Benutzers.
- [x] **AK-19:** Gegeben ein Benutzer, der noch für Aufgaben ohne Projekt zuständig ist, wenn er gelöscht
  werden soll, dann wird das Löschen verhindert; die Meldung nennt die Anzahl dieser Aufgaben
  (Erweiterung von ANF-005 AK-32).
- [x] **AK-20:** Gegeben ein gesperrter Benutzer, dann bleiben seine Aufgaben ohne Projekt erhalten und
  zugewiesen; in der Benutzer-Auswahl (AK-2) ist er nicht wählbar, seine Aufgaben sind aber über die
  Webadresse weiterhin erreichbar.

**Allgemein**
- [x] **AK-21:** Die Datenbank wird per neuer sqlx-Migration erweitert; bestehende Migrationen bleiben
  unverändert, bestehende Projektaufgaben bleiben unverändert.

## 4. Daten / Felder

**Aufgabe ohne Projekt:**

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Titel | Text (200) | Ja | |
| Beschreibung | Langtext | Nein | |
| Status | Auswahl | Ja | Offen · Erledigt (Standard: Offen) |
| Zuständig (`benutzer_id`) | Fremdschlüssel → `benutzer` | Ja | Nur aktive Benutzer wählbar; `ON DELETE RESTRICT` (AK-19) |
| Erstellt am | Zeitstempel | Ja | automatisch |
| Geändert am | Zeitstempel | Ja | automatisch |

**Ablage (Vorschlag):** in der bestehenden Tabelle `aufgabe`:

- `aufgabe.vorgang_id` wird **optional** (leer = Aufgabe ohne Projekt),
- neue Spalte `aufgabe.benutzer_id` (Fremdschlüssel → `benutzer`, `ON DELETE RESTRICT`),
- Prüfregel in der Datenbank: **genau eines** von beiden ist gesetzt –
  `CHECK ((vorgang_id IS NULL) <> (benutzer_id IS NULL))`.

Damit bekommt eine Projektaufgabe weiterhin **keinen** eigenen Benutzer (ANF-005: Verantwortung nur über
Vorgang, Teilprojekt, Projekt), und eine Aufgabe ohne Projekt hat immer genau einen.
`reihenfolge` ist bei Aufgaben ohne Projekt leer bzw. ohne Bedeutung.

**Beziehungen:**

```
vorgang  1 ──── 0..n aufgabe   (Projektaufgabe:        aufgabe.vorgang_id gesetzt,  benutzer_id leer)
benutzer 1 ──── 0..n aufgabe   (Aufgabe ohne Projekt:  aufgabe.benutzer_id gesetzt, vorgang_id leer)
```

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst, Hauptnavigation), ANF-003 (Projektaufgaben),
  ANF-005 (Benutzer, Verantwortung in Projekten)
- Wird benötigt von: Anmeldung (noch zu erfassen) – dann ersetzt der angemeldete Benutzer die Auswahl (AK-2)

## 6. Nicht im Umfang

- Anlegen, Bearbeiten oder Löschen von Projektaufgaben in der Navigation „Aufgaben“ (bleibt im Projekt)
- Eine Aufgabe ohne Projekt nachträglich einem Projekt zuordnen oder umgekehrt
- Fälligkeitsdatum, Priorität, Erinnerungen, Benachrichtigungen
- Mehrere zuständige Benutzer für eine Aufgabe ohne Projekt
- Aufgaben anderer Benutzer zuweisen mit Berechtigungsprüfung (gehört zu Rollen und Berechtigungen)

## 7. Offene Fragen

Vorläufige Annahmen in Klammern.

- [ ] Wessen Aufgaben werden ohne Anmeldung angezeigt? *(vorerst: Benutzer-Auswahl oben auf der Seite, AK-2)*
- [ ] Ablage in der Tabelle `aufgabe` oder in einer eigenen Tabelle? *(vorerst: Tabelle `aufgabe`, `vorgang_id` optional, siehe Abschnitt 4)*
- [ ] Brauchen Aufgaben ohne Projekt eine eigene Nummer (z. B. `AU-00001`)? *(vorerst: nein)*
- [ ] Darf man Aufgaben ohne Projekt auch einem **anderen** Benutzer zuweisen? *(vorerst: ja, „Zuständig“ ist frei wählbar)*
- [ ] Sollen Aufgaben ohne Projekt ein Fälligkeitsdatum erhalten? *(vorerst: nein, wie Projektaufgaben)*
- [ ] Benutzer mit Aufgaben ohne Projekt löschen: verhindern oder Aufgaben mitlöschen? *(vorerst: verhindern, AK-19)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261112180000_aufgabe_ohne_projekt.sql` | `aufgabe.vorgang_id` optional, neue Spalte `aufgabe.benutzer_id` (`ON DELETE RESTRICT`), Prüfregel „genau eines von beiden“ |
| `src/backend/tasks/model.rs` | `TaskItem` (Zeile der Liste), Eingabe und Prüfregeln für Aufgaben ohne Projekt + Modultests |
| `src/backend/tasks/server_fns.rs` + `repository.rs` | Liste je Benutzer mit Filtern (Projektaufgaben über dieselbe Vererbung wie ANF-005), Anzeigen, Anlegen, Bearbeiten, Löschen |
| `src/components/tasks/list.rs`, `detail.rs`, `form.rs` | Seite „Aufgaben“ mit Benutzer-Auswahl, Filtern und Abhaken; Detailansicht; Modal-Fenster |
| `src/backend/projects/repository.rs` | Projektbereich ändert, verschiebt und löscht nur Aufgaben mit Vorgang (AK-13) |
| `src/backend/users/…`, `src/components/users/detail.rs` | Offene Aufgaben inkl. Aufgaben ohne Projekt, Verweis „Alle Aufgaben dieses Benutzers anzeigen“ (AK-18), Löschschutz (AK-19) |
| `src/forms.rs` | `SearchPicker` mit optionaler Rückmeldung `on_change` (für die Benutzer-Auswahl) |
| `src/navigation.rs`, `src/app.rs`, `src/icons.rs` | Eintrag „Aufgaben“ in der Hauptnavigation, Routen, Symbol |

Hinweise:
- **AK-8:** Abhaken nutzt für beide Arten dieselbe Server-Funktion wie der Strukturbaum (`toggle_task`).
- **AK-4:** Bei Verantwortung auf mehreren Ebenen wird als Herkunft die oberste Ebene genannt.

Geprüft: Migration auf einer frischen Testdatenbank; alle Server-Funktionen über HTTP (Liste mit Vererbung
und Filtern, Anlegen mit gültigen und ungültigen Eingaben, gesperrte Benutzer, Bearbeiten, Löschen,
Trennung vom Projektbereich, Löschschutz beim Benutzer); Seitenausgabe von Liste, Detailansicht und
Modal-Fenstern; 58 Modultests. Bedienung im Browser (Benutzer-Auswahl, Abhaken) noch nicht von Hand geprüft.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-12 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-12 | Hatem Ben Belgacem | Umgesetzt |
| 2026-11-12 | Hatem Ben Belgacem | ANF-007: Standard ist der angemeldete Benutzer |
