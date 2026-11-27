# ANF-020: Zahlungen und offene Posten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-020                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-27                                        |

## 1. Beschreibung

**Zahlungen** gleichen **offene Posten** aus: Zahlungseingänge reduzieren offene Debitorenrechnungen
(ANF-018), Zahlungsausgänge offene Kreditorenrechnungen (ANF-019). Eine Zahlung kann eine oder mehrere
Rechnungen desselben Partners ganz oder teilweise ausgleichen. Jede Zahlung erzeugt eine Buchung (ANF-017):

```
Zahlungseingang:  Soll Bank            / Haben Debitoren
                  Soll Skonto gewährt  / Haben Debitoren        (bei Skontoabzug, inkl. MWST-Korrektur)
Zahlungsausgang:  Soll Kreditoren      / Haben Bank
                  Soll Kreditoren      / Haben Skonto erhalten  (bei Skontoabzug, inkl. Vorsteuer-Korrektur)
Fremdwährung:     Differenz zwischen Buchungs- und Zahlungskurs → Konto Kursdifferenzen
```

Der Reiter „Zahlungen“ (`/finanzen/zahlungen`) zeigt die Zahlungen und die **Liste der offenen Posten**
(Debitoren und Kreditoren mit Altersgliederung).

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Zahlungseingänge den Kundenrechnungen zuordnen und Lieferantenrechnungen
> als bezahlt erfassen**, damit **die offenen Posten immer stimmen und ich weiss, wer uns was schuldet**.

## 3. Akzeptanzkriterien

**Zahlungseingang**
- [x] **AK-1:** Gegeben der Reiter „Zahlungen“, wenn der Benutzer „Zahlungseingang“ wählt, dann öffnet
  sich ein Modal-Fenster (`?neu=eingang`) mit Datum (Vorgabe heute), Bankkonto (Vorgabe Standard, ANF-015),
  Betrag, Währung und Kunde (Suche) bzw. Rechnungsnummer.
- [x] **AK-2:** Gegeben ein Kunde, dann listet das Fenster seine offenen Rechnungen und Gutschriften mit
  offenem Betrag; der Benutzer verteilt den Betrag auf die Rechnungen (Vorschlag: älteste Fälligkeit
  zuerst). Gegeben eine Rechnungsnummer, dann wird die Rechnung direkt vorgewählt.
- [x] **AK-3:** Gegeben eine Zahlung innerhalb der Skontofrist mit Fehlbetrag bis zum Skontobetrag, dann
  schlägt das System den Skontoabzug vor; der Benutzer kann ihn bestätigen oder als Teilzahlung belassen.
- [x] **AK-4:** Gegeben eine Restdifferenz (z. B. Bankspesen, Kleinbetrag), dann kann der Benutzer sie als
  „Differenz ausbuchen“ auf ein wählbares Konto buchen und die Rechnung damit ausgleichen.
- [x] **AK-5:** Gegeben eine Überzahlung, dann bleibt der Mehrbetrag als **Vorauszahlung** des Kunden
  offen und kann später mit einer Rechnung verrechnet werden.

**Zahlungsausgang**
- [x] **AK-6:** Gegeben „Zahlungsausgang“ (`?neu=ausgang`), dann gelten AK-1 bis AK-5 sinngemäss für
  Lieferanten und Kreditorenrechnungen.
- [x] **AK-7:** Gegeben die Liste der offenen Kreditoren, dann kann der Benutzer mehrere Rechnungen
  markieren und mit „Als bezahlt erfassen“ in einem Schritt je Rechnung einen Zahlungsausgang mit Datum und
  Bankkonto erzeugen (mit Skontoabzug, wenn die Frist am Zahlungsdatum noch läuft).

**Buchung und Status**
- [x] **AK-8:** Gegeben eine gespeicherte Zahlung, dann wird die Buchung erzeugt (Herkunft `zahlung`), der
  offene Betrag jeder ausgeglichenen Rechnung reduziert und deren Status gesetzt: offener Betrag 0 →
  „Bezahlt“, sonst „Teilweise bezahlt“. Alles geschieht in einer Transaktion.
- [x] **AK-9:** Gegeben eine Zahlung in Fremdwährung, dann wird sie zum Kurs des Zahlungsdatums
  umgerechnet und die Differenz zum Buchungskurs der Rechnung auf das Konto Kursdifferenzen gebucht.
- [x] **AK-10:** Gegeben eine Zahlung, wenn der Benutzer „Stornieren“ bestätigt, dann wird die Buchung
  storniert und die offenen Beträge und Status der Rechnungen werden zurückgesetzt.
- [x] **AK-11:** Gegeben eine gebuchte Gutschrift und eine offene Rechnung desselben Partners, dann kann
  der Benutzer beide mit „Verrechnen“ ausgleichen (Ausgleich ohne Bankbewegung, ohne neue Buchung).

**Listen**
- [x] **AK-12:** Gegeben die Zahlungsliste, dann zeigt sie Datum, Art (Eingang/Ausgang), Partner,
  Bankkonto, Betrag, ausgeglichene Rechnungen und Status; Suche, Filter (Art, Zeitraum, Bankkonto) und
  Sortierung stehen in der Webadresse.
- [x] **AK-13:** Gegeben die Ansicht „Offene Posten“ (`/finanzen/zahlungen?ansicht=offene-posten`), dann
  zeigt sie getrennt Debitoren und Kreditoren je Partner mit Summe und Altersgliederung
  (nicht fällig, 1–30, 31–60, 61–90, über 90 Tage überfällig) zu einem wählbaren Stichtag.
- [x] **AK-14:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `zahlung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Art | Auswahl | Ja | `eingang`, `ausgang`, `verrechnung` |
| Datum | Datum | Ja | |
| Partner (`adresse_id`) | Verweis | Ja | |
| Bankkonto (`bankkonto_id`) | Verweis | bei Eingang/Ausgang | ANF-015 |
| Betrag / Währung / Kurs | Dezimal / Text (3) / Dezimal | Ja | |
| Betrag CHF | Dezimal (14,2) | Ja | |
| Vorauszahlung (nicht zugeordnet) | Dezimal (14,2) | Ja | Standard 0 |
| Status | Auswahl | Ja | `gebucht`, `storniert` |
| Text | Text (200) | Nein | |
| Buchung (`buchung_id`) | Verweis | Nein | leer bei reiner Verrechnung |
| Erfasst von (`benutzer_id`) / Erstellt am | Verweis / Zeitstempel | Ja | |

Tabelle `zahlungsausgleich`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Zahlung (`zahlung_id`) | Verweis | Ja | |
| Debitorenrechnung (`debitorenrechnung_id`) | Verweis | Nein | genau eines der beiden |
| Kreditorenrechnung (`kreditorenrechnung_id`) | Verweis | Nein | |
| Betrag | Dezimal (14,2) | Ja | in Rechnungswährung |
| Skonto | Dezimal (14,2) | Ja | Standard 0 |
| Differenz ausgebucht / Konto | Dezimal / Verweis | Nein | AK-4 |

## 5. Abhängigkeiten

- Hängt ab von: ANF-014, ANF-015, ANF-016, ANF-017, ANF-018, ANF-019
- Wird benötigt von: ANF-010 AK-4, ANF-021, ANF-022 (vereinnahmt), ANF-024

## 6. Nicht im Umfang

- Import von Bankauszügen (camt.053/054) und automatische Zuordnung über QR-Referenz
- Export von Zahlungsaufträgen (pain.001) an die Bank
- Lastschriftverfahren (LSV / eBill)

## 7. Offene Fragen

- [x] Wird der Skonto mit MWST-Korrektur gebucht (effektive Methode)? **Geklärt 2026-11-27:** ja, anteilig nach den
  MWST-Codes der Rechnung.
- [x] Bis zu welchem Betrag dürfen Differenzen ohne Rückfrage ausgebucht werden? **Geklärt 2026-11-27:** keine Grenze,
  immer vom Benutzer bestätigt.

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261206100000_zahlung.sql` | Tabellen `zahlung` (mit `seite` und offener Vorauszahlung) und `zahlungsausgleich` (Rechnung, Gutschrift oder Vorauszahlung); Trigger: Zahlungen nur stornieren, nicht ändern oder löschen |
| `src/backend/finance/payments/model.rs` | Prüfregeln, Vorschlag zur Verteilung (AK-2, AK-3), Skonto nach MWST-Codes, Buchungszeilen mit Kursdifferenz (AK-8, AK-9), Altersgliederung (AK-13), Modultests |
| `src/backend/finance/payments/repository.rs`, `server_fns.rs` | Offene Posten, Zahlung bzw. Verrechnung speichern, „Als bezahlt erfassen“, Storno, Liste, Altersgliederung zum Stichtag |
| `src/components/finance/payments/` | Seite „Zahlungen“ mit Liste und „Offene Posten“, Zahlungsfenster, Detailansicht, Abschnitt „Zahlungen“ in den Rechnungen |
| `src/components/finance/creditors/list.rs` | Markieren offener Rechnungen und „Als bezahlt erfassen“ (AK-7, `?bezahlt=<Kennungen>`) |

Festlegungen bei der Umsetzung:
- **Freigabe (2026-11-27):** Skonto mit MWST-Korrektur anteilig nach den MWST-Codes der Rechnung; Differenzen ohne Grenze,
  immer vom Benutzer bestätigt.
- **Währung:** Alle Posten einer Zahlung haben die Währung der Zahlung. Die Debitoren bzw. Kreditoren werden zum
  Buchungskurs des Belegs entlastet; die Bank zum Kurs am Zahlungsdatum; die Differenz geht auf „Kursdifferenzen“.
- **AK-2:** Das Fenster lädt die offenen Posten des Partners in der gewählten Währung; „Betrag verteilen“ füllt die
  Rechnungen nach ältester Fälligkeit. Gutschriften werden nur mitgenommen, wenn der Benutzer einen Betrag einträgt;
  sie verringern dann den Zahlungsbetrag. Die Rechnungsnummer (bei Kreditoren auch die des Lieferanten) wählt den
  Beleg vor, ebenso der Knopf „Zahlung erfassen“ in der Rechnung (`?beleg=r:<Kennung>`).
- **AK-3:** Skonto = Prozent vom Gesamtbetrag, höchstens der offene Betrag. Der Vorschlag setzt ihn, wenn der Rest
  innerhalb der Frist nicht grösser als der Skonto ist; der Benutzer kann ihn leeren (Teilzahlung).
- **AK-4:** Die Differenz bucht auf ein frei wählbares aktives Konto (Eingang Soll, Ausgang Haben); nur Fehlbeträge.
- **AK-5, AK-11:** Eine Vorauszahlung bleibt an der Zahlung offen (Konto Debitoren bzw. Kreditoren) und wird über
  „Verrechnen“ mit einer Rechnung ausgeglichen. Eine Verrechnung gleicht Rechnungen mit Gutschriften bzw.
  Vorauszahlungen in gleicher Währung aus und erzeugt keine Buchung; Kursunterschiede zwischen den Belegen werden
  dabei nicht ausgeglichen.
- **AK-7:** Alle markierten Rechnungen werden in einer Transaktion bezahlt; scheitert eine, wird keine gebucht.
- **AK-10:** Eine Zahlung, deren Vorauszahlung schon verrechnet ist, lässt sich erst nach dem Storno der Verrechnung
  stornieren. Die Gegenbuchung erhält das Zahlungsdatum, wenn dessen Periode offen ist, sonst heute.
- **AK-13:** Offen am Stichtag = Gesamtbetrag minus Ausgleiche gültiger Zahlungen bis zum Stichtag, in CHF zum
  Buchungskurs; Gutschriften und Vorauszahlungen negativ. Stornierte Belege zählen nicht, auch wenn der Storno nach
  dem Stichtag liegt.
- **Geprüft (2026-11-27):** 11 Modultests; Ablauf über den laufenden Server gegen eine lokale Testdatenbank:
  Eingang mit Skonto (Bank 105.94, Debitoren 108.10, Skonto 2.00 mit Code, Umsatzsteuer 0.16), EUR-Eingang mit
  Überzahlung (Kursgewinn 21.62, Vorauszahlung 19 EUR), Teilzahlung mit Spesen-Differenz, Verrechnung Gutschrift und
  Vorauszahlung, Storno-Regeln, „Als bezahlt erfassen“ mit Skonto, Altersgliederung nachgerechnet, Löschsperre
  Bankkonto, alle Seiten auf dem Server dargestellt. Die Bedienung im Browser (Laden der Posten, Verteilen, Eingaben
  im Fenster) ist noch nicht von Hand geprüft (TS-020-01 bis TS-020-16).

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-27 | Hatem Ben Belgacem | Freigegeben; offene Fragen nach Vorschlag entschieden |
| 2026-11-27 | Claude Code | Umgesetzt; Browser-Prüfung offen |
