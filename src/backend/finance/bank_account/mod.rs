//! Eigene Bankkonten: Bank, Post und Kasse mit Verknüpfung zum Kontenplan (ANF-015).

pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
