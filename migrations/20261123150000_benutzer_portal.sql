-- ANF-026: Anmeldung nur über das Portal.
-- Jeder Benutzer der Mandanten-Datenbank kann einem Zugang der Portal-Datenbank zugeordnet sein
-- (kein Fremdschlüssel: die Portal-Datenbank ist eine andere Datenbank). Das Passwort steht nur noch im Portal;
-- die Spalte passwort_hash entfällt mit dem Umbau der Benutzerverwaltung (ANF-026 Schritt 3b).
ALTER TABLE benutzer ADD COLUMN portal_benutzer_id BIGINT UNIQUE;
ALTER TABLE benutzer ALTER COLUMN passwort_hash DROP NOT NULL;
