-- ANF-013: Geschäftsjahre und Perioden

-- Geschäftsjahr: Beginn am 1. eines Monats, Ende am letzten Tag eines Monats (Perioden sind Kalendermonate).
-- Die Ausschlussbedingung verhindert überlappende Geschäftsjahre (AK-3).
CREATE TABLE geschaeftsjahr (
    id                 BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    bezeichnung        VARCHAR(20)  NOT NULL CHECK (btrim(bezeichnung) <> ''),
    beginn             DATE         NOT NULL,
    ende               DATE         NOT NULL,
    status             VARCHAR(20)  NOT NULL DEFAULT 'offen' CHECK (status IN ('offen', 'abgeschlossen')),
    abgeschlossen_am   TIMESTAMPTZ,
    abgeschlossen_von  BIGINT       REFERENCES benutzer (id) ON DELETE SET NULL,
    erstellt_am        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am       TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK (ende > beginn),
    CHECK ((status = 'abgeschlossen') = (abgeschlossen_am IS NOT NULL)),
    EXCLUDE USING gist (daterange(beginn, ende, '[]') WITH &&)
);

-- Periode: ein Kalendermonat eines Geschäftsjahrs. Wird mit dem Geschäftsjahr gelöscht.
CREATE TABLE periode (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    geschaeftsjahr_id   BIGINT       NOT NULL REFERENCES geschaeftsjahr (id) ON DELETE CASCADE,
    beginn              DATE         NOT NULL,
    ende                DATE         NOT NULL,
    status              VARCHAR(20)  NOT NULL DEFAULT 'offen' CHECK (status IN ('offen', 'gesperrt')),
    -- Vermerk der letzten Sperrung bzw. Entsperrung (AK-5); bleibt beim Löschen des Benutzers erhalten
    geaendert_von       BIGINT       REFERENCES benutzer (id) ON DELETE SET NULL,
    geaendert_am        TIMESTAMPTZ,
    CHECK (ende > beginn),
    UNIQUE (geschaeftsjahr_id, beginn)
);
