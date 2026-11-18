# ANF-019: Kreditoren – Eingangsrechnungen

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-019                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Das ERP erfasst **Eingangsrechnungen** von Lieferanten (Kreditoren). Eine Rechnung kann frei erfasst oder
aus einem **Einkaufsauftrag** (ANF-004) übernommen werden. Mit dem **Buchen** wird sie zum offenen Posten
und erzeugt automatisch eine Buchung (ANF-017):

```
Soll  Aufwandskonto der Position       Nettobetrag je Position
Soll  Vorsteuer (MWST-Konto Code)      MWST je Code
Haben Kreditoren (Systemkonto)         Gesamtbetrag
```

Gutschriften von Lieferanten werden als Kreditorenrechnung der Art „Gutschrift“ erfasst.

**Status:**
```
Entwurf → Offen → Teilweise bezahlt → Bezahlt
   │        │
   └────────┴──→ Storniert
```

Die Kreditoren liegen im Reiter „Kreditoren“ unter `/finanzen/kreditoren`.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Lieferantenrechnungen schnell erfassen, gegen den Einkaufsauftrag prüfen
> und buchen**, damit **Aufwand und Verbindlichkeiten stimmen und keine Rechnung doppelt bezahlt wird**.

## 3. Akzeptanzkriterien

**Liste (Anzeigen)**
- [ ] **AK-1:** Gegeben der Reiter „Kreditoren“, dann erscheint eine Tabelle mit interner Nummer,
  Rechnungsnummer des Lieferanten, Lieferant, Einkaufsauftrag, Projekt, Rechnungsdatum, Fälligkeit,
  Skonto bis, Gesamtbetrag, offener Betrag, Währung und Status.
- [ ] **AK-2:** Gegeben die Liste, dann kann gesucht, gefiltert (Status, Zeitraum, fällig bis) und
  sortiert werden. Standard: Status Offen und Teilweise bezahlt, nach Fälligkeit. Zustand in der
  Webadresse; ab 50 Einträgen seitenweise.
- [ ] **AK-3:** Gegeben eine offene Rechnung mit überschrittener Fälligkeit, dann ist sie als überfällig
  markiert; eine Rechnung mit laufender Skontofrist zeigt das Skontodatum hervorgehoben (`text-secondary`).

**Erfassen**
- [ ] **AK-4:** Gegeben „Neue Eingangsrechnung“, dann öffnet sich ein Modal-Fenster (`?neu=rechnung`).
  Lieferant (Adressstamm), Rechnungsnummer des Lieferanten und Rechnungsdatum sind Pflicht; Einkaufsauftrag,
  Projekt, Währung, Zahlungsbedingung (Vorgabe der Adresse bzw. Standard), Fälligkeit (überschreibbar) und
  Bemerkung sind wählbar.
- [ ] **AK-5:** Gegeben Lieferant und Rechnungsnummer, die bereits in einer nicht stornierten Rechnung
  vorkommen, dann wird das Speichern mit dem Hinweis „Rechnung bereits erfasst“ verhindert.
- [ ] **AK-6:** Gegeben ein Entwurf, dann lassen sich Positionen mit Bezeichnung, Aufwandskonto, MWST-Code,
  Nettobetrag und optional Projekt erfassen (Schnellerfassung: eine Zeile pro Konto/MWST-Code genügt).
  Der Benutzer kann alternativ den Bruttobetrag eingeben; Netto und MWST werden dann berechnet.
- [ ] **AK-7:** Gegeben der Gesamtbetrag laut Beleg (Pflichtfeld „Betrag gemäss Rechnung“), dann zeigt das
  Formular die Differenz zum berechneten Total; Buchen ist nur bei Differenz 0.00 möglich. Rundungsdifferenzen
  bis 0.05 kann der Benutzer mit „Rundung ausgleichen“ auf das Systemkonto Rundungsdifferenzen buchen.

**Aus Einkaufsauftrag übernehmen**
- [ ] **AK-8:** Gegeben ein Einkaufsauftrag im Status Bestellt, Teilweise geliefert oder Geliefert, wenn
  der Benutzer in dessen Detailansicht „Eingangsrechnung erfassen“ wählt, dann wird ein Entwurf mit
  Lieferant, Projekt, Währung und den Positionen (gelieferte Menge × Einzelpreis abzüglich Rabatt, MWST-Code
  nach Satz) angelegt.
- [ ] **AK-9:** Gegeben ein Einkaufsauftrag, dann zeigt seine Detailansicht die zugeordneten
  Eingangsrechnungen und den Vergleich „bestellt / verrechnet“; übersteigt der verrechnete Betrag den
  bestellten, erscheint ein Hinweis (Speichern bleibt erlaubt).

**Buchen**
- [ ] **AK-10:** Gegeben ein vollständiger Entwurf, wenn der Benutzer „Buchen“ klickt, dann erhält die
  Rechnung eine interne Nummer (z. B. `K-2026-0001`), die Buchung wird erzeugt (Herkunft `kreditor`,
  Fremdwährung zum Kurs des Rechnungsdatums) und der Status wechselt auf „Offen“.
- [ ] **AK-11:** Gegeben eine gesperrte Periode, fehlende Systemkonten oder ungültige Codes, dann wird das
  Buchen mit Meldung verhindert und nichts gespeichert.
- [ ] **AK-12:** Gegeben eine gebuchte Rechnung, dann sind Lieferant, Positionen und Beträge gesperrt; nur
  Fälligkeit und Bemerkung bleiben änderbar.

**Stornieren und Löschen**
- [ ] **AK-13:** Gegeben ein Entwurf, dann kann er nach Bestätigung gelöscht werden.
- [ ] **AK-14:** Gegeben eine offene Rechnung ohne Zahlungen, dann kann sie storniert werden
  (Gegenbuchung, Status „Storniert“). Mit Zahlungen ist Stornieren nicht möglich.

**Verknüpfungen**
- [ ] **AK-15:** Gegeben ein Projekt bzw. eine Adresse, dann zeigt deren Detailansicht die zugeordneten
  Eingangsrechnungen bzw. offenen Kreditoren. Adressen, Projekte und Einkaufsaufträge mit Eingangsrechnungen
  können nicht gelöscht werden.

**Allgemein**
- [ ] **AK-16:** Gegeben eine nicht existierende Kennung, dann erscheint „Rechnung nicht gefunden“.
- [ ] **AK-17:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `kreditorenrechnung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Nummer | Text (20) | bei gebucht | intern, fortlaufend je Jahr |
| Art | Auswahl | Ja | `rechnung`, `gutschrift` |
| Lieferant (`adresse_id`) | Verweis | Ja | ANF-002 |
| Rechnungsnummer Lieferant | Text (50) | Ja | eindeutig je Lieferant (ausser storniert) |
| Einkaufsauftrag (`einkaufsauftrag_id`) | Verweis | Nein | ANF-004 |
| Projekt (`projekt_id`) | Verweis | Nein | ANF-003 |
| Status | Auswahl | Ja | `entwurf`, `offen`, `teilweise_bezahlt`, `bezahlt`, `storniert` |
| Rechnungsdatum | Datum | Ja | |
| Zahlungsbedingung (`zahlungsbedingung_id`) | Verweis | Ja | |
| Fälligkeit / Skonto bis / Skonto % | Datum / Datum / Dezimal | bei gebucht | berechnet, Fälligkeit überschreibbar |
| Währung / Kurs | Text (3) / Dezimal (14,6) | Ja | |
| Betrag gemäss Rechnung | Dezimal (14,2) | Ja | zur Kontrolle |
| Gesamtbetrag / Gesamtbetrag CHF | Dezimal (14,2) | bei gebucht | |
| Offener Betrag | Dezimal (14,2) | Ja | |
| Zahlungsreferenz | Text (35) | Nein | QR- oder Kreditorreferenz für die Zahlung |
| Bemerkung | Langtext | Nein | |
| Buchung (`buchung_id`) | Verweis | bei gebucht | |
| Erstellt / Geändert am | Zeitstempel | Ja | |

Tabelle `kreditorenposition`: Reihenfolge, Bezeichnung, Aufwandskonto (`konto_id`), MWST-Code
(`mwst_code_id`), Nettobetrag, MWST-Betrag, Projekt (`projekt_id`, optional),
Einkaufsposition (`einkaufsposition_id`, optional).

## 5. Abhängigkeiten

- Hängt ab von: ANF-002, ANF-003, ANF-004, ANF-011, ANF-012, ANF-014, ANF-016, ANF-017
- Wird benötigt von: ANF-020, ANF-021, ANF-022

## 6. Nicht im Umfang

- Einlesen von QR-Rechnungen (Scanner, PDF-Erkennung) und E-Rechnungen
- Visum- bzw. Freigabe-Ablauf vor dem Buchen
- Bankverbindungen der Lieferanten (für Zahlungsdateien; siehe offene Fragen)
- Belegablage (Bild/PDF der Rechnung)

## 7. Offene Fragen

- [ ] Wird ein Freigabe-Schritt (Visum) vor dem Buchen benötigt? *(Vorschlag: nein, spätere Anforderung)*
- [ ] Sollen Bankverbindungen (IBAN) der Lieferanten an der Adresse gespeichert werden? *(Vorschlag: ja, zusammen mit späterem Zahlungsexport)*
- [ ] Soll beim Erfassen ein Bild/PDF der Rechnung angehängt werden können? *(Vorschlag: später)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
