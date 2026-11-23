//! Modul Anmeldung (ANF-007, ANF-026): Anmeldeseite und Registrierung, beide ohne Grundgerüst.
//! Abmelden und Name stehen in der Kopfzeile (`src/layout.rs`).

mod login;
mod register;

pub use login::LoginPage;
pub use register::RegisterPage;
