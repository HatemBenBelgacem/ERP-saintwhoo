# ANF-015: Eigene Bankkonten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-015                                           |
| **Status**    | In Umsetzung                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Die **eigenen Bank- und Postkonten** sowie die **Kasse** des Unternehmens werden unter
`/einstellungen/finanzen/bankkonten` gepflegt. Jedes Bankkonto ist mit einem Konto im Kontenplan (ANF-011)
verknüpft. Es wird für Zahlungen (ANF-020) ausgewählt und auf Ausgangsrechnungen (ANF-018) als
Zahlungsverbindung (IBAN, QR-IBAN) angegeben.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **unsere Bankverbindungen einmal erfassen**, damit **sie auf Rechnungen
> erscheinen und Zahlungen dem richtigen Konto zugeordnet werden**.

## 3. Akzeptanzkriterien

- [x] **AK-1:** Gegeben die Einstellungsseite „Bankkonten“, dann erscheinen alle Bankkonten mit
  Bezeichnung, Art (Bank, Post, Kasse), Bank, IBAN, Konto im Kontenplan und Kennzeichen „Standard“ und „aktiv“.
- [x] **AK-2:** Gegeben „Neues Bankkonto“, dann öffnet sich ein Modal-Fenster (`?neu=1`). Bezeichnung, Art
  und Konto im Kontenplan (Kontoart Aktiv) sind Pflicht; ein Konto im Kontenplan darf nur einem Bankkonto
  zugeordnet sein.
- [x] **AK-3:** Gegeben die Art „Bank“ oder „Post“, dann ist die IBAN Pflicht und wird geprüft
  (Länderkennung, Länge, Prüfziffer nach ISO 13616). Die Anzeige erfolgt in Vierergruppen.
- [x] **AK-4:** Gegeben eine QR-IBAN, dann wird sie zusätzlich geprüft (Schweizer IBAN mit
  Institutskennung 30000–31999); sie ist optional.
- [x] **AK-5:** Gegeben mehrere Bankkonten, dann ist genau eines als **Standard** markiert; es wird in
  Zahlungen und auf Rechnungen vorgeschlagen. Das erste angelegte Konto wird automatisch Standard.
- [ ] **AK-6:** Gegeben ein Bankkonto mit Zahlungen, dann kann es nicht gelöscht, nur deaktiviert werden.
- [x] **AK-7:** Die Tabelle wird per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `bankkonto`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Bezeichnung | Text (100) | Ja | z. B. „ZKB Geschäftskonto“ |
| Art | Auswahl | Ja | `bank`, `post`, `kasse` |
| Bank / Institut | Text (100) | Nein | |
| IBAN | Text (34) | bei Bank/Post | ohne Leerzeichen gespeichert |
| QR-IBAN | Text (34) | Nein | |
| BIC | Text (11) | Nein | |
| Konto (`konto_id`) | Verweis | Ja | Aktivkonto aus ANF-011, eindeutig |
| Standard | Ja/Nein | Ja | genau ein Konto |
| Aktiv | Ja/Nein | Ja | |
| Erstellt am / Geändert am | Zeitstempel | Ja | automatisch |

## 5. Abhängigkeiten

- Hängt ab von: ANF-011
- Hängt zusätzlich ab von: ANF-025 (Bankkonten gehören zur eigenen Firma)
- Wird benötigt von: ANF-018, ANF-020, ANF-010 AK-4 (Saldo der Bankkonten)

## 6. Nicht im Umfang

- Bankkonten in Fremdwährung (siehe ANF-014, offene Frage)
- Kontoauszüge importieren und Bankabgleich (camt.053/054)
- Bankverbindungen von Kunden und Lieferanten (gehören zur Adresse, siehe ANF-019 offene Fragen)

## 7. Offene Fragen

- [x] Wird eine QR-IBAN (für QR-Rechnungen mit Referenz) eingesetzt? **Entschieden 2026-11-25: Vorschlag übernommen** (Feld ist vorgesehen). *(QR-Rechnung wird benötigt, Zeitpunkt offen;
  Vorschlag: Feld jetzt vorsehen)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261201100000_bankkonto.sql` | Tabelle `bankkonto`: Art, Bank, IBAN, QR-IBAN, BIC, Konto im Kontenplan (eindeutig), genau ein Standard-Konto (Teilindex) |
| `src/backend/finance/bank_account/` | Modell mit Prüfregeln (IBAN nach ISO 13616 mit Prüfsumme Modulo 97, QR-IBAN mit Institutsnummer 30000 bis 31999, BIC), Gruppierung der IBAN, SQL, Server-Funktionen |
| `src/components/settings/finance/bank_accounts/` | Seite `/einstellungen/finanzen/bankkonten` mit Liste, Fenster „Neues Bankkonto“ und „Bankkonto bearbeiten“ |

Festlegungen bei der Umsetzung:
- **Standard:** Das erste Konto wird automatisch Standard. Das Standard-Konto lässt sich nur ersetzen, indem man ein anderes als Standard festlegt; es bleibt aktiv und lässt sich nicht löschen.
- **Kasse:** Bank, IBAN, QR-IBAN und BIC entfallen; sie werden beim Speichern gelöscht.
- **Speicherung:** IBAN und BIC ohne Leerzeichen, in Grossbuchstaben. Die Liste zeigt die IBAN in Vierergruppen.
- **AK-6 (teilweise):** Ein Bankkonto lässt sich löschen, weil es noch keine Zahlungen gibt. Die Prüfung auf Zahlungen folgt mit ANF-020.
- **Offen aus ANF-025 (AK-2):** Die Firmenseite verweist noch nicht auf die Bankkonten.
- Geprüft: 8 Modultests (IBAN, QR-IBAN, BIC, Gruppierung, Kasse, Pflichtfelder), Migration auf einer leeren Datenbank, Regeln direkt in der Datenbank (ein Konto nur einmal, ein Standard-Konto).
- Die Browser-Prüfung steht noch aus.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | QR-Rechnung wird benötigt; Bezug zu ANF-025 |
| 2026-11-25 | Claude Code | Freigegeben (Umsetzung von Block 1 beauftragt); umgesetzt bis auf AK-6 (Prüfung auf Zahlungen mit ANF-020) und den Verweis aus ANF-025 |
