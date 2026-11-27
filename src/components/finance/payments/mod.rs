//! Zahlungen (ANF-020): Zahlungsliste, offene Posten, Zahlungsfenster, Detailansicht und der Abschnitt
//! „Zahlungen“ in den Rechnungen.

mod detail;
mod form;
mod list;

pub use detail::{DocumentPayments, PaymentDetailPage};
pub use form::PayCreditorsModal;
pub use list::PaymentsPage;
