-- ANF-012 AK-9: Einkaufspositionen verweisen auf einen MWST-Code. Der Satz bleibt als Stand zur Erfassung
-- gespeichert; bestehende Positionen ohne Code behalten ihren Satz. Ein verwendeter Code kann nicht gelöscht werden.
ALTER TABLE einkaufsposition
    ADD COLUMN mwst_code_id BIGINT REFERENCES mwst_code (id) ON DELETE RESTRICT;
CREATE INDEX einkaufsposition_mwst_code_id_idx ON einkaufsposition (mwst_code_id);
