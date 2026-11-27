# ANF-018: Debitoren – Ausgangsrechnungen

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-018                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-27                                        |

## 1. Beschreibung

Das ERP erstellt **Ausgangsrechnungen** an Kunden (Debitoren). Eine Rechnung hat Kopfdaten (Kunde,
Datum, Zahlungsbedingung, Projekt …) und **Positionen** wie im Einkauf (ANF-004). Mit dem **Buchen** wird
die Rechnung zum offenen Posten und erzeugt automatisch eine Buchung im Journal (ANF-017):

```
Soll  Debitoren (Systemkonto)          Gesamtbetrag
Haben Ertragskonto der Position        Nettobetrag je Position
Haben Umsatzsteuer (MWST-Konto Code)   MWST je Code
```

Eine **Gutschrift** ist eine Rechnung mit umgekehrtem Vorzeichen (z. B. Rückerstattung) und kann auf eine
Rechnung verweisen.

**Status:**
```
Entwurf → Offen → Teilweise bezahlt → Bezahlt
   │        │
   └────────┴──→ Storniert
```

Die Debitoren liegen im Reiter „Debitoren“ unter `/finanzen/debitoren`.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Rechnungen an Kunden erstellen, drucken und buchen**, damit **Erlöse
> und Forderungen korrekt erfasst sind und ich sehe, was noch offen ist**.

> Als **Projektleiter** möchte ich **Rechnungen meinem Projekt zuordnen**, damit **ich sehe, was im Projekt
> bereits verrechnet wurde**.

## 3. Akzeptanzkriterien

**Liste (Anzeigen)**
- [x] **AK-1:** Gegeben der Reiter „Debitoren“, dann erscheint eine Tabelle mit Nummer, Art (Rechnung/
  Gutschrift), Kunde, Projekt, Rechnungsdatum, Fälligkeit, Gesamtbetrag, offener Betrag, Währung und Status.
- [x] **AK-2:** Gegeben die Liste, dann kann nach Nummer, Kunde, Projekt und Referenz gesucht, nach Status
  und Zeitraum gefiltert und sortiert werden. Standard: Status Offen und Teilweise bezahlt, nach Fälligkeit.
  Filter, Sortierung und Seite stehen in der Webadresse; ab 50 Einträgen seitenweise.
- [x] **AK-3:** Gegeben eine offene Rechnung mit überschrittener Fälligkeit, dann ist sie als überfällig
  markiert (Signalfarbe `error`) und zeigt die Tage seit Fälligkeit.

**Anlegen und Bearbeiten (Entwurf)**
- [x] **AK-4:** Gegeben „Neue Rechnung“ bzw. „Neue Gutschrift“, dann öffnet sich ein Modal-Fenster
  (`?neu=rechnung` bzw. `?neu=gutschrift`). Kunde (Suche im Adressstamm) und Rechnungsdatum sind Pflicht;
  Projekt, Referenz des Kunden, Währung (Vorgabe CHF), Zahlungsbedingung (Vorgabe der Adresse bzw.
  Standard), Einleitungs- und Schlusstext sind wählbar.
- [x] **AK-5:** Gegeben eine Rechnung im Entwurf, dann lassen sich Positionen anlegen, bearbeiten,
  löschen und verschieben (`?neu=position`, `?bearbeiten=…`). Bezeichnung, Menge (> 0), Einheit,
  Einzelpreis (≥ 0), Ertragskonto und MWST-Code sind Pflicht; Rabatt 0–100 %. Das Ertragskonto wird aus
  einer Vorgabe (Systemkonto Ertrag, sonst 3400) vorgeschlagen.
- [x] **AK-6:** Gegeben die Positionen, dann werden Positionsbetrag, Zwischensumme, MWST je Code und
  Gesamtbetrag wie in ANF-004 berechnet; der Gesamtbetrag in CHF wird auf 0.05 gerundet, die Differenz
  als Rundungszeile ausgewiesen.
- [x] **AK-7:** Gegeben ein Projekt mit Kunde, wenn der Benutzer das Projekt zuerst wählt, dann wird der
  Kunde des Projekts vorgeschlagen.

**Buchen**
- [x] **AK-8:** Gegeben ein Entwurf mit mindestens einer Position, wenn der Benutzer „Buchen“ klickt, dann
  erhält die Rechnung eine fortlaufende Nummer (z. B. `R-2026-0001`, Gutschrift `G-2026-0001`), Fälligkeit
  und Skontofrist werden berechnet (ANF-016), die Buchung wird im Journal erzeugt (Herkunft `debitor`) und
  der Status wechselt auf „Offen“. Bei Fremdwährung wird zum Kurs des Rechnungsdatums umgerechnet (ANF-014).
- [x] **AK-9:** Gegeben eine gesperrte Periode, fehlende Systemkonten oder ein ungültiger MWST-Code, dann
  wird das Buchen mit Meldung verhindert und nichts gespeichert.
- [x] **AK-10:** Gegeben eine gebuchte Rechnung, dann sind Kunde, Positionen und Beträge gesperrt; nur
  Referenz, Bemerkung und Texte für den Druck bleiben änderbar.

**Druck**
- [x] **AK-11:** Gegeben eine Rechnung, wenn der Benutzer „Drucken“ wählt, dann öffnet sich eine
  Druckansicht (`/finanzen/debitoren/<kennung>/druck`) mit Absender (Firma und ihre Adresse, ANF-025),
  Kundenadresse, Rechnungsnummer, Datum,
  MWST-Nummer, Positionen, MWST-Ausweis je Satz, Gesamtbetrag, Zahlungsbedingung und Bankverbindung
  (Standard-Bankkonto, ANF-015). Sie ist für A4 gestaltet; ein Entwurf ist als „Entwurf“ gekennzeichnet.

**Stornieren, Löschen, Gutschrift**
- [x] **AK-12:** Gegeben eine Rechnung im Entwurf, dann kann sie nach Bestätigung gelöscht werden.
- [x] **AK-13:** Gegeben eine offene Rechnung ohne Zahlungen, wenn der Benutzer „Stornieren“ bestätigt,
  dann wird die Buchung storniert (Gegenbuchung) und der Status ist „Storniert“. Rechnungen mit Zahlungen
  können nicht storniert werden; stattdessen wird eine Gutschrift erstellt.
- [x] **AK-14:** Gegeben eine gebuchte Rechnung, wenn der Benutzer „Gutschrift erstellen“ wählt, dann wird
  ein Gutschrift-Entwurf mit denselben Positionen und Verweis auf die Rechnung angelegt. Eine gebuchte
  Gutschrift kann mit der Rechnung verrechnet werden (ANF-020).

**Verknüpfungen**
- [x] **AK-15:** Gegeben ein Projekt, dann zeigt dessen Detailansicht die zugeordneten Ausgangsrechnungen
  mit Nummer, Datum, Status, Gesamtbetrag und offenem Betrag.
- [x] **AK-16:** Gegeben eine Adresse, dann zeigt ihre Detailansicht einen Abschnitt „Debitoren“ mit den
  offenen Rechnungen und dem offenen Gesamtbetrag. Adressen und Projekte mit Rechnungen können nicht
  gelöscht werden.

**Allgemein**
- [x] **AK-17:** Gegeben eine nicht existierende Kennung, dann erscheint „Rechnung nicht gefunden“.
- [x] **AK-18:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `debitorenrechnung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Nummer | Text (20) | bei gebucht | fortlaufend je Jahr und Art, eindeutig |
| Art | Auswahl | Ja | `rechnung`, `gutschrift` |
| Kunde (`adresse_id`) | Verweis | Ja | ANF-002 |
| Projekt (`projekt_id`) | Verweis | Nein | ANF-003 |
| Bezug auf Rechnung (`bezug_rechnung_id`) | Verweis | Nein | bei Gutschriften |
| Status | Auswahl | Ja | `entwurf`, `offen`, `teilweise_bezahlt`, `bezahlt`, `storniert` |
| Rechnungsdatum | Datum | Ja | |
| Zahlungsbedingung (`zahlungsbedingung_id`) | Verweis | Ja | ANF-016 |
| Fälligkeit / Skonto bis / Skonto % | Datum / Datum / Dezimal | bei gebucht | beim Buchen berechnet |
| Währung / Kurs | Text (3) / Dezimal (14,6) | Ja | ANF-014, Kurs 1 bei CHF |
| Referenz Kunde | Text (100) | Nein | z. B. Bestellnummer des Kunden |
| Einleitungstext / Schlusstext | Langtext | Nein | für den Druck |
| Gesamtbetrag / Gesamtbetrag CHF | Dezimal (14,2) | bei gebucht | festgeschrieben beim Buchen |
| Offener Betrag | Dezimal (14,2) | Ja | wird durch Zahlungen reduziert (ANF-020) |
| Buchung (`buchung_id`) | Verweis | bei gebucht | ANF-017 |
| Erstellt / Geändert am | Zeitstempel | Ja | automatisch |

Tabelle `debitorenposition`: wie `einkaufsposition` (ANF-004) ohne gelieferte Menge, zusätzlich
`konto_id` (Ertragskonto, Pflicht) und `mwst_code_id` (Pflicht) statt freiem MWST-Satz.

## 5. Abhängigkeiten

- Hängt ab von: ANF-002, ANF-003, ANF-025, ANF-011, ANF-012, ANF-014, ANF-015, ANF-016, ANF-017
- Wird benötigt von: ANF-020, ANF-021, ANF-022, ANF-024

## 6. Nicht im Umfang

- QR-Rechnung mit Zahlteil und PDF-Erzeugung auf dem Server (wird benötigt, eigene spätere Anforderung)
- Versand per E-Mail
- Angebote, Auftragsbestätigungen, Lieferscheine (Verkaufsmodul)
- Automatische Rechnungen aus Projektaufwand (Stunden) oder Abonnements
- Artikelstamm (Positionen sind Freitext)

## 7. Offene Fragen

- [x] Nummernformat? **Geklärt 2026-11-27:** `R-JJJJ-NNNN` und `G-JJJJ-NNNN`, Jahr des Rechnungsdatums.
- [ ] Wann wird die QR-Rechnung umgesetzt? *(Geklärt 2026-11-18: wird benötigt; Zeitpunkt vorerst offen)*
- [x] Absenderangaben – woher? **Geklärt 2026-11-18:** aus der Firmenverwaltung (ANF-025), die mit einer
  Adresse verknüpft ist.
- [x] Sollen Teilrechnungen / Akontorechnungen für Projekte unterstützt werden? **Geklärt 2026-11-27:** später,
  nicht in dieser Anforderung.

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261204100000_debitorenrechnung.sql` | Tabellen `debitorenrechnung`, `debitorenposition`, `debitorenrechnung_nummer`; Systemkonto-Rolle `ertrag`; Trigger sperren gebuchte Belege und ihre Positionen (AK-10, AK-12) |
| `src/backend/finance/debtors/model.rs` | Datentypen, Prüfregeln, Summen je MWST-Code (AK-6), Zeilen der Buchung (AK-8), Modultests |
| `src/backend/finance/debtors/repository.rs`, `server_fns.rs` | Liste, Beleg, Positionen, Buchen, Storno, Gutschrift, Druckdaten, Verknüpfungen |
| `src/backend/finance/date.rs` | `add_days` und `days_between` für Fälligkeit und Skontofrist (ANF-016 AK-6) |
| `src/backend/finance/journal/server_fns.rs` | `write_reversal`: Gegenbuchung, gemeinsam für Journal-Storno und Beleg-Storno |
| `src/components/finance/debtors/` | Liste, Detailansicht, Modal-Fenster, Druckansicht (`print.rs`), Abschnitte für Projekt und Adresse (`links.rs`) |
| `src/components/finance/tabs.rs`, `src/app.rs` | Reiter „Debitoren“ und Routen `/finanzen/debitoren`, `/finanzen/debitoren/<Kennung>`, `/finanzen/debitoren/<Kennung>/druck` |

Festlegungen bei der Umsetzung:
- **Freigabe (2026-11-27):** Offene Fragen nach Vorschlag entschieden: Nummern `R-JJJJ-NNNN` und `G-JJJJ-NNNN`
  (Jahr des Rechnungsdatums, Zähler je Art und Jahr), Teilrechnungen später.
- **Beträge:** Gutschriften werden mit positiven Beträgen gespeichert; die Art bestimmt die Richtung der Buchung.
  In Listen erscheinen Gutschriften mit Minus.
- **AK-2:** Der Standardfilter „Offen und teilweise bezahlt“ zeigt keine Entwürfe; der Filter „Entwurf“ oder „Alle“
  zeigt sie. Der Zeitraum filtert nach Rechnungsdatum.
- **AK-4:** Die Zahlungsbedingung steht auf „Vorgabe“; beim Speichern gilt die Bedingung der Adresse, sonst die
  Standard-Bedingung. Eine andere aktive Bedingung kann gewählt werden.
- **AK-5:** Das Ertragskonto wird aus dem neuen Systemkonto „Ertrag (Vorgabe Debitoren)“ vorgeschlagen, sonst aus
  Konto 3400. Gewählt werden nur aktive Ertragskonten. Der MWST-Code ist Pflicht, solange am Rechnungsdatum die
  MWST-Pflicht gilt (ANF-012 AK-10); ohne MWST-Pflicht wird die Position „ohne MWST“ erfasst. Gewählt werden nur
  aktive Umsatzsteuer-Codes, die am Rechnungsdatum gelten.
- **AK-6:** Die MWST wird je Code berechnet (Basis × Satz), nicht je Position. Gerundet auf 0.05 wird nur in CHF.
- **AK-7:** Wählt der Benutzer zuerst das Projekt, wird dessen Kunde vorgeschlagen; ohne Kunde übernimmt auch der
  Server beim Speichern den Kunden des Projekts.
- **AK-8:** Die Buchung schreibt Debitoren gegen Ertrag je Konto und Code (mit MWST-Basis) und die Steuer je Code
  auf das MWST-Konto des Codes; die Rundungsdifferenz geht auf das Systemkonto „Rundungsdifferenzen“. Ertragszeilen
  tragen das Projekt. In Fremdwährung wird jede Zeile zum Kurs (neuester Kurs ≤ Rechnungsdatum) umgerechnet; die
  Debitorenzeile ist die Summe, damit Soll und Haben gleich sind.
- **AK-9:** Das Buchen braucht zusätzlich die eigene Firma (ANF-025 AK-10), einen Kurs bei Fremdwährung und einen
  Gesamtbetrag über null.
- **AK-13:** Die Gegenbuchung erhält das Rechnungsdatum, wenn dessen Periode offen ist, sonst das heutige Datum.
  „Ohne Zahlungen“ heisst: offener Betrag = Gesamtbetrag (Zahlungen folgen mit ANF-020).
- **AK-14:** Die Gutschrift übernimmt Kunde, Projekt, Währung, Zahlungsbedingung und Referenz; das Datum ist heute.
  Verrechnet wird sie seit ANF-020 über „Verrechnen“ (Abschnitt „Zahlungen“ der Gutschrift).
- **Geprüft (2026-11-27):** 18 Modultests; Migrationen auf leerer Datenbank; Ablauf über den laufenden Server gegen eine
  lokale Testdatenbank (Registrierung, Mandant, Kontenvorlage): Anlegen nur mit Projekt (Kunde übernommen), Position,
  Buchen mit Journalzeilen 1100/3400/2200/6990, gesperrte Periode, EUR ohne und mit Kurs, Gutschrift, Storno,
  Löschsperren, Druckdaten, alle Seiten auf dem Server dargestellt. Die Bedienung im Browser ist noch nicht von Hand
  geprüft (Szenarien TS-018-01 bis TS-018-20).
- **AK-11:** Die Druckansicht hat kein Grundgerüst und ist auf A4-Breite begrenzt (Ausnahme von der ganzen
  Bildschirmbreite); `@page` in `styles/app.css` setzt A4. Die MWST-Nummer erscheint nur, wenn am Rechnungsdatum die
  MWST-Pflicht gilt.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | Absender aus Firmenverwaltung (ANF-025); QR-Rechnung benötigt, Zeitpunkt offen |
| 2026-11-27 | Hatem Ben Belgacem | Freigegeben; Nummernformat und Teilrechnungen nach Vorschlag entschieden |
| 2026-11-27 | Claude Code | Umgesetzt bis auf das Verrechnen von Gutschriften (AK-14, mit ANF-020); Browser-Prüfung offen |
| 2026-11-27 | Claude Code | AK-14 vollständig (Verrechnen mit ANF-020); Abschnitt „Zahlungen“ in der Detailansicht |
