-- ANF-018: Debitoren – Ausgangsrechnungen und Gutschriften mit Positionen

-- Systemkonto „Ertrag“ (Vorgabe für das Ertragskonto neuer Positionen, ANF-018 AK-5)
ALTER TABLE systemkonto DROP CONSTRAINT systemkonto_rolle_check;
ALTER TABLE systemkonto ADD CONSTRAINT systemkonto_rolle_check CHECK (rolle IN (
    'debitoren', 'kreditoren', 'vorsteuer', 'umsatzsteuer', 'mwst_abrechnung', 'bank',
    'kursdifferenzen', 'skonto_gewaehrt', 'skonto_erhalten', 'rundungsdifferenzen',
    'gewinnvortrag', 'eroeffnungsbilanz', 'jahresergebnis', 'ertrag'));

-- Zähler der Nummern je Art und Jahr des Rechnungsdatums (R-JJJJ-NNNN, G-JJJJ-NNNN). Die Zeile wird in der
-- Transaktion des Buchens gesperrt; bricht das Buchen ab, wird die Nummer nicht verbraucht.
CREATE TABLE debitorenrechnung_nummer (
    art     VARCHAR(10) NOT NULL CHECK (art IN ('rechnung', 'gutschrift')),
    jahr    INTEGER     NOT NULL,
    letzte  INTEGER     NOT NULL CHECK (letzte >= 0),
    PRIMARY KEY (art, jahr)
);

-- Rechnung bzw. Gutschrift (Kopf). Beträge sind immer positiv; die Art bestimmt die Richtung der Buchung.
-- Nummer, Fälligkeit, Kurs, Beträge und Buchung werden beim Buchen gesetzt (AK-8).
CREATE TABLE debitorenrechnung (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nummer                VARCHAR(20)   UNIQUE,
    art                   VARCHAR(10)   NOT NULL CHECK (art IN ('rechnung', 'gutschrift')),
    -- Kunde (ANF-002); eine Adresse mit Rechnungen lässt sich nicht löschen (AK-16)
    adresse_id            BIGINT        NOT NULL REFERENCES adresse (id) ON DELETE RESTRICT,
    -- optionales Projekt (ANF-003); ein Projekt mit Rechnungen lässt sich nicht löschen (AK-16)
    projekt_id            BIGINT        REFERENCES projekt (id) ON DELETE RESTRICT,
    -- Gutschrift → Rechnung (AK-14)
    bezug_rechnung_id     BIGINT        REFERENCES debitorenrechnung (id) ON DELETE RESTRICT,
    status                VARCHAR(20)   NOT NULL DEFAULT 'entwurf'
                          CHECK (status IN ('entwurf', 'offen', 'teilweise_bezahlt', 'bezahlt', 'storniert')),
    rechnungsdatum        DATE          NOT NULL,
    zahlungsbedingung_id  BIGINT        NOT NULL REFERENCES zahlungsbedingung (id) ON DELETE RESTRICT,
    faelligkeit           DATE,
    skonto_bis            DATE,
    skonto_prozent        NUMERIC(5,2)  CHECK (skonto_prozent > 0 AND skonto_prozent < 100),
    waehrung              VARCHAR(3)    NOT NULL DEFAULT 'CHF' REFERENCES waehrung (code) ON DELETE RESTRICT,
    -- CHF für 1 Einheit der Währung (ANF-014), beim Buchen festgeschrieben
    kurs                  NUMERIC(14,6) CHECK (kurs > 0),
    referenz_kunde        VARCHAR(100),
    einleitungstext       TEXT,
    schlusstext           TEXT,
    bemerkung             TEXT,
    gesamtbetrag          NUMERIC(14,2) CHECK (gesamtbetrag >= 0),
    gesamtbetrag_chf      NUMERIC(14,2) CHECK (gesamtbetrag_chf >= 0),
    -- wird durch Zahlungen reduziert (ANF-020)
    offener_betrag        NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (offener_betrag >= 0),
    -- Buchung im Journal (ANF-017); eine Buchung gehört höchstens einer Rechnung
    buchung_id            BIGINT        UNIQUE REFERENCES buchung (id) ON DELETE RESTRICT,
    erstellt_am           TIMESTAMPTZ   NOT NULL DEFAULT now(),
    geaendert_am          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    CHECK ((status = 'entwurf') = (nummer IS NULL)),
    CHECK (status = 'entwurf' OR (buchung_id IS NOT NULL AND faelligkeit IS NOT NULL AND kurs IS NOT NULL
                                  AND gesamtbetrag IS NOT NULL AND gesamtbetrag_chf IS NOT NULL)),
    CHECK ((skonto_bis IS NULL) = (skonto_prozent IS NULL)),
    CHECK (bezug_rechnung_id IS NULL OR art = 'gutschrift')
);
CREATE INDEX debitorenrechnung_adresse_id_idx ON debitorenrechnung (adresse_id);
CREATE INDEX debitorenrechnung_projekt_id_idx ON debitorenrechnung (projekt_id);
CREATE INDEX debitorenrechnung_status_idx ON debitorenrechnung (status);
CREATE INDEX debitorenrechnung_faelligkeit_idx ON debitorenrechnung (faelligkeit);
CREATE INDEX debitorenrechnung_bezug_idx ON debitorenrechnung (bezug_rechnung_id);
CREATE INDEX debitorenrechnung_zahlungsbedingung_id_idx ON debitorenrechnung (zahlungsbedingung_id);

-- Positionen wie im Einkauf (ANF-004), mit Ertragskonto und MWST-Code. Ohne MWST-Pflicht bleibt der Code leer;
-- ob ein Code nötig ist, prüft der Server beim Speichern und beim Buchen (ANF-012 AK-10).
CREATE TABLE debitorenposition (
    id                    BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    debitorenrechnung_id  BIGINT        NOT NULL REFERENCES debitorenrechnung (id) ON DELETE CASCADE,
    reihenfolge           INTEGER       NOT NULL,
    bezeichnung           VARCHAR(200)  NOT NULL CHECK (btrim(bezeichnung) <> ''),
    menge                 NUMERIC(12,3) NOT NULL CHECK (menge > 0),
    einheit               VARCHAR(10)   NOT NULL DEFAULT 'stk'
                          CHECK (einheit IN ('stk', 'h', 'kg', 'm', 'm2', 'm3', 'l', 'pauschal')),
    einzelpreis           NUMERIC(12,2) NOT NULL CHECK (einzelpreis >= 0),
    rabatt_prozent        NUMERIC(5,2)  NOT NULL DEFAULT 0 CHECK (rabatt_prozent BETWEEN 0 AND 100),
    konto_id              BIGINT        NOT NULL REFERENCES konto (id) ON DELETE RESTRICT,
    mwst_code_id          BIGINT        REFERENCES mwst_code (id) ON DELETE RESTRICT,
    erstellt_am           TIMESTAMPTZ   NOT NULL DEFAULT now(),
    geaendert_am          TIMESTAMPTZ   NOT NULL DEFAULT now()
);
CREATE INDEX debitorenposition_rechnung_idx ON debitorenposition (debitorenrechnung_id, reihenfolge);
CREATE INDEX debitorenposition_konto_id_idx ON debitorenposition (konto_id);
CREATE INDEX debitorenposition_mwst_code_id_idx ON debitorenposition (mwst_code_id);

-- Gebuchte Rechnungen (AK-10): Positionen sind gesperrt, auch bei direktem Zugriff auf die Datenbank.
CREATE FUNCTION debitorenposition_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    kopf_status TEXT;
BEGIN
    SELECT status INTO kopf_status FROM debitorenrechnung
     WHERE id = COALESCE(NEW.debitorenrechnung_id, OLD.debitorenrechnung_id);
    IF kopf_status IS NOT NULL AND kopf_status <> 'entwurf' THEN
        RAISE EXCEPTION 'Positionen gebuchter Rechnungen können nicht geändert werden.';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER debitorenposition_kontrolle
    BEFORE INSERT OR UPDATE OR DELETE ON debitorenposition
    FOR EACH ROW EXECUTE FUNCTION debitorenposition_pruefen();

-- Gebuchte Rechnungen (AK-10, AK-12): Kunde, Beträge und Buchung bleiben, gelöscht wird nur ein Entwurf.
CREATE FUNCTION debitorenrechnung_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
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
        OR NEW.projekt_id IS DISTINCT FROM OLD.projekt_id
        OR NEW.rechnungsdatum <> OLD.rechnungsdatum
        OR NEW.zahlungsbedingung_id <> OLD.zahlungsbedingung_id
        OR NEW.waehrung <> OLD.waehrung
        OR NEW.kurs IS DISTINCT FROM OLD.kurs
        OR NEW.gesamtbetrag IS DISTINCT FROM OLD.gesamtbetrag
        OR NEW.gesamtbetrag_chf IS DISTINCT FROM OLD.gesamtbetrag_chf
        OR NEW.buchung_id IS DISTINCT FROM OLD.buchung_id
        OR NEW.status = 'entwurf'
    ) THEN
        RAISE EXCEPTION 'Gebuchte Rechnungen können nicht geändert werden.';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER debitorenrechnung_kontrolle
    BEFORE UPDATE OR DELETE ON debitorenrechnung
    FOR EACH ROW EXECUTE FUNCTION debitorenrechnung_pruefen();
