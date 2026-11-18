# ANF-015: Eigene Bankkonten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-015                                           |
| **Status**    | Entwurf                                           |
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

- [ ] **AK-1:** Gegeben die Einstellungsseite „Bankkonten“, dann erscheinen alle Bankkonten mit
  Bezeichnung, Art (Bank, Post, Kasse), Bank, IBAN, Konto im Kontenplan und Kennzeichen „Standard“ und „aktiv“.
- [ ] **AK-2:** Gegeben „Neues Bankkonto“, dann öffnet sich ein Modal-Fenster (`?neu=1`). Bezeichnung, Art
  und Konto im Kontenplan (Kontoart Aktiv) sind Pflicht; ein Konto im Kontenplan darf nur einem Bankkonto
  zugeordnet sein.
- [ ] **AK-3:** Gegeben die Art „Bank“ oder „Post“, dann ist die IBAN Pflicht und wird geprüft
  (Länderkennung, Länge, Prüfziffer nach ISO 13616). Die Anzeige erfolgt in Vierergruppen.
- [ ] **AK-4:** Gegeben eine QR-IBAN, dann wird sie zusätzlich geprüft (Schweizer IBAN mit
  Institutskennung 30000–31999); sie ist optional.
- [ ] **AK-5:** Gegeben mehrere Bankkonten, dann ist genau eines als **Standard** markiert; es wird in
  Zahlungen und auf Rechnungen vorgeschlagen. Das erste angelegte Konto wird automatisch Standard.
- [ ] **AK-6:** Gegeben ein Bankkonto mit Zahlungen, dann kann es nicht gelöscht, nur deaktiviert werden.
- [ ] **AK-7:** Die Tabelle wird per sqlx-Migration angelegt.

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

- [ ] Wird eine QR-IBAN (für QR-Rechnungen mit Referenz) eingesetzt? *(QR-Rechnung wird benötigt, Zeitpunkt offen;
  Vorschlag: Feld jetzt vorsehen)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | QR-Rechnung wird benötigt; Bezug zu ANF-025 |
