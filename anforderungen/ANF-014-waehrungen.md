# ANF-014: Währungen und Umrechnungskurse

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-014                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Soll                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Die Buchhaltung wird in der **Buchhaltungswährung CHF** geführt. Rechnungen und Zahlungen können aber in
**Fremdwährung** (z. B. EUR, USD) anfallen. Dazu werden **Währungen** und **Umrechnungskurse** unter
`/einstellungen/finanzen/waehrungen` gepflegt. Beim Buchen eines Fremdwährungsbelegs wird der Betrag zum
Kurs des Belegdatums in CHF umgerechnet; Differenzen bei der Zahlung werden als **Kursdifferenz** gebucht
(ANF-020).

Die Währungen ersetzen die bisher fest im Code hinterlegte Liste im Einkauf (ANF-004).

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Rechnungen in Euro erfassen und die Kurse pflegen**, damit **die
> Buchhaltung trotzdem korrekt in Franken geführt wird**.

## 3. Akzeptanzkriterien

- [ ] **AK-1:** Gegeben die Einstellungsseite „Währungen“, dann erscheinen alle Währungen mit ISO-Code,
  Bezeichnung, Rundung, letztem Kurs (mit Datum) und Kennzeichen „aktiv“. CHF ist als Buchhaltungswährung
  markiert und kann weder gelöscht noch deaktiviert werden.
- [ ] **AK-2:** Gegeben „Neue Währung“, dann öffnet sich ein Modal-Fenster (`?neu=1`). ISO-Code (drei
  Grossbuchstaben, eindeutig), Bezeichnung und Rundung (0.01 oder 0.05) sind Pflicht.
- [ ] **AK-3:** Gegeben eine Fremdwährung, wenn der Benutzer „Kurs erfassen“ wählt, dann kann er Datum und
  Kurs (CHF für 1 Einheit, bis 6 Nachkommastellen, > 0) eintragen. Pro Währung und Datum gibt es höchstens
  einen Kurs.
- [ ] **AK-4:** Gegeben ein Beleg in Fremdwährung, dann wird der Kurs mit dem neuesten Datum ≤ Belegdatum
  vorgeschlagen; der Benutzer kann ihn im Beleg überschreiben. Fehlt ein Kurs, wird Speichern mit Meldung
  verhindert.
- [ ] **AK-5:** Gegeben eine Währung, die in Belegen verwendet wird, dann kann sie nicht gelöscht, nur
  deaktiviert werden.
- [ ] **AK-6:** Gegeben die Währungen, dann bietet der Einkauf (ANF-004) die aktiven Währungen statt der
  fest hinterlegten Liste an.
- [ ] **AK-7:** Die Tabellen werden per sqlx-Migration angelegt, mit CHF, EUR und USD als Startwerten.

## 4. Daten / Felder

Tabelle `waehrung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Code | Text (3) | Ja | Schlüssel, ISO 4217 |
| Bezeichnung | Text (50) | Ja | z. B. „Euro“ |
| Rundung | Dezimalzahl (4,2) | Ja | `0.01` oder `0.05` |
| Buchhaltungswährung | Ja/Nein | Ja | genau eine Währung (CHF) |
| Aktiv | Ja/Nein | Ja | |

Tabelle `waehrungskurs`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Währung (`waehrung_code`) | Verweis | Ja | |
| Datum | Datum | Ja | eindeutig je Währung |
| Kurs | Dezimalzahl (14,6) | Ja | CHF pro 1 Einheit |

## 5. Abhängigkeiten

- Hängt ab von: ANF-010, ANF-011 (Systemkonto Kursdifferenzen)
- Wird benötigt von: ANF-018, ANF-019, ANF-020; Anpassung in ANF-004

## 6. Nicht im Umfang

- Automatischer Kursabruf (z. B. ESTV- oder Bankkurse aus dem Internet)
- Bewertung offener Fremdwährungsposten zum Jahresende (Kann für ANF-023)
- Fremdwährungskonten im Kontenplan (Bankkonten werden in CHF geführt)

## 7. Offene Fragen

- [ ] Gibt es Bankkonten in Fremdwährung (z. B. EUR-Konto)? *(Vorschlag: nein in der ersten Stufe)*
- [ ] Welche Kurse gelten – ESTV-Monatsmittelkurse oder Tageskurse der Bank? *(Vorschlag: manuell erfasste Tageskurse)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
