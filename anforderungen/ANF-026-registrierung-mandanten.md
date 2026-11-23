# ANF-026: Registrierung, eigene Datenbank und Mandant

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-026                                           |
| **Status**    | Entwurf                                           |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Portal / Mandanten                                |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-23                                        |
| **Geändert**  | 2026-11-23                                        |

## 1. Beschreibung

uniqwork wird **mandantenfähig**: Mehrere Unternehmen arbeiten mit demselben ERP, jedes mit seinen
eigenen, vollständig getrennten Daten. Jeder Mandant bekommt eine **eigene PostgreSQL-Datenbank**.

Ablauf:

1. Ein Benutzer **registriert sich** selbst im ERP-System. Wer sich registriert, ist automatisch
   **Administrator** seines Zugangs.
2. Nach der Anmeldung kann er eine **Datenbank erstellen**. Die Datenbank wird ausschliesslich
   **durch das System** (das Portal) angelegt – nie direkt durch den Benutzer und nie über einen anderen Weg.
3. Direkt nach dem Erstellen erscheint ein **Modal-Fenster**, in dem er den **Mandanten** definiert
   (z. B. „Firma AG“ mit Anschrift). Erst danach kann er im ERP arbeiten.

```
                 ┌──────────────────────────────┐
 Registrieren ─► │  Portal-Datenbank            │   genau einmal vorhanden
 Anmelden        │  · portal_benutzer           │   Zugänge (Administrator)
                 │  · mandant                   │   welche Datenbank gehört wem
                 │  · portal_sitzung            │   Sitzungen mit gewähltem Mandanten
                 └──────────────┬───────────────┘
                                │ legt an (nur das System)
              ┌─────────────────┼─────────────────┐
              ▼                 ▼                 ▼
        uq_m_000001       uq_m_000002       uq_m_000003      je Mandant eine Datenbank,
        Firma AG          Muster GmbH       …                gleiche Migrationen wie heute
        (adresse, projekt, konto, firma, benutzer …)
```

Die **Portal-Datenbank** enthält nur Zugänge, Sitzungen und das Verzeichnis der Mandanten – keine ERP-Daten.
Jede **Mandanten-Datenbank** enthält das bisherige ERP (alle Migrationen aus `migrations/`).

## 2. Benutzergeschichten

> Als **neuer Kunde** möchte ich **mich selbst registrieren**, damit **ich ohne Zutun eines Dritten
> mit uniqwork beginnen kann**.

> Als **Administrator** möchte ich **mit einem Klick eine eigene Datenbank erstellen**, damit **die
> Daten meiner Firma vollständig von den Daten anderer Firmen getrennt sind**.

> Als **Administrator** möchte ich **nach dem Erstellen meinen Mandanten (Firma AG) erfassen**, damit
> **das ERP von Anfang an weiss, für welche Firma es arbeitet**.

> Als **Betreiber** möchte ich, dass **Datenbanken nur über das Portal entstehen**, damit **niemand
> unkontrolliert Datenbanken anlegen oder fremde Daten erreichen kann**.

## 3. Akzeptanzkriterien

**Registrierung**
- [ ] **AK-1:** Gegeben die Anmeldeseite, dann gibt es einen Verweis **„Registrieren“** auf `/registrieren`.
  Die Seite ist ohne Anmeldung erreichbar (wie `/anmelden`, ohne Grundgerüst).
- [ ] **AK-2:** Gegeben `/registrieren`, dann enthält das Formular Vorname, Nachname, E-Mail (= Anmeldename),
  Passwort und Passwort-Wiederholung. Es gelten die Passwortregeln aus ANF-005/ANF-007.
- [ ] **AK-3:** Gegeben eine bereits registrierte E-Mail (Gross-/Kleinschreibung egal), wenn sich jemand
  damit registriert, dann erscheint eine Fehlermeldung und es wird kein Zugang angelegt.
- [ ] **AK-4:** Gegeben eine gültige Registrierung, dann wird ein Zugang mit Kennzeichen **Administrator**
  angelegt, der Benutzer ist sofort angemeldet und landet auf der Portal-Übersicht `/portal`.
- [ ] **AK-5:** Das Passwort wird nur als Hash gespeichert (Verfahren wie ANF-007); Schutz gegen
  wiederholte Fehlversuche wie ANF-007.

**Datenbank erstellen**
- [ ] **AK-6:** Gegeben ein angemeldeter Administrator ohne Datenbank, dann zeigt `/portal` den Hinweis
  „Noch keine Datenbank“ und die Hauptschaltfläche **„Datenbank erstellen“**.
- [ ] **AK-7:** Wenn der Administrator „Datenbank erstellen“ wählt, dann legt der Server eine neue
  PostgreSQL-Datenbank an, führt alle Migrationen aus und trägt den Mandanten mit Status **„Bereit“** in der
  Portal-Datenbank ein. Während des Erstellens zeigt die Seite einen Ladezustand; die Schaltfläche ist gesperrt.
- [ ] **AK-8:** Der Name der Datenbank wird **vom Server erzeugt** (Muster `uq_m_<laufende Nummer, 6-stellig>`)
  und nie aus einer Eingabe übernommen.
- [ ] **AK-9:** Gegeben ein Fehler beim Erstellen oder bei den Migrationen, dann wird eine halb angelegte
  Datenbank wieder entfernt, der Mandant erhält Status **„Fehler“**, und der Benutzer sieht eine verständliche
  Meldung mit der Möglichkeit, es erneut zu versuchen.
- [ ] **AK-10:** Gegeben die vorläufige Regel **„eine Datenbank pro Administrator“** (siehe Offene Fragen),
  wenn bereits eine Datenbank besteht, dann wird „Datenbank erstellen“ nicht angezeigt, und ein direkter
  Aufruf der Server-Funktion wird abgelehnt – auch bei zwei gleichzeitigen Anfragen (Doppelklick) entsteht
  höchstens eine Datenbank. Die Grenze steht als Prüfregel im Modell und ist an einer Stelle änderbar.
- [ ] **AK-11:** Beim Erstellen wird der Administrator in der neuen Datenbank als **erster aktiver Benutzer**
  (ANF-005) mit Adresse vom Typ „Person“ (ANF-002) angelegt, mit den Angaben aus der Registrierung.

**Mandant definieren (Modal-Fenster)**
- [ ] **AK-12:** Direkt nach erfolgreichem Erstellen öffnet sich das Modal-Fenster **„Mandant definieren“**
  (`/portal?mandant=<kennung>`), damit Neuladen und Zurück-Taste funktionieren.
- [ ] **AK-13:** Das Modal-Fenster enthält Firmenname (Pflicht, z. B. „Firma AG“), Zusatz, Strasse, PLZ, Ort,
  Land (Vorgabe Schweiz), Telefon, E-Mail, Webseite und UID (optional, Prüfung wie ANF-025).
- [ ] **AK-14:** Beim Speichern wird in der **Mandanten-Datenbank** eine Adresse vom Typ „Firma“ (ANF-002) und
  die eigene Firma (ANF-025) mit dieser Adresse angelegt; in der Portal-Datenbank wird der Firmenname beim
  Mandanten gespeichert. Danach leitet das System auf die Startseite des ERP weiter.
- [ ] **AK-15:** Gegeben ein Mandant ohne definierte Firma, dann sind die Seiten des ERP nicht erreichbar:
  der Benutzer wird nach `/portal` geleitet, wo das Modal-Fenster erneut erscheint. Wird es geschlossen,
  zeigt `/portal` den Hinweis „Mandant noch nicht definiert“ mit der Schaltfläche **„Mandant definieren“**.
- [ ] **AK-16:** Der Firmenname erscheint danach in der Kopfzeile des ERP (bzw. auf `/portal` in der Liste
  der Datenbanken).

**Zugriff und Sicherheit**
- [ ] **AK-17:** Datenbanken werden nur von einer eigenen PostgreSQL-Rolle mit dem Recht `CREATEDB`
  angelegt, die nur der Portal-Teil des Servers verwendet. Die Rolle für den laufenden ERP-Betrieb hat
  dieses Recht **nicht**.
- [ ] **AK-18:** Jede Anfrage an das ERP arbeitet mit der Datenbank des Mandanten aus der Sitzung.
  Server-Funktionen holen den Verbindungspool weiterhin über `crate::db::pool()`; dieser liefert den Pool
  des Mandanten.
- [ ] **AK-19:** Gegeben ein Benutzer, der eine fremde Mandanten-Kennung in der Webadresse oder in einer
  Anfrage einsetzt, dann wird der Zugriff abgelehnt (Antwort wie „nicht gefunden“), ohne zu verraten, ob es
  den Mandanten gibt.
- [ ] **AK-20:** Beim Serverstart werden ausstehende Migrationen für die Portal-Datenbank und für alle
  Mandanten-Datenbanken mit Status „Bereit“ ausgeführt.
- [ ] **AK-21:** Die bisherige **Ersteinrichtung** (ANF-007 AK-21/22) entfällt; ihre Aufgabe übernimmt
  die Registrierung (AK-4) mit dem Anlegen des ersten Benutzers (AK-11).

**Nicht-funktional**
- [ ] **AK-22:** Das Erstellen einer Datenbank inklusive Migrationen dauert auf Railway höchstens 30 Sekunden.
- [ ] **AK-23:** Pro Mandant werden höchstens 3 Datenbankverbindungen gleichzeitig gehalten; unbenutzte
  Pools werden nach einer Wartezeit geschlossen, damit die Verbindungsgrenze von PostgreSQL eingehalten wird.

## 4. Daten / Felder

**Portal-Datenbank `uniqwork_portal`** (Variable `PORTAL_DATABASE_URL`)

`portal_benutzer`

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| `id` | Zahl | Ja | Technischer Schlüssel |
| `vorname`, `nachname` | Text | Ja | |
| `email` | Text | Ja | Anmeldename, eindeutig ohne Rücksicht auf Gross-/Kleinschreibung |
| `passwort_hash` | Text | Ja | wie ANF-007 |
| `ist_administrator` | Wahrheitswert | Ja | bei Registrierung immer `true` |
| `erstellt_am` | Zeitpunkt | Ja | |

`mandant`

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| `id` | Zahl | Ja | Technischer Schlüssel, Grundlage für den Datenbanknamen |
| `besitzer_id` | Verweis auf `portal_benutzer` | Ja | Administrator, der die Datenbank erstellt hat |
| `datenbankname` | Text | Ja | vom Server erzeugt, eindeutig, z. B. `uq_m_000001` |
| `firmenname` | Text | Nein | leer, bis der Mandant definiert ist (AK-14) |
| `status` | Auswahl | Ja | `wird_erstellt`, `bereit`, `fehler` |
| `erstellt_am` | Zeitpunkt | Ja | |

`portal_sitzung`: wie `sitzung` aus ANF-007, zusätzlich `mandant_id` (gewählter Mandant, kann leer sein).

**Mandanten-Datenbank:** unverändert die Tabellen aus `migrations/`. Die Tabelle `benutzer` erhält einen
Verweis `portal_benutzer_id`, damit der angemeldete Zugang dem Benutzer im ERP zugeordnet werden kann.

## 5. Abhängigkeiten

- Hängt ab von: ANF-002 (Adressen), ANF-005 (Benutzer), ANF-007 (Anmeldung, Sitzungen, Zugangsschutz),
  ANF-008 (Betrieb auf Railway), ANF-025 (Firma – wird im Modal-Fenster angelegt)
- Ändert: ANF-007 (Ersteinrichtung entfällt, Anmeldung über die Portal-Datenbank)
- Wird benötigt von: künftige Anforderung „Rollen und Berechtigungen“, Einladen weiterer Benutzer

## 6. Nicht im Umfang

- Einladen weiterer Benutzer in eine Mandanten-Datenbank (eigene Anforderung)
- Rollen und Berechtigungen über das Kennzeichen „Administrator“ hinaus
- Löschen, Umbenennen, Sichern und Wiederherstellen einer Mandanten-Datenbank durch den Benutzer
- Bezahlung, Abonnements, Testzeiträume
- Eigene Unteradressen pro Mandant (z. B. `firma.uniqwork.ch`) – der Mandant ergibt sich aus der Sitzung
- Übernahme der bestehenden Datenbank in das neue Modell (eigene Aufgabe bei der Umsetzung)

## 7. Offene Fragen

- [ ] **Darf ein Administrator mehrere Datenbanken erstellen?** *(vorläufig: genau eine; das Datenmodell
  erlaubt mehrere, die Grenze ist eine Prüfregel, AK-10. Bei mehreren braucht es nach der Anmeldung eine
  Auswahl der Datenbank und einen Wechsel in der Kopfzeile.)*
- [ ] Muss die E-Mail-Adresse bei der Registrierung bestätigt werden (Bestätigungsmail)? *(vorläufig: nein)*
- [ ] Ist die Registrierung für alle offen oder nur mit Einladungscode? Schutz gegen massenhafte
  Registrierungen (z. B. Begrenzung pro IP-Adresse, Prüfaufgabe)? *(vorläufig: offen, mit Begrenzung wie ANF-007)*
- [ ] Wie wird die heute bestehende Datenbank (mit echten Daten) zum ersten Mandanten?
- [ ] Liegen Portal- und Mandanten-Datenbanken im selben PostgreSQL-Dienst auf Railway?
  *(vorläufig: ja; Grenze der Verbindungen und des Speichers beachten, AK-23)*
- [ ] Ab welcher Anzahl Mandanten ist eine andere Aufteilung (Schema pro Mandant) zu prüfen?

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-23 | Hatem Ben Belgacem | Erstellt        |
