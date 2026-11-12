-- ANF-002: Adressstamm
CREATE SEQUENCE adresse_nummer_seq;

CREATE TABLE adresse (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nummer        VARCHAR(20)  NOT NULL UNIQUE
                  DEFAULT ('A-' || lpad(nextval('adresse_nummer_seq')::text, 5, '0')),
    typ           VARCHAR(10)  NOT NULL CHECK (typ IN ('firma', 'person')),
    firmenname    VARCHAR(200),
    anrede        VARCHAR(10)  CHECK (anrede IN ('herr', 'frau')),
    vorname       VARCHAR(100),
    nachname      VARCHAR(100),
    zusatz        VARCHAR(200),
    strasse       VARCHAR(200),
    postfach      VARCHAR(50),
    plz           VARCHAR(20),
    ort           VARCHAR(100),
    land          VARCHAR(2)   NOT NULL DEFAULT 'CH',
    email         VARCHAR(254),
    telefon       VARCHAR(50),
    mobiltelefon  VARCHAR(50),
    webseite      VARCHAR(200),
    notiz         TEXT,
    -- Anzeigename: Firma → Firmenname, Person → „Nachname Vorname“
    anzeigename   TEXT GENERATED ALWAYS AS (
                      CASE WHEN typ = 'firma' THEN firmenname
                           ELSE btrim(coalesce(nachname, '') || ' ' || coalesce(vorname, '')) END
                  ) STORED,
    erstellt_am   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am  TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT adresse_firma_pflicht
        CHECK (typ <> 'firma' OR coalesce(btrim(firmenname), '') <> ''),
    CONSTRAINT adresse_person_pflicht
        CHECK (typ <> 'person' OR (coalesce(btrim(vorname), '') <> '' AND coalesce(btrim(nachname), '') <> ''))
);

CREATE INDEX adresse_anzeigename_idx ON adresse (lower(anzeigename));
