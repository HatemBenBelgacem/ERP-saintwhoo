# ANF-020: Zahlungen und offene Posten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-020                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

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
- [ ] **AK-1:** Gegeben der Reiter „Zahlungen“, wenn der Benutzer „Zahlungseingang“ wählt, dann öffnet
  sich ein Modal-Fenster (`?neu=eingang`) mit Datum (Vorgabe heute), Bankkonto (Vorgabe Standard, ANF-015),
  Betrag, Währung und Kunde (Suche) bzw. Rechnungsnummer.
- [ ] **AK-2:** Gegeben ein Kunde, dann listet das Fenster seine offenen Rechnungen und Gutschriften mit
  offenem Betrag; der Benutzer verteilt den Betrag auf die Rechnungen (Vorschlag: älteste Fälligkeit
  zuerst). Gegeben eine Rechnungsnummer, dann wird die Rechnung direkt vorgewählt.
- [ ] **AK-3:** Gegeben eine Zahlung innerhalb der Skontofrist mit Fehlbetrag bis zum Skontobetrag, dann
  schlägt das System den Skontoabzug vor; der Benutzer kann ihn bestätigen oder als Teilzahlung belassen.
- [ ] **AK-4:** Gegeben eine Restdifferenz (z. B. Bankspesen, Kleinbetrag), dann kann der Benutzer sie als
  „Differenz ausbuchen“ auf ein wählbares Konto buchen und die Rechnung damit ausgleichen.
- [ ] **AK-5:** Gegeben eine Überzahlung, dann bleibt der Mehrbetrag als **Vorauszahlung** des Kunden
  offen und kann später mit einer Rechnung verrechnet werden.

**Zahlungsausgang**
- [ ] **AK-6:** Gegeben „Zahlungsausgang“ (`?neu=ausgang`), dann gelten AK-1 bis AK-5 sinngemäss für
  Lieferanten und Kreditorenrechnungen.
- [ ] **AK-7:** Gegeben die Liste der offenen Kreditoren, dann kann der Benutzer mehrere Rechnungen
  markieren und mit „Als bezahlt erfassen“ in einem Schritt je Rechnung einen Zahlungsausgang mit Datum und
  Bankkonto erzeugen (mit Skontoabzug, wenn die Frist am Zahlungsdatum noch läuft).

**Buchung und Status**
- [ ] **AK-8:** Gegeben eine gespeicherte Zahlung, dann wird die Buchung erzeugt (Herkunft `zahlung`), der
  offene Betrag jeder ausgeglichenen Rechnung reduziert und deren Status gesetzt: offener Betrag 0 →
  „Bezahlt“, sonst „Teilweise bezahlt“. Alles geschieht in einer Transaktion.
- [ ] **AK-9:** Gegeben eine Zahlung in Fremdwährung, dann wird sie zum Kurs des Zahlungsdatums
  umgerechnet und die Differenz zum Buchungskurs der Rechnung auf das Konto Kursdifferenzen gebucht.
- [ ] **AK-10:** Gegeben eine Zahlung, wenn der Benutzer „Stornieren“ bestätigt, dann wird die Buchung
  storniert und die offenen Beträge und Status der Rechnungen werden zurückgesetzt.
- [ ] **AK-11:** Gegeben eine gebuchte Gutschrift und eine offene Rechnung desselben Partners, dann kann
  der Benutzer beide mit „Verrechnen“ ausgleichen (Ausgleich ohne Bankbewegung, ohne neue Buchung).

**Listen**
- [ ] **AK-12:** Gegeben die Zahlungsliste, dann zeigt sie Datum, Art (Eingang/Ausgang), Partner,
  Bankkonto, Betrag, ausgeglichene Rechnungen und Status; Suche, Filter (Art, Zeitraum, Bankkonto) und
  Sortierung stehen in der Webadresse.
- [ ] **AK-13:** Gegeben die Ansicht „Offene Posten“ (`/finanzen/zahlungen?ansicht=offene-posten`), dann
  zeigt sie getrennt Debitoren und Kreditoren je Partner mit Summe und Altersgliederung
  (nicht fällig, 1–30, 31–60, 61–90, über 90 Tage überfällig) zu einem wählbaren Stichtag.
- [ ] **AK-14:** Die Tabellen werden per sqlx-Migration angelegt.

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

- [ ] Wird der Skonto mit MWST-Korrektur gebucht (effektive Methode)? *(Vorschlag: ja, anteilig nach MWST-Codes der Rechnung)*
- [ ] Bis zu welchem Betrag dürfen Differenzen ohne Rückfrage ausgebucht werden? *(Vorschlag: keine Grenze, immer manuell bestätigt)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
