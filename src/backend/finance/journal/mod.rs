//! Buchungsjournal (ANF-017): Buchungen mit Zeilen, Entwürfe, Buchen, Storno.

pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
