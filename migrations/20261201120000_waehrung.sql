-- ANF-014: Währungen und Umrechnungskurse

-- Währung: ISO-Code als Schlüssel, Rundung 0.01 oder 0.05, genau eine Buchhaltungswährung (CHF).
CREATE TABLE waehrung (
    code                  VARCHAR(3)   PRIMARY KEY CHECK (code ~ '^[A-Z]{3}$'),
    bezeichnung           VARCHAR(50)  NOT NULL CHECK (btrim(bezeichnung) <> ''),
    rundung               NUMERIC(4,2) NOT NULL CHECK (rundung IN (0.01, 0.05)),
    buchhaltungswaehrung  BOOLEAN      NOT NULL DEFAULT false,
    aktiv                 BOOLEAN      NOT NULL DEFAULT true,
    erstellt_am           TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK (NOT buchhaltungswaehrung OR aktiv)
);

-- Genau eine Buchhaltungswährung (AK-1)
CREATE UNIQUE INDEX waehrung_buchhaltungswaehrung_idx ON waehrung (buchhaltungswaehrung) WHERE buchhaltungswaehrung;

INSERT INTO waehrung (code, bezeichnung, rundung, buchhaltungswaehrung) VALUES
    ('CHF', 'Schweizer Franken', 0.05, true),
    ('EUR', 'Euro', 0.01, false),
    ('USD', 'US-Dollar', 0.01, false);

-- Kurs: CHF für 1 Einheit der Währung. Je Währung und Datum höchstens ein Kurs (AK-3).
-- Ein Kurs gehört zur Währung: Wird die Währung gelöscht, verschwinden ihre Kurse.
CREATE TABLE waehrungskurs (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    waehrung_code   VARCHAR(3)    NOT NULL REFERENCES waehrung (code) ON DELETE CASCADE,
    datum           DATE          NOT NULL,
    kurs            NUMERIC(14,6) NOT NULL CHECK (kurs > 0),
    erstellt_am     TIMESTAMPTZ   NOT NULL DEFAULT now(),
    UNIQUE (waehrung_code, datum)
);

-- Der Einkauf wählt die Währungen aus dieser Tabelle statt aus einer festen Liste (AK-6).
-- Eine verwendete Währung lässt sich deshalb nicht löschen (AK-5).
ALTER TABLE einkaufsauftrag DROP CONSTRAINT einkaufsauftrag_waehrung_check;
ALTER TABLE einkaufsauftrag
    ADD CONSTRAINT einkaufsauftrag_waehrung_fkey
    FOREIGN KEY (waehrung) REFERENCES waehrung (code) ON DELETE RESTRICT;
