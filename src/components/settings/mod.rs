//! Einstellungsseiten.

mod company;
mod finance;
mod general;

pub use company::{CompanyPage, NewAddressFields, SenderBlock};
pub use finance::AccountsPage;
pub use general::GeneralSettingsPage;
