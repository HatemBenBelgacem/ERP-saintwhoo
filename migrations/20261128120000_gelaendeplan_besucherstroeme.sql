-- ANF-028: Besucherströme im Geländeplan – Besucherwege, Zugänge und Zonen

-- Besucherweg: Linienzug mit Richtung und erwarteter Menge
CREATE TABLE gelaendeplan_besucherweg (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gelaendeplan_id       BIGINT           NOT NULL REFERENCES gelaendeplan (id) ON DELETE CASCADE,
    reihenfolge           INTEGER          NOT NULL,
    -- leer = „Weg <Nummer>“
    bezeichnung           VARCHAR(100),
    richtung              VARCHAR(10)      NOT NULL CHECK (richtung IN ('einbahn', 'beide')),
    -- 0 = unbekannt
    personen_pro_stunde   INTEGER          NOT NULL DEFAULT 0 CHECK (personen_pro_stunde BETWEEN 0 AND 1000000),
    breite_m              DOUBLE PRECISION CHECK (breite_m BETWEEN 0.5 AND 100),
    UNIQUE (gelaendeplan_id, reihenfolge)
);

CREATE TABLE gelaendeplan_besucherweg_punkt (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    besucherweg_id        BIGINT           NOT NULL REFERENCES gelaendeplan_besucherweg (id) ON DELETE CASCADE,
    reihenfolge           INTEGER          NOT NULL,
    x_m                   DOUBLE PRECISION NOT NULL,
    z_m                   DOUBLE PRECISION NOT NULL,
    UNIQUE (besucherweg_id, reihenfolge)
);

-- Zugang: Eingang, Ausgang oder Notausgang
CREATE TABLE gelaendeplan_zugang (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gelaendeplan_id       BIGINT           NOT NULL REFERENCES gelaendeplan (id) ON DELETE CASCADE,
    reihenfolge           INTEGER          NOT NULL,
    art                   VARCHAR(20)      NOT NULL CHECK (art IN ('eingang', 'ausgang', 'notausgang')),
    -- leer = Art mit Nummer, z. B. „Eingang 1“
    bezeichnung           VARCHAR(100),
    x_m                   DOUBLE PRECISION NOT NULL,
    z_m                   DOUBLE PRECISION NOT NULL,
    personen_pro_minute   INTEGER          CHECK (personen_pro_minute BETWEEN 0 AND 100000),
    UNIQUE (gelaendeplan_id, reihenfolge)
);

-- Zone: Aufenthaltsfläche mit erwarteten Personen und Personendichte (Fassungsvermögen = Fläche × Dichte)
CREATE TABLE gelaendeplan_zone (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gelaendeplan_id       BIGINT           NOT NULL REFERENCES gelaendeplan (id) ON DELETE CASCADE,
    reihenfolge           INTEGER          NOT NULL,
    -- leer = „Zone <Nummer>“
    bezeichnung           VARCHAR(100),
    -- 0 = unbekannt
    erwartete_personen    INTEGER          NOT NULL DEFAULT 0 CHECK (erwartete_personen BETWEEN 0 AND 1000000),
    -- Personen pro m²
    dichte                DOUBLE PRECISION NOT NULL DEFAULT 2 CHECK (dichte BETWEEN 0.1 AND 10),
    UNIQUE (gelaendeplan_id, reihenfolge)
);

CREATE TABLE gelaendeplan_zone_punkt (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    zone_id               BIGINT           NOT NULL REFERENCES gelaendeplan_zone (id) ON DELETE CASCADE,
    reihenfolge           INTEGER          NOT NULL,
    x_m                   DOUBLE PRECISION NOT NULL,
    z_m                   DOUBLE PRECISION NOT NULL,
    UNIQUE (zone_id, reihenfolge)
);
