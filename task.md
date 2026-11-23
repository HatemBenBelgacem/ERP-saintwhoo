# Aufgaben

Stand: 2026-11-24

## In Arbeit

- [ ] …

## Offen

- [ ] ANF-027: offene Fragen klären (eigene Seite, Speichern, „Verpflegungsstand“, Nutzung der Esri-Bilder, Bibliotheken aus dem Netz, Vorgabe-Standort) und im Browser mit Satellitenbild prüfen
- [ ] ANF-026: vor Inbetriebnahme klären, wie die bestehende Datenbank zum ersten Mandanten wird
- [ ] ANF-025 Rest: Bankkonten als Verweis (mit ANF-015), MWST-Nummer nur bei MWST-Pflicht (mit ANF-012), Absender in Druckansichten und Buchungssperre ohne Firma (mit ANF-018)
- [ ] ANF-011 Rest: Sperre von Nummer/Kontoart nach Buchungen (mit ANF-017), Vorgabe-MWST-Code (mit ANF-012)
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

- [x] ANF-027 Geländeplan eines Projekts erfasst und umgesetzt: 3D-Karte (MapLibre + Three.js) unter `/projekte/<Kennung>/gelaendeplan`, Bühnen/Verpflegungsstände/Festzelte platzieren, drehen, löschen, Rasenfläche abstecken (m², Lageprüfung mit `geo`), Standort im Modal-Fenster, Speichern in `gelaendeplan*`
- [x] ANF-027 AK-4: Elemente als gestaltete 3D-Modelle statt Quadern (Bühne, Verpflegungsstand, Festzelt)
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
