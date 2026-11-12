//! Modul Projekte (ANF-003): Projekt → Teilprojekt → Vorgang → Aufgabe,
//! auf jeder Ebene Anlegen, Anzeigen, Bearbeiten, Löschen.

mod detail;
mod form;
mod list;

pub use detail::{ElementDetailPage, ProjectDetailPage};
pub use list::ProjectListPage;
