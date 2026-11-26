# ANF-014: Währungen und Umrechnungskurse

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-014                                           |
| **Status**    | In Umsetzung                                           |
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

- [x] **AK-1:** Gegeben die Einstellungsseite „Währungen“, dann erscheinen alle Währungen mit ISO-Code,
  Bezeichnung, Rundung, letztem Kurs (mit Datum) und Kennzeichen „aktiv“. CHF ist als Buchhaltungswährung
  markiert und kann weder gelöscht noch deaktiviert werden.
- [x] **AK-2:** Gegeben „Neue Währung“, dann öffnet sich ein Modal-Fenster (`?neu=1`). ISO-Code (drei
  Grossbuchstaben, eindeutig), Bezeichnung und Rundung (0.01 oder 0.05) sind Pflicht.
- [x] **AK-3:** Gegeben eine Fremdwährung, wenn der Benutzer „Kurs erfassen“ wählt, dann kann er Datum und
  Kurs (CHF für 1 Einheit, bis 6 Nachkommastellen, > 0) eintragen. Pro Währung und Datum gibt es höchstens
  einen Kurs.
- [ ] **AK-4:** Gegeben ein Beleg in Fremdwährung, dann wird der Kurs mit dem neuesten Datum ≤ Belegdatum
  vorgeschlagen; der Benutzer kann ihn im Beleg überschreiben. Fehlt ein Kurs, wird Speichern mit Meldung
  verhindert.
- [ ] **AK-5:** Gegeben eine Währung, die in Belegen verwendet wird, dann kann sie nicht gelöscht, nur
  deaktiviert werden.
- [x] **AK-6:** Gegeben die Währungen, dann bietet der Einkauf (ANF-004) die aktiven Währungen statt der
  fest hinterlegten Liste an.
- [x] **AK-7:** Die Tabellen werden per sqlx-Migration angelegt, mit CHF, EUR und USD als Startwerten.

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

- [x] Gibt es Bankkonten in Fremdwährung (z. B. EUR-Konto)? **Entschieden 2026-11-25: Vorschlag übernommen** (nein in der ersten Stufe)
- [x] Welche Kurse gelten – ESTV-Monatsmittelkurse oder Tageskurse der Bank? **Entschieden 2026-11-25: Vorschlag übernommen** (manuell erfasste Tageskurse)

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261201120000_waehrung.sql` | Tabellen `waehrung` (ISO-Code als Schlüssel, Rundung, genau eine Buchhaltungswährung) und `waehrungskurs` (je Währung und Datum höchstens ein Kurs); Startwerte CHF, EUR, USD. Die Prüfregel der Einkaufsaufträge wird durch einen Fremdschlüssel auf `waehrung` ersetzt |
| `src/backend/finance/currency/` | Modell mit Prüfregeln und Kursrechnung, SQL, Server-Funktionen (Anlegen, Ändern, Löschen, Kurs erfassen, Kurs an einem Datum) |
| `src/components/settings/finance/currencies/` | Seite `/einstellungen/finanzen/waehrungen` mit den Fenstern „Neue Währung“, „Währung bearbeiten“ und „Kurs erfassen“ |
| `src/backend/purchase_orders/`, `src/components/purchase_orders/form.rs` (ANF-004) | Die Währung wird aus den aktiven Währungen gewählt; der Server prüft die Aktivität beim Speichern (AK-6) |

Festlegungen bei der Umsetzung:
- **Rundung:** CHF 0.05 (Rappenrundung), EUR und USD 0.01. Die Rundung lässt sich ändern.
- **Kurse:** Gespeichert mit bis zu sechs Nachkommastellen. Ein Kurs gilt ab seinem Datum.
- **Löschen:** Eine Währung ohne Verwendung lässt sich löschen; ihre Kurse gehen dabei verloren (AK-5).
- **AK-1 bis AK-3, AK-6, AK-7:** vollständig umgesetzt. Geprüft: 8 Modultests, Migration auf einer leeren Datenbank, Regeln direkt in der Datenbank (Dublette am Datum, Rundung, Kaskade).
- **AK-4 (teilweise):** Die Abfrage „Kurs am Belegdatum“ steht als Server-Funktion `get_exchange_rate` bereit. Der Kursvorschlag im Beleg folgt mit ANF-018.
- **AK-5 (teilweise):** Eine im Einkauf verwendete Währung lässt sich nicht löschen. Rechnungen und Buchungen folgen mit ANF-018 bis ANF-019.
- Die Browser-Prüfung der Szenarien aus `testszenarien.md` steht noch aus.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-25 | Claude Code | Freigegeben (Umsetzung von Block 1 beauftragt); umgesetzt bis auf AK-4 und AK-5 (Belege folgen mit ANF-018 bis ANF-019) |
