# ANF-021: Auswertungen – Kontoauszug, Saldenliste, Bilanz, Erfolgsrechnung

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-021                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Der Reiter „Auswertungen“ (`/finanzen/auswertungen`) bietet die Standardauswertungen der
Finanzbuchhaltung. Alle Auswertungen lesen ausschliesslich **gebuchte** Buchungen aus dem Journal
(ANF-017) und gelten für einen wählbaren Zeitraum innerhalb eines Geschäftsjahres:

| Auswertung | Inhalt |
|------------|--------|
| **Kontoauszug** | alle Buchungen eines Kontos mit Anfangssaldo, Soll, Haben, laufendem Saldo |
| **Saldenliste** | alle Konten mit Anfangssaldo, Summe Soll, Summe Haben, Endsaldo |
| **Bilanz** | Aktiven und Passiven zum Stichtag, gegliedert nach Kontengruppen |
| **Erfolgsrechnung** | Ertrag und Aufwand im Zeitraum, mit Ergebnis (Gewinn/Verlust) |
| **Projektauswertung** | Erlöse und Aufwand je Projekt aus Buchungszeilen mit Projekt |

## 2. Benutzergeschichte

> Als **Geschäftsleitung** möchte ich **Bilanz und Erfolgsrechnung jederzeit abrufen und mit dem Vorjahr
> vergleichen**, damit **ich die finanzielle Entwicklung beurteilen kann**.

> Als **Buchhalter** möchte ich **jedes Konto im Detail nachvollziehen**, damit **ich Fehler finde und
> Salden abstimmen kann**.

## 3. Akzeptanzkriterien

- [x] **AK-1:** Gegeben der Reiter „Auswertungen“, dann wählt der Benutzer die Auswertung, das
  Geschäftsjahr und den Zeitraum (Vorgabe: Beginn des Geschäftsjahres bis heute). Auswahl und Zeitraum
  stehen in der Webadresse.
- [x] **AK-2:** Gegeben der **Kontoauszug** eines Kontos, dann zeigt er Anfangssaldo, jede Buchung mit
  Datum, Beleg, Text, Gegenkonto, Soll, Haben und laufendem Saldo sowie Summen und Endsaldo. Ein Klick auf
  eine Buchung öffnet sie im Journal. Aus jeder anderen Auswertung führt ein Klick auf ein Konto zum
  Kontoauszug.
- [x] **AK-3:** Gegeben die **Saldenliste**, dann zeigt sie alle Konten mit Bewegung oder Saldo ≠ 0,
  gruppiert nach Kontenklasse mit Zwischensummen; die Summe aller Soll ist gleich der Summe aller Haben.
- [x] **AK-4:** Gegeben die **Bilanz** zum Stichtag, dann zeigt sie Aktiven und Passiven nach
  Kontengruppen mit Summen; das noch nicht verbuchte Ergebnis des laufenden Jahres erscheint als eigene
  Zeile in den Passiven, sodass Aktiven = Passiven.
- [x] **AK-5:** Gegeben die **Erfolgsrechnung**, dann zeigt sie Ertrag und Aufwand nach Kontengruppen mit
  Zwischenergebnissen (Bruttogewinn, Betriebsergebnis vor Abschreibungen, Jahresergebnis); das Ergebnis
  stimmt mit der Bilanzzeile aus AK-4 überein.
- [x] **AK-6:** Gegeben Bilanz oder Erfolgsrechnung, wenn der Benutzer „Vorjahr vergleichen“ wählt, dann
  erscheint eine zusätzliche Spalte mit dem gleichen Zeitraum des Vorjahres und der Abweichung.
- [x] **AK-7:** Gegeben die **Projektauswertung**, dann zeigt sie je Projekt (bzw. für ein gewähltes
  Projekt) Erlöse, Aufwand und Ergebnis sowie offene Einkaufsaufträge (ANF-004) als zugesagte Kosten;
  die Projekt-Detailansicht verweist darauf.
- [x] **AK-8:** Gegeben eine Auswertung, dann kann sie gedruckt (Druckansicht A4) und als CSV-Datei
  heruntergeladen werden.
- [x] **AK-9:** Gegeben Entwürfe oder stornierte Buchungen, dann zählen Entwürfe nicht; stornierte
  Buchungen und ihre Gegenbuchungen heben sich auf und erscheinen nur im Kontoauszug.
- [ ] **AK-10:** Die Auswertungen laufen auch bei 100 000 Buchungszeilen in unter 2 Sekunden
  (Indizes auf Konto und Datum).

## 4. Daten / Felder

Keine neuen Tabellen; Auswertungen werden aus `buchung`, `buchungszeile`, `konto` und
`geschaeftsjahr` berechnet.

## 5. Abhängigkeiten

- Hängt ab von: ANF-011, ANF-013, ANF-017; für Projektauswertung ANF-003, ANF-004
- Wird benötigt von: ANF-010 AK-4 (Kennzahlen), ANF-023

## 6. Nicht im Umfang

- Frei gestaltbare Berichte und Diagramme
- Liquiditätsplanung, Budget-Vergleich, Kennzahlen (z. B. Eigenkapitalquote)
- Export als PDF oder Excel auf dem Server (Drucken über den Browser, CSV)
- Konsolidierung mehrerer Firmen

## 7. Offene Fragen

- [x] Soll die Bilanz nach OR 959a/959b gegliedert werden (gesetzliche Mindestgliederung)? *(Vorschlag: Gliederung nach Kontengruppen des KMU-Kontenrahmens, die OR-konform ist)*
- [x] Braucht es Diagramme in der Übersicht? *(Vorschlag: später)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `src/backend/finance/reports/model.rs` | Rechenregeln für Kontoauszug, Saldenliste, Bilanz, Erfolgsrechnung und Projektauswertung, CSV-Ausgabe, Modultests |
| `src/backend/finance/reports/repository.rs`, `service.rs`, `server_fns.rs` | Salden aus dem Journal ohne Entwürfe, Zeitraum im Geschäftsjahr, Vorjahr, Projekte mit offenen Einkaufsaufträgen |
| `src/backend/finance/reports/csv.rs`, `src/main.rs` | CSV-Download unter `/finanzen/auswertungen/csv` |
| `src/components/finance/reports/page.rs` | Seite mit Auswahl, Zeitraum, Vorjahresvergleich, Drucken und CSV |
| `src/components/projects/detail.rs` | Link „Finanzauswertung“ auf der Projektseite (AK-7) |

Festlegungen bei der Umsetzung:
- **Freigabe:** Der Auftrag „Umsetzen“ gilt als Freigabe; der Status steht auf „In Umsetzung“.
- **Offene Fragen (Vorschläge übernommen):** Die Bilanz ist nach den Kontengruppen des KMU-Kontenrahmens gegliedert. Diagramme folgen später.
- **AK-4:** Das Ergebnis früherer Jahre steht als eigene Zeile, bis der Jahresabschluss (ANF-023) vorliegt. Das laufende Ergebnis ist der Rest. So sind Aktiven und Passiven immer gleich.
- **AK-5:** Ertrag steht positiv, Aufwand negativ. Zwischenergebnisse: Bruttogewinn 1 und 2, Betriebsergebnis vor und nach Abschreibungen (Konten 68xx), Jahresergebnis. Es stimmt mit der Bilanzzeile überein.
- **AK-6:** Der Vergleich nimmt denselben Zeitraum ein Jahr früher; der 29. Februar wird zum 28. Februar.
- **AK-8:** Drucken über den Browser mit ausgeblendeter Seitenleiste. Die Seitengröße A4 setzt der Browser; sie ist nicht festgelegt.
- **AK-9:** Entwürfe zählen nie. Stornos und Gegenbuchungen stehen beide im Journal und heben sich in den Salden auf.
- **Offen:** AK-10 (Laufzeit bei 100 000 Zeilen) ist nicht gemessen. Die Indizes auf `buchung(datum)`, `buchung(status)` und `buchungszeile(konto_id)` sind angelegt.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-26 | Claude Code | Umgesetzt: Kontoauszug, Saldenliste, Bilanz, Erfolgsrechnung mit Vorjahr, Projektauswertung, Drucken und CSV. |
