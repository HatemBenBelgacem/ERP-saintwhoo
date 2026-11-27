# ANF-017: Buchungsjournal (Hauptbuch)

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-017                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Das **Buchungsjournal** ist der Kern der Finanzbuchhaltung. Es enthält alle **Buchungen** in zeitlicher
Reihenfolge. Eine Buchung hat einen Kopf (Datum, Belegnummer, Text) und mindestens zwei **Buchungszeilen**
(Konto, Soll oder Haben, Betrag, MWST-Code). Die Summe Soll ist immer gleich der Summe Haben.

```
Buchung  (Datum, Beleg, Text, Herkunft)
└── Buchungszeile  (Konto, Soll | Haben, Betrag CHF, MWST-Code, Projekt)   mind. 2 Zeilen
```

Buchungen entstehen auf zwei Arten:
- **manuell** im Journal (z. B. Bankspesen, Abgrenzungen, Eröffnungsbilanz)
- **automatisch** aus Debitoren- und Kreditorenrechnungen, Zahlungen, MWST-Abrechnung und Abschluss
  (Herkunft = Beleg, im Journal nur lesbar, Verweis auf den Beleg)

Für manuelle Buchungen mit MWST-Code erzeugt das System die **MWST-Zeile** automatisch
(z. B. Aufwand 100.00 mit VM81 → Aufwand 100.00, Vorsteuer 8.10 an Bank 108.10).

Das Journal liegt im Reiter „Journal“ unter `/finanzen/journal`.

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Buchungen mit Soll und Haben erfassen und alle Buchungen im Journal
> nachvollziehen**, damit **jeder Geschäftsfall lückenlos und belegbar in der Buchhaltung steht**.

## 3. Akzeptanzkriterien

**Liste (Anzeigen)**
- [x] **AK-1:** Gegeben der Reiter „Journal“, dann erscheint eine Tabelle aller Buchungen des gewählten
  Geschäftsjahres mit Datum, Belegnummer, Text, Soll-Konto, Haben-Konto, Betrag, Herkunft und Status;
  Sammelbuchungen (mehr als zwei Zeilen) zeigen „Diverse“ und lassen sich aufklappen.
- [x] **AK-2:** Gegeben die Liste, dann kann der Benutzer nach Text, Belegnummer, Konto und Betrag suchen,
  nach Zeitraum, Herkunft (manuell, Debitoren, Kreditoren, Zahlung, MWST, Abschluss) und Status filtern
  und sortieren. Standard: laufendes Geschäftsjahr, nach Datum absteigend. Filter und Seite stehen in der
  Webadresse; ab 50 Einträgen seitenweise.

**Anlegen**
- [x] **AK-3:** Gegeben „Neue Buchung“, dann öffnet sich ein Modal-Fenster (`?neu=1`) mit Datum (Vorgabe
  heute), Belegnummer (Vorschlag: nächste freie Nummer), Text und einer Zeilentabelle. Für die einfache
  Buchung genügen Soll-Konto, Haben-Konto, Betrag und optional MWST-Code; „Weitere Zeile“ wandelt sie in
  eine Sammelbuchung um.
- [x] **AK-4:** Gegeben die Kontoauswahl, dann sucht sie nach Nummer und Bezeichnung und bietet nur
  aktive Konten an (ANF-011 AK-10). Wählt der Benutzer ein Konto mit Vorgabe-MWST-Code, wird dieser
  vorgeschlagen.
- [x] **AK-5:** Gegeben eine Zeile mit MWST-Code, dann berechnet das System die Steuer (auf 0.01 gerundet)
  und zeigt die zusätzliche Zeile auf dem MWST-Konto des Codes an, bevor gespeichert wird. Ob der
  eingegebene Betrag netto oder brutto ist, wählt der Benutzer pro Buchung (Vorgabe: brutto für manuelle Buchungen).
- [x] **AK-6:** Gegeben die Zeilen, dann zeigt das Formular laufend Summe Soll, Summe Haben und Differenz;
  „Buchen“ ist nur möglich, wenn die Differenz 0.00 ist und jede Zeile ein Konto und einen Betrag > 0 hat.
- [x] **AK-7:** Gegeben ein Buchungsdatum, dann muss es in einem offenen Geschäftsjahr und einer offenen
  Periode liegen (ANF-013); sonst wird mit Meldung abgelehnt. Alle Prüfungen laufen auf dem Server.
- [x] **AK-8:** Gegeben eine Buchungszeile, dann kann optional ein Projekt (ANF-003) zugeordnet werden;
  diese Zuordnung wird in der Projektauswertung (ANF-021) verwendet.
- [x] **AK-9:** Gegeben eine gespeicherte Buchung, dann erhält sie eine fortlaufende, lückenlose
  Journalnummer je Geschäftsjahr und speichert erfassenden Benutzer und Zeitpunkt.

**Entwurf, Bearbeiten, Stornieren**
- [x] **AK-10:** Gegeben das Formular, wenn der Benutzer „Als Entwurf speichern“ wählt, dann wird die
  Buchung ohne Ausgleichsprüfung gespeichert, ist als Entwurf markiert und zählt in keiner Auswertung.
  Entwürfe können bearbeitet, gebucht oder gelöscht werden.
- [x] **AK-11:** Gegeben eine gebuchte manuelle Buchung, wenn der Benutzer „Stornieren“ wählt und
  bestätigt (mit Datum, Vorgabe: Originaldatum bzw. heute, falls die Periode gesperrt ist), dann entsteht
  eine Gegenbuchung mit vertauschtem Soll/Haben; beide Buchungen sind als storniert bzw. Storno markiert
  und verweisen aufeinander.
- [x] **AK-12:** Gegeben eine automatisch erzeugte Buchung, dann ist sie im Journal nicht stornierbar;
  der Verweis führt zum Beleg, der dort storniert wird.

**Detail**
- [x] **AK-13:** Gegeben ein Klick auf eine Buchung, dann öffnet sich die Detailansicht
  (`/finanzen/journal/<kennung>`) mit allen Zeilen, Herkunft, Benutzer, Zeitpunkt und Stornoverweis.
  Eine nicht existierende Kennung zeigt „Buchung nicht gefunden“.

**Vorlagen**
- [x] **AK-14:** Gegeben eine Buchung, wenn der Benutzer „Kopieren“ wählt, dann öffnet sich das
  Erfassungsfenster mit denselben Zeilen und dem heutigen Datum.

**Allgemein**
- [x] **AK-15:** Die Tabellen werden per sqlx-Migration angelegt; die Datenbank verhindert gebuchte
  Buchungen mit Soll ≠ Haben (Prüfung in der Transaktion vor dem Abschluss).

## 4. Daten / Felder

Tabelle `buchung`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Geschäftsjahr (`geschaeftsjahr_id`) | Verweis | Ja | aus dem Datum ermittelt |
| Journalnummer | Ganzzahl | bei gebucht | lückenlos je Geschäftsjahr |
| Datum | Datum | Ja | |
| Belegnummer | Text (30) | Ja | |
| Text | Text (200) | Ja | |
| Status | Auswahl | Ja | `entwurf`, `gebucht`, `storniert` |
| Herkunft | Auswahl | Ja | `manuell`, `debitor`, `kreditor`, `zahlung`, `mwst`, `abschluss`, `eroeffnung` |
| Herkunftsbeleg | Verweis | Nein | je nach Herkunft (`debitorenrechnung_id`, `kreditorenrechnung_id`, `zahlung_id` …) |
| Storno von (`storno_buchung_id`) | Verweis | Nein | Gegenbuchung ↔ Original |
| Erfasst von (`benutzer_id`) | Verweis | Ja | ANF-005 |
| Erstellt am / Gebucht am | Zeitstempel | Ja | automatisch |

Tabelle `buchungszeile`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch |
| Buchung (`buchung_id`) | Verweis | Ja | wird mit Entwurf gelöscht |
| Reihenfolge | Ganzzahl | Ja | |
| Konto (`konto_id`) | Verweis | Ja | aktives Konto |
| Soll | Dezimalzahl (14,2) | Nein | genau eines von Soll/Haben > 0 |
| Haben | Dezimalzahl (14,2) | Nein | |
| MWST-Code (`mwst_code_id`) | Verweis | Nein | |
| MWST-Basis | Dezimalzahl (14,2) | Nein | Nettobetrag, für die MWST-Abrechnung |
| Ist MWST-Zeile | Ja/Nein | Ja | automatisch erzeugte Steuerzeile |
| Projekt (`projekt_id`) | Verweis | Nein | ANF-003 |
| Text | Text (200) | Nein | abweichender Zeilentext |

## 5. Abhängigkeiten

- Hängt ab von: ANF-011, ANF-012, ANF-013, ANF-005, ANF-003
- Wird benötigt von: ANF-018 bis ANF-023

## 6. Nicht im Umfang

- Wiederkehrende (automatisch periodische) Buchungen
- Import von Buchungen aus Dateien (CSV, Bankauszug)
- Fremdwährung auf Zeilenebene im Journal (manuelle Buchungen nur in CHF)
- Beleg-Anhänge (eingescannte Quittungen)

## 7. Offene Fragen

- [x] Format der Belegnummer für manuelle Buchungen? *(Vorschlag: frei, Vorschlag `M-JJJJ-NNNN`)*
- [x] Dürfen Entwürfe von allen Benutzern gesehen und gebucht werden? *(Vorschlag: ja)*
- [x] Wird eine Eröffnungsbilanz für das erste Jahr manuell gebucht? *(Vorschlag: ja, Herkunft `eroeffnung` über Konto Eröffnungsbilanz)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261202100000_buchung.sql` | Tabellen `buchung`, `buchungszeile`, `journal_nummer`; Trigger machen gebuchte Belege unveränderlich und prüfen den Ausgleich am Ende der Transaktion (AK-15) |
| `src/backend/finance/journal/model.rs` | Datentypen, Prüfregeln, MWST-Aufteilung (`tax_split`, `expand`), Modultests |
| `src/backend/finance/journal/repository.rs`, `server_fns.rs` | Liste mit Filter und Seiten, Detail, Entwurf speichern und buchen, Löschen, Storno |
| `src/components/finance/journal/` | Liste (Sammelbuchungen klappen auf), Formular mit Zeilen und MWST-Vorschau, Detailseite, Storno- und Löschfenster |

Festlegungen bei der Umsetzung:
- **Freigabe:** Der Auftrag „Umsetzen“ gilt als Freigabe; der Status steht auf „In Umsetzung“.
- **Offene Fragen (Vorschläge übernommen):** Belegnummer `M-JJJJ-NNNN` als Vorschlag, frei änderbar. Entwürfe sehen und buchen alle angemeldeten Benutzer. Die Eröffnungsbilanz wird manuell mit Herkunft „Eröffnung“ gebucht.
- **AK-5:** Die Vorgabe ist brutto. Die MWST-Zeile liegt auf derselben Seite wie die Zeile, die sie berechnet. Netto oder brutto gilt für alle Zeilen einer Buchung.
- **AK-9:** Die Journalnummer kommt aus einem Zähler je Geschäftsjahr, der in der Buchungstransaktion gesperrt wird. Bricht eine Buchung ab, entsteht keine Lücke.
- **AK-11:** Die Gegenbuchung bekommt eine eigene Journalnummer. Das Original wird „storniert“; ein zweiter Storno ist durch einen eindeutigen Index ausgeschlossen.
- **AK-12:** Buchungen aus Ausgangsrechnungen (ANF-018) verweisen in der Detailansicht auf ihren Beleg. Der Verweis für Eingangsrechnungen folgt mit ANF-019.
- **Offen:** Der Zeilentext (`buchungszeile.text`) ist im Schema, aber noch nicht im Formular. Die Browser-Prüfung steht aus, siehe `task.md`.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-26 | Claude Code | Umgesetzt: Journal mit Zeilen, Entwurf, Buchen, Storno, Journalnummer je Geschäftsjahr, Unveränderlichkeit und Ausgleich in der Datenbank. Reste aus ANF-010, ANF-011, ANF-012, ANF-013 mit umgesetzt. |
| 2026-11-27 | Claude Code | AK-12: Verweis zum Herkunftsbeleg für Ausgangsrechnungen (ANF-018) |
