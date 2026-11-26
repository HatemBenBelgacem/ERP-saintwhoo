-- ANF-016: Zahlungsbedingungen und Vorgabe je Adresse

-- Zahlungsbedingung: Fälligkeit (Tage), optionaler Skonto und der Text für die Rechnung.
-- Skontofrist und Skonto % sind entweder beide leer oder beide gesetzt (AK-2); die Prüfung dazu steht hier und in `model.rs`.
CREATE TABLE zahlungsbedingung (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    bezeichnung         VARCHAR(100) NOT NULL CHECK (btrim(bezeichnung) <> ''),
    zahlungsfrist_tage  INTEGER      NOT NULL CHECK (zahlungsfrist_tage BETWEEN 0 AND 365),
    skontofrist_tage    INTEGER      CHECK (skontofrist_tage >= 0),
    skonto_prozent      NUMERIC(5,2) CHECK (skonto_prozent > 0 AND skonto_prozent < 100),
    rechnungstext       VARCHAR(250) NOT NULL CHECK (btrim(rechnungstext) <> ''),
    standard            BOOLEAN      NOT NULL DEFAULT false,
    aktiv               BOOLEAN      NOT NULL DEFAULT true,
    erstellt_am         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK ((skontofrist_tage IS NULL) = (skonto_prozent IS NULL)),
    CHECK (skontofrist_tage IS NULL OR skontofrist_tage < zahlungsfrist_tage),
    CHECK (NOT standard OR aktiv)
);

-- Genau eine Standard-Zahlungsbedingung (AK-4)
CREATE UNIQUE INDEX zahlungsbedingung_standard_idx ON zahlungsbedingung (standard) WHERE standard;

-- Startwert: „30 Tage netto“ als Standard (AK-8)
INSERT INTO zahlungsbedingung (bezeichnung, zahlungsfrist_tage, rechnungstext, standard)
VALUES ('30 Tage netto', 30, 'Zahlbar innert 30 Tagen netto', true);

-- Vorgabe-Zahlungsbedingung je Adresse (AK-5). Eine verwendete Bedingung lässt sich nicht löschen (AK-7).
ALTER TABLE adresse
    ADD COLUMN zahlungsbedingung_id BIGINT REFERENCES zahlungsbedingung (id) ON DELETE RESTRICT;
CREATE INDEX adresse_zahlungsbedingung_id_idx ON adresse (zahlungsbedingung_id);
