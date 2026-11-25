//! Stammdaten der Finanzen in den Einstellungen (ANF-010).

mod accounts;
mod geschaeftsjahre;
mod mwst;

pub use accounts::AccountsPage;
pub use geschaeftsjahre::FiscalYearPage;
pub use mwst::VatPage;
