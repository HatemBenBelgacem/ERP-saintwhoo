-- ANF-004: Einkaufsaufträge mit Positionen

-- Zähler für Einkaufsnummern je Jahr (E-JJJJ-NNNN)
CREATE TABLE einkaufsauftrag_nummer (
    jahr    INTEGER PRIMARY KEY,
    letzte  INTEGER NOT NULL
);

CREATE TABLE einkaufsauftrag (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nummer              VARCHAR(20)  NOT NULL UNIQUE,
    -- Lieferant: Einkaufsauftrag – Adresse N:1; Löschen einer verwendeten Adresse wird verhindert (AK-21)
    adresse_id          BIGINT       NOT NULL REFERENCES adresse (id) ON DELETE RESTRICT,
    -- optionales Projekt; Löschen eines verwendeten Projekts wird verhindert (AK-23)
    projekt_id          BIGINT       REFERENCES projekt (id) ON DELETE RESTRICT,
    status              VARCHAR(20)  NOT NULL DEFAULT 'entwurf'
                        CHECK (status IN ('entwurf', 'bestellt', 'teilweise_geliefert', 'geliefert', 'storniert')),
    bestelldatum        DATE,
    liefertermin        DATE,
    referenz_lieferant  VARCHAR(100),
    waehrung            VARCHAR(3)   NOT NULL DEFAULT 'CHF' CHECK (waehrung IN ('CHF', 'EUR', 'USD')),
    lieferadresse       TEXT,
    bemerkung           TEXT,
    erstellt_am         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CONSTRAINT einkaufsauftrag_liefertermin_nach_bestelldatum
        CHECK (liefertermin IS NULL OR bestelldatum IS NULL OR liefertermin >= bestelldatum)
);
CREATE INDEX einkaufsauftrag_adresse_id_idx ON einkaufsauftrag (adresse_id);
CREATE INDEX einkaufsauftrag_projekt_id_idx ON einkaufsauftrag (projekt_id);
CREATE INDEX einkaufsauftrag_status_idx ON einkaufsauftrag (status);

-- Beträge als Dezimalzahl (keine Gleitkommazahl, AK-25)
CREATE TABLE einkaufsposition (
    id                       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    einkaufsauftrag_id       BIGINT        NOT NULL REFERENCES einkaufsauftrag (id) ON DELETE CASCADE,
    reihenfolge              INTEGER       NOT NULL,
    bezeichnung              VARCHAR(200)  NOT NULL CHECK (btrim(bezeichnung) <> ''),
    artikelnummer_lieferant  VARCHAR(50),
    menge                    NUMERIC(12,3) NOT NULL CHECK (menge > 0),
    einheit                  VARCHAR(10)   NOT NULL DEFAULT 'stk'
                             CHECK (einheit IN ('stk', 'h', 'kg', 'm', 'm2', 'm3', 'l', 'pauschal')),
    einzelpreis              NUMERIC(12,2) NOT NULL CHECK (einzelpreis >= 0),
    rabatt_prozent           NUMERIC(5,2)  NOT NULL DEFAULT 0 CHECK (rabatt_prozent BETWEEN 0 AND 100),
    mwst_satz                NUMERIC(5,2)  NOT NULL DEFAULT 8.1 CHECK (mwst_satz BETWEEN 0 AND 100),
    gelieferte_menge         NUMERIC(12,3) NOT NULL DEFAULT 0 CHECK (gelieferte_menge >= 0),
    erstellt_am              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    geaendert_am             TIMESTAMPTZ   NOT NULL DEFAULT now()
);
CREATE INDEX einkaufsposition_auftrag_idx ON einkaufsposition (einkaufsauftrag_id, reihenfolge);
