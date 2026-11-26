//! Zahlungsbedingungen: Fälligkeit, Skonto und Rechnungstext, Vorgabe je Adresse (ANF-016).

pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
