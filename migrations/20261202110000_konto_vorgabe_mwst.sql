-- ANF-011 AK-7: Vorgabe-MWST-Code am Konto. Beim Erfassen einer Buchung auf diesem Konto wird der Code
-- vorgeschlagen. Ein Code, der als Vorgabe dient, lässt sich nicht löschen (ANF-012 AK-7).
ALTER TABLE konto
    ADD COLUMN mwst_code_id BIGINT REFERENCES mwst_code (id) ON DELETE RESTRICT;
CREATE INDEX konto_mwst_code_id_idx ON konto (mwst_code_id);
