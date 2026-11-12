# ANF-002: Adressen verwalten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-002                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Adressen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-11                                        |
| **Geändert**  | 2026-11-11                                        |

## 1. Beschreibung

Das ERP führt einen zentralen Adressstamm für Firmen und Personen, zum Beispiel Kunden, Lieferanten
und Partner. Andere Module wie Projekte (ANF-003) verweisen auf diese Adressen, statt eigene
Adressdaten zu speichern.

Die Adressen lassen sich vollständig verwalten – mit den vier Grundfunktionen:

| Grundfunktion | Bedeutung |
|---------------|-----------|
| **Anlegen**    | Neue Adresse erfassen |
| **Anzeigen**   | Adressen als Liste anzeigen, suchen und eine einzelne Adresse im Detail ansehen |
| **Bearbeiten** | Bestehende Adresse ändern |
| **Löschen**    | Adresse entfernen |

Das Modul erscheint als Eintrag **„Adressen“** in der Hauptnavigation (ANF-001) unter dem Pfad `/adressen`.

## 2. Benutzergeschichte

> Als **Benutzer** möchte ich **Adressen von Firmen und Personen an einer zentralen Stelle anlegen,
> finden, ändern und löschen**, damit **alle Module mit denselben, aktuellen Adressdaten arbeiten**.

## 3. Akzeptanzkriterien

**Liste (Anzeigen)**
- [x] **AK-1:** Gegeben der Navigationseintrag „Adressen“, wenn der Benutzer ihn anklickt, dann erscheint
  eine Tabelle aller Adressen mit den Spalten Nummer, Name, PLZ, Ort, Land, E-Mail und Telefon.
- [x] **AK-2:** Gegeben die Adressliste, wenn der Benutzer einen Suchbegriff eingibt, dann zeigt die
  Liste nur Adressen, deren Name, Vorname, Ort, PLZ, E-Mail oder Nummer den Begriff enthält
  (Gross-/Kleinschreibung egal).
- [x] **AK-3:** Gegeben die Adressliste, wenn der Benutzer auf einen Spaltenkopf klickt, dann wird nach
  dieser Spalte sortiert; ein zweiter Klick kehrt die Reihenfolge um. Standard: nach Name aufsteigend.
- [x] **AK-4:** Gegeben mehr als 50 Adressen, dann wird die Liste seitenweise (50 pro Seite) angezeigt.
- [x] **AK-5:** Gegeben keine Adressen oder kein Suchtreffer, dann erscheint ein entsprechender Hinweis
  statt einer leeren Tabelle.
- [x] **AK-6:** Suche, Sortierung und Seite stehen in der Webadresse, damit Neuladen und Zurück-Taste den
  Zustand erhalten.

**Detailansicht (Anzeigen)**
- [x] **AK-7:** Gegeben die Adressliste, wenn der Benutzer eine Zeile anklickt, dann öffnet sich die
  Detailansicht unter `/adressen/<kennung>` mit allen Feldern der Adresse.

**Anlegen**
- [x] **AK-8:** Gegeben die Adressliste, wenn der Benutzer auf „Neue Adresse“ klickt, dann öffnet sich
  ein leeres Formular in einem Modal-Fenster über der Liste (Webadresse `/adressen?neu=1`).
  Schliessen über ✕, Klick daneben, Escape oder die Zurück-Taste.
- [x] **AK-9:** Gegeben das Formular, wenn der Typ „Firma“ gewählt ist, dann ist „Firmenname“ Pflicht;
  bei „Person“ sind „Vorname“ und „Nachname“ Pflicht.
- [x] **AK-10:** Gegeben ein gültig ausgefülltes Formular, wenn der Benutzer speichert, dann wird die
  Adresse mit einer automatisch vergebenen, fortlaufenden Nummer gespeichert, eine Erfolgsmeldung
  erscheint und die Detailansicht der neuen Adresse öffnet sich.
- [x] **AK-11:** Gegeben ein ungültiges Formular (Pflichtfeld leer, E-Mail-Format falsch), wenn der
  Benutzer speichert, dann wird nichts gespeichert und die fehlerhaften Felder werden mit einer
  verständlichen Meldung markiert. Die Prüfung erfolgt auf dem Server (zusätzlich gern im Browser).

**Bearbeiten**
- [x] **AK-12:** Gegeben die Detailansicht, wenn der Benutzer auf „Bearbeiten“ klickt, dann öffnet sich
  das Formular mit den aktuellen Werten in einem Modal-Fenster (Webadresse `/adressen/<kennung>?bearbeiten=1`).
- [x] **AK-13:** Gegeben geänderte, gültige Werte, wenn der Benutzer speichert, dann werden die
  Änderungen übernommen, „Geändert am“ wird aktualisiert und die Detailansicht erscheint wieder.
  Es gelten dieselben Prüfungen wie beim Anlegen.
- [x] **AK-14:** Gegeben das Formular, wenn der Benutzer auf „Abbrechen“ klickt, dann werden keine
  Änderungen gespeichert.

**Löschen**
- [x] **AK-15:** Gegeben die Detailansicht, wenn der Benutzer auf „Löschen“ klickt, dann erscheint ein
  Bestätigungsdialog mit dem Namen der Adresse. Erst nach Bestätigung wird gelöscht und die Liste
  angezeigt.
- [x] **AK-16:** Gegeben eine Adresse, die noch von einem anderen Datensatz verwendet wird (z. B. als
  Kunde eines Projekts), wenn der Benutzer sie löschen will, dann wird das Löschen verhindert und
  eine Meldung nennt, wo sie verwendet wird.

**Allgemein**
- [x] **AK-17:** Gegeben eine nicht existierende Adresskennung in der Webadresse, dann erscheint die Meldung
  „Adresse nicht gefunden“.
- [x] **AK-18:** Die Tabelle wird per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `adresse`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Nummer | Text | Ja | Fachliche Nummer, automatisch, fortlaufend, eindeutig (z. B. `A-00001`) |
| Typ | Auswahl | Ja | `Firma` oder `Person` |
| Firmenname | Text (200) | bei Firma | |
| Anrede | Auswahl | Nein | Herr, Frau, – |
| Vorname | Text (100) | bei Person | Bei Firma optional (Kontaktperson) |
| Nachname | Text (100) | bei Person | Bei Firma optional (Kontaktperson) |
| Zusatz | Text (200) | Nein | z. B. Abteilung, c/o |
| Strasse | Text (200) | Nein | inkl. Hausnummer |
| Postfach | Text (50) | Nein | |
| PLZ | Text (20) | Nein | Text, da ausländische PLZ Buchstaben enthalten |
| Ort | Text (100) | Nein | |
| Land | Text (2) | Ja | ISO-Ländercode, Standard `CH` |
| E-Mail | Text (254) | Nein | Formatprüfung |
| Telefon | Text (50) | Nein | |
| Mobiltelefon | Text (50) | Nein | |
| Webseite | Text (200) | Nein | |
| Notiz | Langtext | Nein | |
| Erstellt am | Zeitstempel | Ja | automatisch |
| Geändert am | Zeitstempel | Ja | automatisch |

**Anzeigename** (in Liste und Auswahlfeldern anderer Module): bei Firma der Firmenname, bei Person
„Nachname Vorname“.

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst, Navigation)
- Wird benötigt von: ANF-003 (Projekte: Kunde)

## 6. Nicht im Umfang

- Mehrere Adressen pro Kontakt (z. B. getrennte Liefer- und Rechnungsadresse)
- Mehrere Kontaktpersonen pro Firma als eigene Datensätze
- Rollen bzw. Kategorien (Kunde, Lieferant, …)
- Datenimport und -export (CSV, vCard), Dublettenprüfung, Adressvalidierung über externe Dienste
- Berechtigungen und Änderungsprotokoll

## 7. Offene Fragen

Für die Umsetzung wurden vorläufige Annahmen getroffen (in Klammern).

- [ ] Braucht es Rollen/Kategorien (Kunde, Lieferant, Partner) schon in der ersten Version? *(vorerst: nein)*
- [ ] Sollen Firmen mehrere Kontaktpersonen als eigene Datensätze haben? *(vorerst: nein, eine Kontaktperson in Vor-/Nachname)*
- [ ] Löschen endgültig oder nur archivieren? *(vorerst: endgültig, nach Bestätigung)*
- [ ] Format der Adressnummer? *(vorerst: `A-00001`, fortlaufend über eine Datenbank-Sequenz)*
- [ ] Welche Länder werden zur Auswahl angeboten? *(vorerst: CH, DE, AT, LI, FR, IT, NL, BE, LU, ES, GB, US – Liste in `model.rs`)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261111120000_adresse.sql` | Tabelle `adresse`, Nummern-Sequenz, Pflichtfeld-Prüfungen, berechneter Anzeigename |
| `src/backend/addresses/model.rs` | Datenmodell, Prüfregeln (Server und Browser), Länderliste + Modultests |
| `src/backend/addresses/server_fns.rs` + `src/backend/addresses/repository.rs` | Server-Funktionen: Liste, Detail, Anlegen, Bearbeiten, Löschen |
| `src/components/addresses/list.rs` | Liste mit Suche, Sortierung, Seiten (Zustand in der Webadresse) |
| `src/components/addresses/detail.rs` | Detailansicht, Löschdialog, „nicht gefunden“ |
| `src/components/addresses/form.rs` | Modal-Fenster für Anlegen und Bearbeiten |
| `src/navigation.rs`, `src/app.rs` | Navigationseintrag „Adressen“ und Routen |

Hinweise:
- **AK-11:** Die Prüfung erfolgt auf dem Server; die Datenbank sichert die Pflichtfelder zusätzlich ab.
- **AK-16:** Erkennt jeden Fremdschlüssel, der auf `adresse` zeigt. Für `projekt` lautet die Meldung
  „in Projekten (als Kunde)“; der Fremdschlüssel in ANF-003 muss dafür ohne `ON DELETE CASCADE` angelegt werden.

Geprüft: alle AK im Browser (grosser Bildschirm und Mobiltelefon, 123 Testadressen, simulierter
Projektverweis) und 13 Modultests.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-11 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-11 | Hatem Ben Belgacem | Begriffe auf Deutsch umgestellt (vorher REQ-002) |
| 2026-11-11 | Hatem Ben Belgacem | Tabellennamen in Einzahl |
| 2026-11-11 | Hatem Ben Belgacem | Umgesetzt |
| 2026-11-11 | Hatem Ben Belgacem | Eingabe in Modal-Fenstern (AK-8, AK-12) |
| 2026-11-12 | Hatem Ben Belgacem | ANF-005: Hinweis auf Benutzer, Löschschutz „als Benutzer“, Typ bleibt Person |
