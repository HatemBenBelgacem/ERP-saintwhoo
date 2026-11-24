-- ANF-027: Beschriftete Messungen (Strecken und Flächen) eines Geländeplans

CREATE TABLE gelaendeplan_messung (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gelaendeplan_id   BIGINT       NOT NULL REFERENCES gelaendeplan (id) ON DELETE CASCADE,
    reihenfolge       INTEGER      NOT NULL,
    art               VARCHAR(20)  NOT NULL CHECK (art IN ('strecke', 'flaeche')),
    -- leer = „Messung <Nummer>“
    bezeichnung       VARCHAR(100),
    UNIQUE (gelaendeplan_id, reihenfolge)
);

-- Punkte einer Messung in Reihenfolge, in Metern relativ zum Standort (wie die Eckpunkte)
CREATE TABLE gelaendeplan_messpunkt (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    messung_id        BIGINT           NOT NULL REFERENCES gelaendeplan_messung (id) ON DELETE CASCADE,
    reihenfolge       INTEGER          NOT NULL,
    x_m               DOUBLE PRECISION NOT NULL,
    z_m               DOUBLE PRECISION NOT NULL,
    UNIQUE (messung_id, reihenfolge)
);
