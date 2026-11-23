# Testszenarien – uniqwork

Stand: 2026-11-12 · Grundlage: Akzeptanzkriterien (AK) der Anforderungen ANF-001 bis ANF-007, ANF-011 und ANF-025

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
| ANF-007 | Anmeldung | 29 | Seit ANF-026 mit E-Mail statt Benutzername; TS-007-26/27 entfallen |
| ANF-026 | Registrierung, Datenbank, Mandant | 32 | Alle Schritte (Portal-Datenbank, Registrierung, Anmeldung, Datenbank erstellen, Mandant definieren, Passwort im Portal) |

Die Anforderungen ANF-005 bis ANF-007 haben bisher keine Prüfung im Browser. Mit diesen Szenarien starten.

## Vorbereitung

1. PostgreSQL bereitstellen und `.env` ausfüllen (Vorlage: `.env.example`). Der Server legt Portal-Datenbank und ERP-Rolle beim Start an, die Mandanten-Datenbank entsteht über „Datenbank erstellen“; die Migrationen laufen automatisch.
2. Den Server mit `cargo leptos watch` starten und `http://127.0.0.1:3002` öffnen.
3. Zwei Browser-Fenster bereithalten, eines normal und eines privat. So sind zwei Anmeldungen gleichzeitig möglich, z. B. für TS-007-21.
4. Die Testdaten in dieser Reihenfolge anlegen. Die Namen stammen aus den Beispielen von ANF-005:
   - Adressen vom Typ Person: **Anna Muster**, **Ben Beispiel**, **Carla Probe**, **Dora Demo** (Dora bleibt ohne Benutzer, für TS-005-10).
   - Eine Firma: **Musterfirma AG**.
   - 123 weitere Adressen, damit die Seitenprüfung funktioniert (ANF-002 AK-4).
   - 61 Projekte insgesamt (ohne P-2026-001 und P-2026-002 mitgezählt, für TS-001-02 und TS-003-05).
   - 51 Einkaufsaufträge, damit die Seitenprüfung funktioniert (für TS-004-03).
   - Benutzer **anna** (entsteht beim Erstellen der Datenbank), **ben** und **carla**, alle aktiv. Seit ANF-026 haben Benutzer kein Passwort; anmelden kann sich nur Anna über ihren Zugang im Portal. Schreib die Passwörter nicht in dieses Dokument.
   - Projekt **P-2026-001**, Kunde **Musterfirma AG**, verantwortlich **Anna**. Struktur:
     - Teilprojekt 1 (ohne Verantwortliche) mit Vorgang 1.1 und Aufgabe 1.1.1 (offen).
     - Teilprojekt 2 (verantwortlich **Ben**):
       - Vorgang 2.1 (ohne Verantwortliche) mit Aufgabe 2.1.1 (offen).
       - Vorgang 2.2 (ohne Verantwortliche) mit Aufgaben 2.2.1 (erledigt) und 2.2.2 (offen).
       - Vorgang 2.3 (verantwortlich **Carla**) mit Aufgabe 2.3.1 (offen).
   - Projekt **P-2026-002** ohne Kunde und ohne Verantwortliche.
   - Ein Einkaufsauftrag im Status Entwurf und einer im Status Bestellt, beide dem Projekt P-2026-001 zugeordnet. Die Lieferanten sind Adressen aus der Liste.
5. Die Ersteinrichtung gibt es seit ANF-026 nicht mehr; der erste Zugang entsteht über `/registrieren`.

> **Hinweis ANF-026:** Seit der Anmeldung über das Portal öffnet sich das ERP nur mit einem bereiten und definierten
> Mandanten, und jeder Mandant hat eine eigene Datenbank. Ablauf: registrieren → „Datenbank erstellen“ → im
> Modal-Fenster „Mandant definieren“ die Firma erfassen (z. B. **Musterfirma AG**). Die Testdaten oben gehören in diese
> Mandanten-Datenbank; die Firma „Musterfirma AG“ entsteht dabei schon.
> In den Szenarien von ANF-007 steht „Benutzername“ für die **E-Mail** des Portal-Zugangs.

## ANF-001 Grundgerüst

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-001-01 | 1 | Startseite `/` aufrufen. | Kopfzeile oben, Seitenleiste links und Inhalt rechts sind sichtbar. |
| TS-001-02 | 2 | Projektübersicht mit 61 Projekten auf dem grossen Bildschirm öffnen und nach unten scrollen. | Kopfzeile und Seitenleiste bleiben stehen. Nur der Inhalt bewegt sich. |
| TS-001-03 | 3 | In der Seitenleiste auf „Adressen“ klicken. | Die Adressliste erscheint. Die Seitenleiste wird nicht neu aufgebaut und flackert nicht. |
| TS-001-04 | 4 | Auf einer geöffneten Seite den Eintrag der Seite ansehen. | Der aktive Eintrag hat einen leichten Indigo-Hintergrund mit dunkel-indigofarbener Schrift. Andere Einträge sind nicht markiert. |
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
| TS-003-18 | 5a | Übersicht und Strukturbaum ansehen. | Projekt ist Indigo, Teilprojekt Himmelblau, Vorgang Petrol und Aufgabe Schiefergrau. Jede Ebene hat einen farbigen Streifen links. Über der Übersicht steht eine Legende. |
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
| TS-005-01 | 1, 4 | Einstellungen → „Benutzer“ öffnen. | Die Tabelle hat die Spalten Benutzername, Name, E-Mail und Status (seit ANF-026 ohne Letzte Anmeldung. Standard ist Benutzername aufsteigend. |
| TS-005-02 | 2 | Nach „anna“, nach „Muster“ und nach einem Teil der E-Mail-Adresse suchen. | Die Treffer stimmen. Gross- und Kleinschreibung spielen keine Rolle. |
| TS-005-03 | 3 | Nach „alle“, „aktiv“ und „gesperrt“ filtern. | Die Liste passt zur Auswahl. |
| TS-005-04 | 4, 5 | Auf den Spaltenkopf „Status“ zweimal klicken. Dann nach „xyz“ suchen. Danach die Seite neu laden. | Die Sortierung kehrt sich um. Ohne Treffer erscheint ein Hinweis. Der Zustand bleibt nach dem Neuladen erhalten. |
| TS-005-05 | 6 | Eine Zeile anklicken. | Die Detailansicht unter `/einstellungen/benutzer/<Kennung>` zeigt Benutzername, Status, Zugang im Portal (statt Letzte Anmeldung, Erstellt am, Geändert am und die Adressdaten. |
| TS-005-06 | 7 | Im Benutzer-Detail den Verweis auf die Adresse anklicken. | Die Adresse öffnet sich unter `/adressen/<Kennung>`. |
| TS-005-07 | 8 | Die Adresse von Anna öffnen. | Ein Hinweis „Benutzer: anna“ erscheint mit einem Verweis auf den Benutzer. |
| TS-005-08 | 9, 10 | „Neuer Benutzer“ öffnen. In der Adress-Suche nach „Musterfirma“ suchen, dann nach „Muster“. | Das Modal-Fenster liegt über der Liste, Webadresse `?neu=1`. Die Firma wird nicht angeboten. Anna wird nicht angeboten, weil sie schon einen Benutzer hat. |
| TS-005-09 | 11 | Das leere Formular speichern. | Benutzername und Adresse sind Pflicht (seit ANF-026 ohne Passwortfelder). Der Status „aktiv“ ist vorausgewählt. |
| TS-005-10 | 12 | Benutzername „dora.demo“, Adresse „Demo Dora“. Speichern. | Die Erfolgsmeldung erscheint. Die Detailansicht des neuen Benutzers öffnet sich. |
| TS-005-11 | 13 | Benutzername nacheinander eingeben: „dora demo“ (mit Leerzeichen), „dora!“, 51 Zeichen, leer, „Dora“ (Grossbuchstabe, siehe Offene Punkte), und „anna“ bei der Adresse von Dora. | Jeder Fehler wird am Feld mit einer Meldung markiert. „anna“ wird als bereits vergeben abgelehnt. Nichts wird gespeichert. |
| TS-005-12 | 13 | Ohne Adresse speichern. Dann prüfen, ob die Adress-Suche die Firma „Musterfirma AG“ oder Anna anbietet. Dann ein Passwort mit 9 Zeichen eingeben. Dann eine Wiederholung eingeben, die nicht passt. | Die Adresse ist Pflicht. Die Firma und Anna werden nicht angeboten. Falls die Auswahl sie trotzdem zulässt, lehnt der Server sie ab. Das kurze Passwort und die ungleiche Wiederholung werden abgelehnt. Die Meldungen nennen den Grund. Nichts wird gespeichert. |
| TS-005-13 | 14 | Einen Benutzer mit „Bearbeiten“ öffnen. | Benutzername, Adresse und Status lassen sich ändern. Passwortfelder fehlen. Webadresse `…?bearbeiten=1`. |
| ~~TS-005-14~~ | 15 | *Entfällt (ANF-026): Passwort ändern im Portal, siehe TS-026-30.* | – |
| TS-005-15 | 16 | Carla auf „gesperrt“ setzen. Dann die Liste und die Detailansicht ansehen. | Carla bleibt mit allen Daten erhalten und ist als „gesperrt“ markiert. Die Anmeldung ist nicht mehr möglich (siehe TS-007-09). |
| TS-005-16 | 17 | Einen Benutzer bearbeiten und einen ungültigen Benutzernamen eingeben. Speichern. Dann eine zweite Änderung mit „Abbrechen“ verwerfen. | Die Prüfung ist dieselbe wie beim Anlegen. „Abbrechen“ speichert nichts. |
| TS-005-17 | 18 | „Löschen“ klicken und im Dialog abbrechen. Dann erneut löschen und bestätigen. | Der Dialog nennt Benutzername und Name. Nach Abbrechen bleibt der Benutzer. Nach Bestätigen ist er weg. Die Adresse „Demo Dora“ bleibt erhalten. |
| TS-005-18 | 19 | Die Adresse von Ben in der Adressverwaltung löschen. | Das Löschen wird verhindert. Die Meldung nennt „als Benutzer ben“. |
| TS-005-19 | 20 | Die Adresse von Ben auf „Firma“ ändern und speichern. | Die Änderung wird verhindert. Die Meldung nennt den Benutzer. |
| ~~TS-005-20~~ | 21 | *Entfällt (ANF-026): keine Passwörter in der Mandanten-Datenbank, siehe TS-026-31.* | – |
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
| TS-007-04 | 4 | `/anmelden` auf dem Mobiltelefon und auf dem grossen Bildschirm öffnen. | Eine zentrierte Karte mit dem Namen „uniqwork“, den Feldern Benutzername und Passwort und der Schaltfläche „Anmelden“ (Indigo). Keine Seitenleiste. |
| TS-007-05 | 5 | Mit Anna anmelden. | Die Startseite erscheint. Die Kopfzeile zeigt „Muster Anna“ und „Abmelden“. |
| TS-007-06 | 5 | Mit Enter statt Klick anmelden. Vorher `/anmelden?zurueck=/projekte/4` aufrufen. | Enter funktioniert. Nach der Anmeldung erscheint `/projekte/4`. |
| TS-007-07 | 5 | `/anmelden?zurueck=https://boese.example.com` mit richtigen Daten verwenden. Dann mit `zurueck=//boese.example.com` wiederholen. | Beide Male landest du auf der Startseite und nicht auf einer fremden Adresse. |
| TS-007-08 | 6 | Benutzername „ ANNA “ mit Leerzeichen und Grossbuchstaben und richtigem Passwort eingeben. Dann das Passwort in falscher Gross- und Kleinschreibung eingeben. | Die erste Anmeldung klappt. Die zweite wird abgelehnt, weil das Passwort unverändert geprüft wird. |
| TS-007-09 | 7 | Dreimal eine falsche Anmeldung: unbekannter Name, falsches Passwort, gesperrte Carla. | Immer dieselbe Meldung: „Benutzername oder Passwort ist falsch.“. Der Name bleibt im Feld. Das Passwortfeld ist leer. |
| TS-007-10 | 8 | Beide Felder leer lassen und „Anmelden“ klicken. | Die Meldung „Bitte Benutzername und Passwort eingeben.“ erscheint. |
| TS-007-11 | 9 | Angemeldet `/anmelden` öffnen. | Du wirst auf die Startseite weitergeleitet, bzw. auf das Ziel von `zurueck`. |
| ~~TS-007-12~~ | 10 | *Entfällt (ANF-026): keine „Letzte Anmeldung“ mehr im ERP.* | – |
| TS-007-13 | 11 | Fünfmal falsch für „ben“ anmelden. Dann das richtige Passwort für „ben“ eingeben. | Der sechste Versuch zeigt „Zu viele Fehlversuche. Bitte später erneut versuchen.“. Nach 15 Minuten geht die Anmeldung wieder. Zum Zurücksetzen in der Testdatenbank `DELETE FROM anmeldeversuch;` ausführen. |
| TS-007-14 | 11 | Fünfmal falsch für den unbekannten Namen „niemand“ anmelden. Dann einen sechsten Versuch machen. | Der sechste Versuch ist gesperrt. Die Meldung ist dieselbe wie in TS-007-13. |
| TS-007-15 | 12 | Je viermal falsch anmelden: für „ben“ (bekannt) und für „niemand“ (unbekannt). Die Dauer jedes Versuchs notieren. | Die Dauern sind ähnlich. Diese Prüfung ist nur grob von Hand möglich. Danach die Fehlversuche in der Testdatenbank löschen. |
| TS-007-16 | 13 | Nach der Anmeldung das Cookie in den Entwicklerwerkzeugen prüfen. | `HttpOnly` ist gesetzt. `SameSite` ist `Lax`. `Path` ist `/`. `Secure` fehlt ohne `SITZUNG_COOKIE_SECURE=1`. Mit HTTPS und der Variable ist `Secure` gesetzt. |
| TS-007-17 | 13 | DB: `SELECT merkmal_hash FROM sitzung;` ausführen. Den Wert mit dem Cookie vergleichen. | Der gespeicherte Wert ist ein 64-stelliger Hash. Er ist nicht gleich dem Cookie-Wert. |
| TS-007-18 | 16 | Den Cookie-Wert notieren. Abmelden. Erneut anmelden. Den Cookie-Wert vergleichen. | Der Cookie-Wert ist neu. Der alte gilt nicht mehr. |
| TS-007-19 | 14 | DB: `UPDATE sitzung SET zuletzt_aktiv = now() - interval '13 hours';` ausführen. Dann eine Seite aufrufen. | Du wirst nach `/anmelden?abgelaufen=1` umgeleitet. Die Meldung „Ihre Sitzung ist abgelaufen. Bitte erneut anmelden.“ erscheint. |
| TS-007-20 | 14 | DB: `UPDATE sitzung SET erstellt_am = now() - interval '8 days', zuletzt_aktiv = now();` ausführen. Dann eine Seite aufrufen. | Die Sitzung ist abgelaufen, obwohl sie aktiv ist. |
| TS-007-21 | 15 | Anna in zwei Fenstern angemeldet lassen. Ben sperrt Anna in seiner Sitzung. Dann in beiden Fenstern eine Seite aufrufen. | Beide Fenster zeigen die Anmeldung mit „Ihre Sitzung ist abgelaufen“. |
| ~~TS-007-22~~ | 15 | *Entfällt (ANF-026): siehe TS-026-30.* | – |
| TS-007-23 | 17 | Mit Anna angemeldet die Kopfzeile ansehen. Dann auf dem Mobiltelefon. | Die Kopfzeile zeigt den Namen und „Abmelden“. Auf dem Mobiltelefon ist der Name gekürzt oder steht nur als Symbol da. |
| TS-007-24 | 18 | „Abmelden“ klicken. Dann die Zurück-Taste drücken. | Die Anmeldeseite erscheint mit „Sie wurden abgemeldet.“. Die Zurück-Taste führt nicht in geschützte Seiten. DB: Die Sitzungszeile ist weg. |
| TS-007-25 | 20 | Als Ben die eigene Zeile in der Benutzerverwaltung sperren, dann löschen. | Beide Male wird es verhindert. Die Meldungen lauten „Sie können sich nicht selbst sperren.“ und „Sie können sich nicht selbst löschen.“. |
| ~~TS-007-26~~ | 21 | *Entfällt (ANF-026).* | – |
| ~~TS-007-27~~ | 22 | *Entfällt (ANF-026).* | – |
| TS-007-28 | 23 | DB: `INSERT INTO anmeldeversuch (benutzername, zeitpunkt) VALUES ('alt', now() - interval '1 day');` ausführen. Dann anmelden. | Der alte Eintrag ist danach gelöscht. Die Tabellen `sitzung` und `anmeldeversuch` sind vorhanden. |
| TS-007-29 | 24 | Anmelden mit den Entwicklerwerkzeugen beobachten: Anfrage, Webadresse und Serverprotokoll. | Das Formular wird per POST gesendet. Die Webadresse enthält kein Passwort. Im Serverprotokoll stehen weder Passwort noch Cookie-Wert. |

## ANF-026 Registrierung, eigene Datenbank und Mandant

Vorbereitung: leere PostgreSQL-Instanz bzw. Datenbanken `uniqwork` und `uniqwork_portal` löschen; `PORTAL_DATABASE_URL` leer lassen.

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-026-01 | 7, 9 | Server starten. Danach `\l` in `psql`. | Beide Datenbanken `uniqwork` und `uniqwork_portal` sind angelegt. Ein zweiter Start legt nichts neu an. |
| TS-026-02 | 8 | In `uniqwork_portal` und in `uniqwork` die Tabellen auflisten (`\dt`). | Portal: `portal_benutzer`, `mandant`, `portal_sitzung`, `portal_anmeldeversuch`. Keine dieser Tabellen in `uniqwork`, keine ERP-Tabelle im Portal. |
| TS-026-03 | 12 | DB: `INSERT INTO mandant (besitzer_id, datenbankname) VALUES (1, 'x');` | Wird abgelehnt: `datenbankname` wird nur von der Datenbank erzeugt (`uq_m_000001` …). |
| TS-026-04 | 1 | Ohne Anmeldung `/anmelden` öffnen, auf „Registrieren“ klicken. | `/registrieren` öffnet sich ohne Anmeldung und ohne Seitenleiste. |
| TS-026-05 | 2 | Leer absenden. Dann E-Mail „falsch“, Passwort mit 5 Zeichen und abweichender Wiederholung. | Meldungen bei Vorname, Nachname, E-Mail und Passwort; nichts wird angelegt. |
| TS-026-06 | 4, 6 | „Anna“, „Muster“, „ Anna@Firma.CH “ und gültiges Passwort absenden. | Weiterleitung auf `/portal` mit „Willkommen, Anna Muster“ und „Sie haben noch keine Datenbank.“. In `portal_benutzer` steht `anna@firma.ch` mit `ist_administrator = true`. In `uniqwork` ändert sich nichts. |
| TS-026-07 | 3 | Abmelden, mit „ANNA@firma.ch“ erneut registrieren. | „Für diese E-Mail-Adresse gibt es bereits einen Zugang.“ |
| TS-026-08 | 5 | DB: `SELECT passwort_hash FROM portal_benutzer;` | Ein Argon2id-Hash (`$argon2id$…`), nie das Passwort. |
| TS-026-09 | 5 | Fünfmal mit falschem Passwort anmelden, dann mit dem richtigen. | Ab dem sechsten Versuch: „Zu viele Fehlversuche. Bitte später erneut versuchen.“ |
| TS-026-10 | 19 | Angemeldet ohne Datenbank `/`, `/adressen` und `/einstellungen/benutzer` aufrufen. | Jedes Mal Weiterleitung auf `/portal`. |
| TS-026-11 | 19 | Angemeldet ohne Datenbank eine ERP-Server-Funktion unter `/api/…` aufrufen (z. B. mit `curl` und dem Cookie). | HTTP 403 „Kein Mandant gewählt.“, keine Daten. |
| TS-026-12 | 19 | DB: Mandant für Anna mit Status `bereit`, aber ohne Firmennamen anlegen, neu anmelden. Dann den Firmennamen setzen und neu anmelden. | Zuerst weiter nur `/portal`. Mit Firmennamen öffnet die Anmeldung das ERP bzw. das Ziel aus `zurueck`. |
| TS-026-13 | 23 | Zweiten Zugang „Ben“ registrieren. DB: Bens Sitzung `mandant_id` auf Annas Mandanten setzen. Dann `/` aufrufen. | Ben bleibt auf `/portal`; Annas Mandant wird nicht verwendet. |
| TS-026-14 | 25 | Mit leerer Portal-Datenbank `/anmelden` öffnen. | Keine Ersteinrichtung mehr, nur Anmeldung mit Verweis auf „Registrieren“. |
| TS-026-15 | 21 | Nach dem Serverstart: `SELECT rolcreatedb, rolsuper FROM pg_roles WHERE rolname = 'uniqwork_erp';` und `SELECT has_database_privilege('uniqwork_erp', 'uniqwork_portal', 'CONNECT');` | Rolle vorhanden, `false`/`false`; Verbindung zur Portal-Datenbank `false`. |
| TS-026-16 | 10, 11, 26 | Als Anna ohne Datenbank `/portal` öffnen, „Datenbank erstellen“ klicken und die Zeit messen. | Während des Erstellens ist die Schaltfläche gesperrt („Datenbank wird erstellt …“). Danach Plakette „Bereit“ und Hinweis auf den Mandanten; Dauer unter 30 Sekunden. |
| TS-026-17 | 12, 15 | DB: `\l` und in `uq_m_000001`: `SELECT benutzername, passwort_hash, portal_benutzer_id FROM benutzer;` | Datenbank `uq_m_000001` vorhanden. Genau ein Benutzer (aus der E-Mail, z. B. `anna`), ohne Passwort, mit der Kennung des Zugangs; dazu eine Adresse „Person“. |
| TS-026-18 | 21 | In `uq_m_000001`: `SELECT has_table_privilege('uniqwork_erp', 'adresse', 'SELECT,INSERT'), has_table_privilege('uniqwork_erp', 'adresse', 'TRUNCATE'), has_schema_privilege('uniqwork_erp', 'public', 'CREATE');` und `SELECT has_database_privilege('public', 'uq_m_000001', 'CONNECT');` | `true`, `false`, `false`; `PUBLIC` darf sich nicht verbinden. |
| TS-026-19 | 14 | Als Anna die Server-Funktion „Datenbank erstellen“ erneut aufrufen (z. B. mit `curl`). Mit einem neuen Zugang zwei Aufrufe gleichzeitig senden. | „Sie haben bereits eine Datenbank.“ bzw. „Ihre Datenbank wird bereits erstellt.“ – es entsteht jeweils genau eine Datenbank. |
| TS-026-20 | 13 | DB: `ALTER ROLE uniqwork_erp RENAME TO weg;`, dann mit einem neuen Zugang „Datenbank erstellen“. Danach zurückbenennen und „Erneut versuchen“. | Meldung „Die Datenbank konnte nicht erstellt werden …“, Plakette „Fehler“, keine halbe Datenbank übrig. Der zweite Versuch verwendet denselben Mandanten und endet mit „Bereit“. |
| TS-026-21 | 22, 23 | Zwei Zugänge mit je eigener Datenbank und Firmennamen. In beiden `/adressen` öffnen. | Jeder sieht nur seine eigene Person; die Daten des anderen sind nicht erreichbar. |
| TS-026-22 | 24 | DB: Einen Mandanten auf `wird_erstellt` setzen, Server neu starten. Danach als ein anderer Zugang weiterarbeiten. | Protokoll „Abgebrochene Erstellung … wird aufgeräumt“; Datenbank gelöscht, Status `fehler`. Die anderen Mandanten arbeiten nach dem Neustart weiter (neues Rollenpasswort). |
| TS-026-23 | 27 | Mit einem Mandanten im ERP arbeiten, dann `SELECT count(*) FROM pg_stat_activity WHERE usename = 'uniqwork_erp';` – und nach 5 Minuten Ruhe erneut. | Höchstens 3 Verbindungen; nach der Ruhezeit 0. |
| TS-026-24 | 16, 17 | Mit einem neuen Zugang „Datenbank erstellen“ klicken. | Danach steht `/portal?mandant=<Kennung>` in der Webadresse und das Modal-Fenster „Mandant definieren“ ist offen: Firmenname, Zusatz, Strasse, Postfach, PLZ, Ort, Land (Schweiz vorgewählt), Telefon, E-Mail, Webseite, UID. |
| TS-026-25 | 17 | Leer speichern. Dann gültige Anschrift mit UID `CHE-123.456.789`. | Meldungen bei Firmenname, Strasse, PLZ und Ort. Danach „Die Prüfziffer stimmt nicht …“ bei der UID; nichts gespeichert. |
| TS-026-26 | 18, 20 | „Firma AG“, „Bahnhofstrasse 1“, „8001“, „Zürich“, UID `che 116.281.710` speichern. | Weiterleitung auf die Startseite des ERP; die Kopfzeile zeigt „Firma AG“. Einstellungen › Firma zeigt Anschrift und `CHE-116.281.710`. In `uniqwork_portal` steht der Firmenname beim Mandanten. |
| TS-026-27 | 19 | Mit einem zweiten Zugang nach „Datenbank erstellen“ das Fenster mit Escape schliessen. | `/portal` zeigt „Mandant noch nicht definiert …“ und die Schaltfläche „Mandant definieren“, die das Fenster wieder öffnet. |
| TS-026-28 | 19 | Mit diesem Zugang `/adressen` aufrufen, abmelden und neu anmelden. | Beide Male erscheint `/portal?mandant=<Kennung>` mit offenem Fenster; das ERP bleibt geschlossen. |
| TS-026-29 | 18 | Als Anna (Mandant definiert) die Server-Funktion „Mandant definieren“ erneut aufrufen. | „Der Mandant ist bereits definiert …“; keine zweite Firma. |
| TS-026-30 | 28 | Als Anna in einem zweiten Browser angemeldet bleiben. Im ersten über den Namen in der Kopfzeile `/portal` öffnen, „Passwort ändern“: zuerst leer, dann falsches bisheriges Passwort, dann neues = bisheriges, dann gültig. | Meldungen „Bitte das bisherige Passwort eingeben.“, „Das bisherige Passwort ist falsch.“, „Das neue Passwort muss sich vom bisherigen unterscheiden.“. Gültig: „Passwort geändert …“; die laufende Sitzung bleibt, die zweite endet („Sitzung abgelaufen“). Anmeldung nur noch mit dem neuen Passwort. |
| TS-026-31 | 8, 29 | In `uq_m_000001`: Spalten von `benutzer` und Tabellen auflisten. Dann in der Benutzerverwaltung „Neuer Benutzer“ öffnen und Ben anlegen. | Keine Spalten `passwort_hash`/`letzte_anmeldung`, keine Tabellen `sitzung`/`anmeldeversuch`. Das Formular hat keine Passwortfelder; Ben zeigt „Zugang im Portal: Nein“, Anna „Ja“. Kein „Passwort setzen“, `?passwort=1` öffnet nichts. |
| TS-026-32 | 24 | Eine vor Schritt 3b erstellte Mandanten-Datenbank (mit `passwort_hash`, `sitzung`) haben und den Server starten. | Die Migration entfernt Spalten und Tabellen; die Benutzer bleiben erhalten und das ERP funktioniert weiter. |

## ANF-011 Kontenplan

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-011-01 | 3, 12 | Mit leerem Kontenplan Einstellungen › Finanzen › Kontenplan öffnen. „KMU-Kontenrahmen übernehmen“ klicken und bestätigen. | Vorher erscheinen der Hinweis „Noch keine Konten erfasst“ und die Warnung zu fehlenden Pflicht-Systemkonten. Danach sind 97 Konten angelegt, die Systemkonten sind vorbelegt, die Warnung ist weg und die Schaltfläche „KMU-Kontenrahmen übernehmen“ ist verschwunden. |
| TS-011-02 | 1 | Die Liste ansehen. | Die Konten sind nach Nummer sortiert und nach Kontenklasse gruppiert (Zwischenzeile „Klasse 1 · Aktiven“ usw.). Systemkonten tragen eine Plakette mit ihrer Rolle. |
| TS-011-03 | 2 | Nach „bank“ suchen, dann Kontoart „Ertrag“ wählen. Die Seite neu laden. | Die Suche findet 1020, 2100, 2400, 6940. Der Filter zeigt nur Ertragskonten. Suche und Filter stehen in der Webadresse und bleiben nach dem Neuladen erhalten. |
| TS-011-04 | 4, 5 | „Neues Konto“ öffnen. Nummer 1021 eingeben, dann 6571. | Die Webadresse enthält `?neu=1`. Die Kontoart springt auf „Aktiv“ bzw. „Aufwand“. Bei 7000 bleibt sie unverändert. Nach einer eigenen Auswahl der Kontoart wird sie nicht mehr überschrieben. |
| TS-011-05 | 4, 8 | Nummer 1020 ohne Bezeichnung speichern. Dann 123 und 0100 versuchen. | Nichts wird gespeichert. Meldungen: „Diese Kontonummer ist bereits vergeben.“, „Bitte eine Bezeichnung eingeben.“ bzw. der Hinweis auf 4–6 Ziffern. |
| TS-011-06 | 4 | 1021 „Bankguthaben UBS“, Kontoart Aktiv speichern. | Das Fenster schliesst, die Meldung „Konto 1021 gespeichert.“ erscheint, die Suche bleibt erhalten und 1021 steht direkt unter 1020. |
| TS-011-07 | 6, 11 | Konto 1100 (Systemkonto Debitoren) öffnen. „Aktiv“ abwählen und speichern. Dann Kontoart „Passiv“ wählen und speichern. | Beide Male wird das Speichern verhindert, mit Hinweis auf das Systemkonto „Debitoren“. |
| TS-011-08 | 9 | Bei Konto 1100 „Löschen“ › „Endgültig löschen“. | Die Meldung nennt „als Systemkonto „Debitoren““ und schlägt Deaktivieren vor. Das Konto bleibt bestehen. |
| TS-011-09 | 10 | Konto 1021 deaktivieren. Danach „Inaktive anzeigen“ wählen. | Ohne Kontrollkästchen ist 1021 ausgeblendet. Mit Kontrollkästchen erscheint es abgeschwächt mit „Inaktiv“. In „Systemkonten bearbeiten“ wird es nicht angeboten. |
| TS-011-10 | 9 | Konto 1021 löschen. | Nach Bestätigung erscheint „Konto 1021 wurde gelöscht.“. |
| TS-011-11 | 11, 12 | „Systemkonten bearbeiten“: bei „Kreditoren“ „– nicht zugeordnet –“ wählen und speichern. | Bei Debitoren werden nur aktive Aktivkonten angeboten. Nach dem Speichern erscheint die Warnung „Pflicht-Systemkonten fehlen: Kreditoren …“. |
| TS-011-12 | 11 | Mit den Entwicklerwerkzeugen die Anfrage beim Speichern der Systemkonten so ändern, dass Debitoren auf 3400 zeigt. | Der Server lehnt ab: „Für „Debitoren“ ist nur ein Konto der Kontoart Aktiv erlaubt.“ |
| TS-011-13 | – | Liste und „Konto bearbeiten“ auf dem Mobiltelefon öffnen. | Keine waagrechte Verschiebung der Seite; die Tabelle lässt sich in ihrem Rahmen verschieben, das Modal-Fenster ist vollständig bedienbar. |

## ANF-025 Firma

| Nr | AK | Schritte | Erwartet |
|----|----|----------|----------|
| TS-025-01 | 1 | Einstellungen öffnen. | In der Einstellungsnavigation steht „Firma“ (`/einstellungen/firma`). |
| TS-025-02 | 4 | Ohne Firma die Seite öffnen, „Firma erfassen“ klicken und ohne Auswahl speichern. | Hinweis „Noch keine Firma erfasst“. Das Fenster öffnet mit `?neu=1`. Meldung „Bitte eine Adresse wählen.“ |
| TS-025-03 | 4 | Im Suchfeld „a“ eingeben. | Es erscheinen nur Adressen vom Typ „Firma“, keine Personen. |
| TS-025-04 | 4, 6, 8 | „Neue Adresse anlegen“ wählen, Firmenname, PLZ, Ort ohne Strasse eingeben, UID CHE-123.456.789. Speichern. Dann Strasse ergänzen und UID „che 116 281 710“. | Zuerst: Strasse „Für den Absender der Belege nötig.“ und „Die Prüfziffer stimmt nicht …“. Danach gespeichert; die neue Adresse steht im Adressstamm, die UID lautet CHE-116.281.710, die MWST-Nummer CHE-116.281.710 MWST. |
| TS-025-05 | 5, 8 | „Bearbeiten“: eine Firmenadresse ohne Strasse wählen und speichern. Dann eine vollständige Firmenadresse wählen. | Zuerst „Die Postanschrift ist unvollständig (fehlt: Strasse) …“. Danach ist die neue Adresse verknüpft. |
| TS-025-06 | 5 | Im Fenster „Bearbeiten“ auf „Adresse bearbeiten“ klicken. | Die Adresse öffnet sich im Adressstamm im Bearbeiten-Fenster. |
| TS-025-07 | 7 | Unter „Logo“ eine GIF-Datei mit Endung .png hochladen, dann eine Datei über 1 MB. | „Erlaubt sind nur PNG-, JPG- oder SVG-Dateien.“ bzw. „Die Datei ist grösser als 1 MB.“ |
| TS-025-08 | 7 | Ein PNG hochladen, danach ein SVG mit `<script>`. `/firma/logo` direkt öffnen. | Das Logo erscheint auf der Seite und in der Belegvorschau. Das SVG wird angezeigt, das Skript läuft nicht (Antwort mit `Content-Security-Policy: … sandbox`). |
| TS-025-09 | 7 | „Logo entfernen“. | „Logo entfernt.“, kein Logo mehr sichtbar. |
| TS-025-10 | 3 | Belegvorschau ansehen. | Logo, Name, Anschrift und UID oben, Fusszeile unten. |
| TS-025-11 | 11 | Die Adresse der Firma im Adressstamm öffnen und löschen. | Plakette „Eigene Firma“. Löschen wird verhindert: „… als eigene Firma verwendet …“. |
| TS-025-12 | 12 | Neues Projekt: im Feld „Kunde“ nach dem Firmennamen suchen. Ebenso bei einem neuen Einkaufsauftrag (Lieferant). | Die eigene Firma wird nicht angeboten. |

## Offene Punkte aus der Prüfung

Bei der Vorbereitung der Szenarien sind diese Punkte aufgefallen. Sie sollten vor dem Test entschieden werden.

1. **Grossbuchstaben im Benutzernamen (TS-005-11).** ANF-005 AK-13 erlaubt nur Kleinbuchstaben. ANF-007 AK-6 ignoriert Gross- und Kleinschreibung beim Anmelden. Zu klären: Wird „Dora“ abgelehnt oder klein gespeichert?
2. **Hinweis ohne Benutzerauswahl (TS-006-02).** ANF-006 AK-2 verlangt bei fehlender Auswahl den Hinweis „Bitte einen Benutzer wählen.“. ANF-007 AK-19 nimmt stattdessen den angemeldeten Benutzer. Im Code ersetzt der angemeldete Benutzer die fehlende Auswahl. Der Hinweis erscheint praktisch nicht mehr. Zu klären: Hinweis entfernen und ANF-006 AK-2 anpassen.
3. **Meldung beim Selbst-Sperren (TS-007-25).** ANF-005 AK-20 nennt eine gemeinsame Meldung für „sperren bzw. löschen“. Im Code gibt es zwei Meldungen. Zu klären: Welche Formulierung gilt?
4. **Rundung des Fortschritts (TS-003-09).** ANF-003 AK-5 gibt keine Rundung vor. Der Code rechnet ganzzahlig ab (2 von 3 ergibt 66 %). Zu klären: Abrunden oder kaufmännisch runden?
5. **Antwortzeit bei der Anmeldung (TS-007-15).** ANF-007 AK-12 lässt sich von Hand nur grob prüfen. Ein Modultest würde die Zeitgleichheit genauer messen.
