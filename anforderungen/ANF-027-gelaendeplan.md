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
| **Geändert**  | 2026-11-24                                        |

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
| 3D-Elemente | **Three.js** 0.170, als eigene Ebene („custom layer“) im selben `<canvas>` wie die Karte; Modelle aus Grundformen (Quader, Zylinder, ausgezogenes Dreieck) |
| Einbindung | Beide als ES-Module aus dem Netz (esm.sh), erst beim Öffnen des Geländeplans geladen – kein npm, kein Bündeln |
| Brücke | `src/components/site_plans/map_bridge.js`, aus Rust über `wasm-bindgen` gesteuert (`bridge.rs`) |
| Flächen und Lage | Rust-Paket **`geo`** (`unsigned_area`, `contains`, `is_valid`) – im Browser und auf dem Server |
| Adresssuche | **Nominatim** (OpenStreetMap), direkt aus dem Browser, nur auf Knopfdruck (höchstens 1 Anfrage pro Sekunde) |
| Messwerkzeuge | Linien, Punkte und Flächen als Ebenen von MapLibre, Beschriftungen als Marken auf der Karte |

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
  „Festzelt (6 × 6 m)“, wenn ich auf die Karte klicke, dann erscheint dort ein 3D-Modell der Art und ein
  Eintrag in der Liste „Platzierte Elemente“ mit Grösse, Lage und Drehung. Die Modelle bestehen aus
  Grundformen von Three.js und bleiben innerhalb ihrer Grundfläche:
  - **Bühne** (4,5 m): rote Podestverkleidung, Bühnenboden, Rückwand, vier Stützen, rotes Dach, zwei Lautsprecher.
  - **Verpflegungsstand** (2,8 m): orange Theke mit heller Platte, Rück- und Seitenwände, Dach, orange-hell
    gestreiftes Vordach nach vorne.
  - **Festzelt** (3 m): helle Zeltwände, blaues Giebeldach mit Firststange, dunkler Eingang vorne.
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
  sie in der Liste als „ausserhalb“ markiert, ihr 3D-Modell ist halbdurchsichtig, und ein Hinweis nennt ihre Anzahl.
- [x] **AK-10:** Gegeben Änderungen am Plan, dann erscheint „Ungespeicherte Änderungen“; mit „Speichern“ wird
  der Plan als Ganzes gespeichert, mit „Verwerfen“ kehrt er zum gespeicherten Stand zurück. Beim Neuladen
  oder Schliessen der Seite mit ungespeicherten Änderungen fragt der Browser nach.
- [x] **AK-11:** Gegeben ein ungültiger Plan (Fläche mit weniger als drei Eckpunkten, mit sich überschneidendem
  Umriss oder kleiner als 1 m², Standort-Adresse länger als 300 Zeichen,
  Element ausserhalb der Fläche, Grösse nicht zwischen 0 und 200 m, Lage mehr als 5 km vom Standort,
  mehr als 500 Elemente oder 100 Eckpunkte), wenn ich speichere, dann prüft der Server den Plan, speichert
  nichts und zeigt die Meldungen.
- [x] **AK-12:** Gegeben das Modal-Fenster „Standort“ (`?standort=1`), wenn ich Breiten- und Längengrad
  eingebe oder „Kartenmitte übernehmen“ wähle und „Übernehmen“ klicke, dann fliegt die Karte zum neuen
  Standort und der ganze Plan wandert mit. Ungültige Werte werden am Feld gemeldet; Dezimalkomma ist erlaubt.
- [x] **AK-12a:** Gegeben das Modal-Fenster „Standort“, wenn ich eine Adresse oder einen Ort eingebe und
  „Suchen“ (oder Enter) wähle, dann erscheinen bis zu fünf Treffer. Wähle ich einen Treffer, werden
  Breiten- und Längengrad sowie die Adresse übernommen. Das Suchfeld ist mit der hinterlegten Adresse,
  sonst mit der Postanschrift des Kunden des Projekts vorbelegt. Ist der Dienst nicht erreichbar, erscheint
  eine Meldung; ohne Treffer der Hinweis „Nichts gefunden“.
- [x] **AK-12b:** Gegeben ein Standort, dann lässt sich seine **Adresse bzw. Bezeichnung** (höchstens 300
  Zeichen, z. B. „Festwiese Kreuzbleiche“) hinterlegen. Sie wird mit dem Plan gespeichert und im Kopf der
  Seite angezeigt („Keine Standort-Adresse hinterlegt“, solange keine erfasst ist).
- [x] **AK-13:** Gegeben ein gespeicherter Plan, wenn ich „Löschen“ wähle und bestätige, dann wird der Plan
  gelöscht und die Projektansicht zeigt „„Geländeplan“ wurde gelöscht.“ Wird das Projekt gelöscht, wird
  auch sein Plan gelöscht.
- [x] **AK-15:** Gegeben das Messwerkzeug **„Strecke messen“**, wenn ich auf die Karte klicke, dann setzt
  jeder Klick einen Messpunkt; die Länge jedes Abschnitts steht auf der Karte und in der Seitenleiste, dazu
  die Gesamtstrecke (bis 1 km in m, darüber in km).
- [x] **AK-16:** Gegeben das Messwerkzeug **„Fläche messen“**, dann wird der Linienzug geschlossen und ab
  drei Punkten gefüllt; Umfang, Fläche in m² und die Abschnitte werden angezeigt. Überschneidet sich der
  Umriss, steht statt der Fläche „–“ und ein Hinweis erscheint.
- [x] **AK-17:** Gegeben eine Messung, dann bleibt sie beim Wechsel zu einem anderen Werkzeug sichtbar;
  „Letzten Punkt entfernen“ und „Messung löschen“ machen sie rückgängig. Messungen gehören nicht zum Plan
  und werden nicht gespeichert.
- [x] **AK-18:** Gegeben das Werkzeug **„Auswählen und verschieben“** (Vorgabe) oder ein Platzier-Werkzeug,
  wenn ich ein Element auf der Karte mit der Maus oder einem Finger anfasse und ziehe, dann folgt es dem
  Zeiger, ohne dass sich die Karte bewegt. Über einem Element zeigt der Mauszeiger „verschieben“. Ein Klick
  auf ein Element wählt es aus (Rahmen in 3D, markierter Eintrag in der Liste) und platziert kein neues;
  ein Klick in der Liste wählt ebenfalls aus. Beim Abstecken und Messen lassen sich Elemente nicht ziehen.
- [x] **AK-19:** Gegeben ein Element, das auf der Rasenfläche steht, wenn ich es so loslasse, dass es nicht
  mehr ganz darauf steht, dann springt es an die Ausgangslage zurück und ein Hinweis erscheint. Während des
  Ziehens ist es ausserhalb als „ausserhalb“ markiert und halbdurchsichtig.
- [x] **AK-20:** Gegeben ein ausgewähltes Element, wenn ich die Pfeiltasten drücke, dann verschiebt es sich
  um 0,5 m (mit Umschalt um 5 m) nach Osten, Westen, Norden bzw. Süden, nicht über den Rand der
  Rasenfläche hinaus; Escape hebt die Auswahl auf. In Eingabefeldern und bei offenem Modal-Fenster nicht.
- [x] **AK-14:** Gegeben die Karte kann nicht geladen werden (z. B. ohne Internet), dann erscheint eine
  Fehlermeldung; der Rest der Seite bleibt bedienbar.

## 4. Daten / Felder

Tabelle `gelaendeplan` (höchstens ein Plan je Projekt):

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| `projekt_id` | Verweis | ja | eindeutig; Löschen des Projekts löscht den Plan |
| `ursprung_laenge` | Zahl | ja | Längengrad −180 … 180 |
| `ursprung_breite` | Zahl | ja | Breitengrad −85 … 85 |
| `standort_adresse` | Text | nein | Adresse oder Bezeichnung des Standorts, höchstens 300 Zeichen |

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
- Externe Dienste: esm.sh / unpkg.com (MapLibre, Three.js), server.arcgisonline.com (Satellitenbilder),
  nominatim.openstreetmap.org (Adresssuche)
- Nutzt ANF-002 (Postanschrift des Kunden als Vorschlag für die Suche)

## 6. Nicht im Umfang

- Suche während des Tippens (Vorschläge); die Nutzungsregeln von Nominatim verbieten das.
- Messungen speichern, Höhen messen, an Elementen oder Eckpunkten einrasten.
- Freie Grösse je Element, Drehen per Ziehen, mehrere Elemente gleichzeitig verschieben.
- Fertige 3D-Modelle aus Dateien (`.glb` über den `GLTFLoader`); die Modelle werden aus Grundformen gebaut.
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
- [ ] Vorgabe-Standort: St. Gallen oder Adresse des Kunden? *Annahme: St. Gallen*; die Kundenadresse wird
  als Suchbegriff vorgeschlagen, aber nicht automatisch gesucht.
- [ ] Adresssuche: Nominatim (weltweit, freiwilliger Dienst mit Nutzungsregeln) oder der Suchdienst von
  swisstopo (nur Schweiz, genauer für Schweizer Adressen)? *Annahme: Nominatim.* Für viele Benutzer im
  Betrieb einen eigenen bzw. kostenpflichtigen Dienst prüfen. Der Suchbegriff geht an den Dienst.

## 8. Änderungshistorie

| Datum      | Autor | Änderung        |
|------------|-------|-----------------|
| 2026-11-23 | Hatem Ben Belgacem | Erstellt und umgesetzt (Vorgabe: Anleitung Leptos + MapLibre + Three.js) |
| 2026-11-24 | Hatem Ben Belgacem | AK-4: gestaltete 3D-Modelle statt Quadern (Bühne, Verpflegungsstand, Festzelt) |
| 2026-11-24 | Hatem Ben Belgacem | AK-18 bis AK-20: Elemente auswählen und verschieben (Ziehen mit Maus/Finger, Pfeiltasten) |
| 2026-11-24 | Hatem Ben Belgacem | AK-12a/b: Adresssuche und Standort-Adresse; AK-15 bis AK-17: Messwerkzeuge Strecke und Fläche; AK-11: Umriss ohne Überschneidung |
