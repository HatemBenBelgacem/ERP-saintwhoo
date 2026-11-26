-- ANF-015: Eigene Bankkonten (Bank, Post, Kasse)

-- Bankkonto: verknüpft mit einem Aktivkonto aus dem Kontenplan (ANF-011). Jedes Konto gehört nur einem Bankkonto (AK-2).
-- IBAN und QR-IBAN ohne Leerzeichen gespeichert; die Prüfung (ISO 13616, QR-IBAN) macht `model.rs`.
CREATE TABLE bankkonto (
    id            BIGINT GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    bezeichnung   VARCHAR(100) NOT NULL CHECK (btrim(bezeichnung) <> ''),
    art           VARCHAR(10)  NOT NULL CHECK (art IN ('bank', 'post', 'kasse')),
    bank          VARCHAR(100),
    -- Pflicht bei Bank und Post (AK-3), bei der Kasse leer
    iban          VARCHAR(34),
    qr_iban       VARCHAR(34),
    bic           VARCHAR(11),
    konto_id      BIGINT       NOT NULL UNIQUE REFERENCES konto (id) ON DELETE RESTRICT,
    standard      BOOLEAN      NOT NULL DEFAULT false,
    aktiv         BOOLEAN      NOT NULL DEFAULT true,
    erstellt_am   TIMESTAMPTZ  NOT NULL DEFAULT now(),
    geaendert_am  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    CHECK (art = 'kasse' OR iban IS NOT NULL),
    CHECK (NOT standard OR aktiv)
);

-- Genau ein Standard-Bankkonto (AK-5): alle Zeilen mit standard = true haben denselben Schlüssel.
CREATE UNIQUE INDEX bankkonto_standard_idx ON bankkonto (standard) WHERE standard;
