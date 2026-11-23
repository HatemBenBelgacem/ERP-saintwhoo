//! Backend Portal (ANF-026): eigene Datenbank erstellen.
//! Registrierung, Anmeldung und Sitzungen liegen im Modul `auth`.

pub mod model;
#[cfg(feature = "ssr")]
pub mod provision;
#[cfg(feature = "ssr")]
pub mod repository;
pub mod server_fns;
