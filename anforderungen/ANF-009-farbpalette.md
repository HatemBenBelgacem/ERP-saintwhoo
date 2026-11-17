# ANF-009: Farbpalette (Primär-, Sekundär-, Neutral- und Signalfarben)

| Feld          | Wert                                              |
|---------------|---------------------------------------------------|
| **Nummer**    | ANF-009                                           |
| **Status**    | In Umsetzung                                      |
| **Priorität** | Muss                                              |
| **Typ**       | Nicht-funktional                                  |
| **Modul**     | Allgemein / Oberfläche                            |
| **Autor**     | Hatem Ben Belgacem                                |
| **Erstellt**  | 2026-11-17                                        |
| **Geändert**  | 2026-11-17                                        |

## 1. Beschreibung

Das ERP erhält eine einheitliche, ruhige Farbpalette. Sie **ersetzt Glutorange** als Grundfarbe (bisher
ANF-001) vollständig. Die Palette besteht aus vier Farbtypen:

| Farbtyp | Zweck / Anwendung | Farbton | daisyUI |
|---------|-------------------|---------|---------|
| **Primärfarbe** (Marke) | Hauptschaltflächen, Hauptnavigation, aktive Zustände, Fokus | **Indigo** `#4338CA` | `primary` |
| **Sekundärfarbe** | Akzente, sekundäre Filter, dezente Hervorhebungen | **Teal** `#0F766E` | `secondary`, `accent` |
| **Neutrale Töne** | Hintergründe, Oberflächen, Rahmen, Text | Reinweiss bis Anthrazit | `base-100` … `base-300`, `base-content`, `neutral` |
| **Signalfarben** | Statusmeldungen und Rückmeldungen des Systems | Rot, Gelb, Grün, Blau | `error`, `warning`, `success`, `info` |

Neutrale Töne im Einzelnen:

| Verwendung | Farbe | daisyUI |
|------------|-------|---------|
| Karten, Modal-Fenster, Eingabefelder | Reinweiss `#FFFFFF` | `base-100` |
| Seitenhintergrund | Hellgrau `#F8FAFC` | `base-200` |
| Rahmen, Trennlinien | Grau `#E2E8F0` | `base-300` |
| Text | Dunkelgrau `#1E293B` | `base-content` |
| Kopfzeile | Anthrazit `#1F2937` | `neutral` |

Signalfarben im Einzelnen:

| Bedeutung | Farbe | daisyUI |
|-----------|-------|---------|
| Fehler | Rot `#DC2626` | `error` |
| Warnung | Gelb `#F59E0B` (mit dunkler Schrift) | `warning` |
| Erfolg | Grün `#15803D` | `success` |
| Information | Blau `#2563EB` | `info` |

## 2. Benutzergeschichte

> Als **Benutzer** möchte ich **eine ruhige, einheitliche Oberfläche mit klaren Farbbedeutungen**, damit
> **ich wichtige Aktionen und Meldungen sofort erkenne und lange konzentriert arbeiten kann**.

## 3. Akzeptanzkriterien

**Palette**
- [ ] **AK-1:** Alle Farbwerte stehen nur in `styles/app.css` (`--primaerfarbe`, `--sekundaerfarbe` und im
  daisyUI-Farbschema). Im Rust-Code stehen keine Farbwerte, nur daisyUI- und Tailwind-Klassen.
- [ ] **AK-2:** Glutorange kommt im ERP nicht mehr vor (keine Variable, keine Klasse, keine Ebene).

**Primärfarbe Indigo**
- [ ] **AK-3:** Hauptschaltflächen (`btn-primary`), Verweise (`link-primary`), Kontrollkästchen, Fortschritts-
  balken und Fokus-Ränder sind Indigo mit weisser Schrift.
- [ ] **AK-4:** Der aktive Navigationseintrag ist in **leichtem Indigo** markiert: 15 % Primärfarbe als
  Hintergrund, Schrift in dunklem Indigo (wie bisher bei Glutorange, ANF-001 AK-4).

**Sekundärfarbe Teal**
- [ ] **AK-5:** Die aktive Sortierspalte in den Listen (Adressen, Projekte, Einkauf, Benutzer) ist in der
  Sekundärfarbe hervorgehoben.
- [ ] **AK-6:** Die Plakette „ohne Projekt“ bei Aufgaben (ANF-006) ist in der Sekundärfarbe.

**Neutrale Töne und Signalfarben**
- [ ] **AK-7:** Seitenhintergrund, Karten, Rahmen, Text und Kopfzeile verwenden die neutralen Töne aus
  Abschnitt 1. Die Kopfzeile bleibt dunkles Anthrazit (`bg-neutral`).
- [ ] **AK-8:** Meldungen (`alert-…`), Plaketten und Feldfehler verwenden die Signalfarben aus Abschnitt 1.
- [ ] **AK-9:** Alle Schriften auf farbigen Flächen erreichen einen Kontrast von mindestens 4.5 : 1 (WCAG AA).

**Ebenen der Projekte (ANF-003 AK-5a)**
- [ ] **AK-10:** Die Ebenen bleiben farblich unterscheidbar: Projekt = **Indigo** (Primärfarbe),
  Teilprojekt = **Himmelblau** `#0284C7`, Vorgang = **Petrol** `#0D9488`, Aufgabe = **Schiefergrau** `#64748B`.

**Dokumentation**
- [ ] **AK-11:** `CLAUDE.md` (Gestaltung), ANF-001, ANF-003 und `testszenarien.md` beschreiben die neue Palette.

## 4. Daten / Felder (optional)

Keine.

## 5. Abhängigkeiten

- Hängt ab von: ANF-001 (Grundgerüst, Farbschema), ANF-003 (Ebenen-Farben)
- Ersetzt: Glutorange als Grundfarbe aus ANF-001
- Wird benötigt von: –

## 6. Nicht im Umfang

- Dunkles Farbschema (Dark Mode)
- Umschalten des Farbschemas durch den Benutzer
- Logo oder Seitensymbol in neuen Farben
- Farben der Status-Plaketten von Projekten und Einkaufsaufträgen (bleiben in der Primärfarbe)

## 7. Offene Fragen

- [x] Welcher Ton als Primärfarbe? → **Indigo** `#4338CA`.
- [x] Welcher Ton als Sekundärfarbe? → **Teal** `#0F766E`.
- [x] Was passiert mit Glutorange? → **Ganz entfernen**, auch bei der Ebene „Projekt“.

## 8. Änderungshistorie

| Datum      | Autor | Änderung        |
|------------|-------|-----------------|
| 2026-11-17 | Hatem Ben Belgacem | Erstellt; Primärfarbe Indigo, Sekundärfarbe Teal, Glutorange entfernt |
