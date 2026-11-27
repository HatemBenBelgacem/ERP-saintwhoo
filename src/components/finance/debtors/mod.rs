//! Debitoren (ANF-018): Liste, Detailansicht mit Positionen, Modal-Fenster, Druckansicht und die
//! Abschnitte in Projekt und Adresse.

mod detail;
mod form;
mod links;
mod list;
mod print;

pub use detail::InvoiceDetailPage;
pub use links::{CustomerInvoices, ProjectInvoices};
pub use list::{DueDate, InvoiceListPage};
pub use print::InvoicePrintPage;
