# ANF-010: Finanzen – Rahmen und Aufteilung in Komponenten

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-010                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Finanzen                                          |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Das ERP erhält ein Modul **Finanzen** als **führende Buchhaltung** des Unternehmens – eine
**doppelte Buchhaltung nach Schweizer Praxis**
(KMU-Kontenrahmen, MWST, CHF mit Rappenrundung). Diese Anforderung ist der **Rahmen**: Sie legt fest,
aus welchen Komponenten das Modul besteht, wo diese in der Oberfläche liegen, wie sie zusammenhängen und
in welcher Reihenfolge sie umgesetzt werden. Die fachlichen Details stehen in je einer eigenen Anforderung.

### Komponenten

**Stammdaten** – werden unter **Einstellungen** gepflegt (`/einstellungen/finanzen/…`, die Firma unter
`/einstellungen/firma`):

| Nummer | Komponente | Pfad | Priorität |
|--------|------------|------|-----------|
| [ANF-025](ANF-025-firma.md) | Firmenverwaltung (eigene Firma, verknüpft mit Adresse, Absender aller Belege) | `/einstellungen/firma` | Muss |
| [ANF-011](ANF-011-kontenplan.md) | Kontenplan und Systemkonten | `/einstellungen/finanzen/kontenplan` | Muss |
| [ANF-012](ANF-012-mwst-codes.md) | MWST-Codes (Steuersätze) | `/einstellungen/finanzen/mwst` | Muss |
| [ANF-013](ANF-013-geschaeftsjahre.md) | Geschäftsjahre und Perioden | `/einstellungen/finanzen/geschaeftsjahre` | Muss |
| [ANF-014](ANF-014-waehrungen.md) | Währungen und Umrechnungskurse | `/einstellungen/finanzen/waehrungen` | Soll |
| [ANF-015](ANF-015-bankkonten.md) | Eigene Bankkonten | `/einstellungen/finanzen/bankkonten` | Muss |
| [ANF-016](ANF-016-zahlungsbedingungen.md) | Zahlungsbedingungen | `/einstellungen/finanzen/zahlungsbedingungen` | Muss |

**Bewegungsdaten und Auswertungen** – unter dem Hauptnavigationseintrag **„Finanzen“** (`/finanzen/…`):

| Nummer | Komponente | Pfad | Priorität |
|--------|------------|------|-----------|
| [ANF-017](ANF-017-buchungsjournal.md) | Buchungsjournal (Hauptbuch) | `/finanzen/journal` | Muss |
| [ANF-018](ANF-018-debitorenrechnungen.md) | Debitoren: Ausgangsrechnungen | `/finanzen/debitoren` | Muss |
| [ANF-019](ANF-019-kreditorenrechnungen.md) | Kreditoren: Eingangsrechnungen | `/finanzen/kreditoren` | Muss |
| [ANF-020](ANF-020-zahlungen.md) | Zahlungen und offene Posten | `/finanzen/zahlungen` | Muss |
| [ANF-021](ANF-021-finanzauswertungen.md) | Auswertungen (Kontoauszug, Saldenliste, Bilanz, Erfolgsrechnung) | `/finanzen/auswertungen` | Muss |
| [ANF-022](ANF-022-mwst-abrechnung.md) | MWST-Abrechnung | `/finanzen/mwst-abrechnung` | Soll |
| [ANF-023](ANF-023-jahresabschluss.md) | Jahresabschluss und Saldovortrag | `/finanzen/abschluss` | Soll |
| [ANF-024](ANF-024-mahnwesen.md) | Mahnwesen (inkl. Mahnstufen in den Einstellungen) | `/finanzen/mahnungen` | Kann |

### Zusammenhang

```
Stammdaten (Einstellungen)
  Kontenplan ─┬─ MWST-Codes ─┬─ Zahlungsbedingungen   Geschäftsjahre/Perioden   Währungen   Bankkonten
              │              │
Bewegungen    ▼              ▼
  Debitorenrechnung ──┐                    ┌── Kreditorenrechnung ◄── Einkaufsauftrag (ANF-004)
     (Kunde: Adresse, │                    │      (Lieferant: Adresse)
      Projekt)        ▼                    ▼
                  ┌──────── Buchungsjournal ────────┐ ◄── manuelle Buchungen
                  │   (jede Buchung: Soll = Haben)  │
  Zahlungen ──────┘                                 └──► Auswertungen, MWST-Abrechnung, Jahresabschluss
  (gleichen offene Posten aus)
```

Grundsätze für alle Komponenten:
- **Doppelte Buchhaltung:** Jeder finanzwirksame Vorgang erzeugt eine Buchung im Journal, deren Soll- und
  Habenbeträge gleich gross sind. Die Auswertungen lesen nur das Journal.
- **Unveränderlichkeit:** Gebuchte Belege werden nicht mehr geändert oder gelöscht, sondern **storniert**
  (Gegenbuchung). Löschen ist nur im Status „Entwurf“ möglich.
- **Gesperrte Perioden:** In gesperrte Perioden bzw. abgeschlossene Geschäftsjahre kann nicht gebucht werden.
- **Beträge:** Festkomma wie in ANF-004 (in der Datenbank `NUMERIC`, in Rust ganze Zahlen in Rappen),
  Buchhaltungswährung CHF; Belege in Fremdwährung werden zum Kurs in CHF umgerechnet (ANF-014).
- **Bezug zu bestehenden Modulen:** Kunden und Lieferanten sind Adressen (ANF-002), Rechnungen können einem
  Projekt (ANF-003) zugeordnet werden, Kreditorenrechnungen können aus einem Einkaufsauftrag (ANF-004)
  übernommen werden, jede Buchung merkt sich den erfassenden Benutzer (ANF-005).

## 2. Benutzergeschichte

> Als **Buchhalter** möchte ich **Rechnungen, Zahlungen und Buchungen an einer Stelle im ERP führen**,
> damit **die Finanzbuchhaltung ohne zusätzliches Programm vollständig und nachvollziehbar ist**.

> Als **Geschäftsleitung** möchte ich **jederzeit Bilanz, Erfolgsrechnung und offene Posten sehen**,
> damit **ich die finanzielle Lage des Unternehmens kenne**.

## 3. Akzeptanzkriterien

**Navigation**
- [x] **AK-1:** Gegeben die Hauptnavigation (ANF-001), dann gibt es einen Eintrag **„Finanzen“** mit Pfad
  `/finanzen`; er erscheint auch als Karte auf der Startseite.
- [x] **AK-2:** Gegeben eine Seite unter `/finanzen/…`, dann zeigt sie oben **Reiter** für Übersicht,
  Journal, Debitoren, Kreditoren, Zahlungen, Auswertungen sowie – sobald umgesetzt – MWST-Abrechnung,
  Abschluss und Mahnungen. Der aktive Reiter ist markiert; der Eintrag „Finanzen“ in der Seitenleiste
  bleibt für alle Unterseiten aktiv.
- [ ] **AK-3:** Gegeben die Einstellungen, dann zeigt die Einstellungsnavigation einen Abschnitt
  **„Finanzen“** mit den Stammdaten-Einträgen aus ANF-011 bis ANF-016 (und ANF-024 Mahnstufen).
  Die Einträge stehen nur in `SETTINGS_NAV` in `src/navigation.rs`.

**Übersicht**
- [ ] **AK-4:** Gegeben der Reiter „Übersicht“ (`/finanzen`), dann sieht der Benutzer Kennzahlen:
  offene Debitoren (Anzahl, Betrag, davon überfällig), offene Kreditoren (Anzahl, Betrag, davon
  überfällig), Saldo der eigenen Bankkonten und das Ergebnis des laufenden Geschäftsjahres.
  Jede Kennzahl ist ein Verweis auf die passende gefilterte Liste.
- [ ] **AK-5:** Gegeben fehlende Stammdaten (keine Firma, kein Kontenplan, kein offenes Geschäftsjahr,
  keine Systemkonten), dann zeigt die Übersicht einen Hinweis mit Verweis auf die betreffende Einstellungsseite,
  statt Buchungen zuzulassen.

**Grundsätze (gelten für alle Finanz-Komponenten)**
- [x] **AK-6:** Gegeben eine Buchung, deren Soll- und Habensumme nicht übereinstimmen, dann wird sie auf
  dem Server abgelehnt (Prüfung in `model.rs` und zusätzlich in der Datenbank bzw. Transaktion).
- [x] **AK-7:** Gegeben ein gebuchter Beleg (Rechnung, Zahlung, Buchung), dann ist er nicht mehr
  bearbeitbar oder löschbar; Korrekturen erfolgen über „Stornieren“, das eine Gegenbuchung erzeugt.
- [x] **AK-8:** Gegeben eine gesperrte Periode oder ein abgeschlossenes Geschäftsjahr, wenn ein Benutzer
  mit Datum darin buchen will, dann wird das Buchen mit einer Meldung verhindert.
- [x] **AK-9:** Gegeben eine Adresse oder ein Projekt mit Finanzbelegen, wenn der Benutzer sie löschen
  will, dann wird das Löschen verhindert und die Meldung nennt die Belege (wie ANF-002 AK-16).

## 4. Daten / Felder

Siehe die einzelnen Anforderungen. Neue Tabellen (Einzahl, deutsch): `konto`, `mwst_code`,
`geschaeftsjahr`, `periode`, `waehrung`, `waehrungskurs`, `bankkonto`, `zahlungsbedingung`, `buchung`,
`buchungszeile`, `debitorenrechnung`, `debitorenposition`, `kreditorenrechnung`, `kreditorenposition`,
`zahlung`, `zahlungsausgleich`, `mwst_abrechnung`, `mahnstufe`, `mahnung`.

Code-Aufteilung nach `system-design.md`: `src/backend/finance/<teil>/` bzw. `src/components/finance/<teil>/`
für die Bewegungsdaten, Stammdaten-Ansichten unter `src/components/settings/finance/`.

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst), ANF-002 (Adressen), ANF-003 (Projekte), ANF-004 (Einkauf),
  ANF-005 (Benutzer), ANF-007 (Anmeldung)
- Wird benötigt von: ANF-011 bis ANF-025

**Reihenfolge der Umsetzung:**
1. Stammdaten: ANF-025 Firma → ANF-011 Kontenplan → ANF-012 MWST-Codes → ANF-013 Geschäftsjahre → ANF-015 Bankkonten
   → ANF-016 Zahlungsbedingungen → ANF-014 Währungen
2. ANF-017 Buchungsjournal (Kern, auf dem alles aufbaut)
3. ANF-018 Debitoren und ANF-019 Kreditoren
4. ANF-020 Zahlungen und offene Posten
5. ANF-021 Auswertungen, danach die Übersicht (AK-4)
6. ANF-022 MWST-Abrechnung, ANF-023 Jahresabschluss, ANF-024 Mahnwesen

## 6. Nicht im Umfang

- Lohnbuchhaltung, Anlagenbuchhaltung, Kostenrechnung (Kostenstellen) und Budget
- Elektronischer Zahlungsverkehr (Export pain.001, Import camt.053/054) und E-Banking-Anbindung
- QR-Rechnung mit Zahlteil: wird benötigt, Umsetzung vorerst offen (eigene spätere Anforderung)
- Mehrere Mandanten (Firmen) in einer Datenbank
- Rollen und Berechtigungen (eigene, noch offene Anforderung); vorerst darf jeder angemeldete Benutzer alles
- Revisionssichere Archivierung nach GeBüV und Belegablage (eingescannte Belege)

## 7. Offene Fragen

- [x] Soll die Finanzbuchhaltung im ERP die **führende** Buchhaltung sein? **Geklärt 2026-11-18: Ja, führend.**
- [x] Ist die Firma MWST-pflichtig? **Geklärt 2026-11-18:** Pflicht ab CHF 100 000 Umsatz; die MWST-Funktion
  muss vollständig vorhanden und einschaltbar sein (ANF-012).
- [x] Braucht es die QR-Rechnung? **Geklärt 2026-11-18:** Ja, Zeitpunkt der Umsetzung vorerst offen.
- [x] Woher kommen die Absenderangaben? **Geklärt 2026-11-18:** Aus einer Firmenverwaltung, die mit einer
  Adresse verknüpft ist (ANF-025).
- [ ] Wer darf Finanzen sehen und buchen? *(Vorschlag: vorerst alle angemeldeten Benutzer, später Rolle „Buchhaltung“)*
- [ ] Gruppierte Einstellungsnavigation (Abschnitt „Finanzen“) oder einfache flache Einträge? *(Vorschlag: Abschnitt)*
- [ ] Braucht es eine Anbindung an E-Banking schon in der ersten Ausbaustufe? *(Vorschlag: nein)*

## Umsetzung

Umgesetzt mit ANF-017 und ANF-021:
- **AK-1, AK-2:** Hauptnavigation „Finanzen“ (`/finanzen`) mit den Reitern. Noch nicht gebaute Reiter (Übersicht, Debitoren, Kreditoren, Zahlungen) sind ausgegraut.
- **AK-6:** Unausgeglichene Buchungen lehnt der Server ab; die Datenbank prüft den Ausgleich am Ende der Transaktion.
- **AK-7:** Gebuchte Belege lassen sich nicht ändern oder löschen. Die Datenbank erzwingt das; Korrektur über Storno.
- **AK-8:** Buchungen in gesperrten Perioden und in abgeschlossenen Geschäftsjahren werden abgelehnt (`check_booking_date`).
- **AK-9:** Projekte mit Buchungen oder Ausgangsrechnungen und Adressen mit Ausgangsrechnungen lassen sich nicht löschen; die Meldung nennt die Art der Belege (ANF-018). Eingangsrechnungen sperren ebenso (ANF-019).
- **Offen:** AK-4 (Übersicht mit Kennzahlen) und AK-5 (Hinweis bei fehlenden Stammdaten) folgen mit ANF-018 bis ANF-021.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt        |
| 2026-11-18 | Hatem Ben Belgacem | Offene Fragen geklärt: führende Buchhaltung, MWST-Pflicht ab 100 000, QR-Rechnung offen, Firmenverwaltung ANF-025 ergänzt |
| 2026-11-26 | Claude Code | Umgesetzt: Navigation und Reiter, Grundsätze für Buchungen (Ausgleich, Unveränderlichkeit, gesperrte Perioden), Löschprüfung für Projekte. Übersicht offen. |
| 2026-11-27 | Claude Code | Reiter „Debitoren“ aktiv; AK-9 für Adressen mit Ausgangsrechnungen (ANF-018) |
| 2026-11-27 | Claude Code | Reiter „Kreditoren“ aktiv; Löschsperre für Eingangsrechnungen (ANF-019) |
