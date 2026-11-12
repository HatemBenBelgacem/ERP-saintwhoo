-- ANF-005: Benutzer verwalten

-- Benutzer – Adresse 1:1: jede Adresse gehört höchstens einem Benutzer (UNIQUE),
-- jeder Benutzer genau einer Adresse (NOT NULL). Löschen einer verwendeten Adresse wird verhindert (AK-19).
CREATE TABLE benutzer (
    id                BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    benutzername      VARCHAR(50)  NOT NULL CHECK (benutzername ~ '^[a-z0-9._-]+$'),
    adresse_id        BIGINT       NOT NULL UNIQUE REFERENCES adresse (id) ON DELETE RESTRICT,
    -- Argon2id-Hash (PHC-Format), nie das Passwort selbst (AK-21)
    passwort_hash     TEXT         NOT NULL,
    aktiv             BOOLEAN      NOT NULL DEFAULT true,
    letzte_anmeldung  TIMESTAMPTZ,
    erstellt_am       TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am      TIMESTAMPTZ  NOT NULL DEFAULT now()
);
-- Benutzername eindeutig ohne Beachtung der Gross-/Kleinschreibung (AK-13)
CREATE UNIQUE INDEX benutzer_benutzername_idx ON benutzer (lower(benutzername));

-- Verantwortung in Projekten (AK-24 bis AK-34): je ein Benutzer bei Projekt, Teilprojekt, Vorgang.
-- Aufgaben erhalten keine eigene Spalte; ihre Verantwortlichen ergeben sich aus der Hierarchie.
ALTER TABLE projekt     ADD COLUMN benutzer_id BIGINT REFERENCES benutzer (id) ON DELETE RESTRICT;
ALTER TABLE teilprojekt ADD COLUMN benutzer_id BIGINT REFERENCES benutzer (id) ON DELETE RESTRICT;
ALTER TABLE vorgang     ADD COLUMN benutzer_id BIGINT REFERENCES benutzer (id) ON DELETE RESTRICT;
CREATE INDEX projekt_benutzer_id_idx     ON projekt (benutzer_id);
CREATE INDEX teilprojekt_benutzer_id_idx ON teilprojekt (benutzer_id);
CREATE INDEX vorgang_benutzer_id_idx     ON vorgang (benutzer_id);
