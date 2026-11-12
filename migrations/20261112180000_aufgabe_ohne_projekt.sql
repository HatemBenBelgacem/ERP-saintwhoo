-- ANF-006: Aufgaben ohne Projekt
-- Eine Aufgabe gehört entweder zu einem Vorgang (Projektaufgabe, ANF-003) oder direkt einem Benutzer
-- (Aufgabe ohne Projekt) – nie beides und nie keines von beiden.
ALTER TABLE aufgabe ALTER COLUMN vorgang_id DROP NOT NULL;
ALTER TABLE aufgabe ADD COLUMN benutzer_id BIGINT REFERENCES benutzer (id) ON DELETE RESTRICT;
ALTER TABLE aufgabe ADD CONSTRAINT aufgabe_vorgang_oder_benutzer
    CHECK ((vorgang_id IS NULL) <> (benutzer_id IS NULL));
CREATE INDEX aufgabe_benutzer_id_idx ON aufgabe (benutzer_id);
