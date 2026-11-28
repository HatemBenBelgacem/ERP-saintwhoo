# ANF-029: Anfragen und Einkaufsaufträge aus einem Vorgang

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-029                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Einkauf / Projekte                                |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-28                                        |
| **Geändert**  | 2026-11-28                                        |

## 1. Beschreibung

Aus einem **Vorgang** (ANF-003) sollen zwei Dinge erzeugt werden können, ohne dass Titel und Beschreibung
von Hand abgetippt werden:

- eine **Anfrage**: Sie übernimmt den **Titel des Teilprojekts** und die **Beschreibung des Vorgangs**.
  Jeder Vorgang kann eine oder mehrere Anfragen haben.
- ein **Einkaufsauftrag** (ANF-004): Er wird aus dem Vorgang erzeugt und merkt sich, aus welchem Vorgang
  er stammt.

```
Projekt
└── Teilprojekt  ── Titel ──────────┐
    └── Vorgang  ── Beschreibung ───┴──► Anfrage (ANF-029)
                 └──────────────────────► Einkaufsauftrag (ANF-004)
```

Die Anfrage ist eine Kopie: Spätere Änderungen am Teilprojekt oder am Vorgang ändern sie nicht.

## 2. Benutzergeschichte

> Als **Projektleiter** möchte ich **aus einem Vorgang mit wenigen Klicks eine Anfrage und einen
> Einkaufsauftrag erzeugen**, damit **der Einkauf für einen Arbeitsschritt nicht von Hand neu erfasst
> werden muss und jeder Einkauf dem Vorgang zugeordnet bleibt**.

## 3. Akzeptanzkriterien

**Anfrage**
- [ ] **AK-1:** Gegeben die Detailansicht eines Vorgangs, dann gibt es den Abschnitt „Anfragen“ mit der
  Schaltfläche „Anfrage erstellen“.
- [ ] **AK-2:** Gegeben der Vorgang, wenn der Benutzer „Anfrage erstellen“ klickt, dann wird eine Anfrage
  gespeichert, deren Titel der Name des Teilprojekts und deren Beschreibung die Beschreibung des Vorgangs ist.
  Beide Werte werden auf dem Server aus der Datenbank gelesen, nicht aus dem Formular übernommen.
- [ ] **AK-3:** Gegeben eine neue Anfrage, dann erhält sie eine fortlaufende Nummer je Jahr im Format
  `A-JJJJ-NNNN` (z. B. `A-2026-0001`).
- [ ] **AK-4:** Gegeben ein Vorgang mit Anfragen, dann listet der Abschnitt „Anfragen“ alle Anfragen mit
  Nummer, Titel, Beschreibung und Erstellt am, neueste zuerst. Ohne Anfragen steht dort ein Hinweis.
- [ ] **AK-5:** Gegeben ein Vorgang ohne Beschreibung, dann ist die Beschreibung der Anfrage leer und die
  Liste zeigt „–“.
- [ ] **AK-6:** Gegeben eine Anfrage, wenn der Benutzer „Löschen“ klickt und bestätigt, dann wird nur die
  Anfrage gelöscht. Der Vorgang bleibt unverändert.

**Einkaufsauftrag aus dem Vorgang**
- [ ] **AK-7:** Gegeben die Detailansicht eines Vorgangs, dann gibt es im Abschnitt „Einkaufsaufträge“ die
  Schaltfläche „Einkaufsauftrag erstellen“. Sie öffnet ein Modal-Fenster (Webadresse `?neu=einkauf`).
- [ ] **AK-8:** Gegeben das Modal-Fenster, dann ist der Lieferant Pflicht und wird aus dem Adressstamm gewählt
  (Suche wie in ANF-004 AK-9). Ein Hinweis nennt, was übernommen wird.
- [ ] **AK-9:** Gegeben ein gültiges Formular, wenn der Benutzer speichert, dann wird ein Einkaufsauftrag im
  Status „Entwurf“ angelegt und die Detailansicht öffnet sich (ANF-004 AK-10). Er enthält:
  - Projekt: das Projekt des Vorgangs,
  - Bemerkung: die Beschreibung des Vorgangs,
  - Währung: CHF,
  - eine Position mit dem Namen des Vorgangs als Bezeichnung, Menge 1, Einheit „Pauschal“, Einzelpreis 0.00
    und dem ersten gültigen Vorsteuer-Code (wie beim Anlegen einer Position).
- [ ] **AK-10:** Gegeben ein Einkaufsauftrag aus einem Vorgang, dann merkt er sich den Vorgang. Der Abschnitt
  „Einkaufsaufträge“ in der Detailansicht des Vorgangs listet alle Einkaufsaufträge dieses Vorgangs mit
  Nummer, Lieferant, Status und Gesamtbetrag.
- [ ] **AK-11:** Gegeben ein Vorgang mit Einkaufsaufträgen, wenn der Vorgang gelöscht wird, dann bleiben die
  Einkaufsaufträge erhalten; ihr Verweis auf den Vorgang wird leer.
- [ ] **AK-12:** Gegeben ein Formular ohne Lieferanten, wenn der Benutzer speichert, dann wird nichts gespeichert
  und das Feld „Lieferant“ wird mit einer Meldung markiert.

**Allgemein**
- [ ] **AK-13:** Gegeben eine Kennung eines Vorgangs, die es nicht (mehr) gibt, wenn eine Anfrage oder ein
  Einkaufsauftrag erzeugt werden soll, dann erscheint die Meldung „Vorgang nicht gefunden.“ und nichts wird
  gespeichert.
- [ ] **AK-14:** Die Tabellen werden per sqlx-Migration angelegt.

## 4. Daten / Felder

Tabelle `anfrage` (neu):

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | Technischer Schlüssel, automatisch |
| Nummer | Text (20) | Ja | Fachliche Nummer, fortlaufend je Jahr, eindeutig (`A-JJJJ-NNNN`) |
| Vorgang (`vorgang_id`) | Verweis | Ja | Löschen des Vorgangs löscht die Anfrage mit (AK-6, Kaskade) |
| Titel | Text (200) | Ja | Kopie des Namens des Teilprojekts bei Erstellung |
| Beschreibung | Langtext | Nein | Kopie der Beschreibung des Vorgangs bei Erstellung |
| Erstellt am | Zeitstempel | Ja | automatisch |
| Geändert am | Zeitstempel | Ja | automatisch |

Tabelle `anfrage_nummer` (neu): Zähler je Jahr (wie `einkaufsauftrag_nummer`).

Tabelle `einkaufsauftrag` (ergänzt):

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Vorgang (`vorgang_id`) | Verweis | Nein | Herkunft des Einkaufsauftrags; beim Löschen des Vorgangs leer (AK-11) |

## 5. Abhängigkeiten

- Hängt ab von: ANF-003 (Vorgang, Teilprojekt), ANF-004 (Einkaufsauftrag, Position), ANF-012 (MWST-Code für die Position)
- Wird benötigt von: –

## 6. Nicht im Umfang

- Lieferant an der Anfrage, Versand an Lieferanten, Angebote und Angebotsvergleich
- Status der Anfrage (Offen, Versendet, Beantwortet …)
- Bearbeiten der Anfrage (Titel und Beschreibung sind Kopien)
- Druck bzw. PDF der Anfrage
- Mehrere Positionen aus den Aufgaben des Vorgangs (es entsteht nur eine Pauschal-Position)
- Anzeige des Vorgangs in der Detailansicht des Einkaufsauftrags
- Mehrere Währungen beim Erzeugen (CHF als Vorgabe, änderbar in der Detailansicht des Auftrags über ANF-004)

## 7. Offene Fragen

Für die Umsetzung wurden die Vorschläge in Klammern als vorläufige Annahmen übernommen.

- [ ] Ist die Anfrage eine interne Bedarfsanfrage oder eine Anfrage an einen Lieferanten? *(vorläufig: intern, ohne Lieferant)*
- [ ] Format der Anfragenummer? *(vorläufig: `A-JJJJ-NNNN`, fortlaufend je Jahr)*
- [ ] Soll die Anfrage Änderungen am Vorgang oder Teilprojekt mitführen? *(vorläufig: nein, Kopie zum Zeitpunkt der Erstellung)*
- [ ] Soll der Einkaufsauftrag aus dem Vorgang Positionen aus den Aufgaben übernehmen? *(vorläufig: nein, eine Pauschal-Position mit dem Namen des Vorgangs)*
- [ ] Soll ein Einkaufsauftrag später auch aus einer Anfrage statt direkt aus dem Vorgang entstehen? *(vorläufig: nein)*
- [ ] Ist die Währung CHF die richtige Vorgabe für Aufträge aus dem Vorgang? *(vorläufig: CHF)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261204100000_anfrage.sql` | Tabellen `anfrage` und `anfrage_nummer`; Spalte `einkaufsauftrag.vorgang_id` (`ON DELETE SET NULL`) |
| `src/backend/inquiries/` | Modul Anfragen: `model.rs` (Zeile der Liste), `repository.rs` (SQL), `server_fns.rs` (Liste, Anlegen, Löschen) |
| `src/backend/purchase_orders/server_fns.rs` + `repository.rs` + `model.rs` | Einkaufsauftrag aus Vorgang (AK-8 bis AK-10, AK-12, AK-13), Liste der Einkaufsaufträge eines Vorgangs |
| `src/components/inquiries/section.rs` | Abschnitt „Anfragen“ in der Vorgang-Detailansicht (AK-1, AK-4 bis AK-6) |
| `src/components/purchase_orders/form.rs` | Modal-Fenster „Einkaufsauftrag aus Vorgang“ (`?neu=einkauf`, AK-7, AK-8, AK-12) |
| `src/components/purchase_orders/detail.rs` | Abschnitt „Einkaufsaufträge“ in der Vorgang-Detailansicht (AK-10) |
| `src/components/projects/detail.rs` | Die Abschnitte im Vorgang einbinden |

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-28 | Hatem Ben Belgacem | Erstellt (aus der Beschreibung im Auftrag) |
| 2026-11-28 | Claude Code        | Umsetzung begonnen; offene Fragen mit vorläufigen Annahmen |
