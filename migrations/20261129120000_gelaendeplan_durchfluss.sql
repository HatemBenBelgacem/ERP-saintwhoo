-- ANF-028 Stufe 3: Planungswert für Engstellen – Personen pro Stunde je Meter Wegbreite
ALTER TABLE gelaendeplan
    ADD COLUMN durchfluss_pro_meter DOUBLE PRECISION NOT NULL DEFAULT 2000
        CHECK (durchfluss_pro_meter BETWEEN 100 AND 10000);
