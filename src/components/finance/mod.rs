//! Finanzen (ANF-010): Reiter, Buchungsjournal (ANF-017) und Auswertungen (ANF-021).
//! Daten kommen über `crate::backend::finance`; Eingaben stehen in Modal-Fenstern.

pub mod journal;
pub mod reports;
pub mod tabs;

pub use journal::{BookingDetailPage, JournalListPage};
pub use reports::ReportsPage;
pub use tabs::FinanceTabs;
