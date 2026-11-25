//! Einstellungsseiten.

mod company;
mod finance;
mod general;

pub use company::{CompanyPage, NewAddressFields, SenderBlock};
pub use finance::{AccountsPage, FiscalYearPage, VatPage};
pub use general::GeneralSettingsPage;
