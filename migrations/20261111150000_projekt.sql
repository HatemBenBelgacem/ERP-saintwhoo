-- ANF-003: Projekte mit Teilprojekten, Vorgängen und Aufgaben

-- Zähler für Projektnummern je Jahr (P-JJJJ-NNN)
CREATE TABLE projekt_nummer (
    jahr    INTEGER PRIMARY KEY,
    letzte  INTEGER NOT NULL
);

CREATE TABLE projekt (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nummer        VARCHAR(20)  NOT NULL UNIQUE,
    name          VARCHAR(200) NOT NULL CHECK (btrim(name) <> ''),
    beschreibung  TEXT,
    status        VARCHAR(20)  NOT NULL DEFAULT 'aktiv'
                  CHECK (status IN ('geplant', 'aktiv', 'pausiert', 'abgeschlossen', 'abgebrochen')),
    start         DATE,
    ende          DATE,
    -- Projekt – Adresse N:1; Löschen einer verwendeten Adresse wird verhindert (ANF-002 AK-16)
    adresse_id    BIGINT REFERENCES adresse (id) ON DELETE RESTRICT,
    erstellt_am   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CONSTRAINT projekt_ende_nach_start CHECK (ende IS NULL OR start IS NULL OR ende >= start)
);
CREATE INDEX projekt_adresse_id_idx ON projekt (adresse_id);
CREATE INDEX projekt_status_idx ON projekt (status);

CREATE TABLE teilprojekt (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    projekt_id    BIGINT       NOT NULL REFERENCES projekt (id) ON DELETE CASCADE,
    reihenfolge   INTEGER      NOT NULL,
    name          VARCHAR(200) NOT NULL CHECK (btrim(name) <> ''),
    beschreibung  TEXT,
    status        VARCHAR(20)  NOT NULL DEFAULT 'aktiv'
                  CHECK (status IN ('geplant', 'aktiv', 'pausiert', 'abgeschlossen', 'abgebrochen')),
    start         DATE,
    ende          DATE,
    erstellt_am   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CONSTRAINT teilprojekt_ende_nach_start CHECK (ende IS NULL OR start IS NULL OR ende >= start)
);
CREATE INDEX teilprojekt_projekt_id_idx ON teilprojekt (projekt_id, reihenfolge);

CREATE TABLE vorgang (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    teilprojekt_id  BIGINT       NOT NULL REFERENCES teilprojekt (id) ON DELETE CASCADE,
    reihenfolge     INTEGER      NOT NULL,
    name            VARCHAR(200) NOT NULL CHECK (btrim(name) <> ''),
    beschreibung    TEXT,
    status          VARCHAR(20)  NOT NULL DEFAULT 'aktiv'
                    CHECK (status IN ('geplant', 'aktiv', 'pausiert', 'abgeschlossen', 'abgebrochen')),
    start           DATE,
    ende            DATE,
    erstellt_am     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am    TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CONSTRAINT vorgang_ende_nach_start CHECK (ende IS NULL OR start IS NULL OR ende >= start)
);
CREATE INDEX vorgang_teilprojekt_id_idx ON vorgang (teilprojekt_id, reihenfolge);

CREATE TABLE aufgabe (
    id               BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    vorgang_id       BIGINT       NOT NULL REFERENCES vorgang (id) ON DELETE CASCADE,
    reihenfolge      INTEGER      NOT NULL,
    name             VARCHAR(200) NOT NULL CHECK (btrim(name) <> ''),
    beschreibung     TEXT,
    status           VARCHAR(20)  NOT NULL DEFAULT 'offen'
                     CHECK (status IN ('offen', 'in_arbeit', 'erledigt')),
    start            DATE,
    ende             DATE,
    prioritaet       VARCHAR(10)  NOT NULL DEFAULT 'normal'
                     CHECK (prioritaet IN ('niedrig', 'normal', 'hoch')),
    aufwand_stunden  NUMERIC(8, 2) CHECK (aufwand_stunden >= 0),
    erstellt_am      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am     TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CONSTRAINT aufgabe_ende_nach_start CHECK (ende IS NULL OR start IS NULL OR ende >= start)
);
CREATE INDEX aufgabe_vorgang_id_idx ON aufgabe (vorgang_id, reihenfolge);
