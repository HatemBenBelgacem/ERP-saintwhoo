-- ANF-003: Aufgaben bestehen nur aus dem Titel (plus erledigt/offen für den Fortschritt)
ALTER TABLE aufgabe DROP CONSTRAINT aufgabe_ende_nach_start;
ALTER TABLE aufgabe
    DROP COLUMN beschreibung,
    DROP COLUMN start,
    DROP COLUMN ende,
    DROP COLUMN prioritaet,
    DROP COLUMN aufwand_stunden;

UPDATE aufgabe SET status = 'offen' WHERE status NOT IN ('offen', 'erledigt');
ALTER TABLE aufgabe DROP CONSTRAINT aufgabe_status_check;
ALTER TABLE aufgabe ADD CONSTRAINT aufgabe_status_check CHECK (status IN ('offen', 'erledigt'));
