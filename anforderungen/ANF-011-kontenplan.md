# ANF-011: Kontenplan und Systemkonten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-011                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen / Einstellungen                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Der **Kontenplan** ist die Grundlage der Finanzbuchhaltung (ANF-010). Er enthält alle Konten, auf die
gebucht werden kann, gegliedert nach **Kontenklassen** und **Kontengruppen** gemäss
**Schweizer KMU-Kontenrahmen**:

```
Klasse 1 Aktiven · 2 Passiven · 3 Betriebsertrag · 4 Material-/Warenaufwand · 5 Personalaufwand
6 Übriger Betriebsaufwand · 7 Betriebliche Nebenerfolge · 8 Betriebsfremder Erfolg · 9 Abschluss
```

Jedes Konto hat eine **Kontoart** (Aktiv, Passiv, Aufwand, Ertrag), die bestimmt, ob es in der Bilanz oder
der Erfolgsrechnung erscheint. Zusätzlich werden **Systemkonten** festgelegt, die die übrigen Komponenten
automatisch verwenden (z. B. Debitoren 1100, Kreditoren 2000, Vorsteuer 1170, Umsatzsteuer 2200).

Der Kontenplan wird in den Einstellungen unter `/einstellungen/finanzen/kontenplan` gepflegt
(Anlegen, Anzeigen, Bearbeiten, Löschen).

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **den Kontenplan nach KMU-Kontenrahmen übernehmen und an unser
> Unternehmen anpassen**, damit **alle Buchungen auf den richtigen Konten landen**.

## 3. Akzeptanzkriterien

**Anzeigen**
- [ ] **AK-1:** Gegeben die Einstellungsseite „Kontenplan“, dann erscheinen alle Konten als Tabelle mit
  Nummer, Bezeichnung, Kontoart, Kontengruppe, MWST-Code (Vorgabe) und Kennzeichen „aktiv“, sortiert nach
  Nummer und gruppiert nach Kontenklasse (Klasse als Zwischenzeile).
- [ ] **AK-2:** Gegeben die Liste, dann kann der Benutzer nach Nummer oder Bezeichnung suchen und nach
  Kontoart filtern; inaktive Konten sind standardmässig ausgeblendet. Suche und Filter stehen in der Webadresse.

**Vorlage übernehmen**
- [ ] **AK-3:** Gegeben ein leerer Kontenplan, wenn der Benutzer auf „KMU-Kontenrahmen übernehmen“ klickt
  und bestätigt, dann wird ein vorbereiteter Kontenplan (gängige Konten des KMU-Kontenrahmens) angelegt und
  die Systemkonten werden vorbelegt. Ist bereits ein Konto vorhanden, ist die Schaltfläche ausgeblendet.

**Anlegen und Bearbeiten**
- [ ] **AK-4:** Gegeben die Liste, wenn der Benutzer auf „Neues Konto“ klickt, dann öffnet sich ein
  Modal-Fenster (`?neu=1`). Nummer (4 bis 6 Ziffern, eindeutig), Bezeichnung und Kontoart sind Pflicht.
- [ ] **AK-5:** Gegeben eine Kontonummer, dann schlägt das System die Kontoart aus der ersten Ziffer vor
  (1 → Aktiv, 2 → Passiv, 3 → Ertrag, 4–6 → Aufwand; bei 7, 8 und 9 kein Vorschlag, da diese Klassen
  Aufwand und Ertrag enthalten); der Benutzer kann sie ändern.
- [ ] **AK-6:** Gegeben ein Konto, dann kann der Benutzer es bearbeiten (`?bearbeiten=<kennung>`).
  Nummer und Kontoart sind nicht mehr änderbar, sobald auf das Konto gebucht wurde.
- [ ] **AK-7:** Gegeben ein Konto, dann kann ein Vorgabe-MWST-Code (ANF-012) hinterlegt werden, der beim
  Erfassen einer Buchung oder Rechnungsposition auf diesem Konto vorgeschlagen wird.
- [ ] **AK-8:** Gegeben ungültige Eingaben, dann wird nichts gespeichert und die Felder sind mit einer
  verständlichen Meldung markiert (Prüfung auf dem Server).

**Löschen und Deaktivieren**
- [ ] **AK-9:** Gegeben ein Konto ohne Buchungen und ohne Verwendung als Systemkonto, Bankkonto oder
  Vorgabe, dann kann es nach Bestätigung gelöscht werden.
- [ ] **AK-10:** Gegeben ein Konto mit Buchungen, dann ist Löschen nicht möglich; stattdessen kann es
  **deaktiviert** werden. Deaktivierte Konten sind in Auswahllisten für neue Buchungen nicht wählbar,
  bleiben aber in Auswertungen sichtbar.

**Systemkonten**
- [ ] **AK-11:** Gegeben der Abschnitt „Systemkonten“ auf derselben Seite, dann kann der Benutzer für jede
  Rolle genau ein aktives Konto der passenden Kontoart wählen: Debitoren, Kreditoren, Vorsteuer,
  Umsatzsteuer, Bank (Vorgabe), Kursdifferenzen, Skonto Ertrag (gewährt), Skonto Aufwand (erhalten),
  Rundungsdifferenzen, Gewinnvortrag, Eröffnungsbilanz und Jahresgewinn/-verlust.
- [ ] **AK-12:** Gegeben fehlende Pflicht-Systemkonten (Debitoren, Kreditoren, Vorsteuer, Umsatzsteuer),
  dann zeigt die Seite einen Warnhinweis, und automatische Buchungen aus Debitoren/Kreditoren sind gesperrt.

**Allgemein**
- [ ] **AK-13:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `konto`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Nummer | Text (6) | Ja | nur Ziffern, eindeutig, z. B. `1020` |
| Bezeichnung | Text (100) | Ja | z. B. „Bankguthaben“ |
| Kontoart | Auswahl | Ja | `aktiv`, `passiv`, `aufwand`, `ertrag` |
| Kontengruppe | Text (100) | Nein | z. B. „Flüssige Mittel“ (zur Gliederung in Bilanz/Erfolgsrechnung) |
| MWST-Code (`mwst_code_id`) | Verweis | Nein | Vorgabe für Buchungen (ANF-012) |
| Aktiv | Ja/Nein | Ja | Standard Ja |
| Bemerkung | Langtext | Nein | |
| Erstellt am / Geändert am | Zeitstempel | Ja | automatisch |

Tabelle `systemkonto`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Rolle | Auswahl | Ja | Schlüssel, z. B. `debitoren`, `kreditoren`, `vorsteuer`, `umsatzsteuer`, … (siehe AK-11) |
| Konto (`konto_id`) | Verweis | Ja | Löschen des Kontos wird verhindert |

## 5. Abhängigkeiten

- Hängt ab von: ANF-010 (Rahmen), ANF-012 (MWST-Codes, nur für die Vorgabe – kann nachgezogen werden)
- Wird benötigt von: ANF-012, ANF-015, ANF-017 bis ANF-024

## 6. Nicht im Umfang

- Mehrere Kontenpläne (z. B. pro Mandant) und Kontenpläne nach anderen Rahmen (z. B. SKR03/04)
- Kostenstellen und Kostenträger
- Sammelkonten mit Unterkonten pro Kunde/Lieferant (offene Posten werden über die Belege geführt, ANF-020)

## 7. Offene Fragen

- [ ] Welche Konten soll die Vorlage enthalten? *(Vorschlag: rund 80 gängige Konten des KMU-Kontenrahmens, Liste in `model.rs`)*
- [ ] Länge der Kontonummer frei (4–6 Ziffern) oder fest 4-stellig? *(Vorschlag: 4–6 Ziffern)*
- [ ] Sollen Kontengruppen eine eigene Tabelle mit fester Reihenfolge für die Bilanzgliederung bekommen? *(Vorschlag: vorerst Freitext; Gliederung nach Nummernbereich)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
