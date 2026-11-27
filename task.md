# Aufgaben

Stand: 2026-11-27

## In Arbeit

- [ ] …

## Offen

- [ ] Browser-Prüfung Kreditoren (ANF-019): Szenarien TS-019-01 bis TS-019-18, besonders Live-Differenz und Netto/Brutto-Vorschau, „Eingangsrechnung erfassen“ im Einkauf
- [ ] Bestehende Mandanten: Systemkonto „Aufwand (Vorgabe Kreditoren)“ zuordnen (die Vorlage setzt es nur bei neuen Kontenplänen; bis dahin gilt Konto 4000)
- [ ] Browser-Prüfung Debitoren (ANF-018): Szenarien TS-018-01 bis TS-018-20, besonders Projekt-zuerst-Vorschlag des Kunden (AK-7), Bestätigungsfenster, Druckvorschau A4 und Mobiltelefon
- [ ] ANF-018 AK-14 Rest: Gutschrift mit Rechnung verrechnen (mit ANF-020)
- [ ] ANF-014 AK-4 Rest: Kurs im Beleg überschreiben (Debitoren buchen heute immer zum Kurs am Rechnungsdatum)
- [ ] Browser-Prüfung Journal und Auswertungen (ANF-017, ANF-021): Liste, Filter, Formular mit Zeilen und MWST-Vorschau, Entwurf, Buchen, Storno, Detail, Kontoauszug, Bilanz, Erfolgsrechnung, Projekte, Drucken und CSV. Dafür ist ein Mandant mit Anmeldung nötig; der Serverstart wendet die neuen Migrationen an
- [ ] ANF-021 AK-10: Laufzeit der Auswertungen mit 100 000 Buchungszeilen messen
- [ ] ANF-010 AK-4 und AK-5: Übersicht Finanzen mit Kennzahlen (folgt mit ANF-018 bis ANF-020; dort auch die Umsatzgrenze)
- [ ] ANF-017: Zeilentext im Formular
- [ ] Testszenarien für ANF-017 und ANF-021 in `testszenarien.md` ergänzen
- [ ] ANF-014 AK-4 und AK-5: Kurs am Belegdatum im Beleg vorschlagen und Löschsperre für Rechnungen und Buchungen (mit ANF-018 bis ANF-019)
- [ ] ANF-015 AK-6: Löschsperre für Bankkonten mit Zahlungen (mit ANF-020)
- [ ] Browser-Prüfung der Bankkonten, Zahlungsbedingungen und Währungen (Layout, Fenster, Speichern; noch nicht gemacht, der Serverstart gegen die Entwicklungsdatenbank ebenfalls nicht)
- [ ] ANF-028: offene Fragen klären (Vorgabe der Personendichte, Besucherzahl je Zone oder Projekt, Grenzen der Ampel, Durchfluss je Meter Wegbreite); später Stufe 4 (Zeitregler)
- [ ] ANF-027: offene Fragen klären (eigene Seite, Speichern, „Verpflegungsstand“, Bibliotheken aus dem Netz, Vorgabe-Standort) und im Browser mit Luftbild prüfen (SWISSIMAGE: Zoomstufe 20 bestätigen)
- [ ] ANF-026: vor Inbetriebnahme klären, wie die bestehende Datenbank zum ersten Mandanten wird
- [ ] ANF-025 Rest: Bankkonten als Verweis (mit ANF-015), Absender in Mahnungen und Auswertungen (AK-9)
- [ ] ANF-012 Rest: AK-6 und AK-10 für Rechnungen (mit ANF-018 bis ANF-019); AK-11 in der Finanz-Übersicht (mit ANF-010 AK-4)
- [ ] ANF-012 offene Entscheidung: bestehende Einkaufspositionen einem Code zuordnen (Vorschlag war ja, umgesetzt nein; siehe `testszenarien.md`, Punkt 6)
- [ ] Browser-Prüfung der Szenarien TS-012-01 bis TS-012-16 und TS-013-01 bis TS-013-11 (Layout ist per Screenshot geprüft)
- [ ] ANF-010 bis ANF-025 (Finanzen, Firma): restliche offene Fragen klären und Anforderungen freigeben (Reihenfolge siehe ANF-010)
- [ ] Anforderung QR-Rechnung erfassen (wird benötigt, Zeitpunkt offen)
- [ ] ANF-008: Railway einrichten – Projekt mit PostgreSQL (Region Europa), GitHub verbinden, `DATABASE_URL` und `SITZUNG_COOKIE_SECURE=1` setzen; danach AK-9, 10, 14, 16 prüfen
- [ ] Module des ERP festlegen (z. B. Lager, Einkauf, Verkauf, Buchhaltung)
- [ ] Erste Anforderungen mit der [Vorlage](anforderungen/_VORLAGE.md) erfassen
- [ ] Datenbank `uniqwork` anlegen und `.env` mit Zugangsdaten füllen
- [ ] ANF-005: offene Fragen klären (vorläufige Annahmen: Adresstyp Person, Löschen/Sperren, Passwortregeln, erster Benutzer, alle Ebenen verantwortlich)
- [ ] Anforderung für Rollen und Berechtigungen erfassen
- [ ] ANF-007: offene Fragen klären (vorläufige Annahmen: 12 h / 7 Tage, 5 Fehlversuche / 15 Min., Ersteinrichtung, HTTPS über `SITZUNG_COOKIE_SECURE`)
- [ ] ANF-006: offene Fragen klären (vorläufige Annahmen: Benutzer-Auswahl ohne Anmeldung, Ablage in `aufgabe`, keine Nummer, keine Fälligkeit)
- [ ] ANF-001: offene Fragen klären (Inhalt der Kopfzeile, Navigationseinträge, Farbschema)
- [ ] ANF-002: offene Fragen klären (Rollen, Archivieren, Nummernformat, Länderliste)
- [ ] ANF-003: offene Fragen klären (Status-Automatik, Nummernformat, Archivieren, Projektleiter, Verschieben)
- [ ] ANF-004: offene Fragen klären (vorläufige Annahmen: Nummernformat, Lieferantenrolle, Freigabe, Rundung, MWST-Sätze, Währungen)
- [ ] Namensregel klären: technischer Schlüssel `id` oder `kennung`; Rust-Bezeichner englisch oder deutsch
- [ ] Testszenarien im Browser durchführen (`testszenarien.md`), zuerst ANF-005 bis ANF-007
- [ ] Offene Punkte aus `testszenarien.md` klären: Grossbuchstaben im Benutzernamen, Hinweis ohne Benutzerauswahl (ANF-006 AK-2), Meldung beim Selbst-Sperren (ANF-005 AK-20), Rundung des Fortschritts (ANF-003 AK-5)

## Erledigt

- [x] ANF-019 Kreditoren freigegeben (kein Visum, IBAN und Belegablage später) und umgesetzt: Liste mit Filter „fällig bis“, Skonto- und Überfälligkeitsmarkierung, Erfassen mit Kontrollbetrag und Rundung bis 0.05, Netto/Brutto je Position, Übernahme aus dem Einkaufsauftrag mit „bestellt / verrechnet“, Buchen ins Journal, Storno, Löschsperren, Abschnitte in Projekt, Adresse und Einkauf. Migration `20261205100000_kreditorenrechnung`; Systemkonto „Aufwand“; Journal-Verweis auch für Eingangsrechnungen. Ablauf über den laufenden Server gegen eine Testdatenbank geprüft
- [x] ANF-018 Debitoren freigegeben (Nummern R-/G-JJJJ-NNNN, Teilrechnungen später) und umgesetzt: Liste mit Filter und Überfälligkeit, Entwurf mit Positionen, Summen je MWST-Code, Buchen ins Journal (auch Fremdwährung), Druckansicht A4, Storno, Gutschrift, Abschnitte in Projekt und Adresse, Löschsperren. Migration `20261204100000_debitorenrechnung`; Systemkonto „Ertrag“; Verweis Journal → Beleg (ANF-017 AK-12); Reste aus ANF-010, ANF-011, ANF-012, ANF-014, ANF-016, ANF-025 mit umgesetzt. fmt, Clippy (Server und Browser) sauber, `cargo test` grün (201 Tests); Ablauf über den laufenden Server gegen eine Testdatenbank geprüft
- [x] ANF-027 AK-26: Elemente beschriften (Bezeichnung je Element, höchstens 100 Zeichen, unter „Masse von …“; auf der Karte über dem Modell und in der Liste; mit dem Plan gespeichert). Migration `20261203110000_gelaendeplan_element_bezeichnung`. Im Browser geprüft: Beschriftung über Sanität und Allgemeinem Element, nach dem Speichern und Neuladen erhalten
- [x] ANF-027: Elemente über die Auswahlliste „Element“ platzieren (statt Schaltflächen); neue Elementarten Sanität (weisse Kabine mit rotem Kreuz) und Allgemeines Element (quadratisch mit Spitzdach); Migration `20261203100000_gelaendeplan_element_art`; AK-4 und AK-25 in ANF-027. `cargo fmt`, Clippy (Server und Browser) sauber, `cargo test` grün (180 Tests). Im Browser geprüft: Sanität (weisse Kabine, rotes Kreuz, dunkle Tür vorne links), Allgemeines Element (petrol Körper, anthrazit Spitzdach, helle Tür) und Festzelt (Giebel unverändert)
- [x] ANF-017 Buchungsjournal und ANF-021 Auswertungen umgesetzt: Journal mit Zeilen, Entwurf, Buchen, Storno, Journalnummer je Geschäftsjahr, Unveränderlichkeit und Ausgleich in der Datenbank (Migrationen `20261202100000`, `20261202110000`); Kontoauszug, Saldenliste, Bilanz, Erfolgsrechnung mit Vorjahr, Projektauswertung, Drucken und CSV. Modultests 21 neu, `cargo test` grün, fmt und Clippy (Server und Browser) sauber; SQL und Trigger gegen eine Testdatenbank geprüft
- [x] Reste: ANF-010 AK-1/2/6 bis 8 und AK-9 für Projekte, ANF-011 AK-6/7 (Vorgabe-MWST, Sperre nach Buchung), ANF-012 AK-6/10/11 (MWST-Seite mit Umsatz), ANF-013 AK-5 bis 7 (Buchungsprüfung, Löschsperre)
- [x] Block 1 umgesetzt: ANF-015 Bankkonten, ANF-016 Zahlungsbedingungen, ANF-014 Währungen. Dazu ANF-016 AK-5 (Vorgabe je Adresse) und ANF-014 AK-6 (Einkauf wählt aktive Währungen). Migrationen `20261201100000` bis `20261201120000`; 23 neue Modultests, fmt und Clippy (Server und Browser) sauber, `cargo test` grün
- [x] ANF-027 AK-23, AK-24: eigene Masse je Element (Breite, Tiefe, Höhe, Drehung) und Elementart Gitterabsperrung (Migration `gelaendeplan_element_masse`)
- [x] ANF-028 Stufe 3 Engstellen: nötige Breite je Weg, freie Breite zwischen Elementen, versperrte Wege, Liste „Engstellen“ und Warnzeichen auf der Karte, Planungswert „Durchfluss je Meter Wegbreite“ (Migration `gelaendeplan_durchfluss`)
- [x] ANF-028 Besucherströme erfasst und Stufen 1 und 2 umgesetzt: Besucherwege (Richtung, Personen pro Stunde, Breite; Linienbreite nach Menge, Pfeile), Zugänge (Eingang, Ausgang, Notausgang), Zonen mit Fassungsvermögen und Ampel; gespeichert in `gelaendeplan_besucherweg`, `gelaendeplan_zugang`, `gelaendeplan_zone`
- [x] ANF-027 Geländeplan eines Projekts erfasst und umgesetzt: 3D-Karte (MapLibre + Three.js) unter `/projekte/<Kennung>/gelaendeplan`, Bühnen/Verpflegungsstände/Festzelte platzieren, drehen, löschen, Rasenfläche abstecken (m², Lageprüfung mit `geo`), Standort im Modal-Fenster, Speichern in `gelaendeplan*`
- [x] ANF-027 AK-4: Elemente als gestaltete 3D-Modelle statt Quadern (Bühne, Verpflegungsstand, Festzelt)
- [x] ANF-027 AK-15 bis AK-17a: mehrere Messungen mit Bezeichnung, rote Messlinien, Messungen werden mit dem Plan gespeichert (`gelaendeplan_messung`, `gelaendeplan_messpunkt`)
- [x] ANF-027 AK-12a: Adresssuche über den Suchdienst von swisstopo statt Nominatim (Geländeplan nur in der Schweiz)
- [x] ANF-027 AK-2: Luftbild swisstopo SWISSIMAGE statt Esri World Imagery (Geländeplan nur in der Schweiz)
- [x] ANF-027 AK-5, AK-21, AK-22: Elemente am Drehgriff um die eigene Achse drehen (Taste R, ↺/↻ in 15°-Schritten), Rasenfläche lila, Eckpunkte ziehbar
- [x] ANF-027 AK-18 bis AK-20: Elemente auswählen und verschieben (Ziehen mit Maus oder Finger, Pfeiltasten, Rücksprung ausserhalb der Rasenfläche)
- [x] ANF-027 AK-12a/b, AK-15 bis AK-17: Adresssuche (Nominatim) mit hinterlegter Standort-Adresse, Messwerkzeuge Strecke und Fläche
- [x] ANF-026 Schritt 3b: Mandanten-Datenbanken ohne Zugangsdaten (Passwort, letzte Anmeldung, Sitzungen), Benutzerverwaltung ohne Passwort, „Passwort ändern“ im Portal – ANF-026 erledigt
- [x] ANF-026 Schritt 4: Modal-Fenster „Mandant definieren“ (`/portal?mandant=…`), Firma und Firmenadresse in der Mandanten-Datenbank, Firmenname in der Kopfzeile
- [x] ANF-026 Schritt 3: Datenbank erstellen (`/portal`), Rolle `uniqwork_erp` ohne `CREATEDB`, Pool je Mandant, Migrationen und Aufräumen beim Serverstart, erster Benutzer mit `portal_benutzer_id`
- [x] ANF-026 Schritt 2: Registrierung (`/registrieren`), Anmeldung mit E-Mail über die Portal-Datenbank, Portal-Übersicht (`/portal`), ERP nur mit Mandant; Ersteinrichtung entfernt
- [x] ANF-026 Schritt 1: Portal-Datenbank mit eigener Verbindung, `migrations_portal/` und Tabellen für Zugänge, Mandanten, Sitzungen
- [x] ANF-026 Registrierung, eigene Datenbank und Mandant erfasst, offene Fragen geklärt und freigegeben
- [x] Projekt umbenannt: ERP-saintwhoo → uniqwork (Paket, Programm, CSS/WASM-Dateien, Docker-Image, Seitentitel, Dokumentation)
- [x] Rust-Projekt angelegt (`cargo new`)
- [x] Projektstruktur für Anforderungen, Übersicht und Aufgaben erstellt
- [x] Tailwind CSS v4 und daisyUI v5 installiert und eingerichtet
- [x] Leptos-Gesamtanwendung (Server und Browser, Axum) mit PostgreSQL (sqlx, Migrationen) aufgesetzt
- [x] ANF-001 Grundgerüst mit Kopfzeile und Seitenleiste umgesetzt
- [x] Anforderungen ANF-002 (Adressen) und ANF-003 (Projekte) erfasst
- [x] Alle Begriffe in Dokumentation und Anforderungen auf Deutsch umgestellt
- [x] ANF-002 Adressen umgesetzt
- [x] ANF-003 Projekte mit Teilprojekten, Vorgängen und Aufgaben umgesetzt
- [x] Projektübersicht mit eingerückter Struktur; alle Eingaben in Modal-Fenstern
- [x] Aufgaben auf Titel reduziert (ANF-003 AK-11a)
- [x] Anforderung ANF-004 (Einkaufsaufträge) erfasst
- [x] ANF-004 Einkaufsaufträge umgesetzt
- [x] ANF-001 AK-12: Startseite zeigt alle Module als Karten
- [x] Ebenen der Projekte farblich abgegrenzt (ANF-003 AK-5a)
- [x] Kopfzeile mit Leitspruch in Tifinagh (ANF-001)
- [x] Kopfzeile neutral: Glutorange und Leitspruch entfernt (ANF-001)
- [x] Aufgaben mit Feld „Beschreibung“ (ANF-003 AK-11a)
- [x] Code nach `backend/<modul>/{model, server_fns, repository}` und `components/<modul>/` umgebaut, Systementwurf in `system-design.md`
- [x] Anforderung ANF-005 (Benutzerverwaltung, Benutzer – Adresse 1:1) erfasst
- [x] ANF-005 Benutzerverwaltung umgesetzt (Benutzer – Adresse 1:1, Verantwortung in Projekten)
- [x] Anforderung ANF-006 (Aufgaben eines Benutzers, Aufgaben ohne Projekt) erfasst
- [x] ANF-006 Aufgaben eines Benutzers und Aufgaben ohne Projekt umgesetzt
- [x] Anforderung ANF-007 (Anmeldeseite) erfasst
- [x] ANF-007 Anmeldeseite umgesetzt (Zugangsschutz, Sitzungen, Ersteinrichtung)
- [x] Testszenarien zu allen Akzeptanzkriterien erstellt (`testszenarien.md`, 170 Szenarien)
- [x] Anforderung ANF-008 (Docker-Image für Railway) erfasst, offene Fragen geklärt und freigegeben
- [x] ANF-008 umgesetzt: `Dockerfile` mit `cargo-chef`, `.dockerignore`, `railway.toml`, Server hört auf `PORT` (lokal mit Docker geprüft)
- [x] ANF-009 Farbpalette umgesetzt (Indigo, Teal, neutrale Töne, Signalfarben; Glutorange entfernt)
- [x] Anforderungen Finanzen erfasst: Rahmen ANF-010, Stammdaten in den Einstellungen ANF-011 bis ANF-016, Komponenten ANF-017 bis ANF-024
- [x] ANF-025 Firmenverwaltung umgesetzt (Firma – Adresse 1:1, UID-Prüfung, Logo, Belegvorschau)
- [x] ANF-011 Kontenplan und Systemkonten umgesetzt (Vorlage KMU-Kontenrahmen, Abschnitt „Finanzen“ in den Einstellungen)
- [x] Fragen zu Finanzen geklärt (führende Buchhaltung, MWST ab 100 000, QR offen); Firmenverwaltung ANF-025 erfasst
- [x] Oberfläche responsiv: Suchfelder der Listen auf dem Mobiltelefon über die volle Breite (ANF-001 AK-10); alle Seiten bei 390, 768 und 1280 px ohne seitliches Überlaufen geprüft
- [x] Modal-Fenster auf dem Mobiltelefon von unten einblenden (daisyUI `modal-bottom`, ab `sm` mittig, Einblend-Animation nur bei Bewegung erlaubt; ANF-001 AK-10)
- [x] Adressdetail: lange E-Mail- und Webadressen brechen auf dem Mobiltelefon um (zuvor 6 px Überlauf); alle Detailseiten mit echten Datensätzen bei 390 px geprüft
- [x] Speichern-Button in Modal-Fenstern auf dem iPhone sichtbar: Fensterhöhe mit `dvh`, Fußzeile mit den Schaltflächen bleibt beim Blättern am unteren Rand (ANF-001 AK-10)
- [x] ANF-012 MWST-Codes und MWST-Einstellungen umgesetzt (Codes, Pflicht mit Beginn und UID-Prüfung, Standard-Codes, Umsatzgrenze als Anzeige, Einkauf auf Codes umgestellt); Reste siehe „Offen“
- [x] ANF-013 Geschäftsjahre und Perioden umgesetzt (Anlegen mit Lücken- und Dauerprüfung, Monatsperioden, Sperren mit Vermerk, Löschen des neuesten Jahres); Reste siehe „Offen“
- [x] ANF-025 MWST-Nummer nur bei eingeschalteter MWST-Pflicht (AK-6)
- [x] Gemeinsame Datumsrechnung für die Finanzen (`src/backend/finance/date.rs`) mit Modultests
