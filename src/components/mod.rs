//! Oberflächen-Komponenten: ein Ordner pro Modul, eine Datei pro Komponente bzw. Ansicht.
//! Daten kommen ausschliesslich über die Server-Funktionen aus `crate::backend`.
//! Aufbau siehe `system-design.md`.

pub mod addresses;
pub mod auth;
pub mod home;
pub mod projects;
pub mod purchase_orders;
pub mod settings;
pub mod tasks;
pub mod users;
