//! Eigene Firma (ANF-025): Einstellungsseite unter `/einstellungen/firma`.

mod form;
mod page;

pub use form::NewAddressFields;

pub use page::{CompanyPage, SenderBlock};
