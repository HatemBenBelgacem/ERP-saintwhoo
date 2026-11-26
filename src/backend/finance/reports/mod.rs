//! Auswertungen (ANF-021): Kontoauszug, Saldenliste, Bilanz, Erfolgsrechnung, Projektauswertung.

#[cfg(feature = "ssr")]
pub mod csv;
pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
#[cfg(feature = "ssr")]
pub mod service;
