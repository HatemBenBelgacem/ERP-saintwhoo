# ANF-013: Geschäftsjahre und Perioden

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-013                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Buchungen sind immer einem **Geschäftsjahr** und darin einer **Periode** (Monat) zugeordnet. Geschäftsjahre
und Perioden werden unter `/einstellungen/finanzen/geschaeftsjahre` gepflegt. Perioden können **gesperrt**
werden (z. B. nach der MWST-Abrechnung eines Quartals), damit dort nicht mehr gebucht wird. Das
**Abschliessen** eines Geschäftsjahres selbst geschieht in ANF-023.

```
Geschäftsjahr:  Offen ──► Abgeschlossen (ANF-023)
Periode:        Offen ◄─► Gesperrt
```

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Geschäftsjahre anlegen und abgerechnete Monate sperren**, damit
> **bereits ausgewertete Zahlen nicht mehr unbemerkt verändert werden**.

## 3. Akzeptanzkriterien

- [ ] **AK-1:** Gegeben die Einstellungsseite „Geschäftsjahre“, dann erscheinen alle Geschäftsjahre mit
  Bezeichnung, Beginn, Ende und Status, das neueste zuoberst. Ein Klick klappt die Perioden mit Status auf.
- [ ] **AK-2:** Gegeben „Neues Geschäftsjahr“, dann öffnet sich ein Modal-Fenster (`?neu=1`) mit
  vorgeschlagenem Beginn (Tag nach dem Ende des letzten Jahres, sonst 1. Januar des laufenden Jahres) und
  Ende (ein Jahr später). Beginn und Ende sind Pflicht; die Dauer liegt zwischen 1 und 23 Monaten
  (verlängertes erstes Geschäftsjahr erlaubt).
- [ ] **AK-3:** Gegeben ein neues Geschäftsjahr, dann darf es sich mit keinem bestehenden überschneiden und
  muss lückenlos an das vorherige anschliessen; sonst wird das Speichern mit Meldung abgelehnt.
- [ ] **AK-4:** Gegeben ein gespeichertes Geschäftsjahr, dann werden automatisch die Monatsperioden
  angelegt (Status „Offen“).
- [ ] **AK-5:** Gegeben eine offene Periode, wenn der Benutzer „Sperren“ wählt und bestätigt, dann ist sie
  gesperrt; Buchungen mit Datum in dieser Periode werden abgelehnt (ANF-010 AK-8). „Entsperren“ ist
  möglich, solange das Geschäftsjahr nicht abgeschlossen ist, und wird mit Benutzer und Zeitpunkt vermerkt.
- [ ] **AK-6:** Gegeben ein Buchungsdatum, für das kein offenes Geschäftsjahr existiert, dann wird das
  Buchen mit der Meldung „Kein offenes Geschäftsjahr für dieses Datum“ verhindert.
- [ ] **AK-7:** Gegeben ein Geschäftsjahr ohne Buchungen, dann kann es gelöscht werden (mit Perioden).
  Ein Geschäftsjahr mit Buchungen kann nicht gelöscht werden.
- [ ] **AK-8:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `geschaeftsjahr`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Bezeichnung | Text (20) | Ja | Vorschlag: Jahr des Endes, z. B. `2026` |
| Beginn | Datum | Ja | |
| Ende | Datum | Ja | nach Beginn |
| Status | Auswahl | Ja | `offen`, `abgeschlossen` |
| Abgeschlossen am / von | Zeitstempel / Verweis Benutzer | Nein | gesetzt durch ANF-023 |

Tabelle `periode`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Geschäftsjahr (`geschaeftsjahr_id`) | Verweis | Ja | wird mit dem Jahr gelöscht |
| Beginn / Ende | Datum | Ja | ein Kalendermonat |
| Status | Auswahl | Ja | `offen`, `gesperrt` |
| Geändert von / am | Verweis Benutzer / Zeitstempel | Nein | letzte Sperrung bzw. Entsperrung |

## 5. Abhängigkeiten

- Hängt ab von: ANF-010, ANF-005 (Benutzer für Vermerk)
- Wird benötigt von: ANF-017 bis ANF-023

## 6. Nicht im Umfang

- Abschluss und Saldovortrag (ANF-023)
- Andere Periodenlängen als Monate (z. B. 13 Perioden)

## 7. Offene Fragen

- [ ] Entspricht das Geschäftsjahr dem Kalenderjahr? *(Vorschlag: Vorgabe Kalenderjahr, frei wählbar)*
- [ ] Wer darf Perioden entsperren? *(Vorschlag: jeder Benutzer, mit Vermerk; später nur Rolle „Buchhaltung“)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
