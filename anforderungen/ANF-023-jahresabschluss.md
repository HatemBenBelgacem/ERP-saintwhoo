# ANF-023: Jahresabschluss und Saldovortrag

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-023                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Soll                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Am Ende eines Geschäftsjahres wird es **abgeschlossen**: Das Ergebnis der Erfolgsrechnung wird auf das
Konto Jahresgewinn/-verlust gebucht, das Jahr wird gesperrt, und die Salden der Bilanzkonten werden als
**Eröffnungsbuchung** ins Folgejahr vorgetragen. Ein **vorläufiger Saldovortrag** ist schon vor dem
Abschluss möglich, damit im neuen Jahr mit richtigen Anfangssalden gearbeitet werden kann.

```
Vorbereitung (Prüfliste) → Vorläufiger Saldovortrag (wiederholbar) → Abschluss (endgültig)
```

Der Abschluss liegt im Reiter „Abschluss“ unter `/finanzen/abschluss`.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **das Geschäftsjahr geführt abschliessen und die Salden ins neue Jahr
> übernehmen**, damit **der Abschluss vollständig ist und das neue Jahr mit den richtigen Anfangsbeständen startet**.

## 3. Akzeptanzkriterien

- [ ] **AK-1:** Gegeben der Reiter „Abschluss“, dann wählt der Benutzer ein offenes Geschäftsjahr und
  sieht eine **Prüfliste**: Buchungsentwürfe vorhanden, nicht abgeschlossene MWST-Abrechnungen, nicht
  gebuchte Debitoren-/Kreditorenentwürfe, Saldenliste ausgeglichen, Folgejahr vorhanden. Jeder Punkt ist
  als erfüllt/offen markiert und verweist auf die betreffende Liste.
- [ ] **AK-2:** Gegeben ein vorhandenes Folgejahr, wenn der Benutzer „Saldovortrag (vorläufig)“ wählt,
  dann wird im Folgejahr eine Eröffnungsbuchung (Herkunft `eroeffnung`, Datum erster Tag) mit den Salden
  aller Bilanzkonten erzeugt; das laufende Ergebnis wird auf das Konto Gewinnvortrag vorgetragen.
  Eine bestehende vorläufige Eröffnungsbuchung wird dabei ersetzt.
- [ ] **AK-3:** Gegeben neue Buchungen im alten Jahr nach einem vorläufigen Saldovortrag, dann zeigt die
  Übersicht und der Reiter „Abschluss“ den Hinweis „Saldovortrag nicht aktuell“.
- [ ] **AK-4:** Gegeben eine vollständig erfüllte Prüfliste (Warnungen dürfen nach Bestätigung übergangen
  werden, ausser „Saldenliste ausgeglichen“ und „Folgejahr vorhanden“), wenn der Benutzer „Geschäftsjahr
  abschliessen“ bestätigt (Eingabe der Jahresbezeichnung), dann wird das Ergebnis auf das Konto
  Jahresgewinn/-verlust gebucht, der Saldovortrag endgültig erzeugt, alle Perioden gesperrt und das Jahr
  auf „Abgeschlossen“ gesetzt (mit Benutzer und Zeitpunkt).
- [ ] **AK-5:** Gegeben ein abgeschlossenes Jahr, dann sind darin keine Buchungen, Stornos oder
  Entsperrungen mehr möglich; alle Auswertungen bleiben abrufbar.
- [ ] **AK-6:** Gegeben offene Posten (Debitoren/Kreditoren) zum Jahresende, dann bleiben sie im neuen
  Jahr offen und können normal bezahlt werden; die Zahlung wird im neuen Jahr gebucht.
- [ ] **AK-7:** Gegeben das erste Geschäftsjahr, dann kann eine **Eröffnungsbilanz** manuell über das
  Journal (ANF-017, Herkunft `eroeffnung`) gebucht werden.

## 4. Daten / Felder

Keine neuen Tabellen. Erweiterung `buchung`: Kennzeichen „vorläufig“ für Eröffnungsbuchungen.
Benötigte Systemkonten (ANF-011): Gewinnvortrag, Jahresgewinn/-verlust, Eröffnungsbilanz.

## 5. Abhängigkeiten

- Hängt ab von: ANF-011, ANF-013, ANF-017, ANF-021; Prüfliste nutzt ANF-018, ANF-019, ANF-022
- Wird benötigt von: –

## 6. Nicht im Umfang

- Wiedereröffnen eines abgeschlossenen Jahres (nur über Datenbank durch den Betreiber)
- Gewinnverwendung (Dividende, Reserven) – wird manuell im Folgejahr gebucht
- Abschlussbuchungen wie Abschreibungen, Abgrenzungen und Bewertung von Fremdwährungsposten
  (manuell im Journal)
- Erstellen des Anhangs bzw. Geschäftsberichts

## 7. Offene Fragen

- [ ] Soll ein abgeschlossenes Jahr durch einen Benutzer wieder geöffnet werden dürfen? *(Vorschlag: nein)*
- [ ] Wird das Ergebnis direkt auf Gewinnvortrag oder zuerst auf „Jahresgewinn/-verlust“ gebucht? *(Vorschlag: Jahresgewinn/-verlust, Vortrag auf Gewinnvortrag im Folgejahr)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
