-- ANF-012: MWST-Codes (Steuersätze) und MWST-Einstellungen

-- MWST-Code: Steuerart, Satz, Gültigkeit und Konto. Satz in Prozent mit zwei Nachkommastellen.
-- Ein Code, der verwendet wird, behält Satz und Art (AK-5); gelöscht wird er nur, wenn er unbenutzt ist (AK-7).
CREATE TABLE mwst_code (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    code            VARCHAR(10)  NOT NULL UNIQUE CHECK (code ~ '^[A-Z0-9_-]{1,10}$'),
    bezeichnung     VARCHAR(100) NOT NULL CHECK (btrim(bezeichnung) <> ''),
    art             VARCHAR(20)  NOT NULL CHECK (art IN ('umsatzsteuer', 'vorsteuer')),
    satz            NUMERIC(5,2) NOT NULL CHECK (satz >= 0 AND satz <= 100),
    gueltig_ab      DATE         NOT NULL,
    gueltig_bis     DATE,
    -- MWST-Konto; Löschen eines verwendeten Kontos wird verhindert (ANF-011)
    konto_id        BIGINT       NOT NULL REFERENCES konto (id) ON DELETE RESTRICT,
    formularziffer  VARCHAR(10),
    aktiv           BOOLEAN      NOT NULL DEFAULT true,
    erstellt_am     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK (gueltig_bis IS NULL OR gueltig_bis >= gueltig_ab)
);
CREATE INDEX mwst_code_konto_id_idx ON mwst_code (konto_id);

-- MWST-Einstellungen: genau eine Zeile (Spalte `einzig` ist immer true und eindeutig).
-- Die Pflicht wird ab einem Beginndatum eingeschaltet; die UID der Firma prüft der Server (AK-8).
CREATE TABLE finanzeinstellung (
    einzig              BOOLEAN       PRIMARY KEY DEFAULT true CHECK (einzig),
    mwst_pflichtig      BOOLEAN       NOT NULL DEFAULT false,
    mwst_pflichtig_ab   DATE,
    -- Umsatzgrenze in CHF (AK-11), Vorgabe 100 000
    umsatzgrenze        NUMERIC(14,2) NOT NULL DEFAULT 100000.00 CHECK (umsatzgrenze > 0),
    methode             VARCHAR(20)   NOT NULL DEFAULT 'effektiv'
                        CHECK (methode IN ('effektiv', 'saldosteuersatz')),
    abrechnungsart      VARCHAR(20)   NOT NULL DEFAULT 'vereinbart'
                        CHECK (abrechnungsart IN ('vereinbart', 'vereinnahmt')),
    periodizitaet       VARCHAR(20)   NOT NULL DEFAULT 'quartal'
                        CHECK (periodizitaet IN ('quartal', 'halbjahr')),
    CHECK (NOT mwst_pflichtig OR mwst_pflichtig_ab IS NOT NULL)
);
INSERT INTO finanzeinstellung (einzig) VALUES (true);
