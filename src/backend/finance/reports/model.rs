//! Auswertungen (ANF-021): Modell, Rechenregeln und CSV. Alle Auswertungen lesen die Buchungen ohne
//! Entwürfe (AK-9): Stornos und Gegenbuchungen heben sich in den Salden auf. Beträge in Rappen;
//! Vorzeichen: Saldo = Soll − Haben, in Bilanz und Erfolgsrechnung positiv = normal (Ertrag, Aktiven, Passiven).

use serde::{Deserialize, Serialize};

use crate::backend::finance::accounts::model::{class_name, class_of};
use crate::backend::finance::date;
use crate::backend::finance::fiscal_year::model::german_date;
use crate::backend::purchase_orders::model::format_fixed;

pub const ARTEN: &[(&str, &str)] = &[
    ("kontoauszug", "Kontoauszug"),
    ("saldenliste", "Saldenliste"),
    ("bilanz", "Bilanz"),
    ("erfolgsrechnung", "Erfolgsrechnung"),
    ("projekte", "Projektauswertung"),
];

pub fn art_label(key: &str) -> &'static str {
    ARTEN
        .iter()
        .find(|(k, _)| *k == key)
        .map_or("Auswertung", |(_, l)| l)
}

/// Auswahl der Auswertung (AK-1). Die Webadresse trägt alle Angaben.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReportQuery {
    pub art: String,
    pub geschaeftsjahr_id: i64,
    pub von: String,
    pub bis: String,
    /// Konto für den Kontoauszug (0 = keines gewählt)
    pub konto_id: i64,
    /// Projekt für die Projektauswertung (0 = alle)
    pub projekt_id: i64,
    /// Vorjahresvergleich (AK-6); leer = aus, sonst an (Kontrollkästchen sendet „1“)
    pub vorjahr: String,
}

impl ReportQuery {
    pub fn vergleich(&self) -> bool {
        !self.vorjahr.is_empty()
    }
}

/// Geschäftsjahr und Zeitraum, für den die Auswertung gilt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Period {
    pub geschaeftsjahr_id: i64,
    pub geschaeftsjahr: String,
    pub beginn: String,
    pub ende: String,
    pub von: String,
    pub bis: String,
    /// Zeitraum des Vorjahres bei Vergleich (AK-6)
    pub vorjahr: Option<(String, String)>,
}

/// Kontosalden für einen Zeitraum (Summen in Rappen). Entwürfe sind nicht enthalten.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct AccountBalance {
    pub konto_id: i64,
    pub nummer: String,
    pub bezeichnung: String,
    pub kontoart: String,
    pub kontengruppe: Option<String>,
    pub soll: i64,
    pub haben: i64,
}

impl AccountBalance {
    pub fn saldo(&self) -> i64 {
        self.soll - self.haben
    }

    /// Erfolgskonto (Ertrag oder Aufwand) – erscheint in der Erfolgsrechnung.
    pub fn is_erfolg(&self) -> bool {
        self.kontoart == "ertrag" || self.kontoart == "aufwand"
    }
}

// ---------------------------------------------------------------------------------------------
// Kontoauszug
// ---------------------------------------------------------------------------------------------

/// Zeile aus der Abfrage: Buchungszeile eines Kontos.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct StatementLine {
    pub buchung_id: i64,
    pub datum: String,
    pub journalnummer: Option<i32>,
    pub belegnummer: String,
    pub text: String,
    pub soll: i64,
    pub haben: i64,
    pub ist_mwst_zeile: bool,
}

/// Gegenkonto-Information einer Zeile einer anderen Zeile derselben Buchung.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct CounterLine {
    pub buchung_id: i64,
    pub konto_id: i64,
    pub nummer: String,
    pub ist_mwst_zeile: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StatementRow {
    pub buchung_id: i64,
    pub datum: String,
    pub journalnummer: Option<i32>,
    pub belegnummer: String,
    pub text: String,
    pub gegenkonto: String,
    pub soll: i64,
    pub haben: i64,
    /// Laufender Saldo nach dieser Zeile (Soll − Haben)
    pub saldo: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Statement {
    pub konto_id: i64,
    pub konto_nummer: String,
    pub konto_bezeichnung: String,
    pub anfangssaldo: i64,
    pub zeilen: Vec<StatementRow>,
    pub soll: i64,
    pub haben: i64,
    pub endsaldo: i64,
}

/// Gegenkonto einer Zeile: MWST-Zeilen tragen den Hinweis „MWST“; sonst das eine andere Konto der
/// Buchung, „Diverse“ bei mehreren und „–“ ohne Gegenkonto (ANF-021 AK-2, ANF-017 AK-1).
pub fn gegenkonto(line: &StatementLine, counter: &[CounterLine], konto_id: i64) -> String {
    if line.ist_mwst_zeile {
        return "MWST".into();
    }
    let mut andere: Vec<&CounterLine> = Vec::new();
    for c in counter
        .iter()
        .filter(|c| c.buchung_id == line.buchung_id && !c.ist_mwst_zeile && c.konto_id != konto_id)
    {
        if !andere.iter().any(|a| a.konto_id == c.konto_id) {
            andere.push(c);
        }
    }
    match andere.as_slice() {
        [] => "–".into(),
        [one] => one.nummer.clone(),
        _ => "Diverse".into(),
    }
}

/// Kontoauszug: Anfangssaldo, Zeilen mit laufendem Saldo, Summen und Endsaldo (AK-2).
pub fn statement(
    konto: &AccountBalance,
    anfangssaldo: i64,
    lines: &[StatementLine],
    counter: &[CounterLine],
) -> Statement {
    let mut saldo = anfangssaldo;
    let mut zeilen = Vec::new();
    let (mut soll_sum, mut haben_sum) = (0, 0);
    for line in lines {
        soll_sum += line.soll;
        haben_sum += line.haben;
        saldo += line.soll - line.haben;
        zeilen.push(StatementRow {
            buchung_id: line.buchung_id,
            datum: line.datum.clone(),
            journalnummer: line.journalnummer,
            belegnummer: line.belegnummer.clone(),
            text: line.text.clone(),
            gegenkonto: gegenkonto(line, counter, konto.konto_id),
            soll: line.soll,
            haben: line.haben,
            saldo,
        });
    }
    Statement {
        konto_id: konto.konto_id,
        konto_nummer: konto.nummer.clone(),
        konto_bezeichnung: konto.bezeichnung.clone(),
        anfangssaldo,
        zeilen,
        soll: soll_sum,
        haben: haben_sum,
        endsaldo: saldo,
    }
}

// ---------------------------------------------------------------------------------------------
// Saldenliste
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrialRow {
    pub konto_id: i64,
    pub nummer: String,
    pub bezeichnung: String,
    pub anfangssaldo: i64,
    pub soll: i64,
    pub haben: i64,
    pub endsaldo: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrialClass {
    pub titel: String,
    pub zeilen: Vec<TrialRow>,
    pub anfangssaldo: i64,
    pub soll: i64,
    pub haben: i64,
    pub endsaldo: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TrialBalance {
    pub klassen: Vec<TrialClass>,
    pub anfangssaldo: i64,
    pub soll: i64,
    pub haben: i64,
    pub endsaldo: i64,
    /// Summe Soll = Summe Haben im Zeitraum (AK-3)
    pub ausgeglichen: bool,
}

/// Saldenliste (AK-3): Konten mit Bewegung oder Saldo ≠ 0, nach Kontenklasse mit Zwischensummen.
/// `anfang` enthält alle Konten bis zum Tag vor dem Zeitraum, `bewegung` alle Konten im Zeitraum.
pub fn trial_balance(anfang: &[AccountBalance], bewegung: &[AccountBalance]) -> TrialBalance {
    let mut klassen: Vec<TrialClass> = Vec::new();
    let mut gesamt = (0, 0, 0);
    for konto in bewegung {
        let anfangssaldo = anfang
            .iter()
            .find(|a| a.konto_id == konto.konto_id)
            .map_or(0, AccountBalance::saldo);
        let endsaldo = anfangssaldo + konto.saldo();
        if konto.soll == 0 && konto.haben == 0 && endsaldo == 0 {
            continue;
        }
        let row = TrialRow {
            konto_id: konto.konto_id,
            nummer: konto.nummer.clone(),
            bezeichnung: konto.bezeichnung.clone(),
            anfangssaldo,
            soll: konto.soll,
            haben: konto.haben,
            endsaldo,
        };
        let titel = class_of(&konto.nummer).map_or("Ohne Kontenklasse".to_string(), |c| {
            format!("Klasse {c} · {}", class_name(c))
        });
        let index = match klassen.iter().position(|k| k.titel == titel) {
            Some(i) => i,
            None => {
                klassen.push(TrialClass {
                    titel,
                    zeilen: Vec::new(),
                    anfangssaldo: 0,
                    soll: 0,
                    haben: 0,
                    endsaldo: 0,
                });
                klassen.len() - 1
            }
        };
        let k = &mut klassen[index];
        k.anfangssaldo += anfangssaldo;
        k.soll += konto.soll;
        k.haben += konto.haben;
        k.endsaldo += endsaldo;
        k.zeilen.push(row);
        gesamt = (
            gesamt.0 + anfangssaldo,
            gesamt.1 + konto.soll,
            gesamt.2 + konto.haben,
        );
    }
    let endsaldo_gesamt: i64 = klassen.iter().map(|k| k.endsaldo).sum();
    TrialBalance {
        klassen,
        anfangssaldo: gesamt.0,
        soll: gesamt.1,
        haben: gesamt.2,
        endsaldo: endsaldo_gesamt,
        ausgeglichen: gesamt.1 == gesamt.2,
    }
}

// ---------------------------------------------------------------------------------------------
// Bilanz und Erfolgsrechnung
// ---------------------------------------------------------------------------------------------

/// Zeile einer Bilanz- oder Erfolgsrechnung. `vorjahr` nur bei Vergleich (AK-6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    pub konto_id: Option<i64>,
    pub nummer: String,
    pub bezeichnung: String,
    pub betrag: i64,
    pub vorjahr: Option<i64>,
}

/// Gruppe von Zeilen mit Summe (Kontengruppe, Kontenklasse, Ergebnis).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub titel: String,
    pub zeilen: Vec<Row>,
    pub summe: i64,
    pub vorjahr: Option<i64>,
}

/// Ergebnis-Zeile für Zwischensummen der Erfolgsrechnung.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Zwischensumme {
    pub titel: String,
    pub betrag: i64,
    pub vorjahr: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub stichtag: String,
    pub aktiven: Vec<Group>,
    pub passiven: Vec<Group>,
    pub summe_aktiven: i64,
    pub summe_passiven: i64,
    pub vorjahr_aktiven: Option<i64>,
    pub vorjahr_passiven: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IncomeStatement {
    pub gruppen: Vec<Group>,
    pub zwischen: Vec<Zwischensumme>,
}

/// Erfolgsrechnung eines Zeitraums (AK-5): Ertrag positiv, Aufwand negativ. Der Vergleich
/// (`vorjahr`) rechnet mit denselben Regeln über den Vorjahreszeitraum.
pub fn income_statement(erfolg: &[AccountBalance]) -> IncomeStatement {
    let mut gruppen: Vec<Group> = Vec::new();
    for class in 3..=8u8 {
        let mut zeilen: Vec<Row> = erfolg
            .iter()
            .filter(|k| k.is_erfolg() && class_of(&k.nummer) == Some(class))
            .filter(|k| k.soll != 0 || k.haben != 0)
            .map(|k| Row {
                konto_id: Some(k.konto_id),
                nummer: k.nummer.clone(),
                bezeichnung: k.bezeichnung.clone(),
                betrag: -k.saldo(),
                vorjahr: None,
            })
            .collect();
        zeilen.sort_by(|a, b| a.nummer.cmp(&b.nummer));
        let summe = zeilen.iter().map(|r| r.betrag).sum();
        gruppen.push(Group {
            titel: format!("Klasse {class} · {}", class_name(class)),
            zeilen,
            summe,
            vorjahr: None,
        });
    }
    let summe_klasse = |class: u8| -> i64 {
        erfolg
            .iter()
            .filter(|k| k.is_erfolg() && class_of(&k.nummer) == Some(class))
            .map(|k| -k.saldo())
            .sum()
    };
    let abschreibungen: i64 = erfolg
        .iter()
        .filter(|k| k.is_erfolg() && k.nummer.starts_with("68"))
        .map(|k| -k.saldo())
        .sum();
    let bruttogewinn1 = summe_klasse(3) + summe_klasse(4);
    let bruttogewinn2 = bruttogewinn1 + summe_klasse(5);
    let vor_abschreibungen = bruttogewinn2 + summe_klasse(6) - abschreibungen;
    let betriebsergebnis = vor_abschreibungen + abschreibungen;
    let jahresergebnis = betriebsergebnis + summe_klasse(7) + summe_klasse(8);
    let zwischen = vec![
        Zwischensumme {
            titel: "Bruttogewinn 1 (nach Materialaufwand)".into(),
            betrag: bruttogewinn1,
            vorjahr: None,
        },
        Zwischensumme {
            titel: "Bruttogewinn 2 (nach Personalaufwand)".into(),
            betrag: bruttogewinn2,
            vorjahr: None,
        },
        Zwischensumme {
            titel: "Betriebsergebnis vor Abschreibungen".into(),
            betrag: vor_abschreibungen,
            vorjahr: None,
        },
        Zwischensumme {
            titel: "Betriebsergebnis nach Abschreibungen".into(),
            betrag: betriebsergebnis,
            vorjahr: None,
        },
        Zwischensumme {
            titel: "Jahresergebnis (Gewinn oder Verlust)".into(),
            betrag: jahresergebnis,
            vorjahr: None,
        },
    ];
    IncomeStatement { gruppen, zwischen }
}

/// Bilanz zum Stichtag (AK-4). `alle` = Bilanzkonten und Erfolgskonten bis zum Stichtag,
/// `frueher` = Erfolg bis zum Tag vor dem Geschäftsjahr. Das laufende Jahresergebnis ist der Rest.
/// So sind Aktiven und Passiven gleich, auch wenn frühere Jahre noch nicht abgeschlossen sind.
pub fn balance_sheet(stichtag: &str, alle: &[AccountBalance], frueher: i64) -> BalanceSheet {
    let mut aktiven: Vec<Group> = Vec::new();
    let mut passiven: Vec<Group> = Vec::new();
    let mut erfolg_gesamt = 0;
    for konto in alle {
        if konto.is_erfolg() {
            erfolg_gesamt += -konto.saldo();
            continue;
        }
        let (art_passiv, betrag) = match konto.kontoart.as_str() {
            "passiv" => (true, -konto.saldo()),
            "aktiv" => (false, konto.saldo()),
            _ => continue,
        };
        if betrag == 0 {
            continue;
        }
        let titel = konto.kontengruppe.clone().unwrap_or_else(|| {
            if art_passiv {
                "Übrige Passiven".into()
            } else {
                "Übrige Aktiven".into()
            }
        });
        let gruppen = if art_passiv {
            &mut passiven
        } else {
            &mut aktiven
        };
        let row = Row {
            konto_id: Some(konto.konto_id),
            nummer: konto.nummer.clone(),
            bezeichnung: konto.bezeichnung.clone(),
            betrag,
            vorjahr: None,
        };
        match gruppen.iter_mut().find(|g| g.titel == titel) {
            Some(g) => {
                g.summe += betrag;
                g.zeilen.push(row);
            }
            None => gruppen.push(Group {
                titel,
                zeilen: vec![row],
                summe: betrag,
                vorjahr: None,
            }),
        }
    }
    let laufend = erfolg_gesamt - frueher;
    passiven.push(Group {
        titel: "Jahresergebnis".into(),
        zeilen: vec![
            Row {
                konto_id: None,
                nummer: String::new(),
                bezeichnung: "Ergebnis früherer Jahre (noch nicht abgeschlossen)".into(),
                betrag: frueher,
                vorjahr: None,
            },
            Row {
                konto_id: None,
                nummer: String::new(),
                bezeichnung: "Ergebnis des laufenden Geschäftsjahres".into(),
                betrag: laufend,
                vorjahr: None,
            },
        ],
        summe: erfolg_gesamt,
        vorjahr: None,
    });
    let summe_aktiven = aktiven.iter().map(|g| g.summe).sum();
    let summe_passiven = passiven.iter().map(|g| g.summe).sum();
    BalanceSheet {
        stichtag: stichtag.to_string(),
        aktiven,
        passiven,
        summe_aktiven,
        summe_passiven,
        vorjahr_aktiven: None,
        vorjahr_passiven: None,
    }
}

/// Fügt die Werte des Vorjahres in Zeilen, Gruppen und Summen ein (AK-6). Zeilen werden über das Konto
/// zugeordnet; Konten ohne Vorjahreswert erhalten 0.
pub fn with_vorjahr_groups(groups: &mut [Group], vorjahr: &[Group]) {
    for g in groups.iter_mut() {
        let prev = vorjahr.iter().find(|p| p.titel == g.titel);
        g.vorjahr = prev.map(|p| p.summe);
        for row in g.zeilen.iter_mut() {
            // Konten über die Kennung, Zeilen ohne Konto (Ergebnis) über die Bezeichnung
            let gleich = |r: &&Row| {
                if row.konto_id.is_some() {
                    r.konto_id == row.konto_id
                } else {
                    r.konto_id.is_none() && r.bezeichnung == row.bezeichnung
                }
            };
            row.vorjahr = Some(
                prev.and_then(|p| p.zeilen.iter().find(gleich).map(|r| r.betrag))
                    .unwrap_or(0),
            );
        }
    }
}

/// Vorjahres-Stand der Erfolgsrechnung in Gruppen und Zwischensummen übernehmen (AK-6).
pub fn with_vorjahr_income(statement: &mut IncomeStatement, prev: &IncomeStatement) {
    with_vorjahr_groups(&mut statement.gruppen, &prev.gruppen);
    for (z, p) in statement.zwischen.iter_mut().zip(prev.zwischen.iter()) {
        z.vorjahr = Some(p.betrag);
    }
}

// ---------------------------------------------------------------------------------------------
// Projektauswertung
// ---------------------------------------------------------------------------------------------

/// Projekt mit Erlösen und Aufwand aus den Buchungszeilen mit diesem Projekt (AK-7).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct ProjectRow {
    pub projekt_id: i64,
    pub nummer: String,
    pub name: String,
    pub erloes: i64,
    pub aufwand: i64,
    /// Offene Einkaufsaufträge als zugesagte Kosten (ANF-004)
    pub offen: i64,
}

impl ProjectRow {
    pub fn ergebnis(&self) -> i64 {
        self.erloes - self.aufwand
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectReport {
    pub zeilen: Vec<ProjectRow>,
    pub erloes: i64,
    pub aufwand: i64,
    pub offen: i64,
}

/// Summen je Projekt; Projekte ohne Bewegung und ohne offene Aufträge fallen weg, außer ein Projekt ist gewählt.
pub fn project_report(rows: Vec<ProjectRow>, einzeln: bool) -> ProjectReport {
    let zeilen: Vec<ProjectRow> = rows
        .into_iter()
        .filter(|r| einzeln || r.erloes != 0 || r.aufwand != 0 || r.offen != 0)
        .collect();
    ProjectReport {
        erloes: zeilen.iter().map(|r| r.erloes).sum(),
        aufwand: zeilen.iter().map(|r| r.aufwand).sum(),
        offen: zeilen.iter().map(|r| r.offen).sum(),
        zeilen,
    }
}

// ---------------------------------------------------------------------------------------------
// Ergebnisseite und Datumsrechnung
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ReportKind {
    /// Noch keine Auswahl (z. B. kein Konto für den Kontoauszug)
    Leer,
    Kontoauszug(Statement),
    Saldenliste(TrialBalance),
    Bilanz(BalanceSheet),
    Erfolgsrechnung(IncomeStatement),
    Projekte(ProjectReport),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReportPage {
    pub art: String,
    pub period: Period,
    pub kind: ReportKind,
}

/// Datum um `jahre` Jahre verschoben (Vorjahr, AK-6). 29. Februar wird zum 28. Februar.
pub fn shift_years(datum: &str, jahre: i32) -> Option<String> {
    let (y, m, d) = date::split(datum)?;
    let jahr = y + jahre;
    let tag = d.min(date::days_in_month(jahr, m));
    Some(date::format(jahr, m, tag))
}

/// Letzter Tag vor `datum`, z. B. `2026-01-01` → `2025-12-31`.
pub fn previous_day(datum: &str) -> Option<String> {
    let (y, m, d) = date::split(datum)?;
    Some(if d > 1 {
        date::format(y, m, d - 1)
    } else if m > 1 {
        date::last_of_month(y, m - 1)
    } else {
        date::format(y - 1, 12, 31)
    })
}

/// Zeitraum zu einem Geschäftsjahr: Vorgabe Beginn bis heute, aber nie über das Geschäftsjahr hinaus (AK-1).
pub fn default_range(beginn: &str, ende: &str, heute: &str) -> (String, String) {
    let bis = if heute < ende { heute } else { ende };
    let bis = if bis < beginn { beginn } else { bis };
    (beginn.to_string(), bis.to_string())
}

// ---------------------------------------------------------------------------------------------
// CSV (AK-8)
// ---------------------------------------------------------------------------------------------

/// Betrag für CSV: Festkomma mit Punkt, z. B. `1234.50`.
fn csv_amount(cents: i64) -> String {
    format_fixed(cents, 2)
}

/// Feld für CSV: Semikolon als Trenner (Excel), Anführungszeichen bei Bedarf verdoppelt.
fn csv_field(value: &str) -> String {
    if value.contains([';', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn csv_line(fields: &[String]) -> String {
    fields
        .iter()
        .map(|f| csv_field(f))
        .collect::<Vec<_>>()
        .join(";")
}

/// Auswertung als CSV-Text. Eine Zeile je Konto bzw. Buchung; Summen am Ende.
pub fn to_csv(page: &ReportPage) -> String {
    let mut out = Vec::<String>::new();
    let s = |v: &str| v.to_string();
    match &page.kind {
        ReportKind::Leer => {}
        ReportKind::Kontoauszug(st) => {
            out.push(csv_line(&[
                s("Datum"),
                s("Journalnummer"),
                s("Beleg"),
                s("Text"),
                s("Gegenkonto"),
                s("Soll"),
                s("Haben"),
                s("Saldo"),
            ]));
            out.push(csv_line(&[
                s(""),
                s(""),
                s(""),
                s("Anfangssaldo"),
                s(""),
                s(""),
                s(""),
                csv_amount(st.anfangssaldo),
            ]));
            for z in &st.zeilen {
                out.push(csv_line(&[
                    german_date(&z.datum),
                    z.journalnummer.map_or(String::new(), |n| n.to_string()),
                    z.belegnummer.clone(),
                    z.text.clone(),
                    z.gegenkonto.clone(),
                    csv_amount(z.soll),
                    csv_amount(z.haben),
                    csv_amount(z.saldo),
                ]));
            }
            out.push(csv_line(&[
                s(""),
                s(""),
                s(""),
                s("Summe"),
                s(""),
                csv_amount(st.soll),
                csv_amount(st.haben),
                csv_amount(st.endsaldo),
            ]));
        }
        ReportKind::Saldenliste(tb) => {
            out.push(csv_line(&[
                s("Konto"),
                s("Bezeichnung"),
                s("Anfangssaldo"),
                s("Soll"),
                s("Haben"),
                s("Endsaldo"),
            ]));
            for k in &tb.klassen {
                for z in &k.zeilen {
                    out.push(csv_line(&[
                        z.nummer.clone(),
                        z.bezeichnung.clone(),
                        csv_amount(z.anfangssaldo),
                        csv_amount(z.soll),
                        csv_amount(z.haben),
                        csv_amount(z.endsaldo),
                    ]));
                }
                out.push(csv_line(&[
                    s(""),
                    format!("Total {}", k.titel),
                    csv_amount(k.anfangssaldo),
                    csv_amount(k.soll),
                    csv_amount(k.haben),
                    csv_amount(k.endsaldo),
                ]));
            }
            out.push(csv_line(&[
                s(""),
                s("Summe"),
                csv_amount(tb.anfangssaldo),
                csv_amount(tb.soll),
                csv_amount(tb.haben),
                csv_amount(tb.endsaldo),
            ]));
        }
        ReportKind::Bilanz(b) => {
            out.push(csv_line(&[s("Konto"), s("Bezeichnung"), s("Betrag")]));
            for (seite, gruppen) in [("Aktiven", &b.aktiven), ("Passiven", &b.passiven)] {
                out.push(csv_line(&[s(""), s(seite), s("")]));
                for g in gruppen.iter() {
                    for r in &g.zeilen {
                        out.push(csv_line(&[
                            r.nummer.clone(),
                            r.bezeichnung.clone(),
                            csv_amount(r.betrag),
                        ]));
                    }
                    out.push(csv_line(&[
                        s(""),
                        format!("Total {}", g.titel),
                        csv_amount(g.summe),
                    ]));
                }
            }
            out.push(csv_line(&[
                s(""),
                s("Summe Aktiven"),
                csv_amount(b.summe_aktiven),
            ]));
            out.push(csv_line(&[
                s(""),
                s("Summe Passiven"),
                csv_amount(b.summe_passiven),
            ]));
        }
        ReportKind::Erfolgsrechnung(e) => {
            out.push(csv_line(&[s("Konto"), s("Bezeichnung"), s("Betrag")]));
            for g in &e.gruppen {
                for r in &g.zeilen {
                    out.push(csv_line(&[
                        r.nummer.clone(),
                        r.bezeichnung.clone(),
                        csv_amount(r.betrag),
                    ]));
                }
                out.push(csv_line(&[
                    s(""),
                    format!("Total {}", g.titel),
                    csv_amount(g.summe),
                ]));
            }
            for z in &e.zwischen {
                out.push(csv_line(&[s(""), z.titel.clone(), csv_amount(z.betrag)]));
            }
        }
        ReportKind::Projekte(p) => {
            out.push(csv_line(&[
                s("Projekt"),
                s("Name"),
                s("Erlöse"),
                s("Aufwand"),
                s("Ergebnis"),
                s("Offene Aufträge"),
            ]));
            for r in &p.zeilen {
                out.push(csv_line(&[
                    r.nummer.clone(),
                    r.name.clone(),
                    csv_amount(r.erloes),
                    csv_amount(r.aufwand),
                    csv_amount(r.ergebnis()),
                    csv_amount(r.offen),
                ]));
            }
            out.push(csv_line(&[
                s(""),
                s("Summe"),
                csv_amount(p.erloes),
                csv_amount(p.aufwand),
                csv_amount(p.erloes - p.aufwand),
                csv_amount(p.offen),
            ]));
        }
    }
    out.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn konto(id: i64, nummer: &str, art: &str, soll: i64, haben: i64) -> AccountBalance {
        AccountBalance {
            konto_id: id,
            nummer: nummer.into(),
            bezeichnung: format!("Konto {nummer}"),
            kontoart: art.into(),
            kontengruppe: None,
            soll,
            haben,
        }
    }

    /// Kleine Probebuchhaltung (Salden): Kapital 10'000 eingezahlt, Ertrag 5'000, Aufwand 3'000,
    /// Abschreibung 500. Bank = 10000 + 5000 − 3000 − 500 = 11500. Die Summe aller Salden ist 0.
    fn books() -> Vec<AccountBalance> {
        vec![
            konto(1, "1020", "aktiv", 11_500_00, 0),
            konto(2, "2800", "passiv", 0, 10_000_00),
            konto(3, "3000", "ertrag", 0, 5_000_00),
            konto(4, "4000", "aufwand", 3_000_00, 0),
            konto(5, "6800", "aufwand", 500_00, 0),
        ]
    }

    #[test]
    fn balance_sheet_balances_with_current_result() {
        let bilanz = balance_sheet("2026-12-31", &books(), 0);
        // Aktiven 11500 = Kapital 10000 + Jahresergebnis 1500
        assert_eq!(bilanz.summe_aktiven, 11_500_00);
        assert_eq!(bilanz.summe_passiven, 11_500_00);
        let jahr = bilanz.passiven.last().unwrap();
        assert_eq!(jahr.summe, 1_500_00);
        assert_eq!(jahr.zeilen[1].betrag, 1_500_00);
    }

    #[test]
    fn earlier_result_is_shown_separately() {
        // 800 Gewinn kommen aus früheren Jahren; das laufende Jahr ist der Rest (1500 − 800 = 700)
        let bilanz = balance_sheet("2026-12-31", &books(), 800_00);
        let jahr = bilanz.passiven.last().unwrap();
        assert_eq!(jahr.zeilen[0].betrag, 800_00);
        assert_eq!(jahr.zeilen[1].betrag, 700_00);
        assert_eq!(bilanz.summe_aktiven, bilanz.summe_passiven);
    }

    #[test]
    fn income_statement_matches_balance_result() {
        let er = income_statement(&books());
        // Ertrag 5000, Aufwand 3000 und Abschreibung 500 → Jahresergebnis 1500
        let jahr = er.zwischen.last().unwrap();
        assert_eq!(jahr.betrag, 1_500_00);
        // Bruttogewinn 1: 5000 − 3000; vor Abschreibungen bleibt 2000, nach Abschreibungen 1500
        assert_eq!(er.zwischen[0].betrag, 2_000_00);
        assert_eq!(er.zwischen[2].betrag, 2_000_00);
        assert_eq!(er.zwischen[3].betrag, 1_500_00);
    }

    #[test]
    fn trial_balance_skips_empty_accounts_and_sums_classes() {
        let anfang = vec![konto(1, "1020", "aktiv", 1_000_00, 0)];
        let bewegung = vec![
            konto(1, "1020", "aktiv", 0, 0),
            konto(2, "3000", "ertrag", 0, 5_000_00),
            konto(3, "4000", "aufwand", 3_000_00, 0),
            konto(4, "1100", "aktiv", 0, 0),
        ];
        let tb = trial_balance(&anfang, &bewegung);
        // 1020 hat Anfangssaldo, also aufgenommen; 1100 ohne Bewegung und Saldo wird weggelassen
        let nummern: Vec<&str> = tb
            .klassen
            .iter()
            .flat_map(|k| k.zeilen.iter().map(|z| z.nummer.as_str()))
            .collect();
        assert_eq!(nummern, vec!["1020", "3000", "4000"]);
        assert_eq!(tb.anfangssaldo, 1_000_00);
        assert_eq!(tb.soll, 3_000_00);
        assert_eq!(tb.haben, 5_000_00);
        assert!(!tb.ausgeglichen);
    }

    #[test]
    fn statement_running_balance_and_counter_account() {
        let k = konto(7, "1020", "aktiv", 0, 0);
        let lines = vec![
            StatementLine {
                buchung_id: 1,
                datum: "2026-01-05".into(),
                journalnummer: Some(1),
                belegnummer: "M-1".into(),
                text: "Einzahlung".into(),
                soll: 1_000_00,
                haben: 0,
                ist_mwst_zeile: false,
            },
            StatementLine {
                buchung_id: 2,
                datum: "2026-01-06".into(),
                journalnummer: Some(2),
                belegnummer: "M-2".into(),
                text: "Miete".into(),
                soll: 0,
                haben: 400_00,
                ist_mwst_zeile: false,
            },
        ];
        let counter = vec![
            CounterLine {
                buchung_id: 1,
                konto_id: 3,
                nummer: "2800".into(),
                ist_mwst_zeile: false,
            },
            CounterLine {
                buchung_id: 2,
                konto_id: 5,
                nummer: "6000".into(),
                ist_mwst_zeile: false,
            },
            CounterLine {
                buchung_id: 2,
                konto_id: 6,
                nummer: "1170".into(),
                ist_mwst_zeile: true,
            },
        ];
        let st = statement(&k, 200_00, &lines, &counter);
        assert_eq!(st.zeilen[0].saldo, 1_200_00);
        assert_eq!(st.zeilen[0].gegenkonto, "2800");
        assert_eq!(st.endsaldo, 800_00);
        assert_eq!(st.soll, 1_000_00);
        assert_eq!(st.haben, 400_00);
        assert_eq!(st.zeilen[1].gegenkonto, "6000");
    }

    #[test]
    fn gegenkonto_for_collective_and_tax_lines() {
        let line = StatementLine {
            buchung_id: 9,
            datum: String::new(),
            journalnummer: None,
            belegnummer: String::new(),
            text: String::new(),
            soll: 0,
            haben: 0,
            ist_mwst_zeile: false,
        };
        let c = |konto_id: i64, nummer: &str| CounterLine {
            buchung_id: 9,
            konto_id,
            nummer: nummer.into(),
            ist_mwst_zeile: false,
        };
        assert_eq!(
            gegenkonto(&line, &[c(2, "1020"), c(3, "4000")], 1),
            "Diverse"
        );
        assert_eq!(gegenkonto(&line, &[c(2, "1020"), c(2, "1020")], 1), "1020");
        assert_eq!(gegenkonto(&line, &[], 1), "–");
        let tax = StatementLine {
            ist_mwst_zeile: true,
            ..line
        };
        assert_eq!(gegenkonto(&tax, &[], 1), "MWST");
    }

    #[test]
    fn vorjahr_values_follow_accounts() {
        let now = income_statement(&books());
        let mut prev_books = books();
        prev_books[2] = konto(3, "3000", "ertrag", 0, 4_000_00);
        let prev = income_statement(&prev_books);
        let mut current = now.clone();
        with_vorjahr_income(&mut current, &prev);
        assert_eq!(
            current.zwischen.last().unwrap().vorjahr,
            Some(prev.zwischen.last().unwrap().betrag)
        );
        let ertrag = current
            .gruppen
            .iter()
            .find(|g| g.titel.starts_with("Klasse 3"))
            .unwrap();
        assert_eq!(ertrag.zeilen[0].vorjahr, Some(4_000_00));
    }

    #[test]
    fn shift_years_clamps_leap_day_and_default_range() {
        assert_eq!(shift_years("2028-02-29", -1).as_deref(), Some("2027-02-28"));
        assert_eq!(shift_years("2026-11-26", -1).as_deref(), Some("2025-11-26"));
        assert_eq!(shift_years("x", -1), None);
        assert_eq!(
            default_range("2026-01-01", "2026-12-31", "2026-11-26"),
            ("2026-01-01".into(), "2026-11-26".into())
        );
        assert_eq!(
            default_range("2026-01-01", "2026-12-31", "2027-03-01"),
            ("2026-01-01".into(), "2026-12-31".into())
        );
    }

    #[test]
    fn projects_without_movement_are_dropped() {
        let rows = vec![
            ProjectRow {
                projekt_id: 1,
                nummer: "P-1".into(),
                name: "A".into(),
                erloes: 0,
                aufwand: 0,
                offen: 0,
            },
            ProjectRow {
                projekt_id: 2,
                nummer: "P-2".into(),
                name: "B".into(),
                erloes: 1_000_00,
                aufwand: 400_00,
                offen: 0,
            },
        ];
        let report = project_report(rows.clone(), false);
        assert_eq!(report.zeilen.len(), 1);
        assert_eq!(report.zeilen[0].ergebnis(), 600_00);
        assert_eq!(project_report(rows, true).zeilen.len(), 2);
    }

    #[test]
    fn csv_quotes_separators() {
        let page = ReportPage {
            art: "projekte".into(),
            period: Period {
                geschaeftsjahr_id: 1,
                geschaeftsjahr: "2026".into(),
                beginn: "2026-01-01".into(),
                ende: "2026-12-31".into(),
                von: "2026-01-01".into(),
                bis: "2026-12-31".into(),
                vorjahr: None,
            },
            kind: ReportKind::Projekte(ProjectReport {
                zeilen: vec![ProjectRow {
                    projekt_id: 1,
                    nummer: "P-1".into(),
                    name: "Halle; \"Nord\"".into(),
                    erloes: 1_234_50,
                    aufwand: 200_00,
                    offen: 0,
                }],
                erloes: 1_234_50,
                aufwand: 200_00,
                offen: 0,
            }),
        };
        let csv = to_csv(&page);
        assert!(csv.contains("P-1;\"Halle; \"\"Nord\"\"\";1234.50;200.00;1034.50;0.00"));
        assert!(csv.starts_with("Projekt;Name;"));
    }
}
