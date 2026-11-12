# ANF-003: Projekte verwalten mit Teilprojekten, Vorgängen und Aufgaben

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-003                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Projekte                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-11                                        |
| **Geändert**  | 2026-11-12                                        |

## 1. Beschreibung

Das ERP verwaltet Projekte in einer festen Hierarchie mit vier Ebenen:

```
Projekt
└── Teilprojekt        (1 Projekt hat 0..n Teilprojekte)
    └── Vorgang        (1 Teilprojekt hat 0..n Vorgänge)
        └── Aufgabe    (1 Vorgang hat 0..n Aufgaben)
```

Zusätzlich ist jedem Projekt eine **Adresse** (ANF-002) als Kunde zugeordnet:
**Projekt – Adresse N:1** – viele Projekte können dieselbe Adresse haben, ein Projekt hat höchstens eine Adresse.

Jedes Element gehört genau zu einem übergeordneten Element. Auf **jeder Ebene** gibt es die
vier Grundfunktionen:

| Grundfunktion | Bedeutung |
|---------------|-----------|
| **Anlegen**    | Neues Element unter dem übergeordneten Element erfassen |
| **Anzeigen**   | Elemente auflisten und im Detail ansehen |
| **Bearbeiten** | Element ändern |
| **Löschen**    | Element entfernen, inkl. aller untergeordneten Elemente |

Das Modul erscheint als Eintrag **„Projekte“** in der Hauptnavigation (ANF-001) unter dem Pfad `/projekte`.

## 2. Benutzergeschichte

> Als **Projektleiter** möchte ich **Projekte in Teilprojekte, Vorgänge und Aufgaben gliedern**,
> damit **ich grosse Vorhaben strukturiert planen und den Fortschritt verfolgen kann**.

> Als **Mitarbeiter** möchte ich **meine Aufgaben innerhalb eines Projekts finden und ihren Status
> ändern**, damit **der Projektstand immer aktuell ist**.

## 3. Akzeptanzkriterien

**Projektliste**
- [x] **AK-1:** Gegeben der Navigationseintrag „Projekte“, wenn der Benutzer ihn anklickt, dann erscheint
  eine Tabelle aller Projekte mit den Spalten Nummer, Name, Kunde, Status, Start, Ende und Fortschritt.
- [x] **AK-1a:** Gegeben die Projektübersicht, dann erscheint unter jedem Projekt seine Struktur
  (Teilprojekte → Vorgänge → Aufgaben) eingerückt nach Ebene, mit Nummer, Status und Fortschritt.
  Jede Ebene lässt sich auf- und zuklappen, zusätzlich „Alle aufklappen“ / „Alle zuklappen“.
  Aufgaben lassen sich direkt abhaken; „+“ und „✎“ in jeder Zeile öffnen Anlegen bzw. Bearbeiten.
- [x] **AK-2:** Gegeben die Projektliste, dann kann der Benutzer nach Nummer, Name oder Kunde suchen,
  nach Status filtern und nach jeder Spalte sortieren. Standard: Status „Aktiv“, nach Nummer absteigend.
- [x] **AK-3:** Gegeben mehr als 50 Projekte, dann wird die Liste seitenweise angezeigt. Suche, Filter,
  Sortierung und Seite stehen in der Webadresse.

**Projektdetail mit Struktur**
- [x] **AK-4:** Gegeben die Projektliste, wenn der Benutzer ein Projekt anklickt, dann öffnet sich unter
  `/projekte/<kennung>` die Projektansicht mit den Projektdaten und darunter der Struktur als aufklappbarer
  Baum: Teilprojekte → Vorgänge → Aufgaben.
- [x] **AK-5:** Gegeben der Strukturbaum, dann zeigt jedes Element Nummer, Name und Status; jede
  Ebene zeigt die Anzahl der untergeordneten Elemente und den Fortschritt in Prozent
  (erledigte Aufgaben / alle Aufgaben darunter).
- [x] **AK-6:** Gegeben der Strukturbaum, wenn der Benutzer ein Element anklickt, dann öffnet sich dessen
  Detailansicht. Eine Pfadleiste zeigt immer die übergeordneten Ebenen,
  z. B. „Projekt › Teilprojekt › Vorgang › Aufgabe“, jeweils anklickbar.

**Anlegen – alle Ebenen**
- [x] **AK-7:** Gegeben die Projektliste, wenn der Benutzer auf „Neues Projekt“ klickt, dann öffnet sich
  ein Formular in einem Modal-Fenster. Nach dem Speichern erhält das Projekt eine fortlaufende Nummer und die
  Projektansicht öffnet sich.
- [x] **AK-8:** Gegeben ein Projekt, Teilprojekt oder Vorgang, dann gibt es eine Schaltfläche, um darunter
  ein Element der nächsten Ebene anzulegen („Neues Teilprojekt“, „Neuer Vorgang“, „Neue Aufgabe“).
  Das übergeordnete Element ist dabei fest vorgegeben. Das Formular öffnet sich in einem Modal-Fenster
  über der aktuellen Seite; nach dem Speichern bleibt der Benutzer auf dieser Seite (Übersicht oder
  Detailansicht), die Struktur wird aktualisiert.
- [x] **AK-9:** Gegeben ein neues Element unter einem übergeordneten Element, dann wird es ans Ende
  der Reihenfolge gestellt und erhält eine Nummer innerhalb des Übergeordneten
  (z. B. Teilprojekt `2`, Vorgang `2.3`, Aufgabe `2.3.1`).
- [x] **AK-10:** Gegeben ein Formular, wenn Pflichtfelder fehlen oder das Ende vor dem Start liegt, dann
  wird nichts gespeichert und die Felder werden mit einer Meldung markiert. Die Prüfung erfolgt
  auf dem Server.

**Bearbeiten – alle Ebenen**
- [x] **AK-11:** Gegeben die Detailansicht eines Elements oder eine Zeile im Strukturbaum, wenn der Benutzer
  auf „Bearbeiten“ (✎) klickt, dann öffnet sich das Formular mit den aktuellen Werten in einem Modal-Fenster;
  Speichern übernimmt die Änderungen, „Abbrechen“ nicht.
- [x] **AK-11a:** Gegeben eine Aufgabe, dann enthält das Formular (Anlegen und Bearbeiten) nur die Felder „Titel“
  (Pflicht) und „Beschreibung“ (optional); die Detailansicht der Aufgabe zeigt die Beschreibung.
- [x] **AK-5a:** Projekt, Teilprojekt, Vorgang und Aufgabe sind farblich abgegrenzt (Übersicht, Strukturbaum,
  Kopf der Detailansicht): farbiger Streifen links, Ebenen-Plakette, leichter Hintergrund; Legende über der Übersicht.
  Farben: Projekt Glutorange, Teilprojekt Blau, Vorgang Petrol, Aufgabe Schiefergrau.
- [x] **AK-12:** Gegeben eine Aufgabe, dann kann der Benutzer ihren Status direkt im Strukturbaum ändern
  (z. B. per Kontrollkästchen „Erledigt“), ohne das Formular zu öffnen.
- [x] **AK-13:** Gegeben mehrere Elemente auf derselben Ebene, dann kann der Benutzer ihre Reihenfolge
  ändern (nach oben / nach unten).

**Löschen – alle Ebenen**
- [x] **AK-14:** Gegeben ein Element, wenn der Benutzer auf „Löschen“ klickt, dann erscheint ein
  Bestätigungsdialog, der nennt, wie viele untergeordnete Elemente mitgelöscht werden
  (z. B. „3 Teilprojekte, 12 Vorgänge, 48 Aufgaben“).
- [x] **AK-15:** Gegeben die Bestätigung, dann werden das Element und alle untergeordneten Elemente in
  einer Transaktion gelöscht (ganz oder gar nicht) und die übergeordnete Ansicht erscheint.

**Allgemein**
- [x] **AK-16:** Gegeben ein Projekt, dann kann als Kunde eine Adresse aus ANF-002 gewählt werden
  (Auswahlfeld mit Suche). Eine Adresse, die als Kunde verwendet wird, kann nicht gelöscht werden.
- [x] **AK-17:** Gegeben eine nicht existierende Kennung in der Webadresse, dann erscheint „nicht gefunden“.
- [x] **AK-18:** Die Tabellen werden per sqlx-Migration angelegt; Fremdschlüssel sichern die Hierarchie.

## 4. Daten / Felder

**Felder** von Projekt, Teilprojekt und Vorgang. **Aufgaben haben nur Titel und Beschreibung** (plus erledigt/offen):

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Name / Titel | Text (200) | Ja | |
| Beschreibung | Langtext | Nein | |
| Status | Auswahl | Ja | siehe unten |
| Start | Datum | Nein | |
| Ende / Fällig | Datum | Nein | Nicht vor Start |
| Erstellt am | Zeitstempel | Ja | automatisch |
| Geändert am | Zeitstempel | Ja | automatisch |

**Zusätzliche Felder je Ebene:**

| Ebene | Tabelle | Zusätzliche Felder |
|-------|---------|--------------------|
| Projekt | `projekt` | Nummer (eindeutig, fortlaufend, z. B. `P-2026-001`), Kunde (→ `adresse`, optional) |
| Teilprojekt | `teilprojekt` | Projekt (→ `projekt`, Pflicht), Reihenfolge |
| Vorgang | `vorgang` | Teilprojekt (→ `teilprojekt`, Pflicht), Reihenfolge |
| Aufgabe | `aufgabe` | Nur: Titel, Beschreibung, erledigt/offen, Vorgang (→ `vorgang`, Pflicht), Reihenfolge |

**Status:**

| Ebene | Werte |
|-------|-------|
| Projekt, Teilprojekt, Vorgang | Geplant · Aktiv · Pausiert · Abgeschlossen · Abgebrochen |
| Aufgabe | Offen · Erledigt (nur über das Kontrollkästchen, nicht im Formular) |

**Beziehungen:**

| Von | Zu | Art | Fremdschlüssel | Pflicht | Beim Löschen des Ziels |
|-----|----|-----|----------------|---------|------------------------|
| `projekt` | `adresse` | N:1 | `projekt.adresse_id` | Nein | verhindert (AK-16) |
| `teilprojekt` | `projekt` | N:1 | `teilprojekt.projekt_id` | Ja | mitlöschen |
| `vorgang` | `teilprojekt` | N:1 | `vorgang.teilprojekt_id` | Ja | mitlöschen |
| `aufgabe` | `vorgang` | N:1 | `aufgabe.vorgang_id` | Ja | mitlöschen |

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst, Navigation), ANF-002 (Adressen als Kunde)
- Wird benötigt von: –

## 6. Nicht im Umfang

- Verantwortliche Benutzer bei Projekt, Teilprojekt und Vorgang – geregelt in ANF-005 (Verantwortung in Projekten)
- Zeiterfassung, Ist-Aufwand, Kosten und Budget
- Balkenplan (Terminplan), Abhängigkeiten zwischen Vorgängen
- Verschieben eines Elements zu einem anderen übergeordneten Element
- Vorlagen und Kopieren von Projekten
- Anhänge und Kommentare

## 7. Offene Fragen

Für die Umsetzung wurden vorläufige Annahmen getroffen (in Klammern).

- [ ] Soll der Status übergeordneter Ebenen automatisch aus den Aufgaben berechnet werden? *(vorerst: manuell; der Fortschritt in % wird automatisch berechnet)*
- [ ] Format der Projektnummer? *(vorerst: `P-JJJJ-NNN`, fortlaufend je Kalenderjahr, ab 1000 vierstellig)*
- [ ] Löschen endgültig oder archivieren? *(vorerst: endgültig, in jedem Status, nach Bestätigung)*
- [ ] Braucht ein Projekt einen Projektleiter? *(vorerst: nein)*
- [ ] Sollen Elemente zwischen übergeordneten Elementen verschoben werden können? *(vorerst: nein)*
- [ ] Ist die Hierarchie immer vollständig? *(vorerst: ja – Aufgaben nur unter Vorgängen, Vorgänge nur unter Teilprojekten)*
- [ ] Standardstatus neuer Elemente? *(vorerst: „Aktiv“, Aufgaben „Offen“ – damit neue Projekte in der Standardliste erscheinen)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261111150000_projekt.sql` | Tabellen `projekt`, `teilprojekt`, `vorgang`, `aufgabe`, Zähler `projekt_nummer`; Fremdschlüssel mit `ON DELETE CASCADE` innerhalb der Hierarchie, `ON DELETE RESTRICT` zur Adresse |
| `migrations/20261111180000_aufgabe_nur_titel.sql` | Aufgaben: Beschreibung, Start, Fällig, Priorität und Aufwand entfernt; Status nur offen/erledigt |
| `migrations/20261112120000_aufgabe_beschreibung.sql` | Aufgaben: Beschreibung wieder ergänzt (optional, AK-11a) |
| `src/backend/projects/model.rs` | Ebenen, Status, Prüfregeln, Nummerierung, Fortschritt + Modultests |
| `src/backend/projects/server_fns.rs` + `src/backend/projects/repository.rs` | Server-Funktionen: Liste, Struktur laden, Anlegen, Bearbeiten, Löschen, Abhaken, Verschieben, Kundensuche |
| `src/components/projects/list.rs` | Projektübersicht mit Struktur (auf-/zuklappbar), Suche, Statusfilter, Sortierung, Seiten |
| `src/components/projects/detail.rs` | Detailansichten aller Ebenen, Strukturbaum, Fortschritt, Löschdialog |
| `src/components/projects/form.rs` | Modal-Fenster für alle Ebenen (gesteuert über die Webadresse), Kundenauswahl mit Suche |

**Webadressen:**

| Ansicht | Webadresse |
|---------|-----------|
| Übersicht | `/projekte` |
| Projekt | `/projekte/<kennung>` |
| Teilprojekt | `/projekte/teilprojekt/<kennung>` |
| Vorgang | `/projekte/vorgang/<kennung>` |
| Aufgabe | `/projekte/aufgabe/<kennung>` |

Modal-Fenster über jeder dieser Seiten (Parameter in der Webadresse):

| Fenster | Parameter |
|---------|-----------|
| Neues Projekt | `?neu=projekt` |
| Neues Element unter einem anderen | `?neu=<ebene>&unter=<kennung>`, z. B. `?neu=vorgang&unter=5` |
| Bearbeiten | `?bearbeiten=<ebene>&id=<kennung>`, z. B. `?bearbeiten=aufgabe&id=12` |

Hinweise:
- **AK-9:** Die Nummern (`2.3.1`) ergeben sich aus der Reihenfolge und ändern sich beim Verschieben.
- **AK-15:** Ein einziger Löschbefehl; die Datenbank löscht alle Unterelemente mit (ganz oder gar nicht).

Geprüft: alle AK im Browser (grosser Bildschirm und Mobiltelefon, 61 Testprojekte mit Struktur) und Modultests.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-11 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-11 | Hatem Ben Belgacem | Begriffe auf Deutsch umgestellt (vorher REQ-003) |
| 2026-11-11 | Hatem Ben Belgacem | Tabellennamen in Einzahl |
| 2026-11-11 | Hatem Ben Belgacem | Beziehung Projekt – Adresse N:1 ergänzt, Beziehungstabelle |
| 2026-11-11 | Hatem Ben Belgacem | Umgesetzt |
| 2026-11-11 | Hatem Ben Belgacem | Projektübersicht mit Struktur (AK-1a), Eingabe in Modal-Fenstern (AK-7, AK-8, AK-11) |
| 2026-11-11 | Hatem Ben Belgacem | Aufgaben nur noch mit Titel (AK-11a); Felder Beschreibung, Start, Fällig, Priorität, Aufwand entfernt |
| 2026-11-11 | Hatem Ben Belgacem | Ebenen farblich abgegrenzt (AK-5a) |
| 2026-11-12 | Hatem Ben Belgacem | Aufgaben wieder mit Feld „Beschreibung“ (AK-11a) |
| 2026-11-12 | Hatem Ben Belgacem | Verweis auf ANF-005 für verantwortliche Benutzer |
| 2026-11-12 | Hatem Ben Belgacem | Feld „Verantwortlich“ bei Projekt, Teilprojekt und Vorgang umgesetzt (ANF-005) |
| 2026-11-12 | Hatem Ben Belgacem | ANF-006: Aufgaben ohne Projekt erscheinen nicht im Projektbereich |
