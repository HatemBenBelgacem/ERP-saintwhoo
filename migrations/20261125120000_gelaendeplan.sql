-- ANF-027: Geländeplan eines Projekts (3D-Karte mit Bühnen, Verpflegungsständen, Festzelten und Rasenfläche)

-- Höchstens ein Geländeplan je Projekt. Der Ursprung ist ein Punkt auf der Karte (WGS84);
-- Elemente und Eckpunkte stehen in Metern relativ dazu (x nach Osten, z nach Süden).
CREATE TABLE gelaendeplan (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    projekt_id        BIGINT           NOT NULL UNIQUE REFERENCES projekt (id) ON DELETE CASCADE,
    ursprung_laenge   DOUBLE PRECISION NOT NULL CHECK (ursprung_laenge BETWEEN -180 AND 180),
    ursprung_breite   DOUBLE PRECISION NOT NULL CHECK (ursprung_breite BETWEEN -85 AND 85),
    erstellt_am       TIMESTAMPTZ      NOT NULL DEFAULT now(),
    geaendert_am      TIMESTAMPTZ      NOT NULL DEFAULT now()
);

-- Platzierte Elemente (Grundfläche als Rechteck, um den Mittelpunkt gedreht)
CREATE TABLE gelaendeplan_element (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gelaendeplan_id   BIGINT           NOT NULL REFERENCES gelaendeplan (id) ON DELETE CASCADE,
    reihenfolge       INTEGER          NOT NULL,
    art               VARCHAR(30)      NOT NULL
                      CHECK (art IN ('buehne', 'verpflegungsstand', 'festzelt')),
    x_m               DOUBLE PRECISION NOT NULL,
    z_m               DOUBLE PRECISION NOT NULL,
    drehung_grad      DOUBLE PRECISION NOT NULL DEFAULT 0 CHECK (drehung_grad >= 0 AND drehung_grad < 360),
    breite_m          DOUBLE PRECISION NOT NULL CHECK (breite_m > 0),
    tiefe_m           DOUBLE PRECISION NOT NULL CHECK (tiefe_m > 0),
    UNIQUE (gelaendeplan_id, reihenfolge)
);

-- Eckpunkte der abgesteckten Rasenfläche in Reihenfolge (leer = keine Fläche)
CREATE TABLE gelaendeplan_eckpunkt (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    gelaendeplan_id   BIGINT           NOT NULL REFERENCES gelaendeplan (id) ON DELETE CASCADE,
    reihenfolge       INTEGER          NOT NULL,
    x_m               DOUBLE PRECISION NOT NULL,
    z_m               DOUBLE PRECISION NOT NULL,
    UNIQUE (gelaendeplan_id, reihenfolge)
);
