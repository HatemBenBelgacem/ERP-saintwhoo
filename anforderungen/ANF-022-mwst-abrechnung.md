# ANF-022: MWST-Abrechnung

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-022                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Soll                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Das ERP erstellt die **MWST-Abrechnung** für die Eidgenössische Steuerverwaltung (ESTV) je
Abrechnungsperiode (Quartal oder Halbjahr, ANF-012). Es summiert Umsätze und Steuern aus den Buchungszeilen
mit MWST-Code nach den **Ziffern des ESTV-Formulars** (z. B. 200 Umsatz, 303 Steuer Normalsatz,
400/405 Vorsteuer), zeigt die Steuerschuld bzw. das Guthaben und bucht beim **Abschliessen** die
Umbuchung auf das Abrechnungskonto. Danach werden die Perioden gesperrt (ANF-013).

```
Entwurf (berechnet, jederzeit neu berechenbar) → Abgeschlossen (gebucht, Perioden gesperrt)
```

Die Abrechnung liegt im Reiter „MWST-Abrechnung“ unter `/finanzen/mwst-abrechnung`.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **die Werte für das MWST-Formular direkt aus der Buchhaltung erhalten**,
> damit **ich die Abrechnung ohne Nebenrechnung korrekt einreichen kann**.

## 3. Akzeptanzkriterien

- [ ] **AK-1:** Gegeben der Reiter „MWST-Abrechnung“, dann erscheinen alle Abrechnungsperioden des
  Geschäftsjahres (nach Periodizität) mit Zeitraum, Steuerschuld, Vorsteuer, Zahllast/Guthaben und Status.
- [ ] **AK-2:** Gegeben eine Periode ohne Abrechnung, wenn der Benutzer „Abrechnung erstellen“ wählt, dann
  berechnet das System je Formularziffer Umsatz (MWST-Basis) und Steuer aus allen gebuchten Zeilen mit
  Buchungsdatum im Zeitraum (Methode „vereinbart“) bzw. aus Zahlungen im Zeitraum (Methode „vereinnahmt“).
- [ ] **AK-3:** Gegeben die berechnete Abrechnung, dann zeigt sie die Ziffern in der Reihenfolge des
  ESTV-Formulars mit Umsatz und Steuer, die Summe Umsatzsteuer, die Summe Vorsteuer und den zu zahlenden
  bzw. zurückzufordernden Betrag.
- [ ] **AK-4:** Gegeben eine Ziffer, wenn der Benutzer sie anklickt, dann sieht er die zugrunde liegenden
  Buchungszeilen (Abstimmung).
- [ ] **AK-5:** Gegeben eine Abweichung zwischen berechneter Steuer und Saldo der MWST-Konten im Zeitraum,
  dann zeigt die Abrechnung einen Warnhinweis mit der Differenz.
- [ ] **AK-6:** Gegeben eine Abrechnung im Entwurf, wenn der Benutzer „Abschliessen“ bestätigt, dann
  werden die Salden von Umsatzsteuer- und Vorsteuerkonto per Periodenende auf das Konto
  „Abrechnungskonto MWST“ umgebucht (Herkunft `mwst`), die Perioden des Zeitraums werden gesperrt und die
  Abrechnung ist unveränderlich.
- [ ] **AK-7:** Gegeben eine abgeschlossene Abrechnung, dann kann sie gedruckt werden (Druckansicht mit Absender und MWST-Nummer der Firma (ANF-025),
  allen Ziffern zum Übertrag ins ESTV-Portal) und als CSV heruntergeladen werden.
- [ ] **AK-8:** Gegeben eine abgeschlossene Abrechnung, wenn der Benutzer „Wieder öffnen“ bestätigt, dann
  wird die Umbuchung storniert und die Perioden entsperrt; dies ist nur möglich, solange das Geschäftsjahr
  offen ist und keine spätere Abrechnung abgeschlossen ist.
- [ ] **AK-9:** Gegeben die Einstellung „nicht MWST-pflichtig“, dann ist der Reiter ausgeblendet.
- [ ] **AK-10:** Die Tabelle wird per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `mwst_abrechnung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Geschäftsjahr (`geschaeftsjahr_id`) | Verweis | Ja | |
| Beginn / Ende | Datum | Ja | Quartal oder Halbjahr |
| Status | Auswahl | Ja | `entwurf`, `abgeschlossen` |
| Werte je Ziffer | JSON | Ja | festgeschrieben beim Abschliessen |
| Zahllast / Guthaben | Dezimal (14,2) | Ja | positiv = Zahllast |
| Buchung (`buchung_id`) | Verweis | bei abgeschlossen | Umbuchung |
| Abgeschlossen von / am | Verweis / Zeitstempel | Nein | |

Zusätzliches Systemkonto (ANF-011): „Abrechnungskonto MWST“ (Vorschlag 2201).

## 5. Abhängigkeiten

- Hängt ab von: ANF-011, ANF-012, ANF-013, ANF-017; für „vereinnahmt“ ANF-020
- Wird benötigt von: ANF-023

## 6. Nicht im Umfang

- Elektronische Übermittlung an die ESTV (eMWST / XML)
- Saldosteuersatz-Abrechnung mit Pauschalsätzen (nur Hinweis, falls Methode so eingestellt)
- Korrektur- und Jahresabstimmung (Formular „Finalisierung“)

## 7. Offene Fragen

- [ ] Wird die Saldosteuersatzmethode benötigt? *(Vorschlag: nein, nur effektive Methode)*
- [ ] Welche Formularziffern sollen in der ersten Stufe unterstützt werden? *(Vorschlag: 200, 220, 221, 280, 289, 299, 302, 303, 312, 313, 342, 343, 382, 383, 400, 405, 410, 415, 420, 500, 510)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | Absender aus Firmenverwaltung (ANF-025) |
