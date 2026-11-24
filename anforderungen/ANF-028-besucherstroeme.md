# ANF-028: Besucherströme im Geländeplan

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-028                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Soll                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Projekte                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-24                                        |
| **Geändert**  | 2026-11-24                                        |

## 1. Beschreibung

Im Geländeplan (ANF-027) lassen sich die **Besucherströme** einer Veranstaltung planen und darstellen:

- **Besucherwege**: Linienzüge mit Richtung (Einbahn oder beide Richtungen) und erwarteter Menge
  (Personen pro Stunde). Die Linie ist umso breiter, je mehr Personen erwartet werden; Pfeile zeigen die
  Richtung. So sind die Hauptströme auf einen Blick sichtbar (Eingang → Bühne → Verpflegung → Ausgang).
- **Zugänge**: Eingänge, Ausgänge und Notausgänge als Punkte, mit Bezeichnung und optionalem Durchlass
  (Personen pro Minute).
- **Zonen**: Flächen, in denen sich Besucher aufhalten (z. B. „Publikum vor der Bühne“). Aus Fläche (m²) und
  Personendichte (Personen pro m²) ergibt sich das **Fassungsvermögen**; verglichen mit den erwarteten
  Personen zeigt eine **Ampel**, ob die Zone ausreicht.

Die Darstellung ist eine Planungshilfe und ersetzt keine Personenstrom-Simulation und keine Prüfung durch
Brandschutz oder Bewilligungsbehörde. Die Personendichte ist je Zone einstellbar.

Umsetzung in Stufen (siehe ANF-027-Gespräch): **1** Besucherwege, **2** Zugänge und Zonen (diese
Anforderung). Später: **3** Engstellen erkennen, **4** zeitliche Abläufe, **5** bewegte Darstellung.

## 2. Benutzergeschichte

> Als **Projektleiter einer Veranstaltung** möchte ich **im Geländeplan die Wege der Besucher, die Ein- und
> Ausgänge und die Aufenthaltszonen mit ihrem Fassungsvermögen einzeichnen**, damit **ich früh sehe, wo sich
> die Besucher bewegen und ob der Platz reicht**.

## 3. Akzeptanzkriterien

- [x] **AK-1:** Gegeben der Geländeplan, dann gibt es in der Seitenleiste den Bereich **„Besucherströme“**
  mit den Werkzeugen „Neuer Weg“, „Neuer Zugang“ (Eingang, Ausgang, Notausgang) und „Neue Zone“ sowie je
  einer Liste der Wege, Zugänge und Zonen.
- [x] **AK-2:** Gegeben „Neuer Weg“, wenn ich auf die Karte klicke, dann beginnt ein neuer Besucherweg; jeder
  weitere Klick setzt einen Punkt. In der Liste erfasse ich Bezeichnung, Richtung (Einbahn / beide
  Richtungen), Personen pro Stunde und optional die Wegbreite in Metern.
- [x] **AK-3:** Gegeben ein Besucherweg, dann zeichnet die Karte ihn als Linie in der Farbe der Besucherwege;
  die Linienbreite wächst mit den Personen pro Stunde. Bei „Einbahn“ zeigen Pfeile entlang der Linie die
  Richtung (vom ersten zum letzten Punkt). Am Weg steht „Bezeichnung: … P/h“.
- [x] **AK-4:** Gegeben „Neuer Zugang“ mit Art Eingang, Ausgang oder Notausgang, wenn ich auf die Karte
  klicke, dann steht dort ein Zugang mit Kurzzeichen und Bezeichnung (z. B. „E Haupteingang“). In der Liste
  erfasse ich Bezeichnung, Art und optional den Durchlass in Personen pro Minute.
- [x] **AK-5:** Gegeben „Neue Zone“, wenn ich auf die Karte klicke, dann setzt jeder Klick einen Eckpunkt;
  ab drei Punkten ist die Zone eine Fläche. In der Liste erfasse ich Bezeichnung, erwartete Personen und die
  Personendichte (Vorgabe 2 Personen pro m², Dezimalkomma erlaubt); angezeigt werden Fläche in m²,
  Fassungsvermögen (Fläche × Dichte, abgerundet) und Auslastung in Prozent.
- [x] **AK-6:** Gegeben eine Zone mit erwarteten Personen, dann zeigt die **Ampel** in der Liste und als
  Füllfarbe auf der Karte: **grün** bis 80 % Auslastung, **gelb** über 80 % bis 100 %, **rot** über 100 %.
  Ohne erwartete Personen oder mit überschneidendem Umriss ist die Zone neutral (grau) mit Hinweis.
- [x] **AK-7:** Gegeben Wege und Zonen, dann lassen sie sich wie Messungen bearbeiten: „Weiter zeichnen“
  setzt weitere Punkte, „Letzten Punkt entfernen“, „✕“ löscht, „Fertig“ beendet das Zeichnen.
- [x] **AK-8:** Gegeben Besucherströme, dann gehören sie zum Plan: Sie werden mit „Speichern“ gespeichert,
  mit „Verwerfen“ zurückgesetzt und beim Öffnen wieder angezeigt.
- [x] **AK-9:** Gegeben ein ungültiger Plan, dann weist der Server ihn mit Meldung ab: Weg mit weniger als
  2 Punkten, Zone mit weniger als 3 Punkten oder überschneidendem Umriss, Bezeichnung länger als 100 Zeichen,
  Personen pro Stunde bzw. erwartete Personen über 1 000 000, Durchlass über 100 000 pro Minute, Wegbreite
  ausserhalb 0,5 bis 100 m, Dichte ausserhalb 0,1 bis 10 Personen pro m², mehr als 50 Wege, Zugänge
  oder Zonen bzw. mehr als 100 Punkte je Weg oder Zone, Punkte mehr als 5 km vom Standort.

## 4. Daten / Felder

| Tabelle | Feld | Typ | Pflicht | Bemerkung |
|---------|------|-----|---------|-----------|
| `gelaendeplan_besucherweg` | `reihenfolge`, `bezeichnung` | Zahl, Text | ja, nein | leer = „Weg <Nummer>“ |
| | `richtung` | Text | ja | `einbahn`, `beide` |
| | `personen_pro_stunde` | Zahl | ja | 0 … 1 000 000 (0 = unbekannt) |
| | `breite_m` | Zahl | nein | 0,5 … 100 m |
| `gelaendeplan_besucherweg_punkt` | `reihenfolge`, `x_m`, `z_m` | Zahl | ja | Meter relativ zum Standort |
| `gelaendeplan_zugang` | `reihenfolge`, `art`, `bezeichnung` | Zahl, Text, Text | ja, ja, nein | `eingang`, `ausgang`, `notausgang` |
| | `x_m`, `z_m` | Zahl | ja | Lage |
| | `personen_pro_minute` | Zahl | nein | Durchlass |
| `gelaendeplan_zone` | `reihenfolge`, `bezeichnung` | Zahl, Text | ja, nein | leer = „Zone <Nummer>“ |
| | `erwartete_personen` | Zahl | ja | 0 = unbekannt |
| | `dichte` | Zahl | ja | Personen pro m², Vorgabe 2 |
| `gelaendeplan_zone_punkt` | `reihenfolge`, `x_m`, `z_m` | Zahl | ja | Eckpunkte |

## 5. Abhängigkeiten

- Hängt ab von: ANF-027 (Geländeplan)

## 6. Nicht im Umfang

- Engstellen erkennen (Menge eines Wegs gegen Breite bzw. Abstand zwischen Elementen) – Stufe 3.
- Zeitliche Abläufe mit Zeitregler (Einlass, Hauptprogramm, Auslass) – Stufe 4.
- Bewegte Punkte, Dichtekarte – Stufe 5.
- Personenstrom-Simulation einzelner Personen; verbindliche Bemessung von Fluchtwegen.
- Punkte von Wegen und Zonen ziehen (nur über „Letzten Punkt entfernen“ und neu setzen).

## 7. Offene Fragen

Vorläufige Annahmen, bis sie geklärt sind:

- [ ] Vorgabe der Personendichte? *Annahme: 2 Personen pro m²*, je Zone änderbar. Verbindliche Werte
  liefern Brandschutz bzw. Bewilligungsbehörde.
- [ ] Gehört die erwartete Besucherzahl zum Projekt (gesamt) oder zu den Zonen? *Annahme: je Zone.*
- [ ] Grenzen der Ampel? *Annahme: grün bis 80 %, gelb bis 100 %, rot darüber.*
- [ ] Braucht es den Zeitregler (Stufe 4) schon früh? *Annahme: nein.*

## 8. Änderungshistorie

| Datum      | Autor | Änderung        |
|------------|-------|-----------------|
| 2026-11-24 | Hatem Ben Belgacem | Erstellt; Stufen 1 (Besucherwege) und 2 (Zugänge, Zonen) umgesetzt |
