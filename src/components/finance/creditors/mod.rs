//! Kreditoren (ANF-019): Liste, Detailansicht mit Positionen und Kontrollbetrag, Modal-Fenster und die
//! Abschnitte in Projekt, Adresse und Einkaufsauftrag.

mod detail;
mod form;
mod links;
mod list;

pub use detail::CreditorDetailPage;
pub use links::{OrderCreditors, ProjectCreditors, SupplierCreditors};
pub use list::CreditorListPage;
