-- ANF-026 Schritt 3b: Anmeldung nur über das Portal.
-- Die Mandanten-Datenbank enthält keine Zugangsdaten mehr (ANF-026 AK-8): Passwort, letzte Anmeldung,
-- Sitzungen und Fehlversuche stehen nur noch in der Portal-Datenbank.
ALTER TABLE benutzer DROP COLUMN passwort_hash;
ALTER TABLE benutzer DROP COLUMN letzte_anmeldung;
DROP TABLE sitzung;
DROP TABLE anmeldeversuch;
