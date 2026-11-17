# ANF-001: Grundgerüst mit Kopfzeile und Seitenleiste

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-001                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Allgemein / Oberfläche                            |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-11                                        |
| **Geändert**  | 2026-11-12                                        |

## 1. Beschreibung

Alle Seiten der Anwendung nutzen ein gemeinsames Grundgerüst. Es besteht aus:

- einer **Kopfzeile** am oberen Rand,
- einer **Seitenleiste** links, die die gesamte Navigation enthält,
- dem **Inhaltsbereich**, in dem die jeweilige Seite angezeigt wird.

Die Seitenleiste hat zwei Ansichten:

1. **Hauptnavigation**: alle Bereiche des ERP (z. B. Lager, Einkauf, Verkauf).
2. **Einstellungsnavigation**: alle Einstellungsseiten.

In der **Fusszeile der Seitenleiste** befindet sich der Eintrag **„Einstellungen“**. Ein Klick darauf
ersetzt die Hauptnavigation durch die Einstellungsnavigation. Von dort führt ein Eintrag **„Zurück“**
wieder zur Hauptnavigation.

```
┌──────────────────────────────────────────────────┐
│ Kopfzeile (neutral, ohne Überschrift)            │
├──────────────┬───────────────────────────────────┤
│ Navigation   │                                   │
│  • Eintrag 1 │                                   │
│  • Eintrag 2 │        Inhaltsbereich             │
│  • …         │                                   │
│              │                                   │
├──────────────┤                                   │
│ ⚙ Einstell.  │                                   │
└──────────────┴───────────────────────────────────┘
```

## 2. Benutzergeschichte

> Als **Benutzer** möchte ich **auf jeder Seite dieselbe Navigation in einer Seitenleiste finden**,
> damit **ich mich schnell zwischen den Bereichen des ERP bewegen kann**.

> Als **Benutzer** möchte ich **die Einstellungen über einen festen Platz unten in der Seitenleiste
> erreichen**, damit **sie die tägliche Navigation nicht überladen**.

## 3. Akzeptanzkriterien

**Aufbau**
- [x] **AK-1:** Gegeben eine beliebige Seite der Anwendung, wenn sie geladen wird, dann werden Kopfzeile,
  Seitenleiste und Inhaltsbereich angezeigt.
- [x] **AK-2:** Gegeben eine Seite mit viel Inhalt, wenn der Benutzer blättert, dann bleiben Kopfzeile und
  Seitenleiste stehen und nur der Inhaltsbereich bewegt sich.

**Hauptnavigation**
- [x] **AK-3:** Gegeben die Hauptnavigation, wenn der Benutzer einen Eintrag anklickt, dann wird die
  zugehörige Seite im Inhaltsbereich geladen, ohne dass die ganze Seite neu lädt.
- [x] **AK-4:** Gegeben eine geöffnete Seite, dann ist der zugehörige Navigationseintrag als aktiv
  hervorgehoben – in leichtem Indigo (heller Hintergrund, Schrift in dunklem Indigo; Farbe der Primärfarbe, ANF-009).
- [x] **AK-5:** Die Navigationseinträge werden an einer zentralen Stelle im Code definiert, damit neue
  Module ohne Änderung am Grundgerüst ergänzt werden können.

**Einstellungen**
- [x] **AK-6:** Gegeben die Hauptnavigation, dann steht der Eintrag „Einstellungen“ in der Fusszeile
  der Seitenleiste, getrennt von den übrigen Einträgen.
- [x] **AK-7:** Gegeben die Hauptnavigation, wenn der Benutzer auf „Einstellungen“ klickt, dann zeigt die
  Seitenleiste die Einstellungsnavigation und der Inhaltsbereich die erste Einstellungsseite.
- [x] **AK-8:** Gegeben die Einstellungsnavigation, wenn der Benutzer auf „Zurück“ klickt, dann zeigt
  die Seitenleiste wieder die Hauptnavigation und der Inhaltsbereich die Startseite.
- [x] **AK-9:** Welche Navigation angezeigt wird, ergibt sich aus der Webadresse: Unter `/einstellungen/…`
  erscheint die Einstellungsnavigation, sonst die Hauptnavigation. Ein Neuladen oder ein Lesezeichen
  zeigt deshalb immer die richtige Seitenleiste.

**Ansicht auf Mobilgeräten**
- [x] **AK-10:** Gegeben ein schmaler Bildschirm (Mobiltelefon), dann ist die Seitenleiste ausgeblendet
  und lässt sich über ein Menü-Symbol in der Kopfzeile ein- und ausblenden.
- [x] **AK-11:** Gegeben die eingeblendete Seitenleiste auf dem Mobiltelefon, wenn der Benutzer einen
  Eintrag wählt, dann schliesst sich die Seitenleiste automatisch.

**Startseite**
- [x] **AK-12:** Gegeben die Startseite, dann erscheint jedes Modul der Hauptnavigation (ausser der Startseite
  selbst) als Karte mit Symbol, Titel und Kurzbeschreibung. Ein Klick auf die Karte öffnet das Modul. Neue Module
  erscheinen automatisch, sobald sie in der Hauptnavigation eingetragen sind. Darunter zeigt eine kleine Zeile,
  ob die Datenbank verbunden ist.

## 4. Daten / Felder

Aufbau eines Navigationseintrags:

| Feld   | Typ    | Pflicht | Bemerkung                                  |
|--------|--------|---------|--------------------------------------------|
| Titel  | Text   | Ja      | Angezeigter Name, z. B. „Lager“            |
| Pfad   | Text   | Ja      | Ziel-Webadresse, z. B. `/lager`            |
| Symbol | Text   | Nein    | Symbol vor dem Titel                       |
| Beschreibung | Text | Ja   | Kurzbeschreibung für die Karte auf der Startseite (AK-12) |

## 5. Abhängigkeiten

- Hängt ab von: –
- Wird benötigt von: allen weiteren Seiten und Modulen

## 6. Nicht im Umfang

- Die konkreten Inhalte der Navigation und der Einstellungsseiten (eigene Anforderungen je Modul).
- Benutzerrechte: Ob ein Eintrag je nach Rolle ausgeblendet wird, regelt eine spätere Anforderung.
- Anmeldung / Abmeldung.

## 7. Offene Fragen

Für die Umsetzung wurden vorläufige Annahmen getroffen (in Klammern). Weitere Wünsche als neue Anforderung erfassen.

- [x] Farbe der Kopfzeile: neutral in dunklem Anthrazit (daisyUI `neutral`, Schrift `neutral-content`), kein Glutorange.
- [x] ~~Glutorange ist die Grundfarbe des ERP~~ → ersetzt durch die Farbpalette aus ANF-009 (Primärfarbe Indigo, daisyUI `primary`). Der aktive Navigationseintrag ist in leichtem Indigo markiert.
- [x] Inhalt der Kopfzeile: keine Überschrift, kein Leitspruch, kein Anwendungsname; nur das Menü-Symbol auf kleinen Bildschirmen.
- [ ] Soll die Kopfzeile später zusätzlich ein Benutzermenü oder eine Suche erhalten?
- [ ] Welche Einträge hat die Hauptnavigation? *(vorerst: „Startseite“)*
- [ ] Welche Einträge hat die Einstellungsnavigation? *(vorerst: „Allgemein“ als Platzhalter)*
- [ ] Soll die Seitenleiste auf grossen Bildschirmen einklappbar sein? *(vorerst: nein)*
- [ ] Gibt es Untermenüs? *(vorerst: nur eine Ebene)*
- [ ] Welches Farbschema, Dunkelmodus? *(vorerst: helles daisyUI-Farbschema „light“, kein Dunkelmodus)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `src/layout.rs` | Grundgerüst (`AppLayout`), Kopfzeile, Seitenleiste (daisyUI `drawer` + `menu`) |
| `src/navigation.rs` | Zentrale Navigationseinträge `MAIN_NAV` (Hauptnavigation), `SETTINGS_NAV` (Einstellungen) + Modultests |
| `src/icons.rs` | Eingebettete SVG-Symbole |
| `src/components/home/home_page.rs` | Startseite mit Modulkarten aus `navigation::modules()` und Datenbank-Status (AK-12) |
| `src/app.rs` | Routen: `/`, `/einstellungen` (→ Weiterleitung auf erste Einstellungsseite), `/einstellungen/allgemein` |
| `styles/app.css` | Farbpalette (ANF-009) im daisyUI-Farbschema, Markierung der Navigation |

Geprüft: alle AK im Browser (grosser Bildschirm 1280 px, Mobiltelefon 390 px) und Modultests für die Pfadlogik.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-11 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-11 | Hatem Ben Belgacem | Umgesetzt, Farbe der Kopfzeile `--glutorange` ergänzt |
| 2026-11-12 | Hatem Ben Belgacem | AK-12: Startseite zeigt alle Module als Karten |
| 2026-11-11 | Hatem Ben Belgacem | Begriffe auf Deutsch umgestellt (vorher REQ-001 „AppLayout mit Header und Seitenleiste“) |
| 2026-11-11 | Hatem Ben Belgacem | Glutorange als Grundfarbe, auch für die Markierung der Navigation |
| 2026-11-11 | Hatem Ben Belgacem | Markierung der Navigation in leichtem Glutorange |
| 2026-11-11 | Hatem Ben Belgacem | Kopfzeile: Leitspruch in Tifinagh mittig, Anwendungsname entfernt |
| 2026-11-12 | Hatem Ben Belgacem | Kopfzeile neutral: Glutorange und Leitspruch entfernt |
| 2026-11-17 | Hatem Ben Belgacem | Glutorange durch die Farbpalette aus ANF-009 ersetzt (Primärfarbe Indigo) |
