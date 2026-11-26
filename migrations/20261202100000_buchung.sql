-- ANF-017: Buchungsjournal (Hauptbuch)

-- Zähler der lückenlosen Journalnummer je Geschäftsjahr (AK-9). Die Zeile wird in der Transaktion
-- des Buchens gesperrt; bricht die Transaktion ab, gilt auch der Zähler wieder als vorher.
CREATE TABLE journal_nummer (
    geschaeftsjahr_id  BIGINT  PRIMARY KEY REFERENCES geschaeftsjahr (id) ON DELETE RESTRICT,
    letzte             INTEGER NOT NULL CHECK (letzte >= 0)
);

-- Buchung (Kopf): Datum, Beleg, Text, Status und Herkunft. Entwürfe haben keine Journalnummer.
CREATE TABLE buchung (
    id                  BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- aus dem Datum ermittelt; ein Geschäftsjahr mit Buchungen lässt sich nicht löschen (ANF-013 AK-7)
    geschaeftsjahr_id   BIGINT       NOT NULL REFERENCES geschaeftsjahr (id) ON DELETE RESTRICT,
    journalnummer       INTEGER,
    datum               DATE         NOT NULL,
    belegnummer         VARCHAR(30)  NOT NULL CHECK (btrim(belegnummer) <> ''),
    text                VARCHAR(200) NOT NULL CHECK (btrim(text) <> ''),
    status              VARCHAR(20)  NOT NULL DEFAULT 'entwurf'
                        CHECK (status IN ('entwurf', 'gebucht', 'storniert')),
    herkunft            VARCHAR(20)  NOT NULL DEFAULT 'manuell'
                        CHECK (herkunft IN ('manuell', 'debitor', 'kreditor', 'zahlung', 'mwst', 'abschluss', 'eroeffnung')),
    -- Gegenbuchung ↔ Original (AK-11): die Gegenbuchung nennt das stornierte Original
    storno_von_id       BIGINT       REFERENCES buchung (id) ON DELETE RESTRICT,
    -- erfassender Benutzer (ANF-005); bleibt beim Löschen des Benutzers erhalten (ohne Verweis)
    erfasst_von         BIGINT       REFERENCES benutzer (id) ON DELETE SET NULL,
    erstellt_am         TIMESTAMPTZ  NOT NULL DEFAULT now(),
    gebucht_am          TIMESTAMPTZ,
    geaendert_am        TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK ((status = 'entwurf') = (journalnummer IS NULL)),
    CHECK ((status = 'entwurf') = (gebucht_am IS NULL)),
    CHECK (storno_von_id IS NULL OR herkunft = 'manuell'),
    UNIQUE (geschaeftsjahr_id, journalnummer)
);
-- Ein Original kann nur einmal storniert werden (AK-11)
CREATE UNIQUE INDEX buchung_storno_von_id_idx ON buchung (storno_von_id) WHERE storno_von_id IS NOT NULL;
CREATE INDEX buchung_geschaeftsjahr_id_idx ON buchung (geschaeftsjahr_id);
CREATE INDEX buchung_datum_idx ON buchung (datum);
CREATE INDEX buchung_status_idx ON buchung (status);
CREATE INDEX buchung_erfasst_von_idx ON buchung (erfasst_von);

-- Buchungszeile: genau eines von Soll/Haben ist größer als null. Eine Zeile mit MWST-Code trägt
-- die Nettobasis (`mwst_basis`); die automatisch erzeugte Steuerzeile hat `ist_mwst_zeile`.
CREATE TABLE buchungszeile (
    id              BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    buchung_id      BIGINT        NOT NULL REFERENCES buchung (id) ON DELETE CASCADE,
    reihenfolge     INTEGER       NOT NULL,
    konto_id        BIGINT        NOT NULL REFERENCES konto (id) ON DELETE RESTRICT,
    soll            NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (soll >= 0),
    haben           NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (haben >= 0),
    CHECK ((soll > 0) <> (haben > 0)),
    mwst_code_id    BIGINT        REFERENCES mwst_code (id) ON DELETE RESTRICT,
    mwst_basis      NUMERIC(14,2) CHECK (mwst_basis IS NULL OR mwst_basis >= 0),
    ist_mwst_zeile  BOOLEAN       NOT NULL DEFAULT false,
    -- optionales Projekt (ANF-003); ein verwendetes Projekt lässt sich nicht löschen (ANF-010 AK-9)
    projekt_id      BIGINT        REFERENCES projekt (id) ON DELETE RESTRICT,
    text            VARCHAR(200),
    UNIQUE (buchung_id, reihenfolge),
    CHECK (NOT ist_mwst_zeile OR mwst_code_id IS NULL)
);
CREATE INDEX buchungszeile_konto_id_idx ON buchungszeile (konto_id);
CREATE INDEX buchungszeile_projekt_id_idx ON buchungszeile (projekt_id);
CREATE INDEX buchungszeile_mwst_code_id_idx ON buchungszeile (mwst_code_id);

-- Unveränderlichkeit (ANF-010 AK-7): Eine gebuchte Buchung ändert nur ihren Status von
-- „gebucht“ auf „storniert“. Gelöscht wird nur ein Entwurf.
CREATE FUNCTION buchung_kopf_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status <> 'entwurf' THEN
            RAISE EXCEPTION 'Gebuchte Buchungen können nicht gelöscht werden.';
        END IF;
        RETURN OLD;
    END IF;
    IF OLD.status <> 'entwurf' AND (
        NEW.datum <> OLD.datum
        OR NEW.belegnummer <> OLD.belegnummer
        OR NEW.text <> OLD.text
        OR NEW.geschaeftsjahr_id <> OLD.geschaeftsjahr_id
        OR NEW.journalnummer IS DISTINCT FROM OLD.journalnummer
        OR NEW.herkunft <> OLD.herkunft
        OR NEW.storno_von_id IS DISTINCT FROM OLD.storno_von_id
        OR NEW.gebucht_am IS DISTINCT FROM OLD.gebucht_am
        OR (NEW.status <> OLD.status AND NOT (OLD.status = 'gebucht' AND NEW.status = 'storniert'))
    ) THEN
        RAISE EXCEPTION 'Gebuchte Buchungen können nicht geändert werden.';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER buchung_kopf
    BEFORE UPDATE OR DELETE ON buchung
    FOR EACH ROW EXECUTE FUNCTION buchung_kopf_pruefen();

-- Zeilen gehören zu einem Entwurf; an gebuchten Buchungen lässt sich nichts ändern.
CREATE FUNCTION buchungszeile_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    kopf_status TEXT;
BEGIN
    SELECT status INTO kopf_status FROM buchung WHERE id = COALESCE(NEW.buchung_id, OLD.buchung_id);
    IF kopf_status IS NOT NULL AND kopf_status <> 'entwurf' THEN
        RAISE EXCEPTION 'Zeilen gebuchter Buchungen können nicht geändert werden.';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER buchungszeile_kontrolle
    BEFORE INSERT OR UPDATE OR DELETE ON buchungszeile
    FOR EACH ROW EXECUTE FUNCTION buchungszeile_pruefen();

-- Ausgleich (AK-15): Am Ende der Transaktion muss jede gebuchte Buchung ausgeglichen sein,
-- also Summe Soll = Summe Haben, und mindestens zwei Zeilen haben.
CREATE FUNCTION buchung_ausgeglichen_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    kopf_id     BIGINT := COALESCE(NEW.buchung_id, OLD.buchung_id);
    kopf_status TEXT;
    summe_soll  NUMERIC(14,2);
    summe_haben NUMERIC(14,2);
    anzahl      INTEGER;
BEGIN
    SELECT status INTO kopf_status FROM buchung WHERE id = kopf_id;
    IF kopf_status = 'gebucht' THEN
        SELECT COALESCE(SUM(soll), 0), COALESCE(SUM(haben), 0), COUNT(*)
          INTO summe_soll, summe_haben, anzahl
          FROM buchungszeile WHERE buchung_id = kopf_id;
        IF summe_soll <> summe_haben THEN
            RAISE EXCEPTION 'Buchung ist nicht ausgeglichen: Soll % ungleich Haben %', summe_soll, summe_haben;
        END IF;
        IF anzahl < 2 THEN
            RAISE EXCEPTION 'Eine gebuchte Buchung braucht mindestens zwei Zeilen.';
        END IF;
    END IF;
    RETURN NULL;
END $$;

CREATE CONSTRAINT TRIGGER buchungszeile_ausgeglichen
    AFTER INSERT OR UPDATE OR DELETE ON buchungszeile
    DEFERRABLE INITIALLY DEFERRED
    FOR EACH ROW EXECUTE FUNCTION buchung_ausgeglichen_pruefen();
