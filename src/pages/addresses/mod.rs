//! Modul Adressen (ANF-002): Liste, Detailansicht, Anlegen, Bearbeiten, Löschen.

pub mod api;
mod detail;
mod form;
mod list;
pub mod model;

pub use detail::AddressDetailPage;
pub use list::AddressListPage;
