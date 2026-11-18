# ANF-012: MWST-Codes (Steuersätze)

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-012                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Die **Mehrwertsteuer** wird über **MWST-Codes** gesteuert. Ein Code fasst Steuerart (Umsatzsteuer oder
Vorsteuer), Satz, Gültigkeit und die Ziffer im MWST-Formular der ESTV zusammen, z. B.:

| Code | Bezeichnung | Art | Satz |
|------|-------------|-----|------|
| `UN81` | Umsatzsteuer Normalsatz | Umsatzsteuer | 8.1 % |
| `UR26` | Umsatzsteuer reduzierter Satz | Umsatzsteuer | 2.6 % |
| `UB38` | Umsatzsteuer Beherbergung | Umsatzsteuer | 3.8 % |
| `VM81` | Vorsteuer Material/Dienstleistungen 8.1 % | Vorsteuer | 8.1 % |
| `VI81` | Vorsteuer Investitionen/übriger Betriebsaufwand 8.1 % | Vorsteuer | 8.1 % |
| `UEX` | Steuerbefreit / Export | Umsatzsteuer | 0 % |

Ein Unternehmen wird **ab CHF 100 000 Jahresumsatz** MWST-pflichtig. Die MWST-Funktion muss deshalb
vollständig vorhanden sein und lässt sich mit dem Beginn der Pflicht **einschalten**; solange sie aus ist,
überwacht das System den Umsatz und warnt vor dem Erreichen der Grenze.

Die Codes ersetzen die bisher fest im Code hinterlegte Liste der MWST-Sätze im Einkauf (ANF-004).
Sie werden unter `/einstellungen/finanzen/mwst` gepflegt.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **MWST-Sätze zentral pflegen und bei Satzänderungen neue Codes mit
> Gültigkeitsdatum anlegen**, damit **Rechnungen und die MWST-Abrechnung immer mit dem richtigen Satz rechnen**.

## 3. Akzeptanzkriterien

- [ ] **AK-1:** Gegeben die Einstellungsseite „MWST“, dann erscheinen alle Codes als Tabelle mit Code,
  Bezeichnung, Art, Satz, gültig ab, gültig bis, MWST-Konto, Formularziffer und Kennzeichen „aktiv“.
- [ ] **AK-2:** Gegeben die Seite, wenn noch keine Codes vorhanden sind, dann kann der Benutzer über
  „Schweizer Standard-Codes übernehmen“ die gängigen Codes (Tabelle oben) anlegen.
- [ ] **AK-3:** Gegeben „Neuer MWST-Code“, dann öffnet sich ein Modal-Fenster (`?neu=1`). Code (eindeutig,
  max. 10 Zeichen), Bezeichnung, Art, Satz (0–100 %, zwei Nachkommastellen) und gültig ab sind Pflicht.
- [ ] **AK-4:** Gegeben die Art „Umsatzsteuer“ bzw. „Vorsteuer“, dann wird als MWST-Konto das passende
  Systemkonto (ANF-011) vorgeschlagen; ein anderes Konto ist wählbar.
- [ ] **AK-5:** Gegeben ein Code, der bereits in Buchungen oder Rechnungen verwendet wird, dann sind Satz
  und Art nicht mehr änderbar; der Code kann nur mit „gültig bis“ beendet und durch einen neuen ersetzt werden.
- [ ] **AK-6:** Gegeben ein Belegdatum ausserhalb der Gültigkeit eines Codes, dann ist der Code für diesen
  Beleg nicht wählbar bzw. die Speicherung wird mit Meldung abgelehnt.
- [ ] **AK-7:** Gegeben ein unbenutzter Code, dann kann er gelöscht werden; ein benutzter Code nur deaktiviert.
- [ ] **AK-8:** Gegeben der Abschnitt „MWST-Einstellungen“, dann kann der Benutzer die MWST-Pflicht mit
  **Beginndatum** einschalten, die Abrechnungsmethode (effektiv / Saldosteuersatz), die
  Abrechnungsart (vereinbart / vereinnahmt) und die Periodizität (quartalsweise / halbjährlich) festlegen.
  Die MWST-Nummer wird aus der UID der Firma gebildet (`CHE-123.456.789 MWST`, ANF-025); ohne UID kann die
  Pflicht nicht eingeschaltet werden. Ohne MWST-Pflicht sind alle Codes ausgeblendet und Belege werden
  ohne MWST erfasst.
- [ ] **AK-9:** Gegeben die MWST-Codes, dann verwendet der Einkauf (ANF-004) für neue Positionen die
  Codes statt der fest hinterlegten Sätze; bestehende Positionen behalten ihren Satz.
- [ ] **AK-10:** Gegeben die eingeschaltete MWST-Pflicht, dann gelten MWST-Codes für Belege ab dem
  Beginndatum; Belege mit früherem Datum bleiben ohne MWST und werden nicht geändert.
- [ ] **AK-11:** Gegeben ein nicht MWST-pflichtiges Unternehmen, dann berechnet das System den
  massgebenden Umsatz (gebuchte Erlöse der Kontenklasse 3 im laufenden Geschäftsjahr und in den letzten
  12 Monaten) und zeigt auf der MWST-Seite und in der Finanz-Übersicht (ANF-010 AK-4) den Stand zur
  **Umsatzgrenze** (Vorgabe CHF 100 000, in den MWST-Einstellungen änderbar): ab 80 % ein Hinweis
  (`warning`), ab 100 % eine Warnung (`error`) „Umsatzgrenze erreicht – MWST-Pflicht prüfen und bei der
  ESTV anmelden“ mit Verweis auf die Einstellung.
- [ ] **AK-12:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `mwst_code`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Code | Text (10) | Ja | eindeutig |
| Bezeichnung | Text (100) | Ja | |
| Art | Auswahl | Ja | `umsatzsteuer`, `vorsteuer` |
| Satz % | Dezimalzahl (5,2) | Ja | 0–100 |
| Gültig ab | Datum | Ja | |
| Gültig bis | Datum | Nein | nicht vor „gültig ab“ |
| MWST-Konto (`konto_id`) | Verweis | Ja | Konto aus ANF-011 |
| Formularziffer | Text (10) | Nein | Ziffer im ESTV-Formular, z. B. `303`, `400`, `405` |
| Aktiv | Ja/Nein | Ja | Standard Ja |
| Erstellt am / Geändert am | Zeitstempel | Ja | automatisch |

Einstellungen (Tabelle `finanzeinstellung`, eine Zeile): MWST-pflichtig (Ja/Nein), MWST-pflichtig ab (Datum),
Umsatzgrenze (Dezimal, Standard 100 000.00),
Methode (`effektiv`, `saldosteuersatz`), Abrechnungsart (`vereinbart`, `vereinnahmt`),
Periodizität (`quartal`, `halbjahr`).

## 5. Abhängigkeiten

- Hängt ab von: ANF-011 (MWST-Konten), ANF-025 (UID der Firma)
- Wird benötigt von: ANF-017, ANF-018, ANF-019, ANF-022; Anpassung in ANF-004

## 6. Nicht im Umfang

- Berechnung nach Saldo- bzw. Pauschalsteuersatz in der Abrechnung (nur Einstellung; Rechnung siehe ANF-022)
- Bezugsteuer und Einfuhrsteuer (Zoll) als eigene Abläufe
- Vorsteuerkorrekturen (Eigenverbrauch, Einlageentsteuerung)

## 7. Offene Fragen

- [x] Ist das Unternehmen MWST-pflichtig? **Geklärt 2026-11-18:** Pflicht ab CHF 100 000 Umsatz; die Funktion
  muss vorhanden sein (AK-8, AK-10, AK-11).
- [ ] Nach welcher Methode wird abgerechnet? *(Vorschlag: effektiv, vereinbart, quartalsweise)*
- [ ] Sollen bestehende Einkaufspositionen (ANF-004) per Migration einem Code zugeordnet werden? *(Vorschlag: ja, nach Satz)*
- [ ] Werden Preise netto (ohne MWST) oder brutto (inkl. MWST) erfasst? *(Vorschlag: netto, wie im Einkauf; brutto als Kennzeichen pro Beleg später)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | MWST-Pflicht ab CHF 100 000: Einschalten mit Beginndatum, Überwachung der Umsatzgrenze (AK-10, AK-11); MWST-Nummer aus UID der Firma |
