# Aufgaben

Stand: 2026-11-18

## In Arbeit

- [ ] …

## Offen

- [ ] ANF-010 bis ANF-024 (Finanzen): offene Fragen klären und Anforderungen freigeben (Reihenfolge der Umsetzung siehe ANF-010)
- [ ] ANF-008: Railway einrichten – Projekt mit PostgreSQL (Region Europa), GitHub verbinden, `DATABASE_URL` und `SITZUNG_COOKIE_SECURE=1` setzen; danach AK-9, 10, 14, 16 prüfen
- [ ] Module des ERP festlegen (z. B. Lager, Einkauf, Verkauf, Buchhaltung)
- [ ] Erste Anforderungen mit der [Vorlage](anforderungen/_VORLAGE.md) erfassen
- [ ] Datenbank `erp_saintwhoo` anlegen und `.env` mit Zugangsdaten füllen
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
