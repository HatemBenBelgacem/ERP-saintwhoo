# ANF-026: Registrierung, eigene Datenbank und Mandant

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-026                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Portal / Mandanten                                |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-23                                        |
| **Geändert**  | 2026-11-24                                        |

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

Die **Registrierung läuft in einer eigenen Datenbank**, der **Portal-Datenbank**. Sie enthält nur Zugänge,
Sitzungen und das Verzeichnis der Mandanten – keine ERP-Daten. Sie besteht schon vor der ersten Registrierung
und ist unabhängig davon, ob und wie viele Mandanten-Datenbanken es gibt.
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
- [x] **AK-1:** Gegeben die Anmeldeseite, dann gibt es einen Verweis **„Registrieren“** auf `/registrieren`.
  Die Seite ist ohne Anmeldung erreichbar (wie `/anmelden`, ohne Grundgerüst).
- [x] **AK-2:** Gegeben `/registrieren`, dann enthält das Formular Vorname, Nachname, E-Mail (= Anmeldename),
  Passwort und Passwort-Wiederholung. Es gelten die Passwortregeln aus ANF-005/ANF-007.
- [x] **AK-3:** Gegeben eine bereits registrierte E-Mail (Gross-/Kleinschreibung egal), wenn sich jemand
  damit registriert, dann erscheint eine Fehlermeldung und es wird kein Zugang angelegt.
- [x] **AK-4:** Gegeben eine gültige Registrierung, dann wird ein Zugang mit Kennzeichen **Administrator**
  angelegt, der Benutzer ist sofort angemeldet und landet auf der Portal-Übersicht `/portal`.
- [x] **AK-5:** Das Passwort wird nur als Hash gespeichert (Verfahren wie ANF-007); Schutz gegen
  wiederholte Fehlversuche wie ANF-007.

**Eigene Datenbank für die Registrierung (Portal-Datenbank)**
- [x] **AK-6:** Registrierung, Anmeldung und Sitzungen laufen in einer **eigenen Datenbank**
  `uniqwork_portal`, getrennt von allen Mandanten-Datenbanken. Die Registrierung schreibt ausschliesslich in
  diese Datenbank; es wird dabei **keine** Mandanten-Datenbank angelegt oder verändert.
- [x] **AK-7:** Die Portal-Datenbank wird über eine eigene Variable `PORTAL_DATABASE_URL` angesprochen und hat
  eigene Migrationen im Ordner `migrations_portal/`. Die Migrationen in `migrations/` gelten nur für
  Mandanten-Datenbanken. Ist `PORTAL_DATABASE_URL` leer, gilt `DATABASE_URL` mit dem Datenbanknamen
  `uniqwork_portal` (gleicher PostgreSQL-Dienst, keine zusätzliche Einstellung auf Railway nötig).
- [x] **AK-8:** Die Portal-Datenbank enthält keine ERP-Tabellen (Adressen, Projekte, Konten …). Die
  Mandanten-Datenbanken enthalten keine Zugangsdaten des Portals (E-Mail als Anmeldename, Passwort-Hash,
  Sitzungen), sondern nur den Verweis `portal_benutzer_id`.
- [x] **AK-9:** Die Portal-Datenbank ist die einzige Datenbank, die der Server beim Start selbst anlegt, falls
  sie fehlt (wie heute `src/db.rs`). Die erste Registrierung funktioniert damit auf einem leeren
  PostgreSQL-Dienst, ohne dass es schon eine Mandanten-Datenbank gibt.

**Datenbank erstellen**
- [x] **AK-10:** Gegeben ein angemeldeter Administrator ohne Datenbank, dann zeigt `/portal` den Hinweis
  „Noch keine Datenbank“ und die Hauptschaltfläche **„Datenbank erstellen“**.
- [x] **AK-11:** Wenn der Administrator „Datenbank erstellen“ wählt, dann legt der Server eine neue
  PostgreSQL-Datenbank an, führt alle Migrationen aus und trägt den Mandanten mit Status **„Bereit“** in der
  Portal-Datenbank ein. Während des Erstellens zeigt die Seite einen Ladezustand; die Schaltfläche ist gesperrt.
- [x] **AK-12:** Der Name der Datenbank wird **vom Server erzeugt** (Muster `uq_m_<laufende Nummer, 6-stellig>`)
  und nie aus einer Eingabe übernommen.
- [x] **AK-13:** Gegeben ein Fehler beim Erstellen oder bei den Migrationen, dann wird eine halb angelegte
  Datenbank wieder entfernt, der Mandant erhält Status **„Fehler“**, und der Benutzer sieht eine verständliche
  Meldung mit der Möglichkeit, es erneut zu versuchen.
- [x] **AK-14:** Gegeben die Regel **„eine Datenbank pro Administrator“**,
  wenn bereits eine Datenbank besteht, dann wird „Datenbank erstellen“ nicht angezeigt, und ein direkter
  Aufruf der Server-Funktion wird abgelehnt – auch bei zwei gleichzeitigen Anfragen (Doppelklick) entsteht
  höchstens eine Datenbank. Die Grenze steht als Prüfregel im Modell und ist an einer Stelle änderbar.
- [x] **AK-15:** Beim Erstellen wird der Administrator in der neuen Datenbank als **erster aktiver Benutzer**
  (ANF-005) mit Adresse vom Typ „Person“ (ANF-002) angelegt, mit den Angaben aus der Registrierung.

**Mandant definieren (Modal-Fenster)**
- [x] **AK-16:** Direkt nach erfolgreichem Erstellen öffnet sich das Modal-Fenster **„Mandant definieren“**
  (`/portal?mandant=<kennung>`), damit Neuladen und Zurück-Taste funktionieren.
- [x] **AK-17:** Das Modal-Fenster enthält Firmenname (Pflicht, z. B. „Firma AG“), Zusatz, Strasse, PLZ, Ort,
  Land (Vorgabe Schweiz), Telefon, E-Mail, Webseite und UID (optional, Prüfung wie ANF-025).
  Strasse, PLZ und Ort sind wie bei ANF-025 AK-8 Pflicht (Absender der Belege); zusätzlich das Feld Postfach.
- [x] **AK-18:** Beim Speichern wird in der **Mandanten-Datenbank** eine Adresse vom Typ „Firma“ (ANF-002) und
  die eigene Firma (ANF-025) mit dieser Adresse angelegt; in der Portal-Datenbank wird der Firmenname beim
  Mandanten gespeichert. Danach leitet das System auf die Startseite des ERP weiter.
- [x] **AK-19:** Gegeben ein Mandant ohne definierte Firma, dann sind die Seiten des ERP nicht erreichbar:
  der Benutzer wird nach `/portal` geleitet, wo das Modal-Fenster erneut erscheint. Wird es geschlossen,
  zeigt `/portal` den Hinweis „Mandant noch nicht definiert“ mit der Schaltfläche **„Mandant definieren“**.
- [x] **AK-20:** Der Firmenname erscheint danach in der Kopfzeile des ERP (bzw. auf `/portal` in der Liste
  der Datenbanken).

**Zugriff und Sicherheit**
- [x] **AK-21:** Datenbanken werden nur von einer eigenen PostgreSQL-Rolle mit dem Recht `CREATEDB`
  angelegt, die nur der Portal-Teil des Servers verwendet. Die Rolle für den laufenden ERP-Betrieb hat
  dieses Recht **nicht**.
- [x] **AK-22:** Jede Anfrage an das ERP arbeitet mit der Datenbank des Mandanten aus der Sitzung.
  Server-Funktionen holen den Verbindungspool weiterhin über `crate::db::pool()`; dieser liefert den Pool
  des Mandanten.
- [x] **AK-23:** Gegeben ein Benutzer, der eine fremde Mandanten-Kennung in der Webadresse oder in einer
  Anfrage einsetzt, dann wird der Zugriff abgelehnt (Antwort wie „nicht gefunden“), ohne zu verraten, ob es
  den Mandanten gibt.
- [x] **AK-24:** Beim Serverstart werden ausstehende Migrationen für die Portal-Datenbank und für alle
  Mandanten-Datenbanken mit Status „Bereit“ ausgeführt.
- [x] **AK-25:** Die bisherige **Ersteinrichtung** (ANF-007 AK-21/22) entfällt; ihre Aufgabe übernimmt
  die Registrierung (AK-4) mit dem Anlegen des ersten Benutzers (AK-15).

**Passwort und Benutzerverwaltung (Schritt 3b)**
- [x] **AK-28:** Gegeben die Portal-Übersicht, dann zeigt der Abschnitt „Ihr Zugang“ die E-Mail und die Schaltfläche
  **„Passwort ändern“** (Modal-Fenster `/portal?passwort=1`): bisheriges Passwort, neues Passwort mit Wiederholung
  (Regeln wie AK-2). Ein falsches bisheriges Passwort wird abgelehnt und zählt als Fehlversuch (Sperre wie ANF-007);
  das neue muss sich vom bisherigen unterscheiden. Nach dem Speichern enden alle anderen Sitzungen des Zugangs, die
  laufende bleibt. Aus dem ERP führt der Name in der Kopfzeile zum Portal.
- [x] **AK-29:** Die Benutzerverwaltung des ERP (ANF-005) kennt keine Passwörter mehr: Anlegen ohne Passwort, kein
  „Passwort setzen“, keine „Letzte Anmeldung“. Die Detailansicht zeigt „Zugang im Portal: Ja/Nein“ – anmelden kann
  sich nur ein Benutzer, der mit einem Zugang des Portals verknüpft ist.

**Nicht-funktional**
- [x] **AK-26:** Das Erstellen einer Datenbank inklusive Migrationen dauert auf Railway höchstens 30 Sekunden.
- [x] **AK-27:** Pro Mandant werden höchstens 3 Datenbankverbindungen gleichzeitig gehalten; unbenutzte
  Pools werden nach einer Wartezeit geschlossen, damit die Verbindungsgrenze von PostgreSQL eingehalten wird.

## 4. Daten / Felder

**Portal-Datenbank `uniqwork_portal`** (Variable `PORTAL_DATABASE_URL`, Migrationen in `migrations_portal/`)

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
| `firmenname` | Text | Nein | leer, bis der Mandant definiert ist (AK-18) |
| `status` | Auswahl | Ja | `wird_erstellt`, `bereit`, `fehler` |
| `erstellt_am` | Zeitpunkt | Ja | |

`portal_sitzung`: wie `sitzung` aus ANF-007, zusätzlich `mandant_id` (gewählter Mandant, kann leer sein).

**Mandanten-Datenbank:** unverändert die Tabellen aus `migrations/`. Die Tabelle `benutzer` erhält einen
Verweis `portal_benutzer_id`, damit der angemeldete Zugang dem Benutzer im ERP zugeordnet werden kann.
Passwort und Sitzungen stehen nur in der Portal-Datenbank (AK-8).

## 5. Abhängigkeiten

- Hängt ab von: ANF-002 (Adressen), ANF-005 (Benutzer), ANF-007 (Anmeldung, Sitzungen, Zugangsschutz),
  ANF-008 (Betrieb auf Railway), ANF-025 (Firma – wird im Modal-Fenster angelegt)
- Ändert: ANF-007 (Ersteinrichtung entfällt; Anmeldung und Sitzungen nur noch über die Portal-Datenbank),
  ANF-005 (Benutzer in der Mandanten-Datenbank ohne Passwort; Passwort ändern/zurücksetzen erfolgt im Portal)
- Wird benötigt von: künftige Anforderung „Rollen und Berechtigungen“, Einladen weiterer Benutzer

## 6. Nicht im Umfang

- Einladen weiterer Benutzer in eine Mandanten-Datenbank (eigene Anforderung)
- Rollen und Berechtigungen über das Kennzeichen „Administrator“ hinaus
- Löschen, Umbenennen, Sichern und Wiederherstellen einer Mandanten-Datenbank durch den Benutzer
- Bezahlung, Abonnements, Testzeiträume
- Eigene Unteradressen pro Mandant (z. B. `firma.uniqwork.ch`) – der Mandant ergibt sich aus der Sitzung
- Übernahme der bestehenden Datenbank in das neue Modell (eigene Aufgabe bei der Umsetzung)

## 7. Offene Fragen

- [x] **Darf ein Administrator mehrere Datenbanken erstellen?** **Entschieden: nein, genau eine Datenbank
  pro Administrator** (AK-14). Das Datenmodell erlaubt mehrere, die Grenze ist eine Prüfregel; eine spätere
  Öffnung bräuchte eine Auswahl der Datenbank nach der Anmeldung und einen Wechsel in der Kopfzeile.
- [x] Muss die E-Mail-Adresse bei der Registrierung bestätigt werden (Bestätigungsmail)? **Entschieden: nein.**
- [x] Ist die Registrierung für alle offen oder nur mit Einladungscode? Schutz gegen massenhafte
  Registrierungen? **Entschieden: offen für alle, mit Begrenzung der Versuche wie ANF-007.**
- [ ] Wie wird die heute bestehende Datenbank (mit echten Daten) zum ersten Mandanten? *(blockiert die
  Umsetzung nicht; vor der Inbetriebnahme auf Railway klären)*
- [x] Liegen Portal- und Mandanten-Datenbanken im selben PostgreSQL-Dienst auf Railway?
  **Entschieden: ja**; Grenze der Verbindungen und des Speichers beachten (AK-27).
- [x] Was geschieht mit Passwort und Sitzungen der Benutzerverwaltung in der Mandanten-Datenbank
  (ANF-005, ANF-007)? **Entschieden: Anmeldung nur über das Portal.** Passwort und Sitzungen entfallen in der
  Mandanten-Datenbank (AK-8).
- [ ] Firmenname im Portal nachführen, wenn die Firma im ERP umbenannt wird? Die Kopfzeile zeigt den Namen aus dem
  Portal (Stand beim Definieren des Mandanten). *(vorläufig: nein; Änderung über Einstellungen › Firma wirkt nur im ERP)*
- [ ] Ab welcher Anzahl Mandanten ist eine andere Aufteilung (Schema pro Mandant) zu prüfen? *(blockiert die
  Umsetzung nicht; im Betrieb beobachten)*

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-23 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-23 | Hatem Ben Belgacem | Registrierung in eigener Portal-Datenbank (AK-6 bis AK-9), folgende AK neu nummeriert |
| 2026-11-23 | Hatem Ben Belgacem | Entschieden: Anmeldung nur über das Portal |
| 2026-11-23 | Hatem Ben Belgacem | Entschieden: eine Datenbank pro Administrator, vorläufige Annahmen übernommen; Status Freigegeben |
| 2026-11-23 | Hatem Ben Belgacem | Schritt 1 umgesetzt: Portal-Datenbank mit eigener Verbindung und Migrationen (AK-7); Status In Umsetzung |
| 2026-11-23 | Hatem Ben Belgacem | Schritt 2 umgesetzt: Registrierung, Anmeldung über das Portal, Portal-Übersicht, Zugang zum ERP nur mit Mandant (AK-1 bis AK-6, AK-9) |
| 2026-11-23 | Hatem Ben Belgacem | Schritt 3 umgesetzt: Datenbank erstellen, Rolle `uniqwork_erp`, Pools je Mandant, Migrationen beim Start (AK-10 bis AK-15, AK-21 bis AK-27) |
| 2026-11-23 | Hatem Ben Belgacem | Schritt 4 umgesetzt: Modal-Fenster „Mandant definieren“, Firmenname in der Kopfzeile (AK-16 bis AK-20) |
| 2026-11-24 | Hatem Ben Belgacem | Schritt 3b umgesetzt: keine Zugangsdaten mehr in den Mandanten-Datenbanken (AK-8), Passwort ändern im Portal (AK-28), Benutzerverwaltung ohne Passwort (AK-29); Status Erledigt |
