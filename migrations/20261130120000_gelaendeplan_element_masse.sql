-- ANF-027: Elementart Gitterabsperrung und eigene Höhe je Element

ALTER TABLE gelaendeplan_element DROP CONSTRAINT gelaendeplan_element_art_check;
ALTER TABLE gelaendeplan_element ADD CONSTRAINT gelaendeplan_element_art_check
    CHECK (art IN ('buehne', 'verpflegungsstand', 'festzelt', 'gitterabsperrung'));

-- Eigene Höhe in Metern; NULL = Vorgabe der Art (Bühne 4,5 m, Verpflegungsstand 2,8 m, Festzelt 3 m,
-- Gitterabsperrung 1,1 m)
ALTER TABLE gelaendeplan_element
    ADD COLUMN hoehe_m DOUBLE PRECISION CHECK (hoehe_m BETWEEN 0.1 AND 50);
