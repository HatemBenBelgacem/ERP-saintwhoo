# Testszenarien – ERP-saintwhoo

Stand: 2026-11-12 · Grundlage: Akzeptanzkriterien (AK) der Anforderungen ANF-001 bis ANF-007

Dieses Dokument beschreibt, wie du von Hand prüfst, ob das ERP die Anforderungen erfüllt. Jedes Testszenario nennt das Akzeptanzkriterium, das es prüft.

## So liest du die Tabellen

| Spalte | Bedeutung |
|--------|-----------|
| **Nr** | Kennung, z. B. `TS-002-03` = Anforderung 002, Szenario 03 |
| **AK** | Nummer des Akzeptanzkriteriums. `–` = Zusatzprüfung aus dem Abschnitt „Daten / Felder“ |
| **Schritte** | Was du tust |
| **Erwartet** | Was stimmen muss. Nur wenn alles stimmt, ist das Szenario bestanden |

Kennzeichen in den Schritten:
- **DB** = Prüfung direkt in der Datenbank, z. B. mit `psql`. Nur in der Testdatenbank ausführen.
- **Mobiltelefon** = Fensterbreite 390 px.
- **grosser Bildschirm** = Fensterbreite 1280 px.

Schlägt ein Szenario fehl, trage es in [task.md](task.md) als offene Aufgabe ein, mit der Nummer, z. B. „TS-002-05 schlägt fehl“.

## Übersicht

| Anforderung | Titel | Szenarien | Hinweis |
|-------------|-------|-----------|---------|
| ANF-001 | Grundgerüst | 12 | |
| ANF-002 | Adressen | 19 | |
| ANF-003 | Projekte | 25 | |
| ANF-004 | Einkauf | 29 | |
| ANF-005 | Benutzer | 35 | Bedienung im Browser noch nicht von Hand geprüft |
| ANF-006 | Aufgaben | 21 | Bedienung im Browser noch nicht von Hand geprüft |
| ANF-007 | Anmeldung | 29 | Bedienung im Browser noch nicht von Hand geprüft |

Die Anforderungen ANF-005 bis ANF-007 haben bisher keine Prüfung im Browser. Mit diesen Szenarien starten.

## Vorbereitung

1. Eine leere Datenbank `erp_saintwhoo` anlegen und `.env` ausfüllen (Vorlage: `.env.example`). Die Migrationen laufen beim Serverstart automatisch.
2. Den Server mit `cargo leptos watch` starten und `http://127.0.0.1:3002` öffnen.
3. Zwei Browser-Fenster bereithalten, eines normal und eines privat. So sind zwei Anmeldungen gleichzeitig möglich, z. B. für TS-007-21.
4. Die Testdaten in dieser Reihenfolge anlegen. Die Namen stammen aus den Beispielen von ANF-005:
   - Adressen vom Typ Person: **Anna Muster**, **Ben Beispiel**, **Carla Probe**, **Dora Demo** (Dora bleibt ohne Benutzer, für TS-005-10).
   - Eine Firma: **Musterfirma AG**.
   - 123 weitere Adressen, damit die Seitenprüfung funktioniert (ANF-002 AK-4).
   - 61 Projekte insgesamt (ohne P-2026-001 und P-2026-002 mitgezählt, für TS-001-02 und TS-003-05).
   - 51 Einkaufsaufträge, damit die Seitenprüfung funktioniert (für TS-004-03).
   - Benutzer **anna**, **ben** und **carla**, alle aktiv. Die Benutzer haben je ein eigenes Testpasswort mit mindestens 10 Zeichen. Schreib die Passwörter nicht in dieses Dokument.
   - Projekt **P-2026-001**, Kunde **Musterfirma AG**, verantwortlich **Anna**. Struktur:
     - Teilprojekt 1 (ohne Verantwortliche) mit Vorgang 1.1 und Aufgabe 1.1.1 (offen).
     - Teilprojekt 2 (verantwortlich **Ben**):
       - Vorgang 2.1 (ohne Verantwortliche) mit Aufgabe 2.1.1 (offen).
       - Vorgang 2.2 (ohne Verantwortliche) mit Aufgaben 2.2.1 (erledigt) und 2.2.2 (offen).
       - Vorgang 2.3 (verantwortlich **Carla**) mit Aufgabe 2.3.1 (offen).
   - Projekt **P-2026-002** ohne Kunde und ohne Verantwortliche.
   - Ein Einkaufsauftrag im Status Entwurf und einer im Status Bestellt, beide dem Projekt P-2026-001 zugeordnet. Die Lieferanten sind Adressen aus der Liste.
5. Für die Ersteinrichtung (TS-007-26) brauchst du eine zweite, leere Datenbank.

## ANF-001 Grundgerüst

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-001-01 | 1 | Startseite `/` aufrufen. | Kopfzeile oben, Seitenleiste links und Inhalt rechts sind sichtbar. |
| TS-001-02 | 2 | Projektübersicht mit 61 Projekten auf dem grossen Bildschirm öffnen und nach unten scrollen. | Kopfzeile und Seitenleiste bleiben stehen. Nur der Inhalt bewegt sich. |
| TS-001-03 | 3 | In der Seitenleiste auf „Adressen“ klicken. | Die Adressliste erscheint. Die Seitenleiste wird nicht neu aufgebaut und flackert nicht. |
| TS-001-04 | 4 | Auf einer geöffneten Seite den Eintrag der Seite ansehen. | Der aktive Eintrag hat einen leichten Glutorange-Hintergrund mit dunkel-orangefarbener Schrift. Andere Einträge sind nicht markiert. |
| TS-001-05 | 6 | Seitenleiste ansehen. | „Einstellungen“ steht getrennt am unteren Rand der Seitenleiste. |
| TS-001-06 | 7 | In der Fusszeile der Seitenleiste auf „Einstellungen“ klicken. | Die Seitenleiste zeigt die Einstellungsnavigation mit „Allgemein“ und „Benutzer“. Im Inhalt steht „Allgemein“. |
| TS-001-07 | 8 | In der Einstellungsnavigation auf „Zurück“ klicken. | Die Hauptnavigation erscheint und im Inhalt die Startseite. |
| TS-001-08 | 9 | Webadresse `/einstellungen/benutzer` direkt aufrufen. Dann die Seite neu laden. | Die Einstellungsnavigation bleibt sichtbar. Der Eintrag „Benutzer“ ist markiert. |
| TS-001-09 | 10 | Auf dem Mobiltelefon öffnen. Dann das Menü-Symbol in der Kopfzeile antippen und nochmals antippen. | Die Seitenleiste ist zuerst ausgeblendet. Das Symbol blendet sie ein und wieder aus. |
| TS-001-10 | 11 | Auf dem Mobiltelefon die Seitenleiste öffnen und „Adressen“ wählen. | Die Seite lädt und die Seitenleiste schliesst sich von selbst. |
| TS-001-11 | 12 | Startseite öffnen. | Eine Karte pro Modul: Adressen, Projekte, Aufgaben, Einkauf. Jede Karte hat Symbol, Titel und Kurzbeschreibung. Ein Klick auf eine Karte öffnet das Modul. |
| TS-001-12 | 12 | Startseite öffnen. Dann den Datenbankdienst kurz stoppen und die Seite neu laden. Danach den Dienst wieder starten. | Die Zeile unter den Karten zeigt den Datenbankstatus. Bei gestopptem Dienst steht dort „nicht verbunden“. Nach dem Start steht dort „verbunden“. |

Hinweis: AK-5 (Navigation an einer zentralen Stelle definiert) wird im Code geprüft, nicht von Hand. Die Modultests in `src/navigation.rs` decken das ab.

## ANF-002 Adressen

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-002-01 | 1 | Seitenleiste „Adressen“ anklicken. | Tabelle mit den Spalten Nummer, Name, PLZ, Ort, Land, E-Mail, Telefon. |
| TS-002-02 | 2 | Suche nach einem Teil der PLZ, dann nach einem Teil des Orts in Kleinbuchstaben, dann nach einem Teil der E-Mail-Adresse, dann nach dem Vornamen. | Nur passende Adressen erscheinen. Gross- und Kleinschreibung spielen keine Rolle. |
| TS-002-03 | 3 | Auf den Spaltenkopf „Ort“ klicken, dann nochmals. | Erst aufsteigend, dann absteigend. Standard ist Name aufsteigend. |
| TS-002-04 | 4 | Bei 123 Adressen durch die Seiten blättern. | 50 Adressen pro Seite, insgesamt 3 Seiten (50, 50, 23). |
| TS-002-05 | 5 | Suche nach `xyz123` eingeben. | Ein Hinweis erscheint. Es ist keine leere Tabelle zu sehen. |
| TS-002-06 | 6 | Suchbegriff, Sortierung und Seite 2 einstellen. Die Seite neu laden. Danach die Zurück-Taste drücken. | Suche, Sortierung und Seite bleiben erhalten. Sie stehen in der Webadresse. |
| TS-002-07 | 7 | Eine Zeile anklicken. | Die Detailansicht öffnet sich unter `/adressen/<Kennung>` mit allen Feldern. |
| TS-002-08 | 8 | „Neue Adresse“ klicken. Dann das Fenster auf vier Wegen schliessen: ✕, Klick daneben, Escape und Zurück-Taste. | Das Modal-Fenster liegt über der Liste, Webadresse `/adressen?neu=1`. Es schliesst sich auf jedem der vier Wege. Nichts wird gespeichert. |
| TS-002-09 | 9 | Typ „Firma“ wählen, Firmenname leer lassen, speichern. Dann Typ „Person“ wählen, Vorname und Nachname leer lassen, speichern. | Jedes Mal wird das Pflichtfeld markiert. Nichts wird gespeichert. |
| TS-002-10 | 10 | Gültige Firma „Musterfirma AG“ speichern (Land bleibt auf Standard CH). Danach eine zweite Adresse speichern. | Erfolgsmeldung. Die Detailansicht öffnet sich mit einer Nummer wie `A-00001`. Die zweite Nummer ist um eins höher. |
| TS-002-11 | 11 | E-Mail `max.muster@` eingeben, ein Pflichtfeld leer lassen, speichern. | Nichts wird gespeichert. Die Felder sind mit einer verständlichen Meldung markiert. |
| TS-002-12 | 12 | In der Detailansicht auf „Bearbeiten“ klicken. | Das Modal-Fenster zeigt die aktuellen Werte. Webadresse `…?bearbeiten=1`. |
| TS-002-13 | 13 | Ort ändern und speichern. Danach eine ungültige E-Mail eintragen und speichern. | Die Änderung ist sichtbar und „Geändert am“ ist neu. Die ungültige E-Mail wird wie beim Anlegen abgelehnt. |
| TS-002-14 | 14 | Ort ändern und auf „Abbrechen“ klicken. | Die alten Werte bleiben stehen. |
| TS-002-15 | 15 | Auf „Löschen“ klicken. Im Dialog auf „Abbrechen“ klicken. Dann erneut löschen und bestätigen. | Der Dialog nennt den Namen der Adresse. Nach Abbrechen ist sie noch da. Nach Bestätigen ist sie weg und die Liste erscheint. |
| TS-002-16 | 16 | Die Adresse „Musterfirma AG“ löschen (sie ist Kunde von P-2026-001). | Das Löschen wird verhindert. Die Meldung nennt „in Projekten (als Kunde)“. |
| TS-002-17 | 17 | Webadresse `/adressen/99999` aufrufen. | Die Meldung „Adresse nicht gefunden“ erscheint. |
| TS-002-18 | 18 | DB: `\d adresse` in `psql` ausführen. | Die Tabelle `adresse` ist vorhanden. Die Migration ist gelaufen. |
| TS-002-19 | – | Adresse „Muster Anna“ und die Firma „Musterfirma AG“ in der Liste ansehen. | Die Person heisst „Muster Anna“ (Nachname Vorname). Bei der Firma steht der Firmenname. |

## ANF-003 Projekte

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-003-01 | 1 | Seitenleiste „Projekte“ anklicken. | Tabelle mit den Spalten Nummer, Name, Kunde, Status, Start, Ende, Fortschritt. |
| TS-003-02 | 1a | Nach P-2026-001 suchen. Die Struktur darunter ansehen. Ein Teilprojekt zuklappen. Dann „Alle aufklappen“ und „Alle zuklappen“ klicken. | Teilprojekte, Vorgänge und Aufgaben sind nach Ebene eingerückt. Jede Ebene klappt auf und zu. Die Schaltflächen wirken auf alle Ebenen. |
| TS-003-03 | 1a | In der Übersicht eine Aufgabe abhaken. Dann in einer Zeile „+“ und danach „✎“ klicken. | Das Abhaken wirkt sofort. „+“ öffnet das Anlegen-Fenster, „✎“ das Bearbeiten-Fenster. |
| TS-003-04 | 2 | Nach dem Kunden „Musterfirma“ suchen. Dann nach Status „Abgeschlossen“ filtern. Dann die Spalte „Name“ sortieren. Danach die Standardansicht ohne Filter laden. | Suche, Filter und Sortierung wirken. Die Standardansicht zeigt nur Projekte mit Status „Aktiv“, neueste Nummer zuerst. |
| TS-003-05 | 3 | Mit 61 Projekten auf Seite 2 wechseln. Dann einen Filter setzen und die Seite neu laden. | 50 Projekte pro Seite. Filter und Seite stehen in der Webadresse und bleiben nach dem Neuladen erhalten. |
| TS-003-06 | 4 | Eine Projektzeile anklicken. | Die Projektansicht öffnet sich unter `/projekte/<Kennung>`. Darunter steht der Strukturbaum. |
| TS-003-07 | 5 | P-2026-001 im Strukturbaum ansehen. Das Projekt hat 5 Aufgaben, davon 1 erledigt. | Der Fortschritt zeigt 20 % (1 von 5). Jede Ebene zeigt, wie viele Elemente sie untergeordnet hat. |
| TS-003-08 | 5 | Vorgang 1.1 mit einer Aufgabe ansehen, dann einen Vorgang ohne Aufgaben. | Vorgang ohne Aufgaben zeigt „– (keine Aufgaben)“ statt eines Prozentwerts. |
| TS-003-09 | 5 | Vorgang mit 3 Aufgaben anlegen, davon 2 erledigt. | Der Fortschritt zeigt 66 % (2 von 3, abgerundet). |
| TS-003-10 | 6 | In einem Vorgang auf ein Glied der Pfadleiste klicken. Dann eine Aufgabe im Strukturbaum anklicken. | Die Pfadleiste zeigt „Projekt › Teilprojekt › Vorgang › Aufgabe“. Jedes Glied ist anklickbar. Die Aufgabe öffnet ihre Detailansicht. |
| TS-003-11 | 7 | „Neues Projekt“ klicken. Name „Testprojekt Nord“ und Kunde wählen. Speichern. | Das Modal-Fenster liegt über der Liste, Webadresse `?neu=projekt`. Danach erscheint die Projektansicht. Die Nummer folgt dem Muster `P-2026-…` und ist fortlaufend. Der Status ist „Aktiv“. |
| TS-003-12 | 8 | In P-2026-001 die Schaltflächen „Neues Teilprojekt“, „Neuer Vorgang“ und „Neue Aufgabe“ nacheinander nutzen und jeweils speichern. | Das Fenster schliesst sich. Der Benutzer bleibt auf derselben Seite. Die Struktur zeigt das neue Element. |
| TS-003-13 | 9 | Neue Aufgabe unter Vorgang 2.2 anlegen. | Die Aufgabe bekommt die Nummer 2.2.3 und steht am Ende der Reihenfolge. |
| TS-003-14 | 9 | Neuen Vorgang unter Teilprojekt 2 anlegen. | Der Vorgang bekommt die Nummer 2.4 und steht am Ende. |
| TS-003-15 | 10 | Projekt ohne Name speichern. Dann ein Projekt mit Ende vor Start speichern. | Nichts wird gespeichert. Die Felder sind mit einer Meldung markiert. |
| TS-003-16 | 11 | Vorgang 2.3 mit ✎ bearbeiten. Name ändern und speichern. Dann eine zweite Änderung mit „Abbrechen“ verwerfen. | Die erste Änderung ist gespeichert. Die zweite nicht. |
| TS-003-17 | 11a | „Neue Aufgabe“ öffnen. Das Formular ansehen. Dann den Titel leer lassen und speichern. | Das Formular hat nur „Titel“ (Pflicht) und „Beschreibung“ (optional). Ohne Titel wird nichts gespeichert. |
| TS-003-18 | 5a | Übersicht und Strukturbaum ansehen. | Projekt ist Glutorange, Teilprojekt Blau, Vorgang Petrol und Aufgabe Schiefergrau. Jede Ebene hat einen farbigen Streifen links. Über der Übersicht steht eine Legende. |
| TS-003-19 | 12 | Im Strukturbaum das Kontrollkästchen „Erledigt“ einer Aufgabe setzen. | Der Status ändert sich sofort. Das Formular öffnet sich nicht. Der Fortschritt des Projekts ändert sich. |
| TS-003-20 | 13 | Vorgang 2.2 nach oben verschieben. | Vorgang 2.2 wird zu 2.1. Seine Aufgaben tragen die neuen Nummern. |
| TS-003-21 | 14 | Teilprojekt 2 löschen, aber im Dialog abbrechen. Dann erneut löschen. | Der Dialog nennt die Zahl der Teilprojekte, Vorgänge und Aufgaben darunter. Die Zahlen stimmen mit dem Strukturbaum überein. Nach Abbrechen bleibt alles. |
| TS-003-22 | 15 | Das Löschen bestätigen. | Teilprojekt, Vorgänge und Aufgaben sind weg. Die übergeordnete Ansicht erscheint. DB: In den Tabellen `vorgang` und `aufgabe` bleiben keine Zeilen dieser Vorgänge. |
| TS-003-23 | 16 | Kunden-Suche im Projektformular nutzen. Dann die Adresse „Musterfirma AG“ löschen. | Die Suche findet die Adresse. Das Löschen wird verhindert (siehe TS-002-16). |
| TS-003-24 | 17 | Webadressen `/projekte/99999` und `/projekte/teilprojekt/99999` aufrufen. | Jede Seite meldet, dass das Element nicht gefunden wurde. |
| TS-003-25 | 18 | DB: Ein Teilprojekt ohne `projekt_id` einfügen. | Die Datenbank lehnt den Eintrag ab. Das Projekt ist Pflicht. |

## ANF-004 Einkauf

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-004-01 | 1, 5 | Seitenleiste „Einkauf“ anklicken. Einen Auftrag mit Status Bestellt und einem Liefertermin in der Vergangenheit ansehen. | Die Tabelle hat die Spalten Nummer, Lieferant, Projekt, Bestelldatum, Liefertermin, Status und Gesamtbetrag. Der überfällige Liefertermin ist markiert. |
| TS-004-02 | 2 | Nach dem Lieferanten, dem Projekt und der Referenz des Lieferanten suchen. Dann nach Status filtern. Ohne Filter die Standardansicht laden. | Die Suche findet die Aufträge. Die Standardansicht zeigt nur offene Status (Entwurf, Bestellt, Teilweise geliefert), neueste Nummer zuerst. |
| TS-004-03 | 3 | Mit 51 Aufträgen auf Seite 2 wechseln. | 50 Aufträge pro Seite. Suche, Filter und Seite stehen in der Webadresse. |
| TS-004-04 | 4 | Eine Suche ohne Treffer eingeben. Dann ohne Aufträge die Liste ansehen. | Ein Hinweis erscheint statt einer leeren Tabelle. |
| TS-004-05 | 6 | Einen Auftrag anklicken. | Die Detailansicht öffnet sich unter `/einkauf/<Kennung>`. Sie zeigt die Kopfdaten, den Lieferanten und das Projekt als Verweise sowie die Positionstabelle. |
| TS-004-06 | 7 | Auftrag mit zwei Positionen anlegen: Position 1 ist Menge 3, Einzelpreis 100.00, Rabatt 10 %, MWST 8.1 %. Position 2 ist Menge 2, Einzelpreis 50.00, Rabatt 0 %, MWST 2.6 %. Währung CHF. | Position 1 ergibt 270.00 und Position 2 ergibt 100.00. Zwischensumme 370.00. MWST 8.1 % = 21.87 und MWST 2.6 % = 2.60. Der Gesamtbetrag ist 394.47 vor der Rundung und 394.45 nach der CHF-Rundung auf 5 Rappen. |
| TS-004-07 | 8, 9 | „Neuer Einkaufsauftrag“ öffnen. Speichern ohne Lieferant. Dann den Lieferanten über die Suche wählen, einmal nach Name und einmal nach Adressnummer. | Das Modal-Fenster liegt über der Liste, Webadresse `?neu=1`. Ohne Lieferant wird nichts gespeichert. Beide Suchen finden die Adresse. Das Projekt bleibt optional. |
| TS-004-08 | 10 | Gültiges Formular speichern. | Der Status ist Entwurf. Die Nummer folgt dem Muster `E-2026-…` und ist fortlaufend. Die Detailansicht öffnet sich. |
| TS-004-09 | 11 | Im Entwurf „Neue Position“ öffnen, Webadresse `…?neu=position`. Gültige Werte speichern. | Die Position erscheint in der Tabelle. Die Summen aktualisieren sich. |
| TS-004-10 | 11, 12 | Neue Position mit leerer Bezeichnung speichern. Dann Menge 0, dann Einzelpreis −5, dann Rabatt 101 eingeben, jeweils speichern. | Jeder Fehler wird am betroffenen Feld mit einer Meldung angezeigt. Nichts wird gespeichert. |
| TS-004-11 | 13 | Im Entwurf die Kopfdaten ändern, eine Position hinzufügen, eine löschen und eine verschieben. | Alle vier Änderungen sind möglich. Die Reihenfolge stimmt danach. |
| TS-004-12 | 14 | Entwurf ohne Positionen öffnen und auf „Bestellen“ klicken. | Das Bestellen ist nicht möglich. Der Status bleibt Entwurf. |
| TS-004-13 | 14 | Entwurf mit einer Position und leerem Bestelldatum bestellen. | Der Status ist Bestellt. Das Bestelldatum ist heute, also 2026-11-12. |
| TS-004-14 | 14 | Entwurf mit einer Position anlegen. Liefertermin auf 2026-11-10 setzen. Dann bestellen. | Das Bestellen wird verhindert. Eine Meldung erscheint. |
| TS-004-15 | 15 | Auftrag im Status Bestellt öffnen. Versuchen, Lieferant, Positionen, Mengen und Preise zu ändern. Dann Liefertermin, Referenz und Bemerkung ändern. | Lieferant, Positionen, Mengen und Preise sind gesperrt. Liefertermin, Referenz und Bemerkung lassen sich ändern. |
| TS-004-16 | 16 | Bei Position 1 die gelieferte Menge 2 erfassen und speichern. Danach −1 eingeben. | Die Menge 2 wird gespeichert. −1 wird abgelehnt. |
| TS-004-17 | 17 | Nach TS-004-16 den Status ansehen. Dann bei allen Positionen die volle Menge erfassen. | Nach der ersten Erfassung ist der Status „Teilweise geliefert“. Nach der vollen Erfassung ist er „Geliefert“. |
| TS-004-18 | 18 | Bei einer Position mit bestellten 3 eine gelieferte Menge 5 erfassen. | Ein Hinweis zur Überlieferung erscheint. Das Speichern ist trotzdem erlaubt. |
| TS-004-19 | 16, 17 | Bestellten Auftrag öffnen und „Alles geliefert“ klicken. | Jede Position hat die volle Liefermenge. Der Status ist Geliefert. Überlieferungen bleiben erhalten. |
| TS-004-20 | 19 | Entwurf öffnen, „Stornieren“ klicken und bestätigen. | Der Status ist Storniert. Der Auftrag bleibt sichtbar, lässt sich aber nicht mehr bearbeiten. |
| TS-004-21 | 19 | Auftrag im Status Teilweise geliefert öffnen. | „Stornieren“ wird nicht angeboten. |
| TS-004-22 | 20 | Entwurf öffnen, „Löschen“ klicken. Im Dialog die Nummer prüfen, dann bestätigen. | Auftrag und alle Positionen sind weg. DB: Die Tabelle `einkaufsposition` hat keine Zeilen mehr für diesen Auftrag. |
| TS-004-23 | 20 | Bestellten Auftrag öffnen. | „Löschen“ wird nicht angeboten. Nur „Stornieren“ ist möglich. |
| TS-004-24 | 21 | Die Adresse eines Lieferanten löschen. | Das Löschen wird verhindert. Die Meldung nennt „in Einkaufsaufträgen (als Lieferant)“. |
| TS-004-25 | 22 | Projekt P-2026-001 öffnen. | Ein Abschnitt „Einkaufsaufträge“ listet die Aufträge mit Nummer, Lieferant, Status und Gesamtbetrag. |
| TS-004-26 | 23 | Projekt P-2026-001 löschen. | Das Löschen wird verhindert. Die Meldung nennt die Einkaufsaufträge. |
| TS-004-27 | 24 | Webadresse `/einkauf/99999` aufrufen. | Die Meldung „Einkaufsauftrag nicht gefunden“ erscheint. |
| TS-004-28 | 25 | Das Beispiel aus TS-004-06 mit der Währung EUR anlegen. | Der Gesamtbetrag ist 394.47. Es gibt keine CHF-Rundung. |
| TS-004-29 | 26 | DB: `\d+ einkaufsauftrag` ausführen. | Die Fremdschlüssel auf `adresse` und `projekt` haben keine Kaskade (`ON DELETE CASCADE` fehlt). |

## ANF-005 Benutzer

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-005-01 | 1, 4 | Einstellungen → „Benutzer“ öffnen. | Die Tabelle hat die Spalten Benutzername, Name, E-Mail, Status und Letzte Anmeldung. Standard ist Benutzername aufsteigend. |
| TS-005-02 | 2 | Nach „anna“, nach „Muster“ und nach einem Teil der E-Mail-Adresse suchen. | Die Treffer stimmen. Gross- und Kleinschreibung spielen keine Rolle. |
| TS-005-03 | 3 | Nach „alle“, „aktiv“ und „gesperrt“ filtern. | Die Liste passt zur Auswahl. |
| TS-005-04 | 4, 5 | Auf den Spaltenkopf „Status“ zweimal klicken. Dann nach „xyz“ suchen. Danach die Seite neu laden. | Die Sortierung kehrt sich um. Ohne Treffer erscheint ein Hinweis. Der Zustand bleibt nach dem Neuladen erhalten. |
| TS-005-05 | 6 | Eine Zeile anklicken. | Die Detailansicht unter `/einstellungen/benutzer/<Kennung>` zeigt Benutzername, Status, Letzte Anmeldung, Erstellt am, Geändert am und die Adressdaten. |
| TS-005-06 | 7 | Im Benutzer-Detail den Verweis auf die Adresse anklicken. | Die Adresse öffnet sich unter `/adressen/<Kennung>`. |
| TS-005-07 | 8 | Die Adresse von Anna öffnen. | Ein Hinweis „Benutzer: anna“ erscheint mit einem Verweis auf den Benutzer. |
| TS-005-08 | 9, 10 | „Neuer Benutzer“ öffnen. In der Adress-Suche nach „Musterfirma“ suchen, dann nach „Muster“. | Das Modal-Fenster liegt über der Liste, Webadresse `?neu=1`. Die Firma wird nicht angeboten. Anna wird nicht angeboten, weil sie schon einen Benutzer hat. |
| TS-005-09 | 11 | Das leere Formular speichern. | Benutzername, Adresse, Passwort und Wiederholung sind Pflicht. Der Status „aktiv“ ist vorausgewählt. |
| TS-005-10 | 12 | Benutzername „dora.demo“, Adresse „Demo Dora“, gültiges Passwort mit Wiederholung. Speichern. | Die Erfolgsmeldung erscheint. Die Detailansicht des neuen Benutzers öffnet sich. |
| TS-005-11 | 13 | Benutzername nacheinander eingeben: „dora demo“ (mit Leerzeichen), „dora!“, 51 Zeichen, leer, „Dora“ (Grossbuchstabe, siehe Offene Punkte), und „anna“ bei der Adresse von Dora. | Jeder Fehler wird am Feld mit einer Meldung markiert. „anna“ wird als bereits vergeben abgelehnt. Nichts wird gespeichert. |
| TS-005-12 | 13 | Ohne Adresse speichern. Dann prüfen, ob die Adress-Suche die Firma „Musterfirma AG“ oder Anna anbietet. Dann ein Passwort mit 9 Zeichen eingeben. Dann eine Wiederholung eingeben, die nicht passt. | Die Adresse ist Pflicht. Die Firma und Anna werden nicht angeboten. Falls die Auswahl sie trotzdem zulässt, lehnt der Server sie ab. Das kurze Passwort und die ungleiche Wiederholung werden abgelehnt. Die Meldungen nennen den Grund. Nichts wird gespeichert. |
| TS-005-13 | 14 | Einen Benutzer mit „Bearbeiten“ öffnen. | Benutzername, Adresse und Status lassen sich ändern. Passwortfelder fehlen. Webadresse `…?bearbeiten=1`. |
| TS-005-14 | 15 | „Passwort setzen“ öffnen, Webadresse `…?passwort=1`. Ein neues Passwort mit 10 Zeichen und Wiederholung eingeben. Speichern. Danach mit dem alten und dem neuen Passwort anmelden. | Das neue Passwort funktioniert. Das alte wird abgelehnt. |
| TS-005-15 | 16 | Carla auf „gesperrt“ setzen. Dann die Liste und die Detailansicht ansehen. | Carla bleibt mit allen Daten erhalten und ist als „gesperrt“ markiert. Die Anmeldung ist nicht mehr möglich (siehe TS-007-09). |
| TS-005-16 | 17 | Einen Benutzer bearbeiten und einen ungültigen Benutzernamen eingeben. Speichern. Dann eine zweite Änderung mit „Abbrechen“ verwerfen. | Die Prüfung ist dieselbe wie beim Anlegen. „Abbrechen“ speichert nichts. |
| TS-005-17 | 18 | „Löschen“ klicken und im Dialog abbrechen. Dann erneut löschen und bestätigen. | Der Dialog nennt Benutzername und Name. Nach Abbrechen bleibt der Benutzer. Nach Bestätigen ist er weg. Die Adresse „Demo Dora“ bleibt erhalten. |
| TS-005-18 | 19 | Die Adresse von Ben in der Adressverwaltung löschen. | Das Löschen wird verhindert. Die Meldung nennt „als Benutzer ben“. |
| TS-005-19 | 20 | Die Adresse von Ben auf „Firma“ ändern und speichern. | Die Änderung wird verhindert. Die Meldung nennt den Benutzer. |
| TS-005-20 | 21 | DB: `SELECT benutzername, passwort_hash FROM benutzer;` ausführen. Dann die Antworten der Liste und der Detailansicht im Browser-Netzwerkbereich prüfen. | Der Hash beginnt mit `$argon2id$`. Kein Klartext. Keine Antwort enthält den Hash. |
| TS-005-21 | 22 | Webadresse `/einstellungen/benutzer/99999` aufrufen. | Die Meldung „Benutzer nicht gefunden“ erscheint. |
| TS-005-22 | 23 | DB: `\d benutzer` ausführen. | Die Tabelle `benutzer` ist vorhanden. `adresse_id` ist eindeutig. Der Benutzername ist ohne Beachtung der Gross- und Kleinschreibung eindeutig. |
| TS-005-23 | 24 | Projekt-Formular öffnen. Das Feld „Verantwortlich“ ansehen. Carla (gesperrt) suchen. Dann das Feld leeren und speichern. | Nur aktive Benutzer werden angeboten. Carla ist nicht wählbar. Das Feld lässt sich leeren. |
| TS-005-24 | 25 | Benutzer-Detail von Anna öffnen, Abschnitt „Verantwortung“. | Projekt P-2026-001 ist gelistet. Anna hat 4 offene Aufgaben. Alle Aufgaben im Projekt gehören zu ihr. |
| TS-005-25 | 26 | Benutzer-Detail von Ben öffnen, Abschnitt „Verantwortung“. | Teilprojekt 2 ist gelistet. Ben hat 3 offene Aufgaben (2.1.1, 2.2.2, 2.3.1). Teilprojekt 1 ist nicht gelistet. |
| TS-005-26 | 27 | Benutzer-Detail von Carla öffnen, Abschnitt „Verantwortung“. | Vorgang 2.3 ist gelistet. Carla hat 1 offene Aufgabe (2.3.1). Die Vorgänge 2.1 und 2.2 sind nicht gelistet. |
| TS-005-27 | 28 | Vorgang 2.3 öffnen und die Verantwortlichen ansehen. | Es steht „Anna – vom Projekt“, „Ben – vom Teilprojekt 2“ und „Carla – hier hinterlegt“. |
| TS-005-28 | 28 | Anna zusätzlich bei Teilprojekt 2 hinterlegen. Dann Teilprojekt 2 öffnen. Danach die Verantwortung von Teilprojekt 2 wieder auf Ben setzen. | Anna erscheint nur einmal. Die Herkunft ist die oberste Ebene: „vom Projekt“. |
| TS-005-29 | 28 | P-2026-002 öffnen. | Unter „Verantwortlich“ steht „–“. |
| TS-005-30 | 29 | Die Verantwortung von P-2026-001 von Anna auf Ben ändern. Dann unter Vorgang 2.1 eine neue Aufgabe anlegen. Danach Ben's Benutzer-Detail öffnen. | Die neue Aufgabe zählt bei Ben sofort mit, ohne dass sie einzeln zugewiesen wurde. Danach die Verantwortung zurücksetzen. |
| TS-005-31 | 30 | Strukturbaum von P-2026-001 ansehen. | Teilprojekt 2 zeigt „Ben“. Vorgang 2.1 zeigt niemanden. Der geerbte Anna wird dort nicht angezeigt. |
| TS-005-32 | 31 | Benutzer ohne Verantwortung öffnen. Dann Anna öffnen. | Ohne Verantwortung erscheint ein Hinweis statt einer leeren Tabelle. Bei Anna sind Projekt und offene Aufgaben mit Verweisen gelistet. |
| TS-005-33 | 32 | Ben löschen. | Das Löschen wird verhindert. Die Meldung zählt je Ebene: 1 Teilprojekt. Sie empfiehlt, den Benutzer zu sperren oder die Verantwortung neu zu vergeben. |
| TS-005-34 | 33 | Carla bleibt bei Vorgang 2.3 hinterlegt. Im Strukturbaum ansehen. Dann ein neues Element anlegen und Carla im Feld suchen. | Der Strukturbaum markiert Carla als „gesperrt“. Im neuen Element ist sie nicht wählbar. |
| TS-005-35 | 34 | DB: `\d projekt`, `\d teilprojekt` und `\d vorgang` ausführen. | Die Spalten `benutzer_id` sind vorhanden. Sie zeigen auf die Tabelle `benutzer`. |

## ANF-006 Aufgaben

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-006-01 | 1 | Hauptnavigation ansehen. Die Startseite ansehen. | Der Eintrag „Aufgaben“ hat ein eigenes Symbol. Auf der Startseite gibt es eine Karte dafür. |
| TS-006-02 | 2, ANF-007 AK-19 | Als Anna `/aufgaben` ohne Parameter öffnen. Dann bei der Benutzer-Auswahl Ben suchen und wählen. | Ohne Parameter zeigt die Liste die Aufgaben von Anna. Nach der Wahl von Ben wechselt die Liste. Die Webadresse enthält `?benutzer=<Kennung>`. |
| TS-006-03 | 3 | Die Liste von Anna, Ben und Carla ansehen. | Anna sieht alle 5 Projektaufgaben von P-2026-001, Herkunft „vom Projekt“. Ben sieht 4 Aufgaben aus Teilprojekt 2, Herkunft „vom Teilprojekt 2“. Carla sieht nur 2.3.1, Herkunft „vom Vorgang 2.3“. |
| TS-006-04 | 4 | Die Spalten der Liste ansehen. | Die Spalten sind Kontrollkästchen „Erledigt“, Titel, Projekt, Nummer im Projekt, Herkunft und Geändert am. Projektaufgaben und Aufgaben ohne Projekt sind mit einer Plakette unterscheidbar. |
| TS-006-05 | 5 | Standardansicht prüfen (offen, alle Arten). Dann Status „erledigt“ wählen. Dann Art „ohne Projekt“ wählen. Dann nach dem Projektnamen suchen. | Standard zeigt nur offene Aufgaben. Jeder Filter wirkt. Die Suche findet Titel, Beschreibung, Projektnummer und Projektname. Die Filter stehen in der Webadresse. |
| TS-006-06 | 6 | Die Sortierung der Liste mit allen Aufgaben von Anna ansehen. | Aufgaben ohne Projekt stehen zuerst, neueste zuerst. Danach folgen die Projektaufgaben nach Projektnummer und Nummer im Projekt. |
| TS-006-07 | 7 | Einen Filter setzen, der nichts trifft. | Ein Hinweis erscheint statt einer leeren Tabelle. |
| TS-006-08 | 8 | Das Kontrollkästchen bei einer Projektaufgabe klicken, dann bei einer Aufgabe ohne Projekt. Danach jeweils erneut klicken. | Der Status wechselt sofort. Die Liste lädt neu. Der Strukturbaum des Projekts zeigt denselben Stand. |
| TS-006-09 | 9 | Den Titel einer Projektaufgabe anklicken. Dann den Titel einer Aufgabe ohne Projekt anklicken. | Die Projektaufgabe öffnet `/projekte/aufgabe/<Kennung>`. Die andere öffnet `/aufgaben/<Kennung>`. |
| TS-006-10 | 10 | „Neue Aufgabe“ klicken. | Das Modal-Fenster liegt über der Liste, Webadresse `…?neu=1`. Zuständig ist mit Anna vorbelegt. Es gibt Felder für Titel, Beschreibung und Zuständig. Ein Projektfeld fehlt. |
| TS-006-11 | 11 | Titel „Offerte nachfassen“, Beschreibung ausfüllen, Zuständig Ben. Speichern. | Das Fenster schliesst sich. Die Liste zeigt die Aufgabe mit Status offen. Bei Ben steht „ohne Projekt“. |
| TS-006-12 | 12 | Speichern mit leerem Titel. Dann mit einem Titel von 201 Zeichen. Dann mit leerem Zuständig. | Jeder Fall wird abgelehnt. Die Felder sind mit einer Meldung markiert. Nichts wird gespeichert. |
| TS-006-13 | 13 | Im Projekt bei Vorgang 2.1 eine neue Aufgabe anlegen. Dann den Aufgabenbereich mit „ohne Projekt“ und den Projektbereich ansehen. | Die neue Aufgabe ist eine Projektaufgabe. Sie erscheint nicht als Aufgabe ohne Projekt. Im Projektbereich erscheint keine Aufgabe ohne Projekt. |
| TS-006-14 | 14 | Detailansicht von „Offerte nachfassen“ öffnen. Den Status umschalten. Den Verweis auf Zuständig anklicken. | Titel, Beschreibung, Status, Zuständig, Erstellt am und Geändert am sind sichtbar. Die Pfadleiste lautet „Aufgaben › Offerte nachfassen“. Der Verweis führt zum Benutzer. |
| TS-006-15 | 15 | „Bearbeiten“ öffnen. Titel, Beschreibung und Zuständig ändern. Speichern. | Die Änderungen sind gespeichert. Das Modal-Fenster liegt über der Seite, Webadresse `…?bearbeiten=1`. |
| TS-006-16 | 16 | „Löschen“ klicken. Im Dialog bestätigen. | Der Dialog nennt den Titel. Nach Bestätigen ist die Aufgabe weg und die Liste erscheint. |
| TS-006-17 | 17 | Webadresse `/aufgaben/<Kennung einer Projektaufgabe>` aufrufen. Dann `/aufgaben/99999`. | Bei der Projektaufgabe steht ein Verweis auf die Ansicht im Projekt. Bei der unbekannten Kennung erscheint „Aufgabe nicht gefunden“. |
| TS-006-18 | 18 | Benutzer-Detail von Ben öffnen. Dann den Verweis „Alle Aufgaben dieses Benutzers anzeigen“ anklicken. | „Offerte nachfassen“ steht bei den offenen Aufgaben mit Herkunft „direkt zugewiesen“. Der Verweis öffnet `/aufgaben?benutzer=…` von Ben. |
| TS-006-19 | 19 | Ben löschen. Ben ist zuständig für „Offerte nachfassen“ und ist bei Teilprojekt 2 hinterlegt. | Das Löschen wird verhindert. Die Meldung nennt die Anzahl der Aufgaben ohne Projekt (1) und das Teilprojekt. |
| TS-006-20 | 20 | Ben sperren. Dann Ben in der Benutzer-Auswahl suchen. Dann `/aufgaben?benutzer=<Kennung von Ben>` aufrufen. | Ben ist in der Auswahl nicht wählbar. Seine Aufgabe bleibt zugewiesen. Die Webadresse ist weiterhin erreichbar. |
| TS-006-21 | 21 | DB: Eine Aufgabe mit `vorgang_id` und `benutzer_id` zugleich anlegen. Dann eine Aufgabe ohne beides anlegen. | Beide Einträge werden von der Datenbank abgelehnt. Die bestehenden Projektaufgaben sind unverändert. |

## ANF-007 Anmeldung

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-007-01 | 1 | Im privaten Fenster ohne Anmeldung `/projekte/4` aufrufen. Dann die Seite neu laden. | Beide Male wirst du nach `/anmelden?zurueck=/projekte/4` umgeleitet. |
| TS-007-02 | 2 | Ohne Anmeldung und ohne Cookie eine Server-Funktion unter `/api/…` aufrufen, z. B. mit `curl`. | Die Antwort ist HTTP 401 mit „Nicht angemeldet“. Es kommen keine Daten zurück. |
| TS-007-03 | 3 | Ohne Anmeldung `/anmelden` öffnen und das Netzwerk prüfen. | Stylesheet und Seitensymbol laden ohne Umleitung. |
| TS-007-04 | 4 | `/anmelden` auf dem Mobiltelefon und auf dem grossen Bildschirm öffnen. | Eine zentrierte Karte mit dem Namen „ERP-saintwhoo“, den Feldern Benutzername und Passwort und der Schaltfläche „Anmelden“ (Glutorange). Keine Seitenleiste. |
| TS-007-05 | 5 | Mit Anna anmelden. | Die Startseite erscheint. Die Kopfzeile zeigt „Muster Anna“ und „Abmelden“. |
| TS-007-06 | 5 | Mit Enter statt Klick anmelden. Vorher `/anmelden?zurueck=/projekte/4` aufrufen. | Enter funktioniert. Nach der Anmeldung erscheint `/projekte/4`. |
| TS-007-07 | 5 | `/anmelden?zurueck=https://boese.example.com` mit richtigen Daten verwenden. Dann mit `zurueck=//boese.example.com` wiederholen. | Beide Male landest du auf der Startseite und nicht auf einer fremden Adresse. |
| TS-007-08 | 6 | Benutzername „ ANNA “ mit Leerzeichen und Grossbuchstaben und richtigem Passwort eingeben. Dann das Passwort in falscher Gross- und Kleinschreibung eingeben. | Die erste Anmeldung klappt. Die zweite wird abgelehnt, weil das Passwort unverändert geprüft wird. |
| TS-007-09 | 7 | Dreimal eine falsche Anmeldung: unbekannter Name, falsches Passwort, gesperrte Carla. | Immer dieselbe Meldung: „Benutzername oder Passwort ist falsch.“. Der Name bleibt im Feld. Das Passwortfeld ist leer. |
| TS-007-10 | 8 | Beide Felder leer lassen und „Anmelden“ klicken. | Die Meldung „Bitte Benutzername und Passwort eingeben.“ erscheint. |
| TS-007-11 | 9 | Angemeldet `/anmelden` öffnen. | Du wirst auf die Startseite weitergeleitet, bzw. auf das Ziel von `zurueck`. |
| TS-007-12 | 10 | Nach der Anmeldung von Anna den Benutzer-Detail von Anna öffnen. | „Letzte Anmeldung“ zeigt den jetzigen Zeitpunkt. |
| TS-007-13 | 11 | Fünfmal falsch für „ben“ anmelden. Dann das richtige Passwort für „ben“ eingeben. | Der sechste Versuch zeigt „Zu viele Fehlversuche. Bitte später erneut versuchen.“. Nach 15 Minuten geht die Anmeldung wieder. Zum Zurücksetzen in der Testdatenbank `DELETE FROM anmeldeversuch;` ausführen. |
| TS-007-14 | 11 | Fünfmal falsch für den unbekannten Namen „niemand“ anmelden. Dann einen sechsten Versuch machen. | Der sechste Versuch ist gesperrt. Die Meldung ist dieselbe wie in TS-007-13. |
| TS-007-15 | 12 | Je viermal falsch anmelden: für „ben“ (bekannt) und für „niemand“ (unbekannt). Die Dauer jedes Versuchs notieren. | Die Dauern sind ähnlich. Diese Prüfung ist nur grob von Hand möglich. Danach die Fehlversuche in der Testdatenbank löschen. |
| TS-007-16 | 13 | Nach der Anmeldung das Cookie in den Entwicklerwerkzeugen prüfen. | `HttpOnly` ist gesetzt. `SameSite` ist `Lax`. `Path` ist `/`. `Secure` fehlt ohne `SITZUNG_COOKIE_SECURE=1`. Mit HTTPS und der Variable ist `Secure` gesetzt. |
| TS-007-17 | 13 | DB: `SELECT merkmal_hash FROM sitzung;` ausführen. Den Wert mit dem Cookie vergleichen. | Der gespeicherte Wert ist ein 64-stelliger Hash. Er ist nicht gleich dem Cookie-Wert. |
| TS-007-18 | 16 | Den Cookie-Wert notieren. Abmelden. Erneut anmelden. Den Cookie-Wert vergleichen. | Der Cookie-Wert ist neu. Der alte gilt nicht mehr. |
| TS-007-19 | 14 | DB: `UPDATE sitzung SET zuletzt_aktiv = now() - interval '13 hours';` ausführen. Dann eine Seite aufrufen. | Du wirst nach `/anmelden?abgelaufen=1` umgeleitet. Die Meldung „Ihre Sitzung ist abgelaufen. Bitte erneut anmelden.“ erscheint. |
| TS-007-20 | 14 | DB: `UPDATE sitzung SET erstellt_am = now() - interval '8 days', zuletzt_aktiv = now();` ausführen. Dann eine Seite aufrufen. | Die Sitzung ist abgelaufen, obwohl sie aktiv ist. |
| TS-007-21 | 15 | Anna in zwei Fenstern angemeldet lassen. Ben sperrt Anna in seiner Sitzung. Dann in beiden Fenstern eine Seite aufrufen. | Beide Fenster zeigen die Anmeldung mit „Ihre Sitzung ist abgelaufen“. |
| TS-007-22 | 15 | Ben setzt das Passwort von Carla neu. Danach setzt Ben sein eigenes Passwort neu. | Carlas Sitzungen enden. Bei Ben bleibt die laufende Sitzung bestehen. Seine anderen Sitzungen enden. |
| TS-007-23 | 17 | Mit Anna angemeldet die Kopfzeile ansehen. Dann auf dem Mobiltelefon. | Die Kopfzeile zeigt den Namen und „Abmelden“. Auf dem Mobiltelefon ist der Name gekürzt oder steht nur als Symbol da. |
| TS-007-24 | 18 | „Abmelden“ klicken. Dann die Zurück-Taste drücken. | Die Anmeldeseite erscheint mit „Sie wurden abgemeldet.“. Die Zurück-Taste führt nicht in geschützte Seiten. DB: Die Sitzungszeile ist weg. |
| TS-007-25 | 20 | Als Ben die eigene Zeile in der Benutzerverwaltung sperren, dann löschen. | Beide Male wird es verhindert. Die Meldungen lauten „Sie können sich nicht selbst sperren.“ und „Sie können sich nicht selbst löschen.“. |
| TS-007-26 | 21 | Mit einer leeren Datenbank `/anmelden` öffnen. Die Ersteinrichtung mit einem Passwort aus 5 Zeichen absenden. Dann mit gültigen Daten absenden. | Die Ersteinrichtung erscheint. Das kurze Passwort wird abgelehnt. Mit gültigen Daten werden Person und Benutzer angelegt und du bist direkt angemeldet. |
| TS-007-27 | 22 | Nach der Ersteinrichtung `/anmelden` öffnen. Dann die Server-Funktion für die Ersteinrichtung erneut aufrufen. | Die Anmeldung erscheint statt der Ersteinrichtung. Der Aufruf meldet „Die Ersteinrichtung ist bereits abgeschlossen. Bitte anmelden.“. |
| TS-007-28 | 23 | DB: `INSERT INTO anmeldeversuch (benutzername, zeitpunkt) VALUES ('alt', now() - interval '1 day');` ausführen. Dann anmelden. | Der alte Eintrag ist danach gelöscht. Die Tabellen `sitzung` und `anmeldeversuch` sind vorhanden. |
| TS-007-29 | 24 | Anmelden mit den Entwicklerwerkzeugen beobachten: Anfrage, Webadresse und Serverprotokoll. | Das Formular wird per POST gesendet. Die Webadresse enthält kein Passwort. Im Serverprotokoll stehen weder Passwort noch Cookie-Wert. |

## Offene Punkte aus der Prüfung

Bei der Vorbereitung der Szenarien sind diese Punkte aufgefallen. Sie sollten vor dem Test entschieden werden.

1. **Grossbuchstaben im Benutzernamen (TS-005-11).** ANF-005 AK-13 erlaubt nur Kleinbuchstaben. ANF-007 AK-6 ignoriert Gross- und Kleinschreibung beim Anmelden. Zu klären: Wird „Dora“ abgelehnt oder klein gespeichert?
2. **Hinweis ohne Benutzerauswahl (TS-006-02).** ANF-006 AK-2 verlangt bei fehlender Auswahl den Hinweis „Bitte einen Benutzer wählen.“. ANF-007 AK-19 nimmt stattdessen den angemeldeten Benutzer. Im Code ersetzt der angemeldete Benutzer die fehlende Auswahl. Der Hinweis erscheint praktisch nicht mehr. Zu klären: Hinweis entfernen und ANF-006 AK-2 anpassen.
3. **Meldung beim Selbst-Sperren (TS-007-25).** ANF-005 AK-20 nennt eine gemeinsame Meldung für „sperren bzw. löschen“. Im Code gibt es zwei Meldungen. Zu klären: Welche Formulierung gilt?
4. **Rundung des Fortschritts (TS-003-09).** ANF-003 AK-5 gibt keine Rundung vor. Der Code rechnet ganzzahlig ab (2 von 3 ergibt 66 %). Zu klären: Abrunden oder kaufmännisch runden?
5. **Antwortzeit bei der Anmeldung (TS-007-15).** ANF-007 AK-12 lässt sich von Hand nur grob prüfen. Ein Modultest würde die Zeitgleichheit genauer messen.
