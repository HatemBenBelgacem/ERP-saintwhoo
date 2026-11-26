-- ANF-027: Elementarten Sanität und Allgemeines Element

ALTER TABLE gelaendeplan_element DROP CONSTRAINT gelaendeplan_element_art_check;
ALTER TABLE gelaendeplan_element ADD CONSTRAINT gelaendeplan_element_art_check
    CHECK (art IN ('buehne', 'verpflegungsstand', 'festzelt', 'gitterabsperrung', 'sanitaet', 'allgemein'));
