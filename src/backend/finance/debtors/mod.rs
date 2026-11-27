//! Debitoren – Ausgangsrechnungen und Gutschriften (ANF-018): Entwurf mit Positionen, Buchen ins Journal,
//! Storno, Gutschrift und Druck.

pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
