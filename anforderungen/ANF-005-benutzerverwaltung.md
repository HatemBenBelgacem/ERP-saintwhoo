# ANF-005: Benutzer verwalten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-005                                           |
| **Status**    | Erledigt                                          |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Benutzer                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-12                                        |
| **Geändert**  | 2026-11-24                                        |

> **Geändert durch ANF-026 (2026-11-24):** Angemeldet wird nur noch über das **Portal** (E-Mail und Passwort des
> Zugangs). Benutzer haben in der Mandanten-Datenbank **kein Passwort** und keine „Letzte Anmeldung“ mehr; Passwort
> setzen (AK-15) entfällt, das eigene Passwort wird im Portal geändert (ANF-026 AK-28). Neu zeigt die Detailansicht,
> ob der Benutzer mit einem Zugang im Portal verknüpft ist (`portal_benutzer_id`).

## 1. Beschreibung

Das ERP führt eine Liste der Benutzer, die mit dem System arbeiten dürfen. Ein Benutzer ist ein
Zugang zum System (Benutzername, Passwort, aktiv/gesperrt). Die persönlichen Angaben wie Name,
E-Mail und Telefon werden **nicht** im Benutzer gespeichert, sondern in einer Adresse aus dem
Adressstamm (ANF-002).

**Beziehung Benutzer – Adresse: 1:1**

- Jeder Benutzer gehört zu **genau einer** Adresse (Pflicht).
- Jede Adresse gehört zu **höchstens einem** Benutzer.
- Die Adresse muss vom Typ **„Person“** sein.

So gibt es Name und Kontaktdaten eines Mitarbeiters nur an einer Stelle; eine Änderung der Adresse
wirkt sofort auch beim Benutzer.

**Verantwortung in Projekten (ANF-003)**

Bei einem Projekt, einem Teilprojekt und einem Vorgang kann je **ein Benutzer als verantwortlich**
hinterlegt werden (freiwillig). Die Verantwortung gilt nach unten für alles, was darunter liegt:

| Benutzer hinterlegt bei | Verantwortlich für |
|-------------------------|--------------------|
| **Projekt**     | das Projekt sowie **alle** Teilprojekte, Vorgänge und Aufgaben dieses Projekts |
| **Teilprojekt** | das Teilprojekt sowie **alle** Vorgänge und Aufgaben dieses Teilprojekts |
| **Vorgang**     | den Vorgang sowie **alle** Aufgaben dieses Vorgangs |

Aufgaben haben **keinen eigenen** verantwortlichen Benutzer; sie erhalten die Verantwortung immer von
ihrem Vorgang, Teilprojekt und Projekt.

Beispiel:

```
Projekt P-2026-001        Verantwortlich: Anna    → Anna für alles im Projekt
└── Teilprojekt 2         Verantwortlich: Ben     → Ben für Teilprojekt 2 und alles darunter
    ├── Vorgang 2.1       (niemand hinterlegt)
    │   └── Aufgabe 2.1.1                         → verantwortlich: Anna, Ben
    └── Vorgang 2.3       Verantwortlich: Carla   → Carla für Vorgang 2.3 und seine Aufgaben
        └── Aufgabe 2.3.1                         → verantwortlich: Anna, Ben, Carla
```

Sind auf mehreren Ebenen Benutzer hinterlegt, sind **alle** verantwortlich (siehe Offene Fragen).

Die Benutzer lassen sich mit den vier Grundfunktionen verwalten:

| Grundfunktion | Bedeutung |
|---------------|-----------|
| **Anlegen**    | Neuen Benutzer erfassen und mit einer Adresse verknüpfen |
| **Anzeigen**   | Benutzer als Liste anzeigen, suchen und einen Benutzer im Detail ansehen |
| **Bearbeiten** | Benutzername, Adresse, Status ändern; Passwort neu setzen |
| **Löschen**    | Benutzer entfernen (die Adresse bleibt erhalten) |

Die Benutzerverwaltung ist eine Einstellungsseite: Eintrag **„Benutzer“** in der
Einstellungsnavigation (ANF-001) unter dem Pfad `/einstellungen/benutzer`.

## 2. Benutzergeschichte

> Als **Verwalter des Systems** möchte ich **Benutzer anlegen, sperren, ändern und löschen und jedem
> Benutzer seine Adresse zuordnen**, damit **ich steuern kann, wer mit dem ERP arbeitet, und die
> Personendaten nur einmal im Adressstamm gepflegt werden**.

> Als **Projektleiter** möchte ich **bei einem Projekt, Teilprojekt oder Vorgang einen verantwortlichen
> Benutzer hinterlegen**, damit **für alles darunter klar ist, wer verantwortlich ist, ohne jede Aufgabe
> einzeln zuzuweisen**.

> Als **Mitarbeiter** möchte ich **sehen, wofür ich verantwortlich bin**, damit **ich nichts übersehe**.

## 3. Akzeptanzkriterien

**Liste (Anzeigen)**
- [x] **AK-1:** Gegeben der Eintrag „Benutzer“ in der Einstellungsnavigation, wenn der Benutzer ihn
  anklickt, dann erscheint eine Tabelle aller Benutzer mit den Spalten Benutzername, Name (Anzeigename
  der Adresse), E-Mail (aus der Adresse) und Status. *(„Letzte Anmeldung“ entfällt, ANF-026.)*
- [x] **AK-2:** Gegeben die Benutzerliste, wenn ein Suchbegriff eingegeben wird, dann zeigt die Liste nur
  Benutzer, deren Benutzername, Name oder E-Mail den Begriff enthält (Gross-/Kleinschreibung egal).
- [x] **AK-3:** Gegeben die Benutzerliste, dann kann nach Status gefiltert werden: alle, aktiv, gesperrt.
- [x] **AK-4:** Gegeben die Benutzerliste, wenn auf einen Spaltenkopf geklickt wird, dann wird nach
  dieser Spalte sortiert; ein zweiter Klick kehrt die Reihenfolge um. Standard: Benutzername aufsteigend.
- [x] **AK-5:** Suche, Filter und Sortierung stehen in der Webadresse, damit Neuladen und Zurück-Taste
  den Zustand erhalten. Gibt es keine Treffer, erscheint ein Hinweis statt einer leeren Tabelle.

**Detailansicht (Anzeigen)**
- [x] **AK-6:** Gegeben die Benutzerliste, wenn eine Zeile angeklickt wird, dann öffnet sich die
  Detailansicht unter `/einstellungen/benutzer/<kennung>` mit Benutzername, Status, Zugang im Portal (ANF-026),
  Erstellt am, Geändert am sowie den Angaben der verknüpften Adresse (Nummer, Name, E-Mail, Telefon,
  Mobiltelefon).
- [x] **AK-7:** Gegeben die Detailansicht, dann führt ein Verweis auf die verknüpfte Adresse zu deren
  Detailansicht unter `/adressen/<kennung>`.
- [x] **AK-8:** Gegeben die Detailansicht einer Adresse (ANF-002), die mit einem Benutzer verknüpft ist,
  dann zeigt sie einen Hinweis „Benutzer: <Benutzername>“ mit Verweis auf den Benutzer.

**Anlegen**
- [x] **AK-9:** Gegeben die Benutzerliste, wenn auf „Neuer Benutzer“ geklickt wird, dann öffnet sich ein
  leeres Formular in einem Modal-Fenster (Webadresse `/einstellungen/benutzer?neu=1`).
- [x] **AK-10:** Gegeben das Formular, dann wird die Adresse über ein Suchfeld (`SearchPicker`) gewählt.
  Angeboten werden nur Adressen vom Typ „Person“, die noch **keinem** Benutzer zugeordnet sind.
- [x] **AK-11:** Gegeben das Formular, dann sind Benutzername und Adresse Pflicht *(Passwort entfällt, ANF-026)*. Der Status ist standardmässig „aktiv“.
- [x] **AK-12:** Gegeben ein gültig ausgefülltes Formular, wenn gespeichert wird, dann wird der Benutzer
  angelegt, eine Erfolgsmeldung erscheint und die Detailansicht des neuen Benutzers öffnet sich.
- [x] **AK-13:** Gegeben ein ungültiges Formular, wenn gespeichert wird, dann wird nichts gespeichert und
  die fehlerhaften Felder werden mit einer verständlichen Meldung markiert, insbesondere:
  - Benutzername leer, zu lang oder mit unerlaubten Zeichen (erlaubt: Kleinbuchstaben, Ziffern, `.`, `-`, `_`)
  - Benutzername bereits vergeben (Gross-/Kleinschreibung egal)
  - Adresse fehlt, ist keine Person oder ist bereits einem anderen Benutzer zugeordnet
  - ~~Passwort kürzer als 10 Zeichen oder Wiederholung stimmt nicht überein~~ *(entfällt, ANF-026)*

  Die Prüfung erfolgt auf dem Server (Prüfregeln in `model.rs`); die Datenbank sichert die
  Eindeutigkeit von Benutzername und Adresse zusätzlich ab.

**Bearbeiten**
- [x] **AK-14:** Gegeben die Detailansicht, wenn auf „Bearbeiten“ geklickt wird, dann öffnet sich das
  Formular mit den aktuellen Werten in einem Modal-Fenster
  (Webadresse `/einstellungen/benutzer/<kennung>?bearbeiten=1`). Benutzername, Adresse und Status sind
  änderbar; die Passwortfelder erscheinen hier nicht.
- [x] ~~**AK-15:**~~ *(Entfällt, ANF-026: Passwort ändern im Portal, ANF-026 AK-28.)* Gegeben die Detailansicht, wenn auf „Passwort setzen“ geklickt wird, dann öffnet sich ein
  eigenes Modal-Fenster (Webadresse `…?passwort=1`) mit Passwort und Wiederholung. Es gelten dieselben
  Regeln wie beim Anlegen.
- [x] **AK-16:** Gegeben ein aktiver Benutzer, wenn der Status auf „gesperrt“ gesetzt wird, dann bleibt
  der Benutzer mit allen Daten erhalten, kann sich aber nicht mehr anmelden (wirksam, sobald die
  Anmeldung umgesetzt ist).
- [x] **AK-17:** Beim Bearbeiten gelten dieselben Prüfungen wie beim Anlegen; „Geändert am“ wird
  aktualisiert. „Abbrechen“ speichert nichts.

**Löschen**
- [x] **AK-18:** Gegeben die Detailansicht, wenn auf „Löschen“ geklickt wird, dann erscheint ein
  Bestätigungsdialog mit Benutzername und Name. Erst nach Bestätigung wird gelöscht und die Liste
  angezeigt. Die verknüpfte Adresse bleibt erhalten. Ausnahme: Benutzer, die in Projekten hinterlegt
  sind (AK-32).
- [x] **AK-19:** Gegeben eine Adresse, die mit einem Benutzer verknüpft ist, wenn sie in der
  Adressverwaltung gelöscht werden soll, dann wird das Löschen verhindert und die Meldung nennt
  „als Benutzer <Benutzername>“ (Erweiterung von ANF-002 AK-16; Fremdschlüssel ohne `ON DELETE CASCADE`).
- [x] **AK-20:** Gegeben die Adresse eines Benutzers, wenn ihr Typ in der Adressverwaltung von „Person“
  auf „Firma“ geändert werden soll, dann wird das verhindert und eine Meldung nennt den Benutzer.

**Sicherheit**
- [x] **AK-21:** *(Gilt seit ANF-026 für die Zugänge der Portal-Datenbank.)* Passwörter werden nie im Klartext gespeichert, sondern nur als Hash mit einem
  anerkannten Verfahren (Argon2id) mit Salz. Hash und Passwort verlassen den Server nie
  (nicht in Liste, Detail oder Server-Antworten).

**Allgemein**
- [x] **AK-22:** Gegeben eine nicht existierende Benutzerkennung in der Webadresse, dann erscheint die
  Meldung „Benutzer nicht gefunden“.
- [x] **AK-23:** Die Tabelle wird per sqlx-Migration angelegt.

**Verantwortung in Projekten**
- [x] **AK-24:** Gegeben das Formular eines Projekts, Teilprojekts oder Vorgangs (ANF-003, Anlegen und
  Bearbeiten), dann gibt es das freiwillige Feld „Verantwortlich“. Die Auswahl erfolgt über ein Suchfeld
  (`SearchPicker`); angeboten werden nur **aktive** Benutzer. Das Feld kann leer bleiben oder geleert
  werden. Der Server prüft, dass der gewählte Benutzer existiert und aktiv ist.
  Das Formular der Aufgabe erhält **kein** solches Feld.
- [x] **AK-25:** Gegeben ein Projekt mit hinterlegtem Benutzer, dann ist dieser Benutzer verantwortlich für
  das Projekt und alle seine Teilprojekte, Vorgänge und Aufgaben.
- [x] **AK-26:** Gegeben ein Teilprojekt mit hinterlegtem Benutzer, dann ist dieser Benutzer verantwortlich
  für das Teilprojekt und alle seine Vorgänge und Aufgaben – nicht aber für andere Teilprojekte desselben
  Projekts.
- [x] **AK-27:** Gegeben ein Vorgang mit hinterlegtem Benutzer, dann ist dieser Benutzer verantwortlich für
  den Vorgang und alle seine Aufgaben – nicht aber für andere Vorgänge desselben Teilprojekts.
- [x] **AK-28:** Gegeben ein Element mit hinterlegten Benutzern auf mehreren übergeordneten Ebenen, dann sind
  alle diese Benutzer verantwortlich. Die Detailansicht jedes Elements zeigt unter „Verantwortlich“ alle
  Benutzer mit ihrer Herkunft (z. B. „Anna – vom Projekt“, „Ben – vom Teilprojekt 2“, „Carla – hier
  hinterlegt“). Ist derselbe Benutzer auf mehreren Ebenen hinterlegt, erscheint er nur einmal (mit der
  obersten Ebene als Herkunft). Ist niemand verantwortlich, steht dort „–“.
- [x] **AK-29:** Die Verantwortung wird nicht bei jedem Element gespeichert, sondern beim Anzeigen aus der
  Hierarchie ermittelt. Wird der Benutzer bei einem Projekt, Teilprojekt oder Vorgang geändert, gilt das
  sofort für alle Elemente darunter, auch für später angelegte.
- [x] **AK-30:** Gegeben die Projektübersicht bzw. der Strukturbaum (ANF-003), dann wird bei Projekten,
  Teilprojekten und Vorgängen der dort **hinterlegte** Benutzer angezeigt (nicht die geerbten).
- [x] **AK-31:** Gegeben die Detailansicht eines Benutzers, dann zeigt ein Abschnitt „Verantwortung“
  - alle Projekte, Teilprojekte und Vorgänge, bei denen er hinterlegt ist (Ebene, Nummer, Name, Projekt),
    jeweils mit Verweis auf das Element;
  - die Anzahl der offenen Aufgaben, für die er (hinterlegt oder geerbt) verantwortlich ist, und auf
    Wunsch deren Liste (Nummer, Titel, Projekt, Status) mit Verweis auf die Aufgabe.

  Gibt es nichts, erscheint ein Hinweis statt einer leeren Tabelle.
- [x] **AK-32:** Gegeben ein Benutzer, der bei mindestens einem Projekt, Teilprojekt oder Vorgang hinterlegt
  ist, wenn er gelöscht werden soll, dann wird das Löschen verhindert. Die Meldung nennt die Anzahl je
  Ebene und empfiehlt, den Benutzer zu sperren oder die Verantwortung neu zu vergeben.
- [x] **AK-33:** Gegeben ein gesperrter Benutzer, dann bleibt er bei seinen Projekten, Teilprojekten und
  Vorgängen hinterlegt und wird dort als „gesperrt“ markiert. Neu hinterlegen lassen sich nur aktive Benutzer.
- [x] **AK-34:** Die Spalten `benutzer_id` in `projekt`, `teilprojekt` und `vorgang` werden per neuer
  sqlx-Migration ergänzt (bestehende Migrationen bleiben unverändert); bestehende Elemente haben danach
  keinen verantwortlichen Benutzer.

## 4. Daten / Felder

Tabelle `benutzer`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Benutzername (`benutzername`) | Text (50) | Ja | Eindeutig (ohne Beachtung der Gross-/Kleinschreibung), nur `a–z`, `0–9`, `.`, `-`, `_` |
| Adresse (`adresse_id`) | Fremdschlüssel → `adresse` | Ja | **Eindeutig** (`UNIQUE`) → 1:1; nur Typ „Person“; ohne `ON DELETE CASCADE` |
| Zugang im Portal (`portal_benutzer_id`) | Ganzzahl | Nein | Verknüpfung zum Zugang der Portal-Datenbank (ANF-026), eindeutig |
| Aktiv (`aktiv`) | Wahrheitswert | Ja | Standard `true`; `false` = gesperrt |
| Erstellt am | Zeitstempel | Ja | automatisch |
| Geändert am | Zeitstempel | Ja | automatisch |

**Anzeigename** des Benutzers: Anzeigename der verknüpften Adresse („Nachname Vorname“), dahinter der
Benutzername in Klammern, z. B. „Muster Anna (anna.muster)“.

Ergänzung der Tabellen aus ANF-003:

| Tabelle | Feld | Typ | Pflicht | Bemerkung |
|---------|------|-----|---------|-----------|
| `projekt` | Verantwortlich (`benutzer_id`) | Fremdschlüssel → `benutzer` | Nein | Index; `ON DELETE RESTRICT` (AK-32) |
| `teilprojekt` | Verantwortlich (`benutzer_id`) | Fremdschlüssel → `benutzer` | Nein | Index; `ON DELETE RESTRICT` (AK-32) |
| `vorgang` | Verantwortlich (`benutzer_id`) | Fremdschlüssel → `benutzer` | Nein | Index; `ON DELETE RESTRICT` (AK-32) |

`aufgabe` erhält **keine** solche Spalte. Die Verantwortlichen einer Aufgabe ergeben sich aus
`vorgang.benutzer_id`, `teilprojekt.benutzer_id` und `projekt.benutzer_id` entlang der Hierarchie (AK-29).

**Beziehungen (Datenbank):**

```
adresse  1 ──── 0..1 benutzer       (benutzer.adresse_id UNIQUE NOT NULL)
benutzer 1 ──── 0..n projekt        (projekt.benutzer_id NULL erlaubt)
benutzer 1 ──── 0..n teilprojekt    (teilprojekt.benutzer_id NULL erlaubt)
benutzer 1 ──── 0..n vorgang        (vorgang.benutzer_id NULL erlaubt)
```

| Tabelle | Bezieht sich auf | Art | Fremdschlüssel | Pflicht | Beim Löschen des Ziels |
|---------|------------------|-----|----------------|---------|------------------------|
| `benutzer` | `adresse` | 1:1 | `benutzer.adresse_id` (eindeutig) | Ja | verhindern |
| `projekt` | `benutzer` | N:1 | `projekt.benutzer_id` | Nein | verhindern |
| `teilprojekt` | `benutzer` | N:1 | `teilprojekt.benutzer_id` | Nein | verhindern |
| `vorgang` | `benutzer` | N:1 | `vorgang.benutzer_id` | Nein | verhindern |

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst, Einstellungsnavigation), ANF-002 (Adressen),
  ANF-003 (Projekte – Projekt, Teilprojekt und Vorgang erhalten das Feld „Verantwortlich“)
- Wird benötigt von: Anmeldung (noch zu erfassen), Rollen und Berechtigungen (noch zu erfassen)

## 6. Nicht im Umfang

- Anmelden und Abmelden, Sitzungen, „Passwort vergessen“ – eigene Anforderung
- Rollen und Berechtigungen (wer darf was) – eigene Anforderung
- Schutz der Benutzerverwaltung selbst (bis zur Anmeldung kann jeder die Seite öffnen)
- Zwei-Faktor-Anmeldung, Anmeldung über externe Dienste (z. B. Microsoft, Google)
- Änderungsprotokoll, wer wann welchen Benutzer geändert hat
- Neue Adresse direkt aus dem Benutzerformular anlegen (vorerst zuerst in der Adressverwaltung anlegen)
- Eigener verantwortlicher Benutzer bei einer einzelnen Aufgabe
- Mehrere verantwortliche Benutzer auf derselben Ebene (z. B. zwei Benutzer bei einem Vorgang)
- Berechtigungen aus der Verantwortung ableiten (wer darf was bearbeiten) – gehört zu Rollen und Berechtigungen
- Benachrichtigung (z. B. E-Mail), wenn ein Benutzer als verantwortlich hinterlegt wird
- Eigene Seite „Meine Aufgaben“ für den angemeldeten Benutzer (setzt die Anmeldung voraus)

## 7. Offene Fragen

Vorläufige Annahmen in Klammern.

- [ ] Muss die Adresse zwingend vom Typ „Person“ sein? *(vorerst: ja)*
- [ ] Löschen endgültig oder nur sperren? *(vorerst: beides möglich; Löschen endgültig nach Bestätigung)*
- [ ] Darf der letzte aktive Benutzer gesperrt oder gelöscht werden? *(vorerst: ja – wird mit der Anmeldung relevant)*
- [ ] Soll beim Anlegen direkt eine neue Adresse erfasst werden können? *(vorerst: nein)*
- [ ] Passwortregeln: Mindestlänge, Zeichenarten, Ablauf? *(vorerst: mindestens 10 Zeichen, keine weiteren Regeln)*
- [ ] Wird ein erster Benutzer automatisch angelegt (z. B. über Umgebungsvariable beim Serverstart)? *(vorerst: nein)*
- [ ] Eigene E-Mail im Benutzer (z. B. für die Anmeldung) oder immer die E-Mail der Adresse? *(vorerst: die der Adresse)*
- [ ] Sind bei Benutzern auf mehreren Ebenen alle verantwortlich, oder ersetzt der Benutzer der tieferen
  Ebene den der höheren (z. B. bei Aufgabe 2.3.1 nur Carla statt Anna, Ben und Carla)? *(vorerst: alle, AK-28)*
- [ ] Benutzer, der in Projekten hinterlegt ist, löschen: verhindern oder Verantwortung automatisch leeren? *(vorerst: verhindern, AK-32)*
- [ ] Soll die Projektliste bzw. der Strukturbaum nach verantwortlichem Benutzer filterbar sein? *(vorerst: nein)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261112150000_benutzer.sql` | Tabelle `benutzer` (Adresse eindeutig → 1:1, Benutzername eindeutig ohne Gross-/Kleinschreibung), Spalte `benutzer_id` in `projekt`, `teilprojekt`, `vorgang` (`ON DELETE RESTRICT`) |
| `src/backend/users/model.rs` | Datentypen, Prüfregeln für Benutzername, Adresse, Status und Passwort, Löschmeldung + Modultests |
| `src/backend/users/password.rs` | Argon2id-Hash mit zufälligem Salz (nur Server) + Modultest |
| `src/backend/users/server_fns.rs` + `repository.rs` | Liste, Detail mit Verantwortung, Suche (aktive Benutzer, freie Personen), Anlegen, Bearbeiten, Passwort setzen, Löschen |
| `src/components/users/list.rs`, `detail.rs`, `form.rs` | Liste mit Suche, Filter und Sortierung; Detailansicht mit Abschnitt „Verantwortung“; Modal-Fenster für Anlegen, Bearbeiten und Passwort |
| `src/backend/projects/model.rs` | `Responsible`, `ProjectTree::responsible_for()` – ermittelt die Verantwortlichen aus der Hierarchie (AK-25 bis AK-29) + Modultests |
| `src/backend/projects/…`, `src/components/projects/…` | Feld „Verantwortlich“ in Projekt, Teilprojekt und Vorgang; Anzeige mit Herkunft (Detailansicht), Plakette im Strukturbaum und Spalte in der Übersicht |
| `src/backend/addresses/…`, `src/components/addresses/detail.rs` | Hinweis „Benutzer: …“ in der Adresse (AK-8), Löschschutz (AK-19), Typ bleibt Person (AK-20) |
| `src/navigation.rs`, `src/app.rs`, `src/forms.rs`, `src/icons.rs` | Eintrag „Benutzer“ in der Einstellungsnavigation, Routen, Fenster-Parameter `passwort`, Symbol |

Hinweise:
- **AK-4:** Die Liste hat keine Seiten (wenige Benutzer); Zeile „n Benutzer“ unter der Tabelle.
- **AK-24, AK-33:** Neu hinterlegen lassen sich nur aktive Benutzer. Ein bereits hinterlegter, inzwischen
  gesperrter Benutzer bleibt beim Bearbeiten erhalten.
- **AK-31:** Die Nummern (z. B. `2.3.1`) werden in SQL nach derselben Reihenfolge wie im Strukturbaum berechnet.
- **Letzte Anmeldung** bleibt leer, bis die Anmeldung umgesetzt ist.

Geprüft: Migration auf einer frischen Testdatenbank; alle Server-Funktionen über HTTP (gültige und ungültige
Eingaben, Vererbung am Beispiel Anna/Ben/Carla, Lösch- und Typschutz, gesperrte Benutzer); Seitenausgabe
aller Ansichten und Modal-Fenster; 54 Modultests. Bedienung im Browser (Suchfelder, Modal-Fenster) noch
nicht von Hand geprüft.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-12 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-12 | Hatem Ben Belgacem | Verantwortung in Projekten ergänzt: Benutzer bei Projekt, Teilprojekt, Vorgang; gilt für alles darunter (AK-24 bis AK-34) |
| 2026-11-12 | Hatem Ben Belgacem | Umgesetzt |
| 2026-11-12 | Hatem Ben Belgacem | ANF-006: offene Aufgaben inkl. Aufgaben ohne Projekt, Löschschutz erweitert |
| 2026-11-12 | Hatem Ben Belgacem | ANF-007: Sitzungen beim Sperren/Passwort setzen beenden, sich nicht selbst sperren/löschen, „Letzte Anmeldung“ |
| 2026-11-24 | Hatem Ben Belgacem | ANF-026: kein Passwort und keine „Letzte Anmeldung“ mehr (AK-1, AK-6, AK-11, AK-13, AK-15), Zugang im Portal |
