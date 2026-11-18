# ANF-024: Mahnwesen

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-024                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Kann                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Überfällige Debitorenrechnungen (ANF-018) werden **gemahnt**. Die **Mahnstufen** (Stammdaten) werden unter
`/einstellungen/finanzen/mahnstufen` gepflegt, z. B.:

| Stufe | Bezeichnung | Tage nach Fälligkeit | Gebühr |
|-------|-------------|----------------------|--------|
| 1 | Zahlungserinnerung | 10 | 0.00 |
| 2 | 1. Mahnung | 30 | 20.00 |
| 3 | 2. Mahnung | 45 | 40.00 |

Der **Mahnlauf** im Reiter „Mahnungen“ (`/finanzen/mahnungen`) schlägt alle fälligen Rechnungen mit der
nächsten Stufe vor; der Benutzer wählt aus und erzeugt die Mahnungen (Druckansicht). Mahngebühren werden
als Debitorenrechnung erfasst.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **überfällige Kunden stufenweise mahnen**, damit **offene Forderungen
> rechtzeitig bezahlt werden**.

## 3. Akzeptanzkriterien

**Mahnstufen (Einstellungen)**
- [ ] **AK-1:** Gegeben die Einstellungsseite „Mahnstufen“, dann können Stufen angelegt, bearbeitet und
  gelöscht werden (Modal-Fenster). Stufe (1–5, eindeutig), Bezeichnung, Tage nach Fälligkeit (aufsteigend
  je Stufe), Gebühr (≥ 0) und Mahntext sind Pflicht. Eine verwendete Stufe kann nicht gelöscht werden.

**Mahnlauf**
- [ ] **AK-2:** Gegeben der Reiter „Mahnungen“, wenn der Benutzer „Mahnlauf vorbereiten“ mit Stichtag
  (Vorgabe heute) wählt, dann listet das System alle offenen Debitorenrechnungen, deren Fälligkeit plus
  Tage der nächsten Stufe erreicht ist, mit Kunde, Rechnung, offenem Betrag, Tagen überfällig, aktueller
  und vorgeschlagener Stufe.
- [ ] **AK-3:** Gegeben die Vorschlagsliste, dann kann der Benutzer Rechnungen abwählen und je Kunde eine
  **Mahnsperre** mit Grund und Enddatum setzen; gesperrte Kunden erscheinen nicht im Vorschlag.
- [ ] **AK-4:** Gegeben die bestätigte Auswahl, dann wird je Kunde eine Mahnung mit allen gewählten
  Rechnungen erzeugt; die Mahnstufe und das Mahndatum werden an jeder Rechnung vermerkt.
- [ ] **AK-5:** Gegeben eine Mahnung mit Gebühr, dann wird automatisch eine Debitorenrechnung über die
  Gebühr (Ertragskonto Mahngebühren, ohne MWST) erzeugt und gebucht.
- [ ] **AK-6:** Gegeben eine Mahnung, dann gibt es eine Druckansicht mit Absender (Firma, ANF-025), Kundenadresse, Mahntext der
  Stufe, Liste der Rechnungen mit Fälligkeit und offenem Betrag, Gebühr und Gesamtbetrag.
- [ ] **AK-7:** Gegeben eine Rechnung, dann zeigt ihre Detailansicht den Mahnverlauf (Stufe, Datum).
  Eine Zahlung zwischen Vorschlag und Erzeugung entfernt die Rechnung aus dem Lauf.
- [ ] **AK-8:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `mahnstufe`: Kennung, Stufe, Bezeichnung, Tage nach Fälligkeit, Gebühr, Mahntext.

Tabelle `mahnung`: Kennung, Kunde (`adresse_id`), Datum, Stufe (`mahnstufe_id`), Gebühr,
Gebührenrechnung (`debitorenrechnung_id`, optional), Erstellt von / am.

Tabelle `mahnung_rechnung`: Mahnung (`mahnung_id`), Debitorenrechnung (`debitorenrechnung_id`),
offener Betrag zum Mahndatum.

Erweiterung `debitorenrechnung`: aktuelle Mahnstufe, letztes Mahndatum.
Erweiterung `adresse`: Mahnsperre bis, Grund.

## 5. Abhängigkeiten

- Hängt ab von: ANF-016, ANF-018, ANF-020
- Wird benötigt von: –

## 6. Nicht im Umfang

- Verzugszinsen
- Versand per E-Mail, Betreibung (Übergabe ans Betreibungsamt bzw. Inkasso)
- Mahnungen an Lieferanten oder für Kreditoren

## 7. Offene Fragen

- [ ] Welche Stufen und Gebühren gelten? *(Vorschlag: Tabelle oben)*
- [ ] Sollen Mahngebühren als eigene Rechnung oder als Zusatz zur Mahnung geführt werden? *(Vorschlag: eigene Rechnung, damit sie in den offenen Posten erscheint)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | Absender aus Firmenverwaltung (ANF-025) |
