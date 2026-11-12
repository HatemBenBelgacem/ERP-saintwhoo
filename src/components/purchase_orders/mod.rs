//! Modul Einkauf (ANF-004): Einkaufsaufträge mit Positionen, Bestellen, Wareneingang, Stornieren.

mod detail;
mod form;
mod list;

pub use detail::{OrderDetailPage, ProjectOrders};
pub use list::OrderListPage;
