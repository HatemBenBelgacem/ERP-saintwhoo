-- ANF-029: Anfragen aus einem Vorgang; Einkaufsauftrag mit Herkunft aus einem Vorgang

-- Zähler für Anfragenummern je Jahr (A-JJJJ-NNNN)
CREATE TABLE anfrage_nummer (
    jahr    INTEGER PRIMARY KEY,
    letzte  INTEGER NOT NULL
);

-- Kopien von Titel (Teilprojekt) und Beschreibung (Vorgang) zum Zeitpunkt des Erstellens (AK-2, AK-5)
CREATE TABLE anfrage (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    nummer        VARCHAR(20)  NOT NULL UNIQUE,
    -- Anfrage – Vorgang N:1; beim Löschen des Vorgangs werden seine Anfragen mitgelöscht (AK-6)
    vorgang_id    BIGINT       NOT NULL REFERENCES vorgang (id) ON DELETE CASCADE,
    titel         VARCHAR(200) NOT NULL CHECK (btrim(titel) <> ''),
    beschreibung  TEXT,
    erstellt_am   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am  TIMESTAMPTZ  NOT NULL DEFAULT now()
);
CREATE INDEX anfrage_vorgang_id_idx ON anfrage (vorgang_id, id);

-- Herkunft: ein Einkaufsauftrag kann aus einem Vorgang stammen; beim Löschen des Vorgangs bleibt er
-- erhalten, der Verweis wird leer (AK-11).
ALTER TABLE einkaufsauftrag
    ADD COLUMN vorgang_id BIGINT REFERENCES vorgang (id) ON DELETE SET NULL;
CREATE INDEX einkaufsauftrag_vorgang_id_idx ON einkaufsauftrag (vorgang_id);
