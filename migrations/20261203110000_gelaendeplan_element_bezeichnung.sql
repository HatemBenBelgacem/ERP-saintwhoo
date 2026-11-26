-- ANF-027: Bezeichnung (Beschriftung) je Element eines Geländeplans

-- leer = keine Beschriftung
ALTER TABLE gelaendeplan_element ADD COLUMN bezeichnung VARCHAR(100);
