//! Datenmodell und Prüfregeln für den Kontenplan und die Systemkonten (ANF-011).
//! Wird auf dem Server und im Browser verwendet.

use serde::{Deserialize, Serialize};

pub use crate::forms::{FieldErrors, SaveOutcome};

pub const NUMBER_MIN_LEN: usize = 4;
pub const NUMBER_MAX_LEN: usize = 6;
pub const NAME_MAX: usize = 100;
pub const GROUP_MAX: usize = 100;
pub const REMARK_MAX: usize = 2000;

/// Kontoarten (Schlüssel in der Datenbank → Beschriftung).
pub const KINDS: &[(&str, &str)] = &[
    ("aktiv", "Aktiv"),
    ("passiv", "Passiv"),
    ("aufwand", "Aufwand"),
    ("ertrag", "Ertrag"),
];

/// Filter der Liste nach Kontoart (AK-2). „alle“ ist der Standard.
pub const KIND_FILTERS: &[(&str, &str)] = &[
    ("alle", "Alle Kontoarten"),
    ("aktiv", "Aktiv"),
    ("passiv", "Passiv"),
    ("aufwand", "Aufwand"),
    ("ertrag", "Ertrag"),
];

pub fn kind_label(kind: &str) -> &'static str {
    KINDS
        .iter()
        .find(|(k, _)| *k == kind)
        .map_or("–", |(_, l)| l)
}

/// Plakette der Kontoart: Bilanzkonten umrandet, Erfolgskonten flach (keine Signalfarben).
pub fn kind_badge(kind: &str) -> &'static str {
    match kind {
        "aktiv" | "passiv" => "badge badge-sm badge-outline",
        _ => "badge badge-sm badge-ghost",
    }
}

/// Kontenklassen des KMU-Kontenrahmens (erste Ziffer der Kontonummer).
pub const CLASSES: &[(u8, &str)] = &[
    (1, "Aktiven"),
    (2, "Passiven"),
    (3, "Betriebsertrag aus Lieferungen und Leistungen"),
    (
        4,
        "Aufwand für Material, Handelswaren, Dienstleistungen und Energie",
    ),
    (5, "Personalaufwand"),
    (
        6,
        "Übriger betrieblicher Aufwand, Abschreibungen und Finanzergebnis",
    ),
    (7, "Betrieblicher Nebenerfolg"),
    (
        8,
        "Betriebsfremder, ausserordentlicher, einmaliger oder periodenfremder Erfolg",
    ),
    (9, "Abschluss"),
];

/// Kontenklasse einer (gültigen) Kontonummer, z. B. „1020“ → 1.
pub fn class_of(number: &str) -> Option<u8> {
    number
        .chars()
        .next()
        .and_then(|c| c.to_digit(10))
        .and_then(|d| u8::try_from(d).ok())
        .filter(|d| *d >= 1)
}

pub fn class_name(class: u8) -> &'static str {
    CLASSES
        .iter()
        .find(|(c, _)| *c == class)
        .map_or("", |(_, n)| n)
}

/// Vorschlag der Kontoart aus der ersten Ziffer (AK-5). Klassen 7–9 enthalten Aufwand und Ertrag –
/// dort gibt es keinen Vorschlag.
pub fn suggested_kind(number: &str) -> Option<&'static str> {
    match class_of(number.trim())? {
        1 => Some("aktiv"),
        2 => Some("passiv"),
        3 => Some("ertrag"),
        4..=6 => Some("aufwand"),
        _ => None,
    }
}

/// 4 bis 6 Ziffern, erste Ziffer 1–9 (AK-4).
pub fn is_valid_number(number: &str) -> bool {
    (NUMBER_MIN_LEN..=NUMBER_MAX_LEN).contains(&number.len())
        && number.chars().all(|c| c.is_ascii_digit())
        && class_of(number).is_some()
}

/// Ein Konto mit den Rollen, in denen es als Systemkonto dient.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Account {
    pub id: i64,
    pub nummer: String,
    pub bezeichnung: String,
    pub kontoart: String,
    pub kontengruppe: Option<String>,
    pub aktiv: bool,
    pub bemerkung: Option<String>,
    /// Schlüssel aus `SYSTEM_ROLES`
    pub rollen: Vec<String>,
    /// Vorgabe-MWST-Code (ANF-011 AK-7) und seine Kennung
    pub mwst_code_id: Option<i64>,
    pub mwst_code: Option<String>,
    /// Auf das Konto ist gebucht: Nummer und Kontoart sind gesperrt (AK-6)
    pub gebucht: bool,
}

impl Account {
    /// „1020 Bankguthaben“
    pub fn label(&self) -> String {
        format!("{} {}", self.nummer, self.bezeichnung)
    }

    pub fn to_input(&self) -> AccountInput {
        AccountInput {
            nummer: self.nummer.clone(),
            bezeichnung: self.bezeichnung.clone(),
            kontoart: self.kontoart.clone(),
            kontengruppe: self.kontengruppe.clone().unwrap_or_default(),
            bemerkung: self.bemerkung.clone().unwrap_or_default(),
            aktiv: if self.aktiv {
                "1".into()
            } else {
                String::new()
            },
            mwst_code_id: self
                .mwst_code_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
        }
    }
}

/// Formulareingabe für Anlegen und Bearbeiten (AK-4, AK-6, AK-10).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct AccountInput {
    pub nummer: String,
    pub bezeichnung: String,
    pub kontoart: String,
    pub kontengruppe: String,
    pub bemerkung: String,
    /// Kontrollkästchen: nicht leer = aktiv
    pub aktiv: String,
    /// Vorgabe-MWST-Code als Kennung; leer = keiner
    pub mwst_code_id: String,
}

impl AccountInput {
    pub fn new_default() -> Self {
        Self {
            aktiv: "1".into(),
            ..Default::default()
        }
    }

    pub fn normalized(&self) -> Self {
        Self {
            nummer: self.nummer.trim().into(),
            bezeichnung: self.bezeichnung.trim().into(),
            kontoart: self.kontoart.trim().to_lowercase(),
            kontengruppe: self.kontengruppe.trim().into(),
            bemerkung: self.bemerkung.trim().into(),
            aktiv: self.aktiv.trim().into(),
            mwst_code_id: self.mwst_code_id.trim().into(),
        }
    }

    /// Prüft die normalisierte Eingabe (AK-4, AK-8). Leere Map = gültig.
    /// Ob die Nummer frei ist und das Konto als Systemkonto dient, prüft der Server zusätzlich.
    pub fn validate(&self) -> FieldErrors {
        let mut errors = FieldErrors::new();
        let mut err = |f: &str, m: String| {
            errors.entry(f.into()).or_insert(m);
        };
        if self.nummer.is_empty() {
            err("nummer", "Bitte eine Kontonummer eingeben.".into());
        } else if !is_valid_number(&self.nummer) {
            err(
                "nummer",
                format!(
                    "{NUMBER_MIN_LEN} bis {NUMBER_MAX_LEN} Ziffern, erste Ziffer 1–9 (Kontenklasse)."
                ),
            );
        }
        if self.bezeichnung.is_empty() {
            err("bezeichnung", "Bitte eine Bezeichnung eingeben.".into());
        } else if self.bezeichnung.chars().count() > NAME_MAX {
            err("bezeichnung", format!("Höchstens {NAME_MAX} Zeichen."));
        }
        if !KINDS.iter().any(|(k, _)| *k == self.kontoart) {
            err("kontoart", "Bitte eine Kontoart wählen.".into());
        }
        if self.kontengruppe.chars().count() > GROUP_MAX {
            err("kontengruppe", format!("Höchstens {GROUP_MAX} Zeichen."));
        }
        if self.bemerkung.chars().count() > REMARK_MAX {
            err("bemerkung", format!("Höchstens {REMARK_MAX} Zeichen."));
        }
        errors
    }

    pub fn is_active(&self) -> bool {
        !self.aktiv.is_empty()
    }

    /// Kennung des Vorgabe-MWST-Codes (`None` = keiner oder keine gültige Zahl).
    pub fn mwst_id(&self) -> Option<i64> {
        self.mwst_code_id.parse().ok()
    }

    pub fn group(&self) -> Option<&str> {
        (!self.kontengruppe.is_empty()).then_some(self.kontengruppe.as_str())
    }

    pub fn remark(&self) -> Option<&str> {
        (!self.bemerkung.is_empty()).then_some(self.bemerkung.as_str())
    }
}

/// Rolle eines Systemkontos (AK-11).
#[derive(Clone, Copy, Debug)]
pub struct SystemRole {
    pub key: &'static str,
    pub label: &'static str,
    /// Erlaubte Kontoarten
    pub kinds: &'static [&'static str],
    /// Pflicht für automatische Buchungen aus Debitoren und Kreditoren (AK-12)
    pub required: bool,
}

/// Alle Rollen in der Reihenfolge der Anzeige.
pub const SYSTEM_ROLES: &[SystemRole] = &[
    SystemRole {
        key: "debitoren",
        label: "Debitoren",
        kinds: &["aktiv"],
        required: true,
    },
    SystemRole {
        key: "kreditoren",
        label: "Kreditoren",
        kinds: &["passiv"],
        required: true,
    },
    SystemRole {
        key: "vorsteuer",
        label: "Vorsteuer",
        kinds: &["aktiv"],
        required: true,
    },
    SystemRole {
        key: "umsatzsteuer",
        label: "Umsatzsteuer",
        kinds: &["passiv"],
        required: true,
    },
    SystemRole {
        key: "mwst_abrechnung",
        label: "Abrechnungskonto MWST",
        kinds: &["aktiv", "passiv"],
        required: false,
    },
    SystemRole {
        key: "bank",
        label: "Bank (Vorgabe)",
        kinds: &["aktiv"],
        required: false,
    },
    SystemRole {
        key: "kursdifferenzen",
        label: "Kursdifferenzen",
        kinds: &["aufwand", "ertrag"],
        required: false,
    },
    SystemRole {
        key: "skonto_gewaehrt",
        label: "Skonto gewährt (Erlösminderung)",
        kinds: &["ertrag", "aufwand"],
        required: false,
    },
    SystemRole {
        key: "skonto_erhalten",
        label: "Skonto erhalten (Aufwandminderung)",
        kinds: &["aufwand", "ertrag"],
        required: false,
    },
    SystemRole {
        key: "rundungsdifferenzen",
        label: "Rundungsdifferenzen",
        kinds: &["aufwand", "ertrag"],
        required: false,
    },
    SystemRole {
        key: "gewinnvortrag",
        label: "Gewinnvortrag",
        kinds: &["passiv"],
        required: false,
    },
    SystemRole {
        key: "eroeffnungsbilanz",
        label: "Eröffnungsbilanz",
        kinds: &["aktiv", "passiv"],
        required: false,
    },
    SystemRole {
        key: "jahresergebnis",
        label: "Jahresgewinn/-verlust",
        kinds: &["passiv"],
        required: false,
    },
    // ANF-018 AK-5: Vorgabe für das Ertragskonto neuer Rechnungspositionen
    SystemRole {
        key: "ertrag",
        label: "Ertrag (Vorgabe Debitoren)",
        kinds: &["ertrag"],
        required: false,
    },
];

pub fn system_role(key: &str) -> Option<&'static SystemRole> {
    SYSTEM_ROLES.iter().find(|r| r.key == key)
}

pub fn role_label(key: &str) -> &'static str {
    system_role(key).map_or("–", |r| r.label)
}

/// „Aktiv oder Passiv“
pub fn kinds_text(kinds: &[&str]) -> String {
    let labels: Vec<&str> = kinds.iter().map(|k| kind_label(k)).collect();
    labels.join(" oder ")
}

/// Fehlermeldung, wenn ein Konto für eine Rolle nicht passt (AK-11), sonst `None`.
pub fn system_assignment_error(role: &SystemRole, kind: &str, active: bool) -> Option<String> {
    if !active {
        Some("Das Konto ist inaktiv.".into())
    } else if !role.kinds.contains(&kind) {
        Some(format!(
            "Für „{}“ ist nur ein Konto der Kontoart {} erlaubt.",
            role.label,
            kinds_text(role.kinds)
        ))
    } else {
        None
    }
}

/// Eine Rolle mit dem zugeordneten Konto (Anzeige der Systemkonten).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SystemAccount {
    pub rolle: String,
    pub konto: Option<Account>,
}

/// Zustand des Kontenplans für die Seite: Anzahl Konten und Systemkonten (AK-3, AK-11, AK-12).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChartSummary {
    pub anzahl: i64,
    pub systemkonten: Vec<SystemAccount>,
}

impl ChartSummary {
    /// Pflicht-Rollen ohne Konto (AK-12).
    pub fn missing_required(&self) -> Vec<&'static str> {
        SYSTEM_ROLES
            .iter()
            .filter(|r| r.required)
            .filter(|r| {
                !self
                    .systemkonten
                    .iter()
                    .any(|s| s.rolle == r.key && s.konto.is_some())
            })
            .map(|r| r.label)
            .collect()
    }
}

/// Konto der Vorlage „KMU-Kontenrahmen“ (AK-3).
#[derive(Clone, Copy, Debug)]
pub struct TemplateAccount {
    pub nummer: &'static str,
    pub bezeichnung: &'static str,
    pub kontoart: &'static str,
    pub gruppe: &'static str,
}

const fn t(
    nummer: &'static str,
    bezeichnung: &'static str,
    kontoart: &'static str,
    gruppe: &'static str,
) -> TemplateAccount {
    TemplateAccount {
        nummer,
        bezeichnung,
        kontoart,
        gruppe,
    }
}

const FLUESSIG: &str = "Flüssige Mittel";
const FORD_LL: &str = "Forderungen aus Lieferungen und Leistungen";
const FORD_UEB: &str = "Übrige kurzfristige Forderungen";
const VORRAT: &str = "Vorräte und nicht fakturierte Dienstleistungen";
const ARA: &str = "Aktive Rechnungsabgrenzungen";
const FINANZ: &str = "Finanzanlagen";
const MOBIL: &str = "Mobile Sachanlagen";
const IMMOBIL: &str = "Immobile Sachanlagen";
const IMMAT: &str = "Immaterielle Werte";
const VERB_LL: &str = "Verbindlichkeiten aus Lieferungen und Leistungen";
const VERB_KZ: &str = "Kurzfristige verzinsliche Verbindlichkeiten";
const VERB_UEB: &str = "Übrige kurzfristige Verbindlichkeiten";
const PRA: &str = "Passive Rechnungsabgrenzungen und kurzfristige Rückstellungen";
const VERB_LZ: &str = "Langfristige Verbindlichkeiten";
const EK: &str = "Eigenkapital";
const ERLOES: &str = "Erlöse aus Lieferungen und Leistungen";
const ERLOES_MIN: &str = "Erlösminderungen";
const MATERIAL: &str = "Aufwand für Material, Waren und Dienstleistungen";
const PERSONAL: &str = "Personalaufwand";
const UEB_AUFW: &str = "Übriger betrieblicher Aufwand";
const ABSCHR: &str = "Abschreibungen";
const FIN_ERG: &str = "Finanzergebnis";
const NEBEN: &str = "Betrieblicher Nebenerfolg";
const BETR_FREMD: &str = "Betriebsfremder und ausserordentlicher Erfolg";
const STEUERN: &str = "Direkte Steuern";
const ABSCHLUSS: &str = "Abschluss";

/// Gängige Konten des Schweizer KMU-Kontenrahmens als Vorlage (AK-3).
pub const KMU_TEMPLATE: &[TemplateAccount] = &[
    // Klasse 1 – Aktiven
    t("1000", "Kasse", "aktiv", FLUESSIG),
    t("1020", "Bankguthaben", "aktiv", FLUESSIG),
    t("1060", "Wertschriften (kurzfristig)", "aktiv", FLUESSIG),
    t(
        "1100",
        "Forderungen aus Lieferungen und Leistungen (Debitoren)",
        "aktiv",
        FORD_LL,
    ),
    t("1109", "Delkredere", "aktiv", FORD_LL),
    t(
        "1170",
        "Vorsteuer MWST Material, Waren, Dienstleistungen, Energie",
        "aktiv",
        FORD_UEB,
    ),
    t(
        "1171",
        "Vorsteuer MWST Investitionen, übriger Betriebsaufwand",
        "aktiv",
        FORD_UEB,
    ),
    t("1176", "Guthaben Verrechnungssteuer", "aktiv", FORD_UEB),
    t(
        "1190",
        "Sonstige kurzfristige Forderungen",
        "aktiv",
        FORD_UEB,
    ),
    t("1200", "Handelswaren", "aktiv", VORRAT),
    t("1210", "Rohstoffe", "aktiv", VORRAT),
    t("1260", "Fertige Erzeugnisse", "aktiv", VORRAT),
    t(
        "1280",
        "Nicht fakturierte Dienstleistungen",
        "aktiv",
        VORRAT,
    ),
    t("1300", "Bezahlter Aufwand des Folgejahres", "aktiv", ARA),
    t("1301", "Noch nicht erhaltener Ertrag", "aktiv", ARA),
    t("1400", "Wertschriften (langfristig)", "aktiv", FINANZ),
    t("1440", "Aktivdarlehen", "aktiv", FINANZ),
    t("1480", "Beteiligungen", "aktiv", FINANZ),
    t("1500", "Maschinen und Apparate", "aktiv", MOBIL),
    t(
        "1509",
        "Wertberichtigungen Maschinen und Apparate",
        "aktiv",
        MOBIL,
    ),
    t("1510", "Mobiliar und Einrichtungen", "aktiv", MOBIL),
    t(
        "1519",
        "Wertberichtigungen Mobiliar und Einrichtungen",
        "aktiv",
        MOBIL,
    ),
    t(
        "1520",
        "Büromaschinen, Informatik, Kommunikationstechnologie",
        "aktiv",
        MOBIL,
    ),
    t(
        "1529",
        "Wertberichtigungen Büromaschinen, Informatik",
        "aktiv",
        MOBIL,
    ),
    t("1530", "Fahrzeuge", "aktiv", MOBIL),
    t("1539", "Wertberichtigungen Fahrzeuge", "aktiv", MOBIL),
    t("1540", "Werkzeuge und Geräte", "aktiv", MOBIL),
    t("1600", "Geschäftsliegenschaften", "aktiv", IMMOBIL),
    t(
        "1609",
        "Wertberichtigungen Geschäftsliegenschaften",
        "aktiv",
        IMMOBIL,
    ),
    t(
        "1700",
        "Patente, Know-how, Lizenzen, Rechte, Entwicklungen",
        "aktiv",
        IMMAT,
    ),
    t("1770", "Goodwill", "aktiv", IMMAT),
    // Klasse 2 – Passiven
    t(
        "2000",
        "Verbindlichkeiten aus Lieferungen und Leistungen (Kreditoren)",
        "passiv",
        VERB_LL,
    ),
    t("2030", "Erhaltene Anzahlungen", "passiv", VERB_LL),
    t(
        "2100",
        "Bankverbindlichkeiten (kurzfristig)",
        "passiv",
        VERB_KZ,
    ),
    t(
        "2200",
        "Geschuldete MWST (Umsatzsteuer)",
        "passiv",
        VERB_UEB,
    ),
    t("2201", "Abrechnungskonto MWST", "passiv", VERB_UEB),
    t("2206", "Verrechnungssteuer", "passiv", VERB_UEB),
    t("2208", "Direkte Steuern", "passiv", VERB_UEB),
    t(
        "2270",
        "Sozialversicherungen und Vorsorgeeinrichtungen",
        "passiv",
        VERB_UEB,
    ),
    t("2300", "Noch nicht bezahlter Aufwand", "passiv", PRA),
    t("2301", "Erhaltener Ertrag des Folgejahres", "passiv", PRA),
    t("2330", "Kurzfristige Rückstellungen", "passiv", PRA),
    t(
        "2400",
        "Bankverbindlichkeiten (langfristig)",
        "passiv",
        VERB_LZ,
    ),
    t("2450", "Darlehen", "passiv", VERB_LZ),
    t("2451", "Hypotheken", "passiv", VERB_LZ),
    t("2600", "Rückstellungen (langfristig)", "passiv", VERB_LZ),
    t(
        "2800",
        "Aktien-, Stamm- oder Anteilscheinkapital",
        "passiv",
        EK,
    ),
    t("2900", "Gesetzliche Kapitalreserve", "passiv", EK),
    t("2950", "Gesetzliche Gewinnreserve", "passiv", EK),
    t("2960", "Freiwillige Gewinnreserven", "passiv", EK),
    t("2970", "Gewinnvortrag oder Verlustvortrag", "passiv", EK),
    t("2979", "Jahresgewinn oder Jahresverlust", "passiv", EK),
    // Klasse 3 – Betriebsertrag
    t("3000", "Produktionserlöse", "ertrag", ERLOES),
    t("3200", "Handelserlöse", "ertrag", ERLOES),
    t("3400", "Dienstleistungserlöse", "ertrag", ERLOES),
    t(
        "3600",
        "Übrige Erlöse aus Lieferungen und Leistungen",
        "ertrag",
        ERLOES,
    ),
    t("3700", "Eigenleistungen", "ertrag", ERLOES),
    t("3710", "Eigenverbrauch", "ertrag", ERLOES),
    t(
        "3800",
        "Erlösminderungen (Skonti, Rabatte)",
        "ertrag",
        ERLOES_MIN,
    ),
    t("3805", "Verluste aus Forderungen", "ertrag", ERLOES_MIN),
    t(
        "3900",
        "Bestandesänderungen unfertige und fertige Erzeugnisse",
        "ertrag",
        ERLOES_MIN,
    ),
    // Klasse 4 – Material- und Warenaufwand
    t("4000", "Materialaufwand Produktion", "aufwand", MATERIAL),
    t("4200", "Handelswarenaufwand", "aufwand", MATERIAL),
    t(
        "4400",
        "Aufwand für bezogene Dienstleistungen",
        "aufwand",
        MATERIAL,
    ),
    t(
        "4500",
        "Energieaufwand zur Leistungserstellung",
        "aufwand",
        MATERIAL,
    ),
    t(
        "4900",
        "Aufwandminderungen (Skonti, Rabatte)",
        "aufwand",
        MATERIAL,
    ),
    // Klasse 5 – Personalaufwand
    t("5000", "Lohnaufwand", "aufwand", PERSONAL),
    t("5700", "Sozialversicherungsaufwand", "aufwand", PERSONAL),
    t("5800", "Übriger Personalaufwand", "aufwand", PERSONAL),
    t("5900", "Leistungen Dritter", "aufwand", PERSONAL),
    // Klasse 6 – Übriger betrieblicher Aufwand und Finanzergebnis
    t("6000", "Raumaufwand", "aufwand", UEB_AUFW),
    t(
        "6100",
        "Unterhalt, Reparaturen, Ersatz mobile Sachanlagen",
        "aufwand",
        UEB_AUFW,
    ),
    t(
        "6105",
        "Leasingaufwand mobile Sachanlagen",
        "aufwand",
        UEB_AUFW,
    ),
    t(
        "6200",
        "Fahrzeug- und Transportaufwand",
        "aufwand",
        UEB_AUFW,
    ),
    t("6260", "Fahrzeugleasing und -mieten", "aufwand", UEB_AUFW),
    t(
        "6300",
        "Sachversicherungen, Abgaben, Gebühren, Bewilligungen",
        "aufwand",
        UEB_AUFW,
    ),
    t(
        "6400",
        "Energie- und Entsorgungsaufwand",
        "aufwand",
        UEB_AUFW,
    ),
    t("6500", "Verwaltungsaufwand", "aufwand", UEB_AUFW),
    t(
        "6570",
        "Informatikaufwand inkl. Leasing",
        "aufwand",
        UEB_AUFW,
    ),
    t("6600", "Werbeaufwand", "aufwand", UEB_AUFW),
    t(
        "6700",
        "Sonstiger betrieblicher Aufwand",
        "aufwand",
        UEB_AUFW,
    ),
    t(
        "6800",
        "Abschreibungen und Wertberichtigungen Anlagevermögen",
        "aufwand",
        ABSCHR,
    ),
    t("6900", "Finanzaufwand", "aufwand", FIN_ERG),
    t("6940", "Bankspesen", "aufwand", FIN_ERG),
    t("6950", "Finanzertrag", "ertrag", FIN_ERG),
    t("6960", "Kursdifferenzen", "aufwand", FIN_ERG),
    t("6990", "Rundungsdifferenzen", "aufwand", FIN_ERG),
    // Klasse 7 – Betrieblicher Nebenerfolg
    t("7000", "Ertrag Nebenbetrieb", "ertrag", NEBEN),
    t("7010", "Aufwand Nebenbetrieb", "aufwand", NEBEN),
    t("7500", "Ertrag betriebliche Liegenschaft", "ertrag", NEBEN),
    t(
        "7510",
        "Aufwand betriebliche Liegenschaft",
        "aufwand",
        NEBEN,
    ),
    // Klasse 8 – Betriebsfremder und ausserordentlicher Erfolg, Steuern
    t("8000", "Betriebsfremder Aufwand", "aufwand", BETR_FREMD),
    t("8100", "Betriebsfremder Ertrag", "ertrag", BETR_FREMD),
    t(
        "8500",
        "Ausserordentlicher, einmaliger oder periodenfremder Aufwand",
        "aufwand",
        BETR_FREMD,
    ),
    t(
        "8510",
        "Ausserordentlicher, einmaliger oder periodenfremder Ertrag",
        "ertrag",
        BETR_FREMD,
    ),
    t("8900", "Direkte Steuern", "aufwand", STEUERN),
    // Klasse 9 – Abschluss
    t("9100", "Eröffnungsbilanz", "passiv", ABSCHLUSS),
];

/// Vorbelegung der Systemkonten beim Übernehmen der Vorlage (Rolle → Kontonummer, AK-3).
pub const KMU_SYSTEM_ACCOUNTS: &[(&str, &str)] = &[
    ("debitoren", "1100"),
    ("kreditoren", "2000"),
    ("vorsteuer", "1170"),
    ("umsatzsteuer", "2200"),
    ("mwst_abrechnung", "2201"),
    ("bank", "1020"),
    ("kursdifferenzen", "6960"),
    ("skonto_gewaehrt", "3800"),
    ("skonto_erhalten", "4900"),
    ("rundungsdifferenzen", "6990"),
    ("gewinnvortrag", "2970"),
    ("eroeffnungsbilanz", "9100"),
    ("jahresergebnis", "2979"),
    ("ertrag", "3400"),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_input() -> AccountInput {
        AccountInput {
            nummer: " 1020 ".into(),
            bezeichnung: " Bankguthaben ".into(),
            kontoart: "Aktiv".into(),
            aktiv: "1".into(),
            ..Default::default()
        }
        .normalized()
    }

    #[test]
    fn numbers() {
        assert!(is_valid_number("1000"));
        assert!(is_valid_number("102001"));
        assert!(!is_valid_number("100"));
        assert!(!is_valid_number("1000000"));
        assert!(!is_valid_number("0100"));
        assert!(!is_valid_number("10a0"));
        assert!(!is_valid_number("１０２０"));
        assert_eq!(class_of("6950"), Some(6));
        assert_eq!(class_name(2), "Passiven");
    }

    #[test]
    fn kind_suggestion_from_first_digit() {
        assert_eq!(suggested_kind("1020"), Some("aktiv"));
        assert_eq!(suggested_kind("2000"), Some("passiv"));
        assert_eq!(suggested_kind("3400"), Some("ertrag"));
        assert_eq!(suggested_kind("4"), Some("aufwand"));
        assert_eq!(suggested_kind("6570"), Some("aufwand"));
        assert_eq!(suggested_kind("7000"), None);
        assert_eq!(suggested_kind("9100"), None);
        assert_eq!(suggested_kind(""), None);
    }

    #[test]
    fn input_validation() {
        let input = valid_input();
        assert_eq!(input.kontoart, "aktiv");
        assert!(input.validate().is_empty());
        assert!(input.is_active());
        assert_eq!(input.group(), None);

        let bad = AccountInput {
            nummer: "12".into(),
            bezeichnung: String::new(),
            kontoart: "bilanz".into(),
            kontengruppe: "x".repeat(GROUP_MAX + 1),
            ..Default::default()
        };
        let errors = bad.validate();
        for field in ["nummer", "bezeichnung", "kontoart", "kontengruppe"] {
            assert!(errors.contains_key(field), "{field}");
        }
        assert!(!bad.is_active());
    }

    #[test]
    fn system_role_rules() {
        let debitoren = system_role("debitoren").unwrap();
        assert!(system_assignment_error(debitoren, "aktiv", true).is_none());
        assert!(system_assignment_error(debitoren, "passiv", true).is_some());
        assert!(system_assignment_error(debitoren, "aktiv", false).is_some());
        assert_eq!(kinds_text(&["aufwand", "ertrag"]), "Aufwand oder Ertrag");
        let required: Vec<_> = SYSTEM_ROLES
            .iter()
            .filter(|r| r.required)
            .map(|r| r.key)
            .collect();
        assert_eq!(
            required,
            ["debitoren", "kreditoren", "vorsteuer", "umsatzsteuer"]
        );
    }

    #[test]
    fn missing_required_roles() {
        let account = Account {
            id: 1,
            nummer: "1100".into(),
            bezeichnung: "Debitoren".into(),
            kontoart: "aktiv".into(),
            kontengruppe: None,
            aktiv: true,
            bemerkung: None,
            rollen: vec!["debitoren".into()],
            mwst_code_id: None,
            mwst_code: None,
            gebucht: false,
        };
        let summary = ChartSummary {
            anzahl: 1,
            systemkonten: vec![
                SystemAccount {
                    rolle: "debitoren".into(),
                    konto: Some(account),
                },
                SystemAccount {
                    rolle: "kreditoren".into(),
                    konto: None,
                },
            ],
        };
        assert_eq!(
            summary.missing_required(),
            ["Kreditoren", "Vorsteuer", "Umsatzsteuer"]
        );
    }

    #[test]
    fn template_is_consistent() {
        let mut numbers: Vec<&str> = KMU_TEMPLATE.iter().map(|a| a.nummer).collect();
        for a in KMU_TEMPLATE {
            assert!(is_valid_number(a.nummer), "{}", a.nummer);
            assert!(KINDS.iter().any(|(k, _)| *k == a.kontoart), "{}", a.nummer);
            assert!(!a.bezeichnung.is_empty() && a.bezeichnung.len() <= NAME_MAX);
            // Vorschlag aus der ersten Ziffer stimmt – ausser bei bewusst abweichenden Konten
            if let Some(kind) = suggested_kind(a.nummer)
                && a.nummer != "6950"
            {
                assert_eq!(kind, a.kontoart, "{}", a.nummer);
            }
        }
        let count = numbers.len();
        numbers.sort_unstable();
        numbers.dedup();
        assert_eq!(numbers.len(), count, "Nummern doppelt");
        assert!(count >= 80, "Vorlage zu klein: {count}");
    }

    #[test]
    fn template_system_accounts_fit_roles() {
        assert_eq!(KMU_SYSTEM_ACCOUNTS.len(), SYSTEM_ROLES.len());
        for (role, number) in KMU_SYSTEM_ACCOUNTS {
            let role = system_role(role).unwrap();
            let account = KMU_TEMPLATE.iter().find(|a| a.nummer == *number).unwrap();
            assert!(
                system_assignment_error(role, account.kontoart, true).is_none(),
                "{} → {number}",
                role.key
            );
        }
    }
}
