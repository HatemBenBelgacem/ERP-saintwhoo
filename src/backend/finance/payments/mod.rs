//! Zahlungen und offene Posten (ANF-020): Zahlungseingang und -ausgang mit Skonto, Differenz,
//! Vorauszahlung und Kursdifferenz, Verrechnung, Storno, Altersgliederung.

pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
