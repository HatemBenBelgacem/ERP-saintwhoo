# ANF-013: Geschäftsjahre und Perioden

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-013                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-19                                        |

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

- [x] **AK-1:** Gegeben die Einstellungsseite „Geschäftsjahre“, dann erscheinen alle Geschäftsjahre mit
  Bezeichnung, Beginn, Ende und Status, das neueste zuoberst. Ein Klick klappt die Perioden mit Status auf.
- [x] **AK-2:** Gegeben „Neues Geschäftsjahr“, dann öffnet sich ein Modal-Fenster (`?neu=1`) mit
  vorgeschlagenem Beginn (Tag nach dem Ende des letzten Jahres, sonst 1. Januar des laufenden Jahres) und
  Ende (ein Jahr später). Beginn und Ende sind Pflicht; die Dauer liegt zwischen 1 und 23 Monaten
  (verlängertes erstes Geschäftsjahr erlaubt).
- [x] **AK-3:** Gegeben ein neues Geschäftsjahr, dann darf es sich mit keinem bestehenden überschneiden und
  muss lückenlos an das vorherige anschliessen; sonst wird das Speichern mit Meldung abgelehnt.
- [x] **AK-4:** Gegeben ein gespeichertes Geschäftsjahr, dann werden automatisch die Monatsperioden
  angelegt (Status „Offen“).
- [x] **AK-5:** Gegeben eine offene Periode, wenn der Benutzer „Sperren“ wählt und bestätigt, dann ist sie
  gesperrt; Buchungen mit Datum in dieser Periode werden abgelehnt (ANF-010 AK-8). „Entsperren“ ist
  möglich, solange das Geschäftsjahr nicht abgeschlossen ist, und wird mit Benutzer und Zeitpunkt vermerkt.
- [x] **AK-6:** Gegeben ein Buchungsdatum, für das kein offenes Geschäftsjahr existiert, dann wird das
  Buchen mit der Meldung „Kein offenes Geschäftsjahr für dieses Datum“ verhindert.
- [x] **AK-7:** Gegeben ein Geschäftsjahr ohne Buchungen, dann kann es gelöscht werden (mit Perioden).
  Ein Geschäftsjahr mit Buchungen kann nicht gelöscht werden.
- [x] **AK-8:** Die Tabellen werden per sqlx-Migration angelegt.

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

- [x] Entspricht das Geschäftsjahr dem Kalenderjahr? **Entschieden 2026-11-19: Vorschlag übernommen** (Kalenderjahr als Vorgabe, frei wählbar).
- [x] Wer darf Perioden entsperren? **Entschieden 2026-11-19: Vorschlag übernommen** (jeder Benutzer, mit Vermerk; später nur Rolle „Buchhaltung“).

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261119110000_geschaeftsjahr.sql` | Tabellen `geschaeftsjahr` (Ausschlussbedingung gegen Überlappung, AK-3) und `periode` (eindeutiger Monat je Jahr, Vermerk von Benutzer und Zeitpunkt) |
| `src/backend/finance/fiscal_year/model.rs` | Prüfregeln (Monatsanfang und -ende, 1 bis 23 Monate, lückenloser Anschluss), Vorschlag, Monatsperioden, Sperrregel beim Entsperren, Buchungsprüfung für ANF-017 (mit Modultests) |
| `src/backend/finance/fiscal_year/repository.rs`, `server_fns.rs` | SQL, Anlegen mit Perioden, Löschen (nur das neueste), Sperren und Entsperren mit Vermerk, Sperre gegen gleichzeitiges Anlegen |
| `src/backend/finance/date.rs` | Datumsrechnung ohne Zeitpaket (Monatslängen, Tag danach, Monate zählen) |
| `src/components/settings/finance/geschaeftsjahre/` | Seite `/einstellungen/finanzen/geschaeftsjahre` mit aufklappbaren Jahren, Fenster für Anlegen, Sperren und Löschen |

Festlegungen bei der Umsetzung:
- **Beginn** am 1. und **Ende** am letzten Tag eines Monats, weil Perioden Kalendermonate sind. Die Dauer zählt Kalendermonate, beide eingerechnet.
- **Bezeichnung** wird aus den Daten abgeleitet: „2026“, bei abweichendem Jahr „2026/2027“.
- **Löschen (AK-7):** Nur das neueste Geschäftsjahr, sonst entstünde eine Lücke. Die Prüfung auf Buchungen folgt mit ANF-017 (Tabelle `buchung`).
- **AK-5 (teilweise):** Sperren und Entsperren mit Bestätigung und Vermerk sind umgesetzt. Die Ablehnung von Buchungen in gesperrten Perioden folgt mit ANF-017.
- **AK-6 (teilweise):** Die Prüffunktion `check_booking_date` ist vorhanden, sie wird beim Buchen mit ANF-017 aufgerufen.
- **AK-7 (teilweise):** Löschen ohne Buchungsprüfung, siehe oben.

Geprüft: 6 Modultests (Vorschlag, Dauer, Monatsgrenzen, Anschluss, Bezeichnung, Buchungsregel) und HTTP-Aufrufe der Server-Funktionen (Anlegen mit allen Fehlerfällen, Sperren mit Vermerk, Entsperren, Löschen). Layout auf grossem Bildschirm und Mobiltelefon per Screenshot geprüft. Die Szenarien TS-013-01 bis TS-013-11 sind noch nicht vollständig von Hand durchgeklickt.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-19 | Claude Code | Umgesetzt: Geschäftsjahre mit Monatsperioden, Lückenprüfung, Sperren mit Vermerk, Löschen des neuesten Jahres. Offen: AK-5, AK-6, AK-7 (Buchungsprüfung mit ANF-017) |
| 2026-11-26 | Claude Code | Nachtrag: AK-5 bis AK-7 mit ANF-017. Buchungen in gesperrten Perioden und ohne offenes Geschäftsjahr werden abgelehnt; ein Geschäftsjahr mit Buchungen lässt sich nicht löschen. |
