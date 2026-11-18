-- ANF-011: Kontenplan und Systemkonten

-- Konten des Kontenplans (Schweizer KMU-Kontenrahmen). Die Kontenklasse ist die erste Ziffer der Nummer.
-- Nummer als Text: so ordnet die Sortierung Unterkonten (z. B. 10200) direkt hinter ihr Hauptkonto (1020).
CREATE TABLE konto (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nummer        VARCHAR(6)   NOT NULL UNIQUE CHECK (nummer ~ '^[1-9][0-9]{3,5}$'),
    bezeichnung   VARCHAR(100) NOT NULL CHECK (btrim(bezeichnung) <> ''),
    kontoart      VARCHAR(10)  NOT NULL CHECK (kontoart IN ('aktiv', 'passiv', 'aufwand', 'ertrag')),
    kontengruppe  VARCHAR(100),
    aktiv         BOOLEAN      NOT NULL DEFAULT true,
    bemerkung     TEXT,
    erstellt_am   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am  TIMESTAMPTZ  NOT NULL DEFAULT now()
);

-- Systemkonten (AK-11): je Rolle genau ein Konto. Ein Konto, das als Systemkonto dient,
-- kann nicht gelöscht werden (AK-9). Die erlaubten Rollen und Kontoarten prüft `model.rs`.
CREATE TABLE systemkonto (
    rolle     VARCHAR(30) PRIMARY KEY CHECK (rolle IN (
                  'debitoren', 'kreditoren', 'vorsteuer', 'umsatzsteuer', 'mwst_abrechnung', 'bank',
                  'kursdifferenzen', 'skonto_gewaehrt', 'skonto_erhalten', 'rundungsdifferenzen',
                  'gewinnvortrag', 'eroeffnungsbilanz', 'jahresergebnis')),
    konto_id  BIGINT      NOT NULL REFERENCES konto (id) ON DELETE RESTRICT
);
CREATE INDEX systemkonto_konto_id_idx ON systemkonto (konto_id);
