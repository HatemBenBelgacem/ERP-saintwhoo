//! Einstellungsseiten.

mod company;
mod finance;
mod general;

pub use company::{CompanyPage, NewAddressFields, SenderBlock};
pub use finance::{
    AccountsPage, BankAccountsPage, CurrenciesPage, FiscalYearPage, PaymentTermsPage, VatPage,
};
pub use general::GeneralSettingsPage;
