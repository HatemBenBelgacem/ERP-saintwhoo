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
| **Geändert**  | 2026-11-26                                        |

## 1. Beschreibung

Zu einem Projekt (ANF-003) lässt sich ein **Geländeplan** erstellen – z. B. für ein Fest oder eine
Veranstaltung. Der Plan zeigt das Gelände als **Satellitenbild in einer geneigten 3D-Karte**. Darauf
werden **Bühnen, Verpflegungsstände, Festzelte, Gitterabsperrungen, Sanität und allgemeine Elemente** als 3D-Modelle platziert, gedreht und gelöscht. Optional
wird die **Rasenfläche abgesteckt**: Ihre Grösse in m² wird laufend angezeigt, und Elemente lassen sich nur
platzieren, wenn sie ganz auf der Fläche stehen.

Technik (Vorgabe des Auftraggebers):

| Teil | Technik |
|------|---------|
| Karte und Luftbild | **MapLibre GL JS** 4.7.1, amtliches Luftbild **swisstopo SWISSIMAGE** (rund 10 cm pro Bildpunkt, nur Schweiz und Liechtenstein) |
| 3D-Elemente | **Three.js** 0.170, als eigene Ebene („custom layer“) im selben `<canvas>` wie die Karte; Modelle aus Grundformen (Quader, Zylinder, ausgezogenes Dreieck) |
| Einbindung | Beide als ES-Module aus dem Netz (esm.sh), erst beim Öffnen des Geländeplans geladen – kein npm, kein Bündeln |
| Brücke | `src/components/site_plans/map_bridge.js`, aus Rust über `wasm-bindgen` gesteuert (`bridge.rs`) |
| Flächen und Lage | Rust-Paket **`geo`** (`unsigned_area`, `contains`, `is_valid`) – im Browser und auf dem Server |
| Adresssuche | **Suchdienst von swisstopo** (geo.admin.ch SearchServer: Adressen, Orte, Flurnamen, Gemeinden, PLZ der Schweiz und Liechtensteins), direkt aus dem Browser, auf Knopfdruck |
| Messungen | Mehrere beschriftete Strecken und Flächen, **rot** gezeichnet als Ebenen von MapLibre, Beschriftungen als Marken auf der Karte, mit dem Plan gespeichert |

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
  Vorgabe-Standort (St. Gallen) geneigt (60°) als Luftbild von swisstopo (Quellenangabe „Luftbild © swisstopo“),
  und der Plan ist als „Noch nicht gespeichert“ markiert. Ausserhalb der Schweiz und Liechtensteins zeigt die
  Karte einen neutralen Hintergrund; im Fenster „Standort“ weist ein Hinweis darauf hin.
- [x] **AK-3:** Gegeben die Karte, wenn ich sie mit der rechten Maustaste bzw. Strg + Ziehen kippe und drehe,
  dann bleiben alle 3D-Elemente an ihrer Stelle auf dem Boden.
- [x] **AK-4:** Gegeben die Auswahlliste „Element“ mit einer gewählten Art, z. B. „Bühne (10 × 8 m)“,
  „Verpflegungsstand (4 × 2,5 m)“ oder „Festzelt (6 × 6 m)“, wenn ich auf die Karte klicke, dann erscheint dort ein 3D-Modell der Art und ein
  Eintrag in der Liste „Platzierte Elemente“ mit Grösse, Lage und Drehung. Die Modelle bestehen aus
  Grundformen von Three.js und bleiben innerhalb ihrer Grundfläche:
  - **Bühne** (4,5 m): rote Podestverkleidung, Bühnenboden, Rückwand, vier Stützen, rotes Dach, zwei Lautsprecher.
  - **Verpflegungsstand** (2,8 m): orange Theke mit heller Platte, Rück- und Seitenwände, Dach, orange-hell
    gestreiftes Vordach nach vorne.
  - **Festzelt** (3 m): helle Zeltwände, blaues Giebeldach mit Firststange, dunkler Eingang vorne.
  - **Gitterabsperrung** „Gitterabsperrung (2,5 × 0,6 m)“ (1,1 m): metallgraue Gitterfelder von rund 2,5 m
    mit Pfosten, Ober- und Unterholm, Gitterstäben und Füssen; mit grösserer Breite eine ganze Absperrlinie.
- [x] **AK-5:** Gegeben ein Element in der Liste, wenn ich „↺“ bzw. „↻“ wähle, dann dreht es sich um 15° gegen
  bzw. im Uhrzeigersinn um die eigene Achse (Mittelpunkt); mit „✕“ wird es entfernt.
- [x] **AK-6:** Gegeben das Werkzeug „Rasenfläche abstecken“, wenn ich auf die Karte klicke, dann setzt jeder
  Klick einen Eckpunkt; ab drei Eckpunkten erscheint die Fläche halbdurchsichtig lila und ihre Grösse in m²
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
- [x] **AK-15:** Gegeben „Neue Strecke“, wenn ich auf die Karte klicke, dann beginnt eine neue Messung; jeder
  weitere Klick setzt einen Punkt. Die Länge jedes Abschnitts steht auf der Karte und in der Liste, dazu die
  Gesamtstrecke (bis 1 km in m, darüber in km). Die Messlinien sind **rot**.
- [x] **AK-16:** Gegeben „Neue Fläche“, dann wird der Linienzug geschlossen und ab drei Punkten rot gefüllt;
  Umfang, Fläche in m² und die Abschnitte werden angezeigt. Überschneidet sich der Umriss, steht statt der
  Fläche „–“ und ein Hinweis erscheint.
- [x] **AK-17:** Gegeben mehrere Messungen, dann stehen sie alle in der Liste „Messungen“ und auf der Karte.
  Jede lässt sich **beschriften** (höchstens 100 Zeichen, leer = „Messung <Nummer>“); auf der Karte steht
  „Bezeichnung: Ergebnis“. Die Messung, an der gerade gemessen wird, ist in der Liste markiert und auf der
  Karte kräftiger gezeichnet, mit Abschnittslängen. „Weiter messen“ setzt weitere Punkte an eine bestehende
  Messung, „Letzten Punkt entfernen“ und „✕“ machen rückgängig, „Fertig“ beendet das Messen.
- [x] **AK-17a:** Gegeben Messungen, dann gehören sie zum Plan: Sie werden mit „Speichern“ gespeichert, mit
  „Verwerfen“ zurückgesetzt und beim Öffnen wieder angezeigt. Höchstens 50 Messungen mit je höchstens 100
  Punkten; eine Strecke braucht mindestens 2, eine Fläche mindestens 3 Punkte, sonst meldet der Server einen Fehler.
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
- [x] **AK-21:** Gegeben ein ausgewähltes Element, dann zeigt die Karte vor seiner Vorderseite einen weissen
  **Drehgriff**, verbunden mit dem Mittelpunkt. Wenn ich den Griff mit Maus oder Finger ziehe, dreht sich das
  Element um die eigene Achse und zeigt zum Zeiger (ganze Grad, mit Umschalt in 15°-Schritten). Taste R dreht
  das ausgewählte Element um 15° im Uhrzeigersinn, Umschalt + R dagegen. Ragt ein Element, das auf der
  Rasenfläche stand, nach dem Loslassen hinaus, springt die Drehung mit Hinweis zurück.
- [x] **AK-22:** Gegeben eine Rasenfläche, wenn ich einen Eckpunkt mit Maus oder Finger anfasse und ziehe
  (beim Auswählen, Platzieren und Abstecken, nicht beim Messen), dann folgt er dem Zeiger; Fläche in m²,
  Lageprüfung der Elemente und Markierung „ausserhalb“ werden laufend nachgeführt. Ein Klick auf einen
  bestehenden Eckpunkt setzt keinen neuen. Fläche, Umriss und Eckpunkte sind **lila**.
- [x] **AK-23:** Gegeben ein ausgewähltes Element, dann zeigt die Liste „Platzierte Elemente“ darunter
  „Eigenschaften von …“ mit **Breite, Tiefe, Höhe** (in Metern, Dezimalkomma erlaubt) und **Drehung** (°). Ein
  geänderter Wert wird beim Verlassen des Felds übernommen; 3D-Modell, Liste, Lageprüfung und Engstellen
  (ANF-028) folgen sofort. Erlaubt sind Breite und Tiefe 0,1 bis 200 m, Höhe 0,1 bis 50 m; ungültige Werte
  werden am Feld gemeldet. Würde ein Element auf der Rasenfläche über den Rand wachsen, wird die Änderung mit
  Hinweis abgewiesen. „Vorgabemasse“ setzt Breite, Tiefe und Höhe auf die Vorgabe der Art zurück. Die Masse
  werden mit dem Plan gespeichert und in der Liste als „Breite × Tiefe × Höhe“ angezeigt.
- [x] **AK-24:** Gegeben das Werkzeug **„Gitterabsperrung (2,5 × 0,6 m)“**, wenn ich auf die Karte klicke,
  dann steht dort ein Gitterelement von 1,1 m Höhe. Mit eigener Breite (AK-23) wird daraus eine Absperrlinie,
  z. B. 20 m; Pfosten und Füsse stehen rund alle 2,5 m. Gitterabsperrungen lassen sich wie alle Elemente
  verschieben und drehen und engen Besucherwege ein bzw. versperren sie (ANF-028).
- [x] **AK-25:** Gegeben die Auswahlliste „Element“, wenn ich „Sanität (6 × 2,5 m)“ oder „Allgemeines Element (4 × 4 m)“
  wähle und auf die Karte klicke, dann erscheint dort das 3D-Modell der Art und ein Eintrag in der Liste „Platzierte
  Elemente“ wie bei AK-4. Die Modelle bestehen aus Grundformen von Three.js und bleiben innerhalb ihrer Grundfläche:
  - **Sanität** (2,8 m): weisse Kabine mit dunklem Flachdach, rotes Kreuz auf dem Dach und an beiden Längsseiten,
    dunkle Tür vorne links.
  - **Allgemeines Element** (4 m, quadratisch): Körper in Petrol mit Spitzdach (Giebel) in Anthrazit und heller Tür
    vorne. Es steht für verschiedene Zwecke.
- [x] **AK-26:** Gegeben ein ausgewähltes Element, dann lässt sich unter „Eigenschaften von …“ eine **Bezeichnung** eintragen
  (höchstens 100 Zeichen, z. B. „Erste Hilfe Nord“). Sie steht auf der Karte als Marke über dem 3D-Modell und in der
  Liste „Platzierte Elemente“ hinter dem Namen. Leer = keine Beschriftung. Die Bezeichnung wird mit dem Plan gespeichert;
  mehr als 100 Zeichen meldet der Server.
- [x] **AK-27:** Gegeben ein ausgewähltes Element, dann lassen sich unter „Eigenschaften von …“ die **Art** (Auswahl wie
  bei der Platzierung), die **Bezeichnung**, die **Masse**, die **Drehung** und die **Lage** ändern. Die Lage ist der Abstand
  vom Standort in Metern: X nach Osten, Z nach Süden (Dezimalkomma erlaubt). Ein Wechsel der Art behält Lage, Drehung,
  Masse und Bezeichnung; „Vorgabemasse“ setzt die Masse auf die Vorgabe der neuen Art zurück. Eine Änderung, die das
  Element über den Rand der Rasenfläche brächte, wird mit Hinweis abgewiesen. Ungültige Zahlen werden am Feld gemeldet.
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
| `art` | Text | ja | `buehne`, `verpflegungsstand`, `festzelt`, `gitterabsperrung`, `sanitaet`, `allgemein` |
| `x_m`, `z_m` | Zahl | ja | Mittelpunkt in Metern relativ zum Standort |
| `drehung_grad` | Zahl | ja | 0 ≤ Wert < 360 |
| `breite_m`, `tiefe_m` | Zahl | ja | 0,1 … 200 m, Vorgabe je Art, änderbar |
| `hoehe_m` | Zahl | nein | 0,1 … 50 m; leer = Vorgabe der Art |
| `bezeichnung` | Text | nein | höchstens 100 Zeichen; leer = keine Beschriftung (AK-26) |

Tabelle `gelaendeplan_eckpunkt`: `reihenfolge`, `x_m`, `z_m` (leer = keine Rasenfläche).

Tabelle `gelaendeplan_messung`: `reihenfolge`, `art` (`strecke`, `flaeche`), `bezeichnung` (höchstens 100
Zeichen, leer = „Messung <Nummer>“). Ihre Punkte in `gelaendeplan_messpunkt`: `messung_id`, `reihenfolge`,
`x_m`, `z_m`.

## 5. Abhängigkeiten

- Hängt ab von: ANF-003 (Projekte)
- Wird benötigt von: ANF-028 (Besucherströme im Geländeplan)
- Externe Dienste: esm.sh / unpkg.com (MapLibre, Three.js), wmts.geo.admin.ch (Luftbild SWISSIMAGE),
  api3.geo.admin.ch (Adresssuche)
- Nutzt ANF-002 (Postanschrift des Kunden als Vorschlag für die Suche)

## 6. Nicht im Umfang

- Suche während des Tippens (Vorschläge) und Adressen ausserhalb der Schweiz und Liechtensteins.
- Höhen messen, Messpunkte ziehen, an Elementen oder Eckpunkten einrasten.
- Gebogene Absperrlinien (eine Gitterabsperrung ist gerade; Kurven aus mehreren Elementen zusammensetzen),
  mehrere Elemente gleichzeitig verschieben, Eckpunkte einfügen oder einzeln löschen.
- Fertige 3D-Modelle aus Dateien (`.glb` über den `GLTFLoader`); die Modelle werden aus Grundformen gebaut.
- Mehrere Pläne oder Versionen je Projekt, Druckansicht, Export.
- Gleichzeitiges Bearbeiten durch mehrere Benutzer (wer zuletzt speichert, gewinnt).

## 7. Offene Fragen

Vorläufige Annahmen, bis sie geklärt sind:

- [ ] Ist der Geländeplan eine eigene Seite? *Annahme: ja.* Die 3D-Karte ist eine Arbeitsfläche, kein
  Formular; sie braucht die ganze Breite. Eingaben (Standort, Löschen) laufen trotzdem in Modal-Fenstern.
- [ ] Speichern ausdrücklich oder automatisch? *Annahme: ausdrücklich mit „Speichern“* (Plan als Ganzes).
- [ ] Begriff „Foodstand“ oder „Verpflegungsstand“? *Annahme: „Verpflegungsstand“* (deutscher Begriff).
- [x] Welches Luftbild? **Geklärt (2026-11-24):** Der Geländeplan wird nur in der Schweiz verwendet → swisstopo
  SWISSIMAGE statt Esri World Imagery (schärfer, amtlich, frei nutzbar mit Quellenangabe).
- [ ] Bibliotheken aus dem Netz (esm.sh) oder selbst ausliefern (`public/`)? *Annahme: aus dem Netz*,
  feste Fassungen. Für Betrieb ohne fremde Dienste später selbst ausliefern.
- [ ] Vorgabe-Standort: St. Gallen oder Adresse des Kunden? *Annahme: St. Gallen*; die Kundenadresse wird
  als Suchbegriff vorgeschlagen, aber nicht automatisch gesucht.
- [x] Adresssuche: Nominatim oder der Suchdienst von swisstopo? **Geklärt (2026-11-24):** swisstopo, da der
  Geländeplan nur in der Schweiz verwendet wird (genauer für Schweizer Adressen, findet auch Flurnamen).
  Der Suchbegriff geht an den Dienst.
- [ ] Ist „Sanität“ ein Sanitätsposten (Erste Hilfe)? *Annahme: ja.* Sanitäranlagen (WC) wären ein eigenes Element.
- [x] Was steht auf dem allgemeinen Element? **Geklärt (2026-11-26):** Jedes Element lässt sich beschriften (AK-26).
  Das allgemeine Element bleibt ein Platzhalter; die Bezeichnung sagt, wofür es steht.

## 8. Änderungshistorie

| Datum      | Autor | Änderung        |
|------------|-------|-----------------|
| 2026-11-23 | Hatem Ben Belgacem | Erstellt und umgesetzt (Vorgabe: Anleitung Leptos + MapLibre + Three.js) |
| 2026-11-24 | Hatem Ben Belgacem | AK-23: eigene Masse je Element (Breite, Tiefe, Höhe, Drehung); AK-24: Gitterabsperrung |
| 2026-11-24 | Hatem Ben Belgacem | AK-4: gestaltete 3D-Modelle statt Quadern (Bühne, Verpflegungsstand, Festzelt) |
| 2026-11-24 | Hatem Ben Belgacem | AK-15 bis AK-17a: mehrere beschriftete Messungen, rot, mit dem Plan gespeichert |
| 2026-11-24 | Hatem Ben Belgacem | AK-12a: Adresssuche über swisstopo statt Nominatim |
| 2026-11-24 | Hatem Ben Belgacem | AK-2: Luftbild swisstopo SWISSIMAGE statt Esri (nur Schweiz) |
| 2026-11-24 | Hatem Ben Belgacem | AK-5, AK-21: Drehen um die eigene Achse (Drehgriff, Taste R, 15°-Schritte); AK-6, AK-22: Rasenfläche lila, Eckpunkte ziehbar |
| 2026-11-24 | Hatem Ben Belgacem | AK-18 bis AK-20: Elemente auswählen und verschieben (Ziehen mit Maus/Finger, Pfeiltasten) |
| 2026-11-24 | Hatem Ben Belgacem | AK-12a/b: Adresssuche und Standort-Adresse; AK-15 bis AK-17: Messwerkzeuge Strecke und Fläche; AK-11: Umriss ohne Überschneidung |
| 2026-11-26 | Hatem Ben Belgacem | AK-4: Elemente über die Auswahlliste „Element“ platzieren statt über Schaltflächen; AK-25: Elementarten Sanität und Allgemeines Element (Spitzdach); Migration `gelaendeplan_element_art` |
| 2026-11-26 | Hatem Ben Belgacem | AK-26: Elemente lassen sich beschriften (Bezeichnung, auf der Karte und in der Liste); Migration `gelaendeplan_element_bezeichnung` |
| 2026-11-26 | Hatem Ben Belgacem | AK-27: Art und Lage (X, Z) eines Elements nachträglich ändern; der Abschnitt heisst jetzt „Eigenschaften von …“ (AK-23, AK-26 angepasst) |
