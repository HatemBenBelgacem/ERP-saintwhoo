# uniqwork – Übersicht

Übersicht über alle Dokumente und Anforderungen des Projekts.

## Dokumente

| Datei | Inhalt |
|-------|--------|
| [CLAUDE.md](CLAUDE.md) | Arbeitsregeln, Begriffe und Projektrahmen für Claude Code |
| [system-design.md](system-design.md) | Systementwurf: Schichten, Ordnerstruktur, Abhängigkeitsregeln |
| [task.md](task.md) | Aufgabenliste und Fortschritt |
| [testszenarien.md](testszenarien.md) | Testszenarien zu den Akzeptanzkriterien aller Anforderungen |
| [.env.example](.env.example) | Vorlage für die Datenbank-Verbindung |
| [anforderungen/_VORLAGE.md](anforderungen/_VORLAGE.md) | Vorlage für neue Anforderungen |

## Anforderungen

| Nummer | Titel | Modul | Priorität | Status |
|--------|-------|-------|-----------|--------|
| [ANF-001](anforderungen/ANF-001-grundgeruest.md) | Grundgerüst mit Kopfzeile und Seitenleiste | Allgemein / Oberfläche | Muss | Erledigt |
| [ANF-002](anforderungen/ANF-002-adressen.md) | Adressen verwalten | Adressen | Muss | Erledigt |
| [ANF-003](anforderungen/ANF-003-projekte.md) | Projekte verwalten mit Teilprojekten, Vorgängen und Aufgaben | Projekte | Muss | Erledigt |
| [ANF-004](anforderungen/ANF-004-einkaufsauftrag.md) | Einkaufsaufträge verwalten | Einkauf | Muss | Erledigt |
| [ANF-005](anforderungen/ANF-005-benutzerverwaltung.md) | Benutzer verwalten (Benutzer – Adresse 1:1, Verantwortung in Projekten) | Benutzer | Muss | Erledigt |
| [ANF-006](anforderungen/ANF-006-meine-aufgaben.md) | Aufgaben eines Benutzers und Aufgaben ohne Projekt | Aufgaben | Muss | Erledigt |
| [ANF-007](anforderungen/ANF-007-anmeldung.md) | Anmeldeseite | Benutzer | Muss | Erledigt |
| [ANF-008](anforderungen/ANF-008-docker-railway.md) | Auslieferung als Docker-Image für Railway | Betrieb | Muss | In Umsetzung |
| [ANF-009](anforderungen/ANF-009-farbpalette.md) | Farbpalette (Primär-, Sekundär-, Neutral- und Signalfarben) | Allgemein / Oberfläche | Muss | Erledigt |
| [ANF-010](anforderungen/ANF-010-finanzen.md) | Finanzen – Rahmen und Aufteilung in Komponenten | Finanzen | Muss | In Umsetzung |
| [ANF-011](anforderungen/ANF-011-kontenplan.md) | Kontenplan und Systemkonten | Finanzen / Einstellungen | Muss | Erledigt |
| [ANF-012](anforderungen/ANF-012-mwst-codes.md) | MWST-Codes (Steuersätze) | Finanzen / Einstellungen | Muss | In Umsetzung |
| [ANF-013](anforderungen/ANF-013-geschaeftsjahre.md) | Geschäftsjahre und Perioden | Finanzen / Einstellungen | Muss | Erledigt |
| [ANF-014](anforderungen/ANF-014-waehrungen.md) | Währungen und Umrechnungskurse | Finanzen / Einstellungen | Soll | In Umsetzung |
| [ANF-015](anforderungen/ANF-015-bankkonten.md) | Eigene Bankkonten | Finanzen / Einstellungen | Muss | In Umsetzung |
| [ANF-016](anforderungen/ANF-016-zahlungsbedingungen.md) | Zahlungsbedingungen | Finanzen / Einstellungen | Muss | In Umsetzung |
| [ANF-017](anforderungen/ANF-017-buchungsjournal.md) | Buchungsjournal (Hauptbuch) | Finanzen | Muss | In Umsetzung |
| [ANF-018](anforderungen/ANF-018-debitorenrechnungen.md) | Debitoren – Ausgangsrechnungen | Finanzen | Muss | Entwurf |
| [ANF-019](anforderungen/ANF-019-kreditorenrechnungen.md) | Kreditoren – Eingangsrechnungen | Finanzen | Muss | Entwurf |
| [ANF-020](anforderungen/ANF-020-zahlungen.md) | Zahlungen und offene Posten | Finanzen | Muss | Entwurf |
| [ANF-021](anforderungen/ANF-021-finanzauswertungen.md) | Auswertungen – Kontoauszug, Saldenliste, Bilanz, Erfolgsrechnung | Finanzen | Muss | In Umsetzung |
| [ANF-022](anforderungen/ANF-022-mwst-abrechnung.md) | MWST-Abrechnung | Finanzen | Soll | Entwurf |
| [ANF-023](anforderungen/ANF-023-jahresabschluss.md) | Jahresabschluss und Saldovortrag | Finanzen | Soll | Entwurf |
| [ANF-024](anforderungen/ANF-024-mahnwesen.md) | Mahnwesen | Finanzen / Einstellungen | Kann | Entwurf |
| [ANF-025](anforderungen/ANF-025-firma.md) | Firmenverwaltung (eigene Firma, verknüpft mit Adresse) | Einstellungen | Muss | In Umsetzung |
| [ANF-026](anforderungen/ANF-026-registrierung-mandanten.md) | Registrierung, eigene Datenbank und Mandant | Portal / Mandanten | Muss | Erledigt |
| [ANF-027](anforderungen/ANF-027-gelaendeplan.md) | Geländeplan eines Projekts (3D-Karte) | Projekte | Soll | In Umsetzung |
| [ANF-028](anforderungen/ANF-028-besucherstroeme.md) | Besucherströme im Geländeplan | Projekte | Soll | In Umsetzung |

<!-- Neue Zeile pro Anforderung, z. B.:
| [ANF-029](anforderungen/ANF-029-artikelstamm.md) | Artikelstamm verwalten | Lager | Muss | Entwurf |
-->
