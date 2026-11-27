-- ANF-019: Kreditoren – Eingangsrechnungen und Gutschriften von Lieferanten mit Positionen

-- Systemkonto „Aufwand“ (Vorgabe für das Aufwandskonto übernommener Positionen, ANF-019 AK-8)
ALTER TABLE systemkonto DROP CONSTRAINT systemkonto_rolle_check;
ALTER TABLE systemkonto ADD CONSTRAINT systemkonto_rolle_check CHECK (rolle IN (
    'debitoren', 'kreditoren', 'vorsteuer', 'umsatzsteuer', 'mwst_abrechnung', 'bank',
    'kursdifferenzen', 'skonto_gewaehrt', 'skonto_erhalten', 'rundungsdifferenzen',
    'gewinnvortrag', 'eroeffnungsbilanz', 'jahresergebnis', 'ertrag', 'aufwand'));

-- Zähler der internen Nummer je Jahr des Rechnungsdatums (K-JJJJ-NNNN, AK-10). Gesperrt in der Transaktion des
-- Buchens; bricht das Buchen ab, wird die Nummer nicht verbraucht.
CREATE TABLE kreditorenrechnung_nummer (
    jahr    INTEGER PRIMARY KEY,
    letzte  INTEGER NOT NULL CHECK (letzte >= 0)
);

-- Eingangsrechnung bzw. Gutschrift eines Lieferanten (Kopf). Beträge sind immer positiv; die Art bestimmt die
-- Richtung der Buchung. Nummer, Kurs, Beträge und Buchung werden beim Buchen gesetzt (AK-10).
CREATE TABLE kreditorenrechnung (
    id                       BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nummer                   VARCHAR(20)   UNIQUE,
    art                      VARCHAR(10)   NOT NULL CHECK (art IN ('rechnung', 'gutschrift')),
    -- Lieferant (ANF-002); eine Adresse mit Eingangsrechnungen lässt sich nicht löschen (AK-15)
    adresse_id               BIGINT        NOT NULL REFERENCES adresse (id) ON DELETE RESTRICT,
    -- Rechnungsnummer des Lieferanten; nur im Entwurf aus einem Einkaufsauftrag noch leer (AK-8)
    rechnungsnummer          VARCHAR(50)   CHECK (btrim(rechnungsnummer) <> ''),
    -- Einkaufsauftrag (ANF-004) und Projekt (ANF-003), beide optional und beim Löschen gesperrt (AK-15)
    einkaufsauftrag_id       BIGINT        REFERENCES einkaufsauftrag (id) ON DELETE RESTRICT,
    projekt_id               BIGINT        REFERENCES projekt (id) ON DELETE RESTRICT,
    status                   VARCHAR(20)   NOT NULL DEFAULT 'entwurf'
                             CHECK (status IN ('entwurf', 'offen', 'teilweise_bezahlt', 'bezahlt', 'storniert')),
    rechnungsdatum           DATE          NOT NULL,
    zahlungsbedingung_id     BIGINT        NOT NULL REFERENCES zahlungsbedingung (id) ON DELETE RESTRICT,
    -- im Entwurf optional (Vorgabe überschreiben, AK-4), beim Buchen gesetzt; danach änderbar (AK-12)
    faelligkeit              DATE,
    skonto_bis               DATE,
    skonto_prozent           NUMERIC(5,2)  CHECK (skonto_prozent > 0 AND skonto_prozent < 100),
    waehrung                 VARCHAR(3)    NOT NULL DEFAULT 'CHF' REFERENCES waehrung (code) ON DELETE RESTRICT,
    kurs                     NUMERIC(14,6) CHECK (kurs > 0),
    -- Kontrollbetrag laut Beleg (AK-7); Pflicht beim Erfassen, nötig zum Buchen
    betrag_gemaess_rechnung  NUMERIC(14,2) CHECK (betrag_gemaess_rechnung >= 0),
    -- ausgeglichene Rundungsdifferenz (AK-7), höchstens 0.05
    rundung                  NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (abs(rundung) <= 0.05),
    gesamtbetrag             NUMERIC(14,2) CHECK (gesamtbetrag >= 0),
    gesamtbetrag_chf         NUMERIC(14,2) CHECK (gesamtbetrag_chf >= 0),
    offener_betrag           NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (offener_betrag >= 0),
    zahlungsreferenz         VARCHAR(35),
    bemerkung                TEXT,
    buchung_id               BIGINT        UNIQUE REFERENCES buchung (id) ON DELETE RESTRICT,
    erstellt_am              TIMESTAMPTZ   NOT NULL DEFAULT now(),
    geaendert_am             TIMESTAMPTZ   NOT NULL DEFAULT now(),
    CHECK ((status = 'entwurf') = (nummer IS NULL)),
    CHECK (status = 'entwurf' OR (buchung_id IS NOT NULL AND faelligkeit IS NOT NULL AND kurs IS NOT NULL
                                  AND gesamtbetrag IS NOT NULL AND gesamtbetrag_chf IS NOT NULL
                                  AND betrag_gemaess_rechnung IS NOT NULL AND rechnungsnummer IS NOT NULL)),
    CHECK ((skonto_bis IS NULL) = (skonto_prozent IS NULL))
);
-- AK-5: je Lieferant jede Rechnungsnummer nur einmal (stornierte ausgenommen)
CREATE UNIQUE INDEX kreditorenrechnung_doppelt_idx ON kreditorenrechnung (adresse_id, lower(rechnungsnummer))
    WHERE status <> 'storniert';
CREATE INDEX kreditorenrechnung_einkaufsauftrag_id_idx ON kreditorenrechnung (einkaufsauftrag_id);
CREATE INDEX kreditorenrechnung_projekt_id_idx ON kreditorenrechnung (projekt_id);
CREATE INDEX kreditorenrechnung_status_idx ON kreditorenrechnung (status);
CREATE INDEX kreditorenrechnung_faelligkeit_idx ON kreditorenrechnung (faelligkeit);
CREATE INDEX kreditorenrechnung_zahlungsbedingung_id_idx ON kreditorenrechnung (zahlungsbedingung_id);

-- Positionen: Aufwandskonto, MWST-Code (ohne MWST-Pflicht leer), Netto und MWST, optional Projekt und
-- Einkaufsposition (AK-6, AK-8).
CREATE TABLE kreditorenposition (
    id                     BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    kreditorenrechnung_id  BIGINT        NOT NULL REFERENCES kreditorenrechnung (id) ON DELETE CASCADE,
    reihenfolge            INTEGER       NOT NULL,
    bezeichnung            VARCHAR(200)  NOT NULL CHECK (btrim(bezeichnung) <> ''),
    konto_id               BIGINT        NOT NULL REFERENCES konto (id) ON DELETE RESTRICT,
    mwst_code_id           BIGINT        REFERENCES mwst_code (id) ON DELETE RESTRICT,
    netto                  NUMERIC(14,2) NOT NULL CHECK (netto >= 0),
    mwst_betrag            NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (mwst_betrag >= 0),
    projekt_id             BIGINT        REFERENCES projekt (id) ON DELETE RESTRICT,
    einkaufsposition_id    BIGINT        REFERENCES einkaufsposition (id) ON DELETE RESTRICT,
    erstellt_am            TIMESTAMPTZ   NOT NULL DEFAULT now(),
    geaendert_am           TIMESTAMPTZ   NOT NULL DEFAULT now(),
    CHECK (mwst_code_id IS NOT NULL OR mwst_betrag = 0)
);
CREATE INDEX kreditorenposition_rechnung_idx ON kreditorenposition (kreditorenrechnung_id, reihenfolge);
CREATE INDEX kreditorenposition_konto_id_idx ON kreditorenposition (konto_id);
CREATE INDEX kreditorenposition_mwst_code_id_idx ON kreditorenposition (mwst_code_id);
CREATE INDEX kreditorenposition_projekt_id_idx ON kreditorenposition (projekt_id);
CREATE INDEX kreditorenposition_einkaufsposition_id_idx ON kreditorenposition (einkaufsposition_id);

-- Gebuchte Rechnungen (AK-12): Positionen sind gesperrt, auch bei direktem Zugriff auf die Datenbank.
CREATE FUNCTION kreditorenposition_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    kopf_status TEXT;
BEGIN
    SELECT status INTO kopf_status FROM kreditorenrechnung
     WHERE id = COALESCE(NEW.kreditorenrechnung_id, OLD.kreditorenrechnung_id);
    IF kopf_status IS NOT NULL AND kopf_status <> 'entwurf' THEN
        RAISE EXCEPTION 'Positionen gebuchter Rechnungen können nicht geändert werden.';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER kreditorenposition_kontrolle
    BEFORE INSERT OR UPDATE OR DELETE ON kreditorenposition
    FOR EACH ROW EXECUTE FUNCTION kreditorenposition_pruefen();

-- Gebuchte Rechnungen (AK-12, AK-13): nur Fälligkeit, Bemerkung, Status und offener Betrag ändern sich;
-- gelöscht wird nur ein Entwurf.
CREATE FUNCTION kreditorenrechnung_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        IF OLD.status <> 'entwurf' THEN
            RAISE EXCEPTION 'Gebuchte Rechnungen können nicht gelöscht werden.';
        END IF;
        RETURN OLD;
    END IF;
    IF OLD.status <> 'entwurf' AND (
        NEW.art <> OLD.art
        OR NEW.nummer IS DISTINCT FROM OLD.nummer
        OR NEW.adresse_id <> OLD.adresse_id
        OR NEW.rechnungsnummer IS DISTINCT FROM OLD.rechnungsnummer
        OR NEW.einkaufsauftrag_id IS DISTINCT FROM OLD.einkaufsauftrag_id
        OR NEW.projekt_id IS DISTINCT FROM OLD.projekt_id
        OR NEW.rechnungsdatum <> OLD.rechnungsdatum
        OR NEW.zahlungsbedingung_id <> OLD.zahlungsbedingung_id
        OR NEW.waehrung <> OLD.waehrung
        OR NEW.kurs IS DISTINCT FROM OLD.kurs
        OR NEW.betrag_gemaess_rechnung IS DISTINCT FROM OLD.betrag_gemaess_rechnung
        OR NEW.rundung <> OLD.rundung
        OR NEW.gesamtbetrag IS DISTINCT FROM OLD.gesamtbetrag
        OR NEW.gesamtbetrag_chf IS DISTINCT FROM OLD.gesamtbetrag_chf
        OR NEW.zahlungsreferenz IS DISTINCT FROM OLD.zahlungsreferenz
        OR NEW.buchung_id IS DISTINCT FROM OLD.buchung_id
        OR NEW.status = 'entwurf'
    ) THEN
        RAISE EXCEPTION 'Gebuchte Rechnungen können nicht geändert werden.';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER kreditorenrechnung_kontrolle
    BEFORE UPDATE OR DELETE ON kreditorenrechnung
    FOR EACH ROW EXECUTE FUNCTION kreditorenrechnung_pruefen();
