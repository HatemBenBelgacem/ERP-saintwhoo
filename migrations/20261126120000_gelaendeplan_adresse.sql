-- ANF-027: Adresse bzw. Bezeichnung des Standorts eines Geländeplans (aus der Adresssuche oder von Hand)
ALTER TABLE gelaendeplan
    ADD COLUMN standort_adresse VARCHAR(300);
