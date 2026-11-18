-- ANF-025: Firmenverwaltung (eigene Firma)

-- Höchstens eine Firma (Spalte `einzig` ist immer true und eindeutig).
-- Firma – Adresse 1:1; Name, Anschrift und Kontakt stehen nur in der Adresse.
-- Löschen der Adresse wird verhindert (AK-11).
CREATE TABLE firma (
    id               BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    einzig           BOOLEAN      NOT NULL DEFAULT true UNIQUE CHECK (einzig),
    adresse_id       BIGINT       NOT NULL UNIQUE REFERENCES adresse (id) ON DELETE RESTRICT,
    uid              VARCHAR(15)  CHECK (uid ~ '^CHE-[0-9]{3}\.[0-9]{3}\.[0-9]{3}$'),
    handelsregister  VARCHAR(100),
    fusszeile        VARCHAR(300),
    -- Logo bis 1 MB (AK-7)
    logo             BYTEA        CHECK (octet_length(logo) <= 1048576),
    logo_typ         VARCHAR(50)  CHECK (logo_typ IN ('image/png', 'image/jpeg', 'image/svg+xml')),
    erstellt_am      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK ((logo IS NULL) = (logo_typ IS NULL))
);
