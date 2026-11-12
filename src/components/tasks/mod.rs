//! Modul Aufgaben (ANF-006): alle Aufgaben eines Benutzers; Aufgaben ohne Projekt anlegen,
//! anzeigen, bearbeiten und löschen. Navigation „Aufgaben“ unter `/aufgaben`.

mod detail;
mod form;
mod list;

pub use detail::TaskDetailPage;
pub use list::TaskListPage;
