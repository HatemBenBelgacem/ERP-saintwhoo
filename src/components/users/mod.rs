//! Modul Benutzer (ANF-005): Liste, Detailansicht, Anlegen, Bearbeiten, Passwort setzen, Löschen.
//! Einstellungsseite unter `/einstellungen/benutzer`.

mod detail;
mod form;
mod list;

pub use detail::UserDetailPage;
pub use list::UserListPage;
