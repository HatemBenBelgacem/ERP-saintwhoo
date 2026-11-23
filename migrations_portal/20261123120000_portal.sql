-- ANF-026: Portal-Datenbank (Registrierung, Anmeldung, Verzeichnis der Mandanten)
-- Diese Migrationen laufen nur in der Portal-Datenbank, nie in einer Mandanten-Datenbank (AK-7).
-- Hier stehen keine ERP-Daten (AK-8).

-- Zugänge: wer sich registriert, ist Administrator (AK-4)
CREATE TABLE portal_benutzer (
    id                 BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    vorname            VARCHAR(100) NOT NULL CHECK (btrim(vorname) <> ''),
    nachname           VARCHAR(100) NOT NULL CHECK (btrim(nachname) <> ''),
    -- Anmeldename (AK-2)
    email              VARCHAR(254) NOT NULL CHECK (position('@' IN email) > 1),
    -- Argon2id-Hash (PHC-Format) wie ANF-007, nie das Passwort selbst (AK-5)
    passwort_hash      TEXT         NOT NULL,
    ist_administrator  BOOLEAN      NOT NULL DEFAULT true,
    aktiv              BOOLEAN      NOT NULL DEFAULT true,
    letzte_anmeldung   TIMESTAMPTZ,
    erstellt_am        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am       TIMESTAMPTZ  NOT NULL DEFAULT now()
);
-- E-Mail eindeutig ohne Beachtung der Gross-/Kleinschreibung (AK-3)
CREATE UNIQUE INDEX portal_benutzer_email_idx ON portal_benutzer (lower(email));

-- Verzeichnis der Mandanten-Datenbanken.
-- Die Grenze „eine Datenbank pro Administrator“ ist eine Prüfregel im Modell (AK-14), kein UNIQUE,
-- damit sie an einer Stelle geändert werden kann.
CREATE TABLE mandant (
    id             BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    besitzer_id    BIGINT       NOT NULL REFERENCES portal_benutzer (id) ON DELETE RESTRICT,
    -- Vom Server erzeugt, nie aus einer Eingabe (AK-12): uq_m_000001, uq_m_000002 …
    datenbankname  VARCHAR(63)  NOT NULL UNIQUE
                   GENERATED ALWAYS AS ('uq_m_' || lpad(id::text, greatest(6, length(id::text)), '0')) STORED,
    -- Leer, bis der Mandant im Modal-Fenster definiert ist (AK-18)
    firmenname     VARCHAR(200) CHECK (firmenname IS NULL OR btrim(firmenname) <> ''),
    status         VARCHAR(20)  NOT NULL DEFAULT 'wird_erstellt'
                   CHECK (status IN ('wird_erstellt', 'bereit', 'fehler')),
    erstellt_am    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am   TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX mandant_besitzer_id_idx ON mandant (besitzer_id);

-- Sitzungen wie ANF-007: im Browser nur ein zufälliges Merkmal, hier nur dessen SHA-256-Hash.
-- mandant_id = gewählter Mandant; leer, solange keiner gewählt bzw. erstellt ist.
CREATE TABLE portal_sitzung (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    merkmal_hash        TEXT        NOT NULL UNIQUE,
    portal_benutzer_id  BIGINT      NOT NULL REFERENCES portal_benutzer (id) ON DELETE CASCADE,
    mandant_id          BIGINT      REFERENCES mandant (id) ON DELETE SET NULL,
    erstellt_am         TIMESTAMPTZ NOT NULL DEFAULT now(),
    zuletzt_aktiv       TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX portal_sitzung_portal_benutzer_id_idx ON portal_sitzung (portal_benutzer_id);

-- Fehlgeschlagene Anmeldungen je E-Mail (auch unbekannte) für die Sperre wie ANF-007 (AK-5)
CREATE TABLE portal_anmeldeversuch (
    id         BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    email      VARCHAR(254) NOT NULL,
    zeitpunkt  TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX portal_anmeldeversuch_email_idx ON portal_anmeldeversuch (lower(email), zeitpunkt);
