//! Finanzen (ANF-010): Reiter, Buchungsjournal (ANF-017), Debitoren (ANF-018), Kreditoren (ANF-019), Zahlungen (ANF-020) und Auswertungen (ANF-021).
//! Daten kommen über `crate::backend::finance`; Eingaben stehen in Modal-Fenstern.

pub mod creditors;
pub mod debtors;
pub mod journal;
pub mod payments;
pub mod reports;
pub mod tabs;

pub use creditors::{CreditorDetailPage, CreditorListPage};
pub use debtors::{InvoiceDetailPage, InvoiceListPage, InvoicePrintPage};
pub use journal::{BookingDetailPage, JournalListPage};
pub use payments::{PaymentDetailPage, PaymentsPage};
pub use reports::ReportsPage;
pub use tabs::FinanceTabs;
