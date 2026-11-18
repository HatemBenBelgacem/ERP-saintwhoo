# ANF-018: Debitoren – Ausgangsrechnungen

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-018                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

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
- [ ] **AK-1:** Gegeben der Reiter „Debitoren“, dann erscheint eine Tabelle mit Nummer, Art (Rechnung/
  Gutschrift), Kunde, Projekt, Rechnungsdatum, Fälligkeit, Gesamtbetrag, offener Betrag, Währung und Status.
- [ ] **AK-2:** Gegeben die Liste, dann kann nach Nummer, Kunde, Projekt und Referenz gesucht, nach Status
  und Zeitraum gefiltert und sortiert werden. Standard: Status Offen und Teilweise bezahlt, nach Fälligkeit.
  Filter, Sortierung und Seite stehen in der Webadresse; ab 50 Einträgen seitenweise.
- [ ] **AK-3:** Gegeben eine offene Rechnung mit überschrittener Fälligkeit, dann ist sie als überfällig
  markiert (Signalfarbe `error`) und zeigt die Tage seit Fälligkeit.

**Anlegen und Bearbeiten (Entwurf)**
- [ ] **AK-4:** Gegeben „Neue Rechnung“ bzw. „Neue Gutschrift“, dann öffnet sich ein Modal-Fenster
  (`?neu=rechnung` bzw. `?neu=gutschrift`). Kunde (Suche im Adressstamm) und Rechnungsdatum sind Pflicht;
  Projekt, Referenz des Kunden, Währung (Vorgabe CHF), Zahlungsbedingung (Vorgabe der Adresse bzw.
  Standard), Einleitungs- und Schlusstext sind wählbar.
- [ ] **AK-5:** Gegeben eine Rechnung im Entwurf, dann lassen sich Positionen anlegen, bearbeiten,
  löschen und verschieben (`?neu=position`, `?bearbeiten=…`). Bezeichnung, Menge (> 0), Einheit,
  Einzelpreis (≥ 0), Ertragskonto und MWST-Code sind Pflicht; Rabatt 0–100 %. Das Ertragskonto wird aus
  einer Vorgabe (Systemkonto Ertrag, sonst 3400) vorgeschlagen.
- [ ] **AK-6:** Gegeben die Positionen, dann werden Positionsbetrag, Zwischensumme, MWST je Code und
  Gesamtbetrag wie in ANF-004 berechnet; der Gesamtbetrag in CHF wird auf 0.05 gerundet, die Differenz
  als Rundungszeile ausgewiesen.
- [ ] **AK-7:** Gegeben ein Projekt mit Kunde, wenn der Benutzer das Projekt zuerst wählt, dann wird der
  Kunde des Projekts vorgeschlagen.

**Buchen**
- [ ] **AK-8:** Gegeben ein Entwurf mit mindestens einer Position, wenn der Benutzer „Buchen“ klickt, dann
  erhält die Rechnung eine fortlaufende Nummer (z. B. `R-2026-0001`, Gutschrift `G-2026-0001`), Fälligkeit
  und Skontofrist werden berechnet (ANF-016), die Buchung wird im Journal erzeugt (Herkunft `debitor`) und
  der Status wechselt auf „Offen“. Bei Fremdwährung wird zum Kurs des Rechnungsdatums umgerechnet (ANF-014).
- [ ] **AK-9:** Gegeben eine gesperrte Periode, fehlende Systemkonten oder ein ungültiger MWST-Code, dann
  wird das Buchen mit Meldung verhindert und nichts gespeichert.
- [ ] **AK-10:** Gegeben eine gebuchte Rechnung, dann sind Kunde, Positionen und Beträge gesperrt; nur
  Referenz, Bemerkung und Texte für den Druck bleiben änderbar.

**Druck**
- [ ] **AK-11:** Gegeben eine Rechnung, wenn der Benutzer „Drucken“ wählt, dann öffnet sich eine
  Druckansicht (`/finanzen/debitoren/<kennung>/druck`) mit Absender (Firma und ihre Adresse, ANF-025),
  Kundenadresse, Rechnungsnummer, Datum,
  MWST-Nummer, Positionen, MWST-Ausweis je Satz, Gesamtbetrag, Zahlungsbedingung und Bankverbindung
  (Standard-Bankkonto, ANF-015). Sie ist für A4 gestaltet; ein Entwurf ist als „Entwurf“ gekennzeichnet.

**Stornieren, Löschen, Gutschrift**
- [ ] **AK-12:** Gegeben eine Rechnung im Entwurf, dann kann sie nach Bestätigung gelöscht werden.
- [ ] **AK-13:** Gegeben eine offene Rechnung ohne Zahlungen, wenn der Benutzer „Stornieren“ bestätigt,
  dann wird die Buchung storniert (Gegenbuchung) und der Status ist „Storniert“. Rechnungen mit Zahlungen
  können nicht storniert werden; stattdessen wird eine Gutschrift erstellt.
- [ ] **AK-14:** Gegeben eine gebuchte Rechnung, wenn der Benutzer „Gutschrift erstellen“ wählt, dann wird
  ein Gutschrift-Entwurf mit denselben Positionen und Verweis auf die Rechnung angelegt. Eine gebuchte
  Gutschrift kann mit der Rechnung verrechnet werden (ANF-020).

**Verknüpfungen**
- [ ] **AK-15:** Gegeben ein Projekt, dann zeigt dessen Detailansicht die zugeordneten Ausgangsrechnungen
  mit Nummer, Datum, Status, Gesamtbetrag und offenem Betrag.
- [ ] **AK-16:** Gegeben eine Adresse, dann zeigt ihre Detailansicht einen Abschnitt „Debitoren“ mit den
  offenen Rechnungen und dem offenen Gesamtbetrag. Adressen und Projekte mit Rechnungen können nicht
  gelöscht werden.

**Allgemein**
- [ ] **AK-17:** Gegeben eine nicht existierende Kennung, dann erscheint „Rechnung nicht gefunden“.
- [ ] **AK-18:** Die Tabellen werden per sqlx-Migration angelegt.

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

- [ ] Nummernformat? *(Vorschlag: `R-JJJJ-NNNN` und `G-JJJJ-NNNN`, wie Einkauf)*
- [ ] Wann wird die QR-Rechnung umgesetzt? *(Geklärt 2026-11-18: wird benötigt; Zeitpunkt vorerst offen)*
- [x] Absenderangaben – woher? **Geklärt 2026-11-18:** aus der Firmenverwaltung (ANF-025), die mit einer
  Adresse verknüpft ist.
- [ ] Sollen Teilrechnungen / Akontorechnungen für Projekte unterstützt werden? *(Vorschlag: später)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | Absender aus Firmenverwaltung (ANF-025); QR-Rechnung benötigt, Zeitpunkt offen |
