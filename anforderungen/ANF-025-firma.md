# ANF-025: Firmenverwaltung (eigene Firma)

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-025                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Funktional                                        |
| **Modul**     | Einstellungen                                     |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-18                                        |
| **Geändert**  | 2026-11-18                                        |

## 1. Beschreibung

Das ERP kennt die **eigene Firma** – das Unternehmen, das mit dem ERP arbeitet. Die Firma ist mit einer
**Adresse** aus dem Adressstamm (ANF-002) verknüpft (**Firma – Adresse 1:1**, wie Benutzer – Adresse in
ANF-005). Name, Postanschrift, Telefon, E-Mail und Webseite werden **nicht doppelt gespeichert**, sondern
aus dieser Adresse geholt. Die Firma ergänzt nur, was eine Adresse nicht hat: UID, Handelsregistereintrag,
Logo und Texte für Belege.

```
Firma ──1:1──► Adresse (Typ Firma)   → Firmenname, Strasse, PLZ, Ort, Land, Telefon, E-Mail, Webseite
  ├── UID / MWST-Nummer               → MWST (ANF-012), Rechnungen
  ├── Logo, Fusszeile für Belege      → Druckansichten
  └── Bankkonten (ANF-015)            → Zahlungsverbindung auf Rechnungen
```

Die Angaben der Firma sind der **Absender** auf allen Belegen: Ausgangsrechnungen und Gutschriften
(ANF-018), Mahnungen (ANF-024), MWST-Abrechnung (ANF-022), Auswertungen (ANF-021) und künftig
Bestellungen (ANF-004) und QR-Rechnung.

Die Firmenverwaltung liegt in den Einstellungen unter `/einstellungen/firma`.

## 2. Benutzergeschichte

> Als **Administrator** möchte ich **unsere Firma einmal mit ihrer Adresse und ihren rechtlichen Angaben
> erfassen**, damit **alle Belege den richtigen Absender tragen und Adressänderungen nur an einer Stelle
> nötig sind**.

## 3. Akzeptanzkriterien

**Anzeigen**
- [x] **AK-1:** Gegeben die Einstellungsnavigation, dann gibt es einen Eintrag **„Firma“** mit Pfad
  `/einstellungen/firma` (nur in `SETTINGS_NAV`).
- [ ] **AK-2:** Gegeben eine erfasste Firma, dann zeigt die Seite die Absenderangaben aus der verknüpften
  Adresse (Firmenname, Postanschrift, Telefon, E-Mail, Webseite) mit Verweis auf die Adresse, dazu UID,
  MWST-Nummer, Handelsregistereintrag, Logo, Fusszeile und die Bankkonten (ANF-015) als Verweis.
- [x] **AK-3:** Gegeben eine Belegvorschau, dann zeigt die Seite, wie der Absenderblock auf einer
  Rechnung aussieht (Logo, Name, Anschrift, UID).

**Erfassen und Bearbeiten**
- [x] **AK-4:** Gegeben noch keine Firma, dann zeigt die Seite „Firma erfassen“; das Modal-Fenster
  (`?neu=1`) lässt den Benutzer eine **bestehende Adresse vom Typ Firma wählen** (Suche nach Name oder
  Adressnummer) oder **direkt eine neue Adresse anlegen**, die danach im Adressstamm erscheint.
- [x] **AK-5:** Gegeben die Firma, wenn der Benutzer „Bearbeiten“ wählt (`?bearbeiten=1`), dann kann er
  UID, Handelsregistereintrag, Logo und Fusszeile ändern und eine andere Adresse verknüpfen.
  Die Felder der Adresse selbst werden in der Adressverwaltung (ANF-002) geändert; der Verweis
  „Adresse bearbeiten“ führt dorthin.
- [x] **AK-6:** Gegeben eine UID, dann wird sie im Format `CHE-123.456.789` geprüft (inkl. Prüfziffer
  nach Modulo 11). Die MWST-Nummer wird daraus gebildet (`CHE-123.456.789 MWST`) und angezeigt,
  sobald die MWST-Pflicht eingeschaltet ist (ANF-012).
- [x] **AK-7:** Gegeben ein Logo, dann kann der Benutzer eine PNG-, JPG- oder SVG-Datei bis 1 MB
  hochladen, ersetzen oder entfernen. Das Logo wird in der Datenbank gespeichert.
- [x] **AK-8:** Gegeben eine verknüpfte Adresse, die nicht vom Typ Firma ist oder keine vollständige
  Postanschrift (Strasse, PLZ, Ort, Land) hat, dann wird das Speichern mit Meldung verhindert.

**Verwendung**
- [ ] **AK-9:** Gegeben eine Druckansicht eines Belegs (Rechnung, Gutschrift, Mahnung, Auswertung), dann
  stammt der Absender immer aus der Firma und ihrer Adresse zum Zeitpunkt des Drucks.
- [ ] **AK-10:** Gegeben keine erfasste Firma, dann zeigen Belege im Druck einen Hinweis „Firma nicht
  erfasst“, und das Buchen von Debitorenrechnungen ist mit Verweis auf `/einstellungen/firma` gesperrt.
- [x] **AK-11:** Gegeben die Adresse der Firma, wenn der Benutzer sie löschen will, dann wird das Löschen
  verhindert und die Meldung nennt „als eigene Firma“ (wie ANF-002 AK-16). In der Adress-Detailansicht ist
  sie als „Eigene Firma“ gekennzeichnet.
- [x] **AK-12:** Gegeben eine Adressauswahl für Kunden oder Lieferanten, dann ist die eigene Firma darin
  nicht wählbar.

**Allgemein**
- [x] **AK-13:** Die Tabelle wird per sqlx-Migration angelegt; es gibt höchstens eine Firma
  (durch die Datenbank sichergestellt).

## 4. Daten / Felder

Tabelle `firma`:

| Feld | Typ | Pflicht | Bemerkung |
|------|-----|---------|-----------|
| Kennung (`id`) | Ganzzahl | Ja | automatisch; höchstens eine Zeile |
| Adresse (`adresse_id`) | Verweis | Ja | eindeutig, Typ Firma; Löschen der Adresse wird verhindert |
| UID | Text (15) | Nein | `CHE-123.456.789`, Pflicht sobald MWST-pflichtig (ANF-012) |
| Handelsregistereintrag | Text (100) | Nein | z. B. „Handelsregister des Kantons Zürich, CH-020.3.123.456-7“ |
| Logo | Binärdaten | Nein | max. 1 MB |
| Logo-Dateityp | Text (50) | Nein | `image/png`, `image/jpeg`, `image/svg+xml` |
| Fusszeile für Belege | Text (300) | Nein | z. B. Bankverbindung oder Geschäftsführung |
| Erstellt am / Geändert am | Zeitstempel | Ja | automatisch |

Aus der Adresse (ANF-002) übernommen und nicht gespeichert: Firmenname, Zusatz, Strasse, Postfach, PLZ,
Ort, Land, Telefon, E-Mail, Webseite.

## 5. Abhängigkeiten

- Hängt ab von: ANF-002 (Adressen), ANF-001 (Einstellungsnavigation)
- Wird benötigt von: ANF-012 (MWST-Nummer), ANF-015, ANF-018, ANF-021, ANF-022, ANF-024, spätere QR-Rechnung

## 6. Nicht im Umfang

- Mehrere Firmen bzw. Mandanten mit getrennter Buchhaltung (siehe offene Fragen)
- Filialen bzw. Standorte mit eigenen Absendern
- Gestaltung eigener Belegvorlagen (Schriften, Anordnung)

## 7. Offene Fragen

- [ ] Wird nur **eine** eigene Firma geführt oder sollen **mehrere Firmen** (Mandanten) verwaltet werden?
  *(Annahme: eine Firma; das Datenmodell mit eigener Tabelle `firma` lässt eine spätere Erweiterung zu)*
- [ ] Soll die Bankverbindung auf den Belegen aus den Bankkonten (ANF-015) oder aus der Fusszeile kommen? *(Vorschlag: aus dem Standard-Bankkonto)*

## Umsetzung

| Datei | Inhalt |
|-------|--------|
| `migrations/20261118150000_firma.sql` | Tabelle `firma` (höchstens eine Zeile, Adresse 1:1 ohne Löschen, UID-Format, Logo bis 1 MB mit Dateityp) |
| `src/backend/company/model.rs` | UID normalisieren und Prüfziffer (Modulo 11), MWST-Nummer, Prüfung der Firmenadresse, Erkennen des Logo-Typs am Inhalt, Absenderzeilen + Modultests |
| `src/backend/company/server_fns.rs` + `repository.rs` | Firma lesen, erfassen (bestehende oder neue Adresse in einer Transaktion), bearbeiten, Logo hochladen (`MultipartFormData`) und entfernen |
| `src/backend/company/logo.rs`, `src/main.rs` | Auslieferung des Logos unter `/firma/logo` (geschützt wie alle Seiten; SVG ohne Skriptausführung) |
| `src/components/settings/company/page.rs` | Seite `/einstellungen/firma`: Absenderangaben, rechtliche Angaben, Logo, Belegvorschau; Baustein `SenderBlock` für spätere Druckansichten |
| `src/components/settings/company/form.rs` | Modal-Fenster: Firma erfassen (`?neu=1`), bearbeiten (`?bearbeiten=1`), Logo (`?logo=1`) |
| `src/backend/addresses/…`, `src/components/addresses/detail.rs` | Kennzeichen „Eigene Firma“, Löschmeldung „als eigene Firma“, Adressauswahl für Kunden und Lieferanten ohne eigene Firma |
| `Cargo.toml` | `server_fn` mit Merkmal `multipart` für das Hochladen von Dateien |

Hinweise:
- **AK-2:** Die Bankkonten werden mit ANF-015 als Verweis ergänzt.
- **AK-6:** UID-Prüfung und MWST-Nummer umgesetzt. Die MWST-Nummer erscheint nur bei eingeschalteter MWST-Pflicht (ANF-012).
- **AK-9/AK-10:** Der Absenderblock (`SenderBlock`) liest die Firma zum Zeitpunkt der Anzeige und zeigt ohne Firma
  „Firma nicht erfasst“. Die Druckansichten und die Buchungssperre für Debitorenrechnungen folgen mit ANF-018 ff.
- **AK-7:** Der Dateityp wird am Inhalt erkannt, nicht an der Endung. SVG-Logos werden mit
  `Content-Security-Policy: sandbox` ausgeliefert, damit eingebettete Skripte nicht ausgeführt werden.
- **AK-8:** Geprüft beim Erfassen und Bearbeiten. Wird die Adresse später im Adressstamm unvollständig, zeigt die
  Firmenseite einen Warnhinweis mit Verweis auf die Adresse.
- **AK-12:** Die Adressauswahl für Kunden (Projekte) und Lieferanten (Einkauf) bietet die eigene Firma nicht mehr an.
- Annahme aus den offenen Fragen: genau eine Firma (durch die Datenbank sichergestellt).

Geprüft: AK-1, AK-3 bis AK-8, AK-11, AK-12 im Browser (grosser Bildschirm und Mobiltelefon,
`testszenarien.md` TS-025-01 bis -12), 5 neue Modultests.

## 8. Änderungshistorie

| Datum      | Autor              | Änderung        |
|------------|--------------------|-----------------|
| 2026-11-18 | Hatem Ben Belgacem | Erstellt (Antwort auf offene Frage zu ANF-018: Absender aus Firmenverwaltung) |
| 2026-11-18 | Hatem Ben Belgacem | Umgesetzt bis auf Bankkonten (ANF-015), MWST-Bedingung (ANF-012) und Belege (ANF-018 ff.) |
| 2026-11-19 | Claude Code | MWST-Nummer nur bei eingeschalteter MWST-Pflicht (AK-6, mit ANF-012) |
