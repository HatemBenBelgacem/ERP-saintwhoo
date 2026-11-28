//! Backend Anfragen (ANF-029): Anfragen aus einem Vorgang.

pub mod model;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
