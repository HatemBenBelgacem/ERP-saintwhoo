//! Kreditoren – Eingangsrechnungen und Gutschriften von Lieferanten (ANF-019): Erfassen mit Kontrollbetrag,
//! Übernahme aus dem Einkaufsauftrag, Buchen ins Journal, Storno.

pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
