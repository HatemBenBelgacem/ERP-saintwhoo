# ANF-004: Einkaufsaufträge verwalten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-004                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Einkauf                                           |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-11                                        |
| **Geändert**  | 2026-11-11                                        |

## 1. Beschreibung

Das ERP erfasst **Einkaufsaufträge** (Bestellungen bei Lieferanten). Ein Einkaufsauftrag besteht aus
Kopfdaten (Lieferant, Datum, Status …) und beliebig vielen **Positionen** (bestellte Leistungen oder Waren
mit Menge und Preis):

```
Einkaufsauftrag
└── Position        (1 Einkaufsauftrag hat 0..n Positionen)
```

Beziehungen zu bestehenden Modulen:
- **Einkaufsauftrag – Adresse N:1** (Pflicht): Der Lieferant ist eine Adresse aus dem Adressstamm (ANF-002).
- **Einkaufsauftrag – Projekt N:1** (optional): Ein Einkauf kann einem Projekt (ANF-003) zugeordnet werden,
  damit sichtbar ist, was für ein Projekt bestellt wurde.

Einkaufsaufträge und Positionen lassen sich mit den vier Grundfunktionen verwalten:

| Grundfunktion | Bedeutung |
|---------------|-----------|
| **Anlegen**    | Neuen Einkaufsauftrag bzw. neue Position erfassen |
| **Anzeigen**   | Einkaufsaufträge auflisten, suchen und im Detail mit allen Positionen ansehen |
| **Bearbeiten** | Kopfdaten und Positionen ändern (solange der Auftrag nicht abgeschlossen ist) |
| **Löschen**    | Einkaufsauftrag inkl. Positionen bzw. einzelne Position entfernen |

Ein Einkaufsauftrag durchläuft folgende **Status**:

```
Entwurf → Bestellt → Teilweise geliefert → Geliefert
   │          │
   └──────────┴──→ Storniert
```

Das Modul erscheint als Eintrag **„Einkauf“** in der Hauptnavigation (ANF-001) unter dem Pfad `/einkauf`.

## 2. Benutzergeschichte

> Als **Einkäufer** möchte ich **Bestellungen bei Lieferanten mit allen Positionen erfassen und ihren
> Stand verfolgen**, damit **ich jederzeit weiss, was bestellt, was geliefert und was noch offen ist**.

> Als **Projektleiter** möchte ich **sehen, welche Einkäufe zu meinem Projekt gehören und was sie kosten**,
> damit **ich die Projektkosten im Blick behalte**.

## 3. Akzeptanzkriterien

**Liste (Anzeigen)**
- [x] **AK-1:** Gegeben der Navigationseintrag „Einkauf“, wenn der Benutzer ihn anklickt, dann erscheint
  eine Tabelle aller Einkaufsaufträge mit den Spalten Nummer, Lieferant, Projekt, Bestelldatum,
  Liefertermin, Status und Gesamtbetrag.
- [x] **AK-2:** Gegeben die Liste, dann kann der Benutzer nach Nummer, Lieferant, Projekt oder Referenz
  des Lieferanten suchen (Gross-/Kleinschreibung egal), nach Status filtern und nach jeder Spalte sortieren.
  Standard: alle offenen Status (Entwurf, Bestellt, Teilweise geliefert), nach Nummer absteigend.
- [x] **AK-3:** Gegeben mehr als 50 Einkaufsaufträge, dann wird die Liste seitenweise angezeigt.
  Suche, Filter, Sortierung und Seite stehen in der Webadresse.
- [x] **AK-4:** Gegeben keine Einkaufsaufträge oder kein Suchtreffer, dann erscheint ein entsprechender
  Hinweis statt einer leeren Tabelle.
- [x] **AK-5:** Gegeben ein Einkaufsauftrag mit überschrittenem Liefertermin, der noch nicht geliefert
  oder storniert ist, dann ist der Liefertermin in der Liste als überfällig markiert.

**Detailansicht (Anzeigen)**
- [x] **AK-6:** Gegeben die Liste, wenn der Benutzer eine Zeile anklickt, dann öffnet sich die
  Detailansicht unter `/einkauf/<kennung>` mit allen Kopfdaten, dem Lieferanten (als Verweis auf die
  Adresse), dem Projekt (als Verweis, falls zugeordnet) und einer Tabelle der Positionen.
- [x] **AK-7:** Gegeben die Positionstabelle, dann zeigt jede Zeile Position, Bezeichnung, Menge, Einheit,
  Einzelpreis, Rabatt, Betrag und gelieferte Menge. Darunter stehen Zwischensumme, MWST je Satz und
  Gesamtbetrag in der Währung des Auftrags.

**Anlegen**
- [x] **AK-8:** Gegeben die Liste, wenn der Benutzer auf „Neuer Einkaufsauftrag“ klickt, dann öffnet sich
  ein Formular für die Kopfdaten in einem Modal-Fenster (Webadresse `/einkauf?neu=1`).
- [x] **AK-9:** Gegeben das Formular, dann ist der Lieferant Pflicht und wird aus dem Adressstamm
  ausgewählt (Suche nach Anzeigename oder Adressnummer). Das Projekt ist optional.
- [x] **AK-10:** Gegeben ein gültiges Formular, wenn der Benutzer speichert, dann wird der Einkaufsauftrag
  im Status „Entwurf“ mit einer automatisch vergebenen, fortlaufenden Nummer gespeichert und die
  Detailansicht öffnet sich.
- [x] **AK-11:** Gegeben die Detailansicht eines bearbeitbaren Auftrags, wenn der Benutzer auf
  „Neue Position“ klickt, dann öffnet sich ein Modal-Fenster (Webadresse `/einkauf/<kennung>?neu=position`).
  Bezeichnung, Menge (> 0) und Einzelpreis (≥ 0) sind Pflicht; Rabatt liegt zwischen 0 und 100 %.
- [x] **AK-12:** Gegeben ungültige Eingaben, wenn der Benutzer speichert, dann wird nichts gespeichert und
  die fehlerhaften Felder werden mit einer verständlichen Meldung markiert. Die Prüfung erfolgt auf dem Server.

**Bearbeiten**
- [x] **AK-13:** Gegeben ein Auftrag im Status „Entwurf“, dann lassen sich Kopfdaten und Positionen
  bearbeiten (Webadresse `?bearbeiten=…`), Positionen hinzufügen, löschen und in der Reihenfolge verschieben.
- [x] **AK-14:** Gegeben ein Auftrag im Status „Entwurf“ mit mindestens einer Position, wenn der Benutzer
  auf „Bestellen“ klickt, dann wechselt der Status auf „Bestellt“ und das Bestelldatum wird auf heute
  gesetzt (falls leer). Ohne Positionen ist „Bestellen“ nicht möglich.
- [x] **AK-15:** Gegeben ein Auftrag im Status „Bestellt“ oder „Teilweise geliefert“, dann sind Lieferant,
  Positionen, Mengen und Preise gesperrt; nur Liefertermin, Referenz des Lieferanten und Bemerkung sind
  noch änderbar.

**Wareneingang (Liefermenge erfassen)**
- [x] **AK-16:** Gegeben ein Auftrag im Status „Bestellt“ oder „Teilweise geliefert“, wenn der Benutzer
  bei einer Position die gelieferte Menge erfasst, dann wird sie gespeichert; sie darf nicht negativ sein.
- [x] **AK-17:** Gegeben erfasste Liefermengen, dann setzt das System den Status automatisch:
  alle Positionen vollständig geliefert → „Geliefert“; mindestens eine Position teilweise oder ganz
  geliefert → „Teilweise geliefert“.
- [x] **AK-18:** Gegeben eine gelieferte Menge grösser als die bestellte Menge, dann erscheint ein Hinweis
  (Überlieferung), das Speichern ist aber erlaubt.

**Stornieren und Löschen**
- [x] **AK-19:** Gegeben ein Auftrag im Status „Entwurf“ oder „Bestellt“ (noch nichts geliefert), wenn
  der Benutzer auf „Stornieren“ klickt und bestätigt, dann wechselt der Status auf „Storniert“.
  Der Auftrag bleibt erhalten, ist aber nicht mehr bearbeitbar.
- [x] **AK-20:** Gegeben ein Auftrag im Status „Entwurf“, wenn der Benutzer auf „Löschen“ klickt, dann
  erscheint ein Bestätigungsdialog mit der Nummer. Nach Bestätigung werden Auftrag und alle Positionen
  gelöscht. Aufträge in anderen Status können nicht gelöscht, nur storniert werden.

**Verknüpfungen**
- [x] **AK-21:** Gegeben eine Adresse, die als Lieferant in einem Einkaufsauftrag verwendet wird, wenn der
  Benutzer sie löschen will, dann wird das Löschen verhindert und die Meldung nennt „in Einkaufsaufträgen
  (als Lieferant)“ (siehe ANF-002 AK-16).
- [x] **AK-22:** Gegeben ein Projekt mit zugeordneten Einkaufsaufträgen, dann zeigt die Projekt-Detailansicht
  eine Liste dieser Einkaufsaufträge mit Nummer, Lieferant, Status und Gesamtbetrag.
- [x] **AK-23:** Gegeben ein Projekt mit zugeordneten Einkaufsaufträgen, wenn der Benutzer es löschen will,
  dann wird das Löschen verhindert und eine Meldung nennt die Einkaufsaufträge.

**Allgemein**
- [x] **AK-24:** Gegeben eine nicht existierende Kennung in der Webadresse, dann erscheint die Meldung
  „Einkaufsauftrag nicht gefunden“.
- [x] **AK-25:** Beträge werden mit Rappen-Genauigkeit gerechnet (Dezimalzahl, keine Gleitkommazahl) und
  auf 0.05 bzw. 0.01 gerundet angezeigt (siehe offene Fragen).
- [x] **AK-26:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `einkaufsauftrag`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Nummer | Text (20) | Ja | Fachliche Nummer, automatisch, fortlaufend, eindeutig (z. B. `E-2026-0001`) |
| Lieferant (`adresse_id`) | Verweis | Ja | Adresse aus ANF-002, Löschen der Adresse wird verhindert |
| Projekt (`projekt_id`) | Verweis | Nein | Projekt aus ANF-003, Löschen des Projekts wird verhindert |
| Status | Auswahl | Ja | `entwurf`, `bestellt`, `teilweise_geliefert`, `geliefert`, `storniert`; Standard `entwurf` |
| Bestelldatum | Datum | Nein | Wird beim Bestellen gesetzt, falls leer |
| Liefertermin | Datum | Nein | Gewünschter Liefertermin; nicht vor Bestelldatum |
| Referenz Lieferant | Text (100) | Nein | z. B. Auftragsbestätigungs- oder Angebotsnummer des Lieferanten |
| Währung | Text (3) | Ja | ISO-Währungscode, Standard `CHF` |
| Lieferadresse | Langtext | Nein | Abweichende Lieferadresse als Freitext |
| Bemerkung | Langtext | Nein | |
| Erstellt am | Zeitstempel | Ja | automatisch |
| Geändert am | Zeitstempel | Ja | automatisch |

Tabelle `einkaufsposition`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Einkaufsauftrag (`einkaufsauftrag_id`) | Verweis | Ja | Wird mit dem Auftrag gelöscht |
| Reihenfolge | Ganzzahl | Ja | Positionsnummer (1, 2, 3 …) |
| Bezeichnung | Text (200) | Ja | Freitext, da es noch keinen Artikelstamm gibt |
| Artikelnummer Lieferant | Text (50) | Nein | |
| Menge | Dezimalzahl (12,3) | Ja | > 0 |
| Einheit | Auswahl | Ja | z. B. Stk, h, kg, m, m², m³, l, Pauschal; Standard `Stk` |
| Einzelpreis | Dezimalzahl (12,2) | Ja | ≥ 0, in Auftragswährung, ohne MWST |
| Rabatt % | Dezimalzahl (5,2) | Nein | 0–100, Standard 0 |
| MWST-Satz % | Dezimalzahl (5,2) | Ja | Standard 8.1 (Schweiz, Normalsatz) |
| Gelieferte Menge | Dezimalzahl (12,3) | Ja | ≥ 0, Standard 0 |
| Erstellt am | Zeitstempel | Ja | automatisch |
| Geändert am | Zeitstempel | Ja | automatisch |

**Berechnete Werte** (nicht gespeichert):
- Positionsbetrag = Menge × Einzelpreis × (1 − Rabatt / 100)
- Zwischensumme = Summe der Positionsbeträge
- MWST = je Satz Summe der Positionsbeträge × Satz / 100
- Gesamtbetrag = Zwischensumme + MWST

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst, Navigation), ANF-002 (Adressen: Lieferant),
  ANF-003 (Projekte: optionale Zuordnung)
- Wird benötigt von: – (später z. B. Artikelstamm, Lager, Kreditorenrechnungen)

## 6. Nicht im Umfang

- Artikelstamm und Lagerbestand (Positionen sind vorerst Freitext)
- Bestellanfragen, Angebotsvergleich und Freigabe-Ablauf (Genehmigung durch Vorgesetzte)
- Druck bzw. PDF der Bestellung und Versand per E-Mail an den Lieferanten
- Mehrere Wareneingänge mit eigenem Datum und Lieferschein pro Position (nur Gesamtmenge geliefert)
- Kreditorenrechnungen, Zahlungen und Buchhaltung
- Fremdwährungsumrechnung (Beträge werden nur in der Auftragswährung geführt)
- Rollen bzw. Kategorien für Adressen (jede Adresse kann als Lieferant gewählt werden)
- Berechtigungen und Änderungsprotokoll

## 7. Offene Fragen

Für die Umsetzung wurden die Vorschläge in Klammern als vorläufige Annahmen übernommen.

- [ ] Format der Einkaufsnummer? *(Vorschlag: `E-JJJJ-NNNN`, fortlaufend je Jahr, wie bei Projekten)*
- [ ] Sollen nur Adressen mit Rolle „Lieferant“ wählbar sein? Dazu bräuchte ANF-002 Rollen. *(Vorschlag: vorerst alle Adressen)*
- [ ] Braucht es einen Freigabe-Schritt vor dem Bestellen (z. B. ab einem bestimmten Betrag)? *(Vorschlag: nein, später als eigene Anforderung)*
- [ ] Soll ein Einkaufsauftrag statt einem Projekt auch einem Teilprojekt oder Vorgang zugeordnet werden können? *(Vorschlag: nur Projekt)*
- [ ] Rundung der Beträge: auf 0.05 (Rappenrundung CHF) oder auf 0.01? *(Vorschlag: Gesamtbetrag in CHF auf 0.05, sonst 0.01)*
- [ ] Welche MWST-Sätze werden angeboten? *(Vorschlag: 8.1 %, 2.6 %, 3.8 %, 0 % – Liste in `model.rs`)*
- [ ] Welche Währungen werden angeboten? *(Vorschlag: CHF, EUR, USD)*
- [ ] Dürfen bestellte Aufträge nachträglich geändert werden (Nachtrag)? *(Vorschlag: nein, bei Bedarf stornieren und neu anlegen)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261111200000_einkaufsauftrag.sql` | Tabellen `einkaufsauftrag`, `einkaufsposition`, Nummernzähler je Jahr, Prüfregeln, Fremdschlüssel ohne Kaskade auf Adresse und Projekt |
| `src/backend/purchase_orders/model.rs` | Datenmodell, Prüfregeln, Festkomma-Rechnung (Positionsbetrag, MWST je Satz, Rappenrundung), Status nach Lieferung + Modultests |
| `src/backend/purchase_orders/server_fns.rs` + `src/backend/purchase_orders/repository.rs` | Server-Funktionen: Liste, Detail, Anlegen, Bearbeiten, Positionen, Bestellen, Wareneingang, Alles geliefert, Stornieren, Löschen |
| `src/components/purchase_orders/list.rs` | Liste mit Suche, Statusfilter, Sortierung, Seiten, Markierung „überfällig“ |
| `src/components/purchase_orders/detail.rs` | Detailansicht mit Positionen und Summen, Schaltflächen je Status, Bestätigungsdialoge; Einkaufsaufträge im Projekt |
| `src/components/purchase_orders/form.rs` | Modal-Fenster: Kopfdaten, Position, Wareneingang |
| `src/forms.rs` | Neu gemeinsam: `SearchPicker` (Suchauswahl), `SelectField`; Fenster-Parameter `wareneingang` |
| `src/backend/addresses/server_fns.rs` + `src/backend/addresses/repository.rs`, `src/backend/projects/server_fns.rs` + `src/backend/projects/repository.rs` | `search_addresses`, `search_projects`; Löschsperren mit Meldung (AK-21, AK-23) |
| `src/components/projects/detail.rs` | Abschnitt „Einkaufsaufträge“ in der Projekt-Detailansicht (AK-22) |
| `src/navigation.rs`, `src/app.rs`, `src/icons.rs` | Navigationseintrag „Einkauf“, Routen, Symbol |

Hinweise:
- **AK-14:** Liegt der Liefertermin vor dem heutigen Bestelldatum, wird das Bestellen mit einer Meldung verhindert.
- **AK-16/17:** Wareneingang je Position über „Wareneingang“ (gelieferte Gesamtmenge); zusätzlich „Alles geliefert“
  für den ganzen Auftrag. Der Status wird danach automatisch gesetzt.
- **AK-25:** Mengen, Preise und Prozente werden in der Datenbank als `NUMERIC` gespeichert und in Rust als ganze Zahlen
  in kleinsten Einheiten gerechnet (kaufmännisch gerundet). Gesamtbetrag in CHF auf 0.05, sonst auf 0.01.
- Die Kunden-Suchauswahl der Projekte nutzt jetzt denselben Baustein wie der Einkauf.

Geprüft: alle AK im Browser (grosser Bildschirm und Mobiltelefon) und 13 neue Modultests.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-11 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-11 | Hatem Ben Belgacem | Umgesetzt (offene Fragen mit vorläufigen Annahmen) |
| 2026-11-19 | Claude Code | MWST-Satz kommt aus den MWST-Codes (ANF-012 AK-9). Bestehende Positionen behalten ihren Satz über „Bisheriger Satz (ohne Code)“; „ohne MWST“ ist wählbar |
