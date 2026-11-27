-- ANF-020: Zahlungen, Verrechnungen und Ausgleich offener Posten

-- Zahlung (Eingang, Ausgang) oder Verrechnung ohne Bankbewegung. `seite` sagt, ob Debitoren oder Kreditoren
-- ausgeglichen werden. Beträge in der Währung der Zahlung, Kurs in CHF je Einheit.
CREATE TABLE zahlung (
    id                   BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    art                  VARCHAR(12)   NOT NULL CHECK (art IN ('eingang', 'ausgang', 'verrechnung')),
    seite                VARCHAR(10)   NOT NULL CHECK (seite IN ('debitoren', 'kreditoren')),
    datum                DATE          NOT NULL,
    -- Partner (ANF-002); eine Adresse mit Zahlungen lässt sich nicht löschen
    adresse_id           BIGINT        NOT NULL REFERENCES adresse (id) ON DELETE RESTRICT,
    -- Bankkonto (ANF-015); ein Bankkonto mit Zahlungen lässt sich nicht löschen (ANF-015 AK-6)
    bankkonto_id         BIGINT        REFERENCES bankkonto (id) ON DELETE RESTRICT,
    betrag               NUMERIC(14,2) NOT NULL CHECK (betrag >= 0),
    waehrung             VARCHAR(3)    NOT NULL REFERENCES waehrung (code) ON DELETE RESTRICT,
    kurs                 NUMERIC(14,6) NOT NULL CHECK (kurs > 0),
    betrag_chf           NUMERIC(14,2) NOT NULL CHECK (betrag_chf >= 0),
    -- nicht zugeordneter Mehrbetrag (AK-5) und davon noch nicht verrechneter Rest
    vorauszahlung        NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (vorauszahlung >= 0),
    vorauszahlung_offen  NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (vorauszahlung_offen >= 0),
    status               VARCHAR(10)   NOT NULL DEFAULT 'gebucht' CHECK (status IN ('gebucht', 'storniert')),
    text                 VARCHAR(200),
    -- Buchung im Journal (ANF-017); leer bei reiner Verrechnung (AK-11)
    buchung_id           BIGINT        UNIQUE REFERENCES buchung (id) ON DELETE RESTRICT,
    erfasst_von          BIGINT        REFERENCES benutzer (id) ON DELETE SET NULL,
    erstellt_am          TIMESTAMPTZ   NOT NULL DEFAULT now(),
    geaendert_am         TIMESTAMPTZ   NOT NULL DEFAULT now(),
    CHECK ((art = 'verrechnung') = (bankkonto_id IS NULL)),
    CHECK ((art = 'verrechnung') = (buchung_id IS NULL)),
    CHECK (art = 'verrechnung' OR betrag > 0),
    CHECK (art <> 'eingang' OR seite = 'debitoren'),
    CHECK (art <> 'ausgang' OR seite = 'kreditoren'),
    CHECK (vorauszahlung_offen <= vorauszahlung),
    CHECK (art <> 'verrechnung' OR vorauszahlung = 0)
);
CREATE INDEX zahlung_adresse_id_idx ON zahlung (adresse_id);
CREATE INDEX zahlung_bankkonto_id_idx ON zahlung (bankkonto_id);
CREATE INDEX zahlung_datum_idx ON zahlung (datum);

-- Ausgleich einer Zahlung: genau ein Beleg (Rechnung/Gutschrift) oder eine Vorauszahlung (bei Verrechnung).
-- Beträge in der Währung des Belegs; Skonto und ausgebuchte Differenz nur bei Rechnungen (AK-3, AK-4).
CREATE TABLE zahlungsausgleich (
    id                     BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    zahlung_id             BIGINT        NOT NULL REFERENCES zahlung (id) ON DELETE CASCADE,
    debitorenrechnung_id   BIGINT        REFERENCES debitorenrechnung (id) ON DELETE RESTRICT,
    kreditorenrechnung_id  BIGINT        REFERENCES kreditorenrechnung (id) ON DELETE RESTRICT,
    vorauszahlung_id       BIGINT        REFERENCES zahlung (id) ON DELETE RESTRICT,
    betrag                 NUMERIC(14,2) NOT NULL CHECK (betrag >= 0),
    skonto                 NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (skonto >= 0),
    differenz              NUMERIC(14,2) NOT NULL DEFAULT 0 CHECK (differenz >= 0),
    differenz_konto_id     BIGINT        REFERENCES konto (id) ON DELETE RESTRICT,
    CHECK (num_nonnulls(debitorenrechnung_id, kreditorenrechnung_id, vorauszahlung_id) = 1),
    CHECK ((differenz > 0) = (differenz_konto_id IS NOT NULL)),
    CHECK (betrag + skonto + differenz > 0)
);
CREATE INDEX zahlungsausgleich_zahlung_id_idx ON zahlungsausgleich (zahlung_id);
CREATE INDEX zahlungsausgleich_debitorenrechnung_id_idx ON zahlungsausgleich (debitorenrechnung_id);
CREATE INDEX zahlungsausgleich_kreditorenrechnung_id_idx ON zahlungsausgleich (kreditorenrechnung_id);
CREATE INDEX zahlungsausgleich_vorauszahlung_id_idx ON zahlungsausgleich (vorauszahlung_id);
CREATE INDEX zahlungsausgleich_differenz_konto_id_idx ON zahlungsausgleich (differenz_konto_id);

-- Eine Zahlung ändert nach dem Speichern nur Status und offene Vorauszahlung; Ausgleichszeilen bleiben.
CREATE FUNCTION zahlung_pruefen() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Zahlungen werden storniert, nicht gelöscht.';
    END IF;
    IF NEW.art <> OLD.art OR NEW.seite <> OLD.seite OR NEW.datum <> OLD.datum
        OR NEW.adresse_id <> OLD.adresse_id OR NEW.bankkonto_id IS DISTINCT FROM OLD.bankkonto_id
        OR NEW.betrag <> OLD.betrag OR NEW.waehrung <> OLD.waehrung OR NEW.kurs <> OLD.kurs
        OR NEW.betrag_chf <> OLD.betrag_chf OR NEW.vorauszahlung <> OLD.vorauszahlung
        OR NEW.buchung_id IS DISTINCT FROM OLD.buchung_id
        OR (NEW.status <> OLD.status AND NOT (OLD.status = 'gebucht' AND NEW.status = 'storniert')) THEN
        RAISE EXCEPTION 'Gespeicherte Zahlungen können nicht geändert werden.';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER zahlung_kontrolle
    BEFORE UPDATE OR DELETE ON zahlung
    FOR EACH ROW EXECUTE FUNCTION zahlung_pruefen();
