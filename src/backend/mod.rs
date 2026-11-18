//! Serverseitige Module: je Modul `model` (gemeinsame Datentypen und Prüfregeln),
//! `server_fns` (Server-Funktionen) und `repository` (SQL-Zugriffe, nur Server).
//! Aufbau siehe `system-design.md`.

pub mod addresses;
pub mod auth;
pub mod company;
pub mod finance;
pub mod projects;
pub mod purchase_orders;
pub mod system;
pub mod tasks;
pub mod users;
