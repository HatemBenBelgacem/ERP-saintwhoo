# ANF-011: Kontenplan und Systemkonten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-011                                           |
| **Status**    | Erledigt                                          |
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
- [x] **AK-1:** Gegeben die Einstellungsseite „Kontenplan“, dann erscheinen alle Konten als Tabelle mit
  Nummer, Bezeichnung, Kontoart, Kontengruppe, MWST-Code (Vorgabe) und Kennzeichen „aktiv“, sortiert nach
  Nummer und gruppiert nach Kontenklasse (Klasse als Zwischenzeile).
- [x] **AK-2:** Gegeben die Liste, dann kann der Benutzer nach Nummer oder Bezeichnung suchen und nach
  Kontoart filtern; inaktive Konten sind standardmässig ausgeblendet. Suche und Filter stehen in der Webadresse.

**Vorlage übernehmen**
- [x] **AK-3:** Gegeben ein leerer Kontenplan, wenn der Benutzer auf „KMU-Kontenrahmen übernehmen“ klickt
  und bestätigt, dann wird ein vorbereiteter Kontenplan (gängige Konten des KMU-Kontenrahmens) angelegt und
  die Systemkonten werden vorbelegt. Ist bereits ein Konto vorhanden, ist die Schaltfläche ausgeblendet.

**Anlegen und Bearbeiten**
- [x] **AK-4:** Gegeben die Liste, wenn der Benutzer auf „Neues Konto“ klickt, dann öffnet sich ein
  Modal-Fenster (`?neu=1`). Nummer (4 bis 6 Ziffern, eindeutig), Bezeichnung und Kontoart sind Pflicht.
- [x] **AK-5:** Gegeben eine Kontonummer, dann schlägt das System die Kontoart aus der ersten Ziffer vor
  (1 → Aktiv, 2 → Passiv, 3 → Ertrag, 4–6 → Aufwand; bei 7, 8 und 9 kein Vorschlag, da diese Klassen
  Aufwand und Ertrag enthalten); der Benutzer kann sie ändern.
- [x] **AK-6:** Gegeben ein Konto, dann kann der Benutzer es bearbeiten (`?bearbeiten=<kennung>`).
  Nummer und Kontoart sind nicht mehr änderbar, sobald auf das Konto gebucht wurde.
- [x] **AK-7:** Gegeben ein Konto, dann kann ein Vorgabe-MWST-Code (ANF-012) hinterlegt werden, der beim
  Erfassen einer Buchung oder Rechnungsposition auf diesem Konto vorgeschlagen wird.
- [x] **AK-8:** Gegeben ungültige Eingaben, dann wird nichts gespeichert und die Felder sind mit einer
  verständlichen Meldung markiert (Prüfung auf dem Server).

**Löschen und Deaktivieren**
- [x] **AK-9:** Gegeben ein Konto ohne Buchungen und ohne Verwendung als Systemkonto, Bankkonto oder
  Vorgabe, dann kann es nach Bestätigung gelöscht werden.
- [x] **AK-10:** Gegeben ein Konto mit Buchungen, dann ist Löschen nicht möglich; stattdessen kann es
  **deaktiviert** werden. Deaktivierte Konten sind in Auswahllisten für neue Buchungen nicht wählbar,
  bleiben aber in Auswertungen sichtbar.

**Systemkonten**
- [x] **AK-11:** Gegeben der Abschnitt „Systemkonten“ auf derselben Seite, dann kann der Benutzer für jede
  Rolle genau ein aktives Konto der passenden Kontoart wählen: Debitoren, Kreditoren, Vorsteuer,
  Umsatzsteuer, Bank (Vorgabe), Kursdifferenzen, Skonto Ertrag (gewährt), Skonto Aufwand (erhalten),
  Rundungsdifferenzen, Gewinnvortrag, Eröffnungsbilanz und Jahresgewinn/-verlust.
- [x] **AK-12:** Gegeben fehlende Pflicht-Systemkonten (Debitoren, Kreditoren, Vorsteuer, Umsatzsteuer),
  dann zeigt die Seite einen Warnhinweis, und automatische Buchungen aus Debitoren/Kreditoren sind gesperrt.

**Allgemein**
- [x] **AK-13:** Die Tabellen werden per sqlx-Migration angelegt.

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

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261118120000_konto.sql` | Tabellen `konto` (Nummer eindeutig, 4–6 Ziffern, Kontoart) und `systemkonto` (Rolle → Konto, Löschen des Kontos verhindert) |
| `src/backend/finance/accounts/model.rs` | Datentypen, Prüfregeln, Kontenklassen, Vorschlag der Kontoart, Rollen der Systemkonten, Vorlage KMU-Kontenrahmen (97 Konten) + Modultests |
| `src/backend/finance/accounts/server_fns.rs` + `repository.rs` | Liste, Anlegen, Bearbeiten, Löschen, Vorlage übernehmen, Systemkonten speichern |
| `src/components/settings/finance/accounts/list.rs` | Seite `/einstellungen/finanzen/kontenplan`: Liste nach Klassen, Suche, Filter, inaktive Konten, Vorlage, Systemkonten, Warnung |
| `src/components/settings/finance/accounts/form.rs` | Modal-Fenster: Konto (`?neu=1`, `?bearbeiten=<kennung>`) mit Löschen, Vorlage (`?vorlage=1`), Systemkonten (`?systemkonten=1`) |
| `src/navigation.rs`, `src/layout.rs`, `src/app.rs`, `src/icons.rs`, `src/forms.rs` | Abschnitt „Finanzen“ in der Einstellungsnavigation (`SETTINGS_FINANCE_NAV`, ANF-010 AK-3), Route, Symbol, Fenster-Parameter |

Hinweise:
- **AK-6:** Die Sperre nach der ersten Buchung ist mit ANF-017 umgesetzt (Nummer und Kontoart gesperrt, Kennzeichen „gebucht“). Schon jetzt: Ein Systemkonto behält eine zur Rolle passende Kontoart und bleibt aktiv.
- **AK-7:** Der Vorgabe-MWST-Code ist mit ANF-017 umgesetzt (Spalte `konto.mwst_code_id`, Auswahl im Kontoformular, Vorschlag beim Erfassen).
- **AK-9/AK-10:** Gelöscht werden kann jedes Konto ohne Verweis; Verweise (heute Systemkonten, später Buchungen,
  Bankkonten) verhindern das Löschen über den Fremdschlüssel, die Meldung schlägt Deaktivieren vor.
  Inaktive Konten werden in „Systemkonten bearbeiten“ nicht angeboten; die Auswahl bei Buchungen folgt mit ANF-017.
- **AK-11:** Zusätzliche Rolle „Abrechnungskonto MWST“ (für ANF-022). „Skonto gewährt“ ist als Erlösminderung (Ertrag),
  „Skonto erhalten“ als Aufwandminderung (Aufwand) vorbelegt; beide erlauben Aufwand oder Ertrag.
- **AK-12:** Die Warnung erscheint auf der Seite; die Sperre der automatischen Buchungen setzen ANF-018/ANF-019 um.
- Vorlage: Klasse 9 enthält nur „9100 Eröffnungsbilanz“ (als Passivkonto geführt); Rundungsdifferenzen 6990 und
  Kursdifferenzen 6960 sind ergänzt.
- Unter Linux übersetzt der Server nur mit `RUSTFLAGS="--cfg erase_components"` (wie im Dockerfile).

Geprüft: alle umgesetzten AK im Browser (grosser Bildschirm und Mobiltelefon, `testszenarien.md` TS-011-01 bis -13),
Prüfung der Systemkonten auf dem Server mit veränderter Anfrage, 8 neue Modultests.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | Umgesetzt (AK-6 Sperre nach Buchung mit ANF-017, AK-7 mit ANF-012) |
| 2026-11-26 | Claude Code | Nachtrag: AK-6 (Sperre nach Buchung) und AK-7 (Vorgabe-MWST-Code am Konto) mit ANF-017 umgesetzt (Migration `20261202110000_konto_vorgabe_mwst.sql`). |
