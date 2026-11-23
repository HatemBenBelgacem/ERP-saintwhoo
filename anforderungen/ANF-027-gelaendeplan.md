# ANF-027: Geländeplan eines Projekts (3D-Karte)

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-027                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Soll                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Projekte                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-23                                        |
| **Geändert**  | 2026-11-23                                        |

## 1. Beschreibung

Zu einem Projekt (ANF-003) lässt sich ein **Geländeplan** erstellen – z. B. für ein Fest oder eine
Veranstaltung. Der Plan zeigt das Gelände als **Satellitenbild in einer geneigten 3D-Karte**. Darauf
werden **Bühnen, Verpflegungsstände und Festzelte** als 3D-Quader platziert, gedreht und gelöscht. Optional
wird die **Rasenfläche abgesteckt**: Ihre Grösse in m² wird laufend angezeigt, und Elemente lassen sich nur
platzieren, wenn sie ganz auf der Fläche stehen.

Technik (Vorgabe des Auftraggebers):

| Teil | Technik |
|------|---------|
| Karte und Satellitenbild | **MapLibre GL JS** 4.7.1, Bilder von Esri World Imagery |
| 3D-Elemente | **Three.js** 0.170, als eigene Ebene („custom layer“) im selben `<canvas>` wie die Karte |
| Einbindung | Beide als ES-Module aus dem Netz (esm.sh), erst beim Öffnen des Geländeplans geladen – kein npm, kein Bündeln |
| Brücke | `src/components/site_plans/map_bridge.js`, aus Rust über `wasm-bindgen` gesteuert (`bridge.rs`) |
| Flächen und Lage | Rust-Paket **`geo`** (`unsigned_area`, `contains`) – im Browser und auf dem Server |

Koordinaten: Ein Plan hat einen **Standort** (Ursprung, Längen- und Breitengrad). Elemente und Eckpunkte
stehen in Metern relativ dazu (x nach Osten, z nach Süden). Wird der Standort geändert, wandert der ganze
Plan mit.

## 2. Benutzergeschichte

> Als **Projektleiter** möchte ich **auf einem Satellitenbild des Geländes Bühnen, Verpflegungsstände und
> Festzelte in 3D platzieren und die nutzbare Rasenfläche abstecken**, damit **ich den Aufbau einer
> Veranstaltung planen, ihre Grösse abschätzen und den Plan mit dem Projekt ablegen kann**.

## 3. Akzeptanzkriterien

- [x] **AK-1:** Gegeben ein Projekt, wenn ich in der Projektansicht „Geländeplan“ wähle, dann öffnet sich
  `/projekte/<Kennung>/gelaendeplan` mit Pfadleiste Projekte › Projektnummer › Geländeplan. Für ein
  unbekanntes Projekt erscheint „Projekt nicht gefunden“.
- [x] **AK-2:** Gegeben ein Projekt ohne Plan, wenn ich den Geländeplan öffne, dann zeigt die Karte den
  Vorgabe-Standort (St. Gallen) geneigt (60°) als Satellitenbild, und der Plan ist als „Noch nicht gespeichert“
  markiert.
- [x] **AK-3:** Gegeben die Karte, wenn ich sie mit der rechten Maustaste bzw. Strg + Ziehen kippe und drehe,
  dann bleiben alle 3D-Elemente an ihrer Stelle auf dem Boden.
- [x] **AK-4:** Gegeben ein gewähltes Werkzeug „Bühne (10 × 8 m)“, „Verpflegungsstand (4 × 2,5 m)“ oder
  „Festzelt (6 × 6 m)“, wenn ich auf die Karte klicke, dann erscheint dort ein Quader in der Farbe der Art
  (Bühne rot, 4,5 m hoch; Verpflegungsstand orange, 2,8 m; Festzelt blau, 3 m) und ein Eintrag in der Liste
  „Platzierte Elemente“ mit Grösse, Lage und Drehung.
- [x] **AK-5:** Gegeben ein Element in der Liste, wenn ich „↻“ wähle, dann dreht es sich um 45°; mit „✕“ wird
  es entfernt.
- [x] **AK-6:** Gegeben das Werkzeug „Rasenfläche abstecken“, wenn ich auf die Karte klicke, dann setzt jeder
  Klick einen Eckpunkt; ab drei Eckpunkten erscheint die Fläche halbdurchsichtig grün und ihre Grösse in m²
  wird laufend angezeigt. „Letzten Eckpunkt entfernen“ und „Fläche entfernen“ machen Eckpunkte rückgängig.
- [x] **AK-7:** Gegeben eine abgesteckte Rasenfläche, wenn ich ein Element so platzieren will, dass es nicht
  ganz auf der Fläche steht, dann wird es nicht platziert und ein Hinweis erscheint.
- [x] **AK-8:** Gegeben ein Element auf der Rasenfläche, wenn es durch Drehen hinausragen würde, dann wird es
  nicht gedreht und ein Hinweis erscheint.
- [x] **AK-9:** Gegeben Elemente, die nach einer Änderung der Fläche nicht mehr ganz darauf stehen, dann sind
  sie in der Liste als „ausserhalb“ markiert, in 3D halbdurchsichtig, und ein Hinweis nennt ihre Anzahl.
- [x] **AK-10:** Gegeben Änderungen am Plan, dann erscheint „Ungespeicherte Änderungen“; mit „Speichern“ wird
  der Plan als Ganzes gespeichert, mit „Verwerfen“ kehrt er zum gespeicherten Stand zurück. Beim Neuladen
  oder Schliessen der Seite mit ungespeicherten Änderungen fragt der Browser nach.
- [x] **AK-11:** Gegeben ein ungültiger Plan (Fläche mit weniger als drei Eckpunkten oder kleiner als 1 m²,
  Element ausserhalb der Fläche, Grösse nicht zwischen 0 und 200 m, Lage mehr als 5 km vom Standort,
  mehr als 500 Elemente oder 100 Eckpunkte), wenn ich speichere, dann prüft der Server den Plan, speichert
  nichts und zeigt die Meldungen.
- [x] **AK-12:** Gegeben das Modal-Fenster „Standort“ (`?standort=1`), wenn ich Breiten- und Längengrad
  eingebe oder „Kartenmitte übernehmen“ wähle und „Übernehmen“ klicke, dann fliegt die Karte zum neuen
  Standort und der ganze Plan wandert mit. Ungültige Werte werden am Feld gemeldet; Dezimalkomma ist erlaubt.
- [x] **AK-13:** Gegeben ein gespeicherter Plan, wenn ich „Löschen“ wähle und bestätige, dann wird der Plan
  gelöscht und die Projektansicht zeigt „„Geländeplan“ wurde gelöscht.“ Wird das Projekt gelöscht, wird
  auch sein Plan gelöscht.
- [x] **AK-14:** Gegeben die Karte kann nicht geladen werden (z. B. ohne Internet), dann erscheint eine
  Fehlermeldung; der Rest der Seite bleibt bedienbar.

## 4. Daten / Felder

Tabelle `gelaendeplan` (höchstens ein Plan je Projekt):

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| `projekt_id` | Verweis | ja | eindeutig; Löschen des Projekts löscht den Plan |
| `ursprung_laenge` | Zahl | ja | Längengrad −180 … 180 |
| `ursprung_breite` | Zahl | ja | Breitengrad −85 … 85 |

Tabelle `gelaendeplan_element`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| `reihenfolge` | Zahl | ja | Reihenfolge in der Liste |
| `art` | Text | ja | `buehne`, `verpflegungsstand`, `festzelt` |
| `x_m`, `z_m` | Zahl | ja | Mittelpunkt in Metern relativ zum Standort |
| `drehung_grad` | Zahl | ja | 0 ≤ Wert < 360 |
| `breite_m`, `tiefe_m` | Zahl | ja | > 0, Vorgabe je Art |

Tabelle `gelaendeplan_eckpunkt`: `reihenfolge`, `x_m`, `z_m` (leer = keine Rasenfläche).

## 5. Abhängigkeiten

- Hängt ab von: ANF-003 (Projekte)
- Externe Dienste: esm.sh / unpkg.com (MapLibre, Three.js), server.arcgisonline.com (Satellitenbilder)

## 6. Nicht im Umfang

- Ortssuche nach Adresse (Geocodierung); der Standort wird als Koordinate oder über die Kartenmitte gesetzt.
- Verschieben eines Elements per Ziehen, freie Grösse je Element, Auswahl eines Elements durch Klick in 3D.
- Echte 3D-Modelle (`.glb` über den `GLTFLoader`) statt Quadern.
- Mehrere Pläne oder Versionen je Projekt, Druckansicht, Export.
- Gleichzeitiges Bearbeiten durch mehrere Benutzer (wer zuletzt speichert, gewinnt).

## 7. Offene Fragen

Vorläufige Annahmen, bis sie geklärt sind:

- [ ] Ist der Geländeplan eine eigene Seite? *Annahme: ja.* Die 3D-Karte ist eine Arbeitsfläche, kein
  Formular; sie braucht die ganze Breite. Eingaben (Standort, Löschen) laufen trotzdem in Modal-Fenstern.
- [ ] Speichern ausdrücklich oder automatisch? *Annahme: ausdrücklich mit „Speichern“* (Plan als Ganzes).
- [ ] Begriff „Foodstand“ oder „Verpflegungsstand“? *Annahme: „Verpflegungsstand“* (deutscher Begriff).
- [ ] Satellitenbilder: Esri World Imagery darf nach den Nutzungsbedingungen von Esri verwendet werden?
  Für den Betrieb ggf. einen eigenen Anbieter (z. B. swisstopo SWISSIMAGE) wählen.
- [ ] Bibliotheken aus dem Netz (esm.sh) oder selbst ausliefern (`public/`)? *Annahme: aus dem Netz*,
  feste Fassungen. Für Betrieb ohne fremde Dienste später selbst ausliefern.
- [ ] Vorgabe-Standort: St. Gallen oder Adresse des Kunden? *Annahme: St. Gallen* (keine Geocodierung).

## 8. Änderungshistorie

| Datum      | Autor | Änderung        |
|------------|-------|-----------------|
| 2026-11-23 | Hatem Ben Belgacem | Erstellt und umgesetzt (Vorgabe: Anleitung Leptos + MapLibre + Three.js) |
