# ANF-016: Zahlungsbedingungen

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-016                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

**Zahlungsbedingungen** legen fest, bis wann eine Rechnung zu bezahlen ist und ob bei früher Zahlung ein
**Skonto** gilt, z. B. „30 Tage netto“ oder „10 Tage 2 % Skonto, 30 Tage netto“. Sie werden unter
`/einstellungen/finanzen/zahlungsbedingungen` gepflegt und in Debitoren- und Kreditorenrechnungen
(ANF-018, ANF-019) verwendet, um Fälligkeit und Skontofrist zu berechnen. Zusätzlich kann einer Adresse
(ANF-002) eine Vorgabe-Zahlungsbedingung zugeordnet werden.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Zahlungsbedingungen einmal festlegen und pro Kunde vorbelegen**, damit
> **Fälligkeiten auf allen Rechnungen einheitlich und richtig berechnet werden**.

## 3. Akzeptanzkriterien

- [ ] **AK-1:** Gegeben die Einstellungsseite „Zahlungsbedingungen“, dann erscheinen alle Bedingungen mit
  Bezeichnung, Zahlungsfrist (Tage), Skontofrist (Tage), Skonto %, Rechnungstext und Kennzeichen
  „Standard“ und „aktiv“.
- [ ] **AK-2:** Gegeben „Neue Zahlungsbedingung“, dann öffnet sich ein Modal-Fenster (`?neu=1`).
  Bezeichnung und Zahlungsfrist (0–365 Tage) sind Pflicht. Skontofrist und Skonto % sind entweder beide
  leer oder beide gesetzt; die Skontofrist ist kleiner als die Zahlungsfrist, Skonto liegt über 0 und unter 100 %.
- [ ] **AK-3:** Gegeben die Eingaben, dann schlägt das System einen Rechnungstext vor, z. B.
  „Zahlbar innert 10 Tagen mit 2 % Skonto oder innert 30 Tagen netto“; der Text ist änderbar.
- [ ] **AK-4:** Gegeben mehrere Bedingungen, dann ist genau eine als **Standard** markiert; sie wird bei
  neuen Rechnungen verwendet, wenn die Adresse keine Vorgabe hat.
- [ ] **AK-5:** Gegeben eine Adresse (ANF-002), dann kann im Adressformular eine Vorgabe-Zahlungsbedingung
  gewählt werden; sie wird in neuen Rechnungen an diese Adresse vorgeschlagen.
- [ ] **AK-6:** Gegeben eine Rechnung, dann wird **Fälligkeit = Rechnungsdatum + Zahlungsfrist** und
  **Skonto bis = Rechnungsdatum + Skontofrist** berechnet und in der Rechnung gespeichert (spätere Änderungen
  der Bedingung wirken nicht auf bestehende Rechnungen).
- [ ] **AK-7:** Gegeben eine verwendete Bedingung, dann kann sie nicht gelöscht, nur deaktiviert werden.
- [ ] **AK-8:** Die Tabelle wird per sqlx-Migration angelegt, mit „30 Tage netto“ als Standard.

## 4. Daten / Felder

Tabelle `zahlungsbedingung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Bezeichnung | Text (100) | Ja | |
| Zahlungsfrist Tage | Ganzzahl | Ja | 0–365 |
| Skontofrist Tage | Ganzzahl | Nein | < Zahlungsfrist |
| Skonto % | Dezimalzahl (5,2) | Nein | > 0 und < 100 |
| Rechnungstext | Text (250) | Ja | wird auf die Rechnung gedruckt |
| Standard | Ja/Nein | Ja | genau eine |
| Aktiv | Ja/Nein | Ja | |

Erweiterung Tabelle `adresse`: `zahlungsbedingung_id` (Verweis, optional).

## 5. Abhängigkeiten

- Hängt ab von: ANF-010, ANF-002 (Erweiterung Adresse)
- Wird benötigt von: ANF-018, ANF-019, ANF-020 (Skonto), ANF-024 (Fälligkeit)

## 6. Nicht im Umfang

- Mehrstufige Skonti (z. B. 10 Tage 3 %, 20 Tage 2 %)
- Ratenzahlungen und Zahlungspläne
- Fälligkeit „Ende Monat“ (nur Tage ab Rechnungsdatum)

## 7. Offene Fragen

- [ ] Getrennte Vorgabe-Zahlungsbedingung für Adressen als Kunde und als Lieferant? *(Vorschlag: eine gemeinsame Vorgabe)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
