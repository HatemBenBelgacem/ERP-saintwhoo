# ANF-019: Kreditoren – Eingangsrechnungen

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-019                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-27                                        |

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
- [x] **AK-1:** Gegeben der Reiter „Kreditoren“, dann erscheint eine Tabelle mit interner Nummer,
  Rechnungsnummer des Lieferanten, Lieferant, Einkaufsauftrag, Projekt, Rechnungsdatum, Fälligkeit,
  Skonto bis, Gesamtbetrag, offener Betrag, Währung und Status.
- [x] **AK-2:** Gegeben die Liste, dann kann gesucht, gefiltert (Status, Zeitraum, fällig bis) und
  sortiert werden. Standard: Status Offen und Teilweise bezahlt, nach Fälligkeit. Zustand in der
  Webadresse; ab 50 Einträgen seitenweise.
- [x] **AK-3:** Gegeben eine offene Rechnung mit überschrittener Fälligkeit, dann ist sie als überfällig
  markiert; eine Rechnung mit laufender Skontofrist zeigt das Skontodatum hervorgehoben (`text-secondary`).

**Erfassen**
- [x] **AK-4:** Gegeben „Neue Eingangsrechnung“, dann öffnet sich ein Modal-Fenster (`?neu=rechnung`).
  Lieferant (Adressstamm), Rechnungsnummer des Lieferanten und Rechnungsdatum sind Pflicht; Einkaufsauftrag,
  Projekt, Währung, Zahlungsbedingung (Vorgabe der Adresse bzw. Standard), Fälligkeit (überschreibbar) und
  Bemerkung sind wählbar.
- [x] **AK-5:** Gegeben Lieferant und Rechnungsnummer, die bereits in einer nicht stornierten Rechnung
  vorkommen, dann wird das Speichern mit dem Hinweis „Rechnung bereits erfasst“ verhindert.
- [x] **AK-6:** Gegeben ein Entwurf, dann lassen sich Positionen mit Bezeichnung, Aufwandskonto, MWST-Code,
  Nettobetrag und optional Projekt erfassen (Schnellerfassung: eine Zeile pro Konto/MWST-Code genügt).
  Der Benutzer kann alternativ den Bruttobetrag eingeben; Netto und MWST werden dann berechnet.
- [x] **AK-7:** Gegeben der Gesamtbetrag laut Beleg (Pflichtfeld „Betrag gemäss Rechnung“), dann zeigt das
  Formular die Differenz zum berechneten Total; Buchen ist nur bei Differenz 0.00 möglich. Rundungsdifferenzen
  bis 0.05 kann der Benutzer mit „Rundung ausgleichen“ auf das Systemkonto Rundungsdifferenzen buchen.

**Aus Einkaufsauftrag übernehmen**
- [x] **AK-8:** Gegeben ein Einkaufsauftrag im Status Bestellt, Teilweise geliefert oder Geliefert, wenn
  der Benutzer in dessen Detailansicht „Eingangsrechnung erfassen“ wählt, dann wird ein Entwurf mit
  Lieferant, Projekt, Währung und den Positionen (gelieferte Menge × Einzelpreis abzüglich Rabatt, MWST-Code
  nach Satz) angelegt.
- [x] **AK-9:** Gegeben ein Einkaufsauftrag, dann zeigt seine Detailansicht die zugeordneten
  Eingangsrechnungen und den Vergleich „bestellt / verrechnet“; übersteigt der verrechnete Betrag den
  bestellten, erscheint ein Hinweis (Speichern bleibt erlaubt).

**Buchen**
- [x] **AK-10:** Gegeben ein vollständiger Entwurf, wenn der Benutzer „Buchen“ klickt, dann erhält die
  Rechnung eine interne Nummer (z. B. `K-2026-0001`), die Buchung wird erzeugt (Herkunft `kreditor`,
  Fremdwährung zum Kurs des Rechnungsdatums) und der Status wechselt auf „Offen“.
- [x] **AK-11:** Gegeben eine gesperrte Periode, fehlende Systemkonten oder ungültige Codes, dann wird das
  Buchen mit Meldung verhindert und nichts gespeichert.
- [x] **AK-12:** Gegeben eine gebuchte Rechnung, dann sind Lieferant, Positionen und Beträge gesperrt; nur
  Fälligkeit und Bemerkung bleiben änderbar.

**Stornieren und Löschen**
- [x] **AK-13:** Gegeben ein Entwurf, dann kann er nach Bestätigung gelöscht werden.
- [x] **AK-14:** Gegeben eine offene Rechnung ohne Zahlungen, dann kann sie storniert werden
  (Gegenbuchung, Status „Storniert“). Mit Zahlungen ist Stornieren nicht möglich.

**Verknüpfungen**
- [x] **AK-15:** Gegeben ein Projekt bzw. eine Adresse, dann zeigt deren Detailansicht die zugeordneten
  Eingangsrechnungen bzw. offenen Kreditoren. Adressen, Projekte und Einkaufsaufträge mit Eingangsrechnungen
  können nicht gelöscht werden.

**Allgemein**
- [x] **AK-16:** Gegeben eine nicht existierende Kennung, dann erscheint „Rechnung nicht gefunden“.
- [x] **AK-17:** Die Tabellen werden per sqlx-Migration angelegt.

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

- [x] Wird ein Freigabe-Schritt (Visum) vor dem Buchen benötigt? **Geklärt 2026-11-27:** nein, spätere Anforderung.
- [x] Sollen Bankverbindungen (IBAN) der Lieferanten an der Adresse gespeichert werden? **Geklärt 2026-11-27:** ja,
  aber zusammen mit dem späteren Zahlungsexport, nicht in dieser Anforderung.
- [x] Soll beim Erfassen ein Bild/PDF der Rechnung angehängt werden können? **Geklärt 2026-11-27:** später.

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261205100000_kreditorenrechnung.sql` | Tabellen `kreditorenrechnung`, `kreditorenposition`, `kreditorenrechnung_nummer`; eindeutige Rechnungsnummer je Lieferant (AK-5); Systemkonto-Rolle `aufwand`; Trigger sperren gebuchte Belege (AK-12, AK-13) |
| `src/backend/finance/creditors/model.rs` | Datentypen, Prüfregeln, Netto/Brutto-Aufteilung, Summen mit Kontrollbetrag und Rundung (AK-6, AK-7), Zeilen der Buchung (AK-10), Modultests |
| `src/backend/finance/creditors/repository.rs`, `server_fns.rs` | Liste, Beleg, Positionen, Übernahme aus dem Einkaufsauftrag, Rundung ausgleichen, Buchen, Storno, Verknüpfungen |
| `src/components/finance/creditors/` | Liste, Detailansicht, Modal-Fenster, Abschnitte in Projekt, Adresse und Einkaufsauftrag (`links.rs`) |
| `src/components/purchase_orders/detail.rs` | Abschnitt „Eingangsrechnungen“ mit „bestellt / verrechnet“ und „Eingangsrechnung erfassen“ (AK-8, AK-9) |

Festlegungen bei der Umsetzung:
- **Freigabe (2026-11-27):** Offene Fragen nach Vorschlag: kein Visum, Lieferanten-IBAN mit dem späteren Zahlungsexport,
  Belegablage später.
- **AK-1, AK-10:** Interne Nummer `K-JJJJ-NNNN` (Jahr des Rechnungsdatums); Rechnungen und Gutschriften teilen den
  Zähler, die Art steht als Plakette daneben.
- **AK-4:** Fälligkeit leer = beim Buchen aus der Zahlungsbedingung; eine eingegebene Fälligkeit hat Vorrang.
  Ohne Lieferant, aber mit Einkaufsauftrag übernimmt der Server den Lieferanten des Auftrags; ein Auftrag eines
  anderen Lieferanten wird abgelehnt. Wählbar sind nur bestellte oder gelieferte Aufträge.
- **AK-5:** Der Vergleich der Rechnungsnummer ignoriert Gross- und Kleinschreibung; ein eindeutiger Index sichert
  ihn auch bei gleichzeitigem Speichern.
- **AK-6:** Die MWST wird je Position aus Netto bzw. Brutto berechnet und gespeichert; die Summe je Code ist die
  Summe der Positionen. Nur aktive Aufwandskonten und Vorsteuer-Codes am Rechnungsdatum; ohne MWST-Pflicht „ohne
  MWST“. Eine Position ohne Projekt erbt in der Buchung das Projekt der Rechnung.
- **AK-7:** Gerundet wird nicht automatisch. „Rundung ausgleichen“ erscheint bei einer Differenz bis 0.05 und
  bucht sie auf „Rundungsdifferenzen“. Die Rechnungsnummer des Lieferanten ist zum Buchen Pflicht.
- **AK-8:** Übernommen werden Positionen mit gelieferter Menge > 0 (Betrag = gelieferte Menge × Preis abzüglich
  Rabatt). Aufwandskonto aus dem neuen Systemkonto „Aufwand (Vorgabe Kreditoren)“, sonst 4000. MWST-Code der
  Einkaufsposition, sonst ein Vorsteuer-Code mit gleichem Satz. Danach öffnet sich das Fenster zum Bearbeiten,
  damit Rechnungsnummer und Betrag gemäss Rechnung erfasst werden.
- **AK-9:** Verrechnet = Summe der nicht stornierten Eingangsrechnungen (inkl. Entwürfe), Gutschriften abgezogen;
  bestellt = Gesamtbetrag des Auftrags.
- **AK-12:** Nach dem Buchen sind nur Fälligkeit und Bemerkung änderbar; die Zahlungsreferenz ist mitgesperrt.
- **AK-15:** Die Meldung beim Löschen einer Adresse nennt den ersten blockierenden Beleg (wie bisher bei Adressen).
- **Geprüft (2026-11-27):** 11 Modultests; Ablauf über den laufenden Server gegen eine lokale Testdatenbank:
  Übernahme aus einem gelieferten Auftrag (Aufwand 900/100, Vorsteuer 81.00), Kontrollbetrag mit Differenz 0.05,
  Rundung ausgleichen, Buchen (Soll 4000/1170/6990, Haben 2000), Doppelerfassung, fremder Auftrag, Gutschrift mit
  Bruttobetrag, Änderungssperre, Storno und erneutes Erfassen der Nummer, Löschsperren, alle Seiten auf dem Server
  dargestellt. Die Bedienung im Browser ist noch nicht von Hand geprüft (TS-019-01 bis TS-019-18).

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-27 | Hatem Ben Belgacem | Freigegeben; offene Fragen nach Vorschlag entschieden |
| 2026-11-27 | Claude Code | Umgesetzt; Browser-Prüfung offen |
