//! Stammdaten der Finanzen in den Einstellungen (ANF-010).

mod accounts;
mod bank_accounts;
mod currencies;
mod geschaeftsjahre;
mod mwst;
mod payment_terms;

pub use accounts::AccountsPage;
pub use bank_accounts::BankAccountsPage;
pub use currencies::CurrenciesPage;
pub use geschaeftsjahre::FiscalYearPage;
pub use mwst::VatPage;
pub use payment_terms::PaymentTermsPage;
