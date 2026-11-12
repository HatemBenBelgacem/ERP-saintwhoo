-- ANF-007: Anmeldung

-- Sitzungen: im Browser nur ein zufälliges Merkmal im Cookie, hier nur dessen SHA-256-Hash (AK-13).
-- Gelöschte Benutzer verlieren ihre Sitzungen automatisch (AK-15).
CREATE TABLE sitzung (
    id             BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    merkmal_hash   TEXT        NOT NULL UNIQUE,
    benutzer_id    BIGINT      NOT NULL REFERENCES benutzer (id) ON DELETE CASCADE,
    erstellt_am    TIMESTAMPTZ NOT NULL DEFAULT now(),
    zuletzt_aktiv  TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX sitzung_benutzer_id_idx ON sitzung (benutzer_id);

-- Fehlgeschlagene Anmeldungen je Benutzername (auch unbekannte Namen) für die Sperre (AK-11).
CREATE TABLE anmeldeversuch (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    benutzername  VARCHAR(50) NOT NULL,
    zeitpunkt     TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX anmeldeversuch_benutzername_idx ON anmeldeversuch (benutzername, zeitpunkt);
