use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    ParamSegment, SsrMode, StaticSegment,
    components::{ParentRoute, Redirect, Route, Router, Routes},
};

use crate::backend::projects::model::Level;
use crate::components::{
    addresses::{AddressDetailPage, AddressListPage},
    auth::{LoginPage, RegisterPage},
    finance::{BookingDetailPage, JournalListPage, ReportsPage},
    home::HomePage,
    portal::PortalPage,
    projects::{ElementDetailPage, ProjectDetailPage, ProjectListPage},
    purchase_orders::{OrderDetailPage, OrderListPage},
    settings::{
        AccountsPage, BankAccountsPage, CompanyPage, CurrenciesPage, FiscalYearPage,
        GeneralSettingsPage, PaymentTermsPage, VatPage,
    },
    site_plans::SitePlanPage,
    tasks::{TaskDetailPage, TaskListPage},
    users::{UserDetailPage, UserListPage},
};
use crate::layout::AppLayout;
use crate::navigation::settings_entry_path;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(crate::forms::DataVersion(RwSignal::new(0)));

    view! {
        <Stylesheet id="leptos" href="/pkg/uniqwork.css"/>
        <Title text="uniqwork"/>
        <Router>
            <Routes fallback=|| view! { <p class="p-8">"Seite nicht gefunden."</p> }>
                // ANF-007: Anmeldeseite ohne Grundgerüst (öffentlich, siehe auth::middleware)
                <Route path=StaticSegment("anmelden") view=LoginPage/>
                // ANF-026: Registrierung (öffentlich) und Portal-Übersicht (ohne Mandant erreichbar)
                <Route path=StaticSegment("registrieren") view=RegisterPage/>
                <Route path=StaticSegment("portal") view=PortalPage/>
                <ParentRoute path=StaticSegment("") view=AppLayout>
                    <Route path=StaticSegment("") view=HomePage/>
                    // ANF-010 Finanzen: Übersicht folgt (AK-4); bis dahin führt der Bereich ins Journal
                    <Route
                        path=StaticSegment("finanzen")
                        view=|| view! { <Redirect path="/finanzen/journal"/> }
                        ssr=SsrMode::Async
                    />
                    // ANF-017 Buchungsjournal – Eingabe im Modal-Fenster (?neu=1, ?bearbeiten=, ?kopieren=, ?storno=, ?loeschen=)
                    <Route
                        path=(StaticSegment("finanzen"), StaticSegment("journal"))
                        view=JournalListPage
                    />
                    <Route
                        path=(StaticSegment("finanzen"), StaticSegment("journal"), ParamSegment("id"))
                        view=BookingDetailPage
                    />
                    // ANF-021 Auswertungen (?art=, ?geschaeftsjahr_id=, ?von=, ?bis=, ?konto_id=, ?projekt_id=, ?vorjahr=)
                    <Route
                        path=(StaticSegment("finanzen"), StaticSegment("auswertungen"))
                        view=ReportsPage
                    />
                    // ANF-002 Adressen – Eingabe im Modal-Fenster (?neu=1, ?bearbeiten=1)
                    <Route path=StaticSegment("adressen") view=AddressListPage/>
                    <Route
                        path=(StaticSegment("adressen"), ParamSegment("id"))
                        view=AddressDetailPage
                    />
                    <Route
                        path=StaticSegment("einstellungen")
                        view=|| view! { <Redirect path=settings_entry_path()/> }
                        // Async: Antwort erst nach dem Rendern senden, damit der Server echt umleitet (302)
                        ssr=SsrMode::Async
                    />
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("allgemein"))
                        view=GeneralSettingsPage
                    />
                    // ANF-025 Firma – Eingabe im Modal-Fenster (?neu=1, ?bearbeiten=1, ?logo=1)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("firma"))
                        view=CompanyPage
                    />
                    // ANF-011 Kontenplan – Eingabe im Modal-Fenster (?neu=1, ?bearbeiten=…, ?vorlage=1, ?systemkonten=1)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("finanzen"), StaticSegment("kontenplan"))
                        view=AccountsPage
                    />
                    // ANF-012 MWST – Eingabe im Modal-Fenster (?neu=1, ?bearbeiten=…, ?einstellungen=1, ?standard=1)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("finanzen"), StaticSegment("mwst"))
                        view=VatPage
                    />
                    // ANF-013 Geschäftsjahre – Eingabe im Modal-Fenster (?neu=1, ?periode=…, ?loeschen=…)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("finanzen"), StaticSegment("geschaeftsjahre"))
                        view=FiscalYearPage
                    />
                    // ANF-014 Währungen – Modal-Fenster (?neu=1, ?bearbeiten=<Code>, ?kurs=<Code>)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("finanzen"), StaticSegment("waehrungen"))
                        view=CurrenciesPage
                    />
                    // ANF-015 Bankkonten – Modal-Fenster (?neu=1, ?bearbeiten=<Kennung>)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("finanzen"), StaticSegment("bankkonten"))
                        view=BankAccountsPage
                    />
                    // ANF-016 Zahlungsbedingungen – Modal-Fenster (?neu=1, ?bearbeiten=<Kennung>)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("finanzen"), StaticSegment("zahlungsbedingungen"))
                        view=PaymentTermsPage
                    />
                    // ANF-005 Benutzer – Eingabe im Modal-Fenster (?neu=1, ?bearbeiten=1, ?passwort=1)
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("benutzer"))
                        view=UserListPage
                    />
                    <Route
                        path=(StaticSegment("einstellungen"), StaticSegment("benutzer"), ParamSegment("id"))
                        view=UserDetailPage
                    />
                    // ANF-003 Projekte – Eingabe im Modal-Fenster (?neu=…, ?bearbeiten=…); feste Segmente vor der Kennung
                    <Route path=StaticSegment("projekte") view=ProjectListPage/>
                    <Route
                        path=(StaticSegment("projekte"), StaticSegment("teilprojekt"), ParamSegment("id"))
                        view=|| view! { <ElementDetailPage level=Level::Subproject/> }
                    />
                    <Route
                        path=(StaticSegment("projekte"), StaticSegment("vorgang"), ParamSegment("id"))
                        view=|| view! { <ElementDetailPage level=Level::Process/> }
                    />
                    <Route
                        path=(StaticSegment("projekte"), StaticSegment("aufgabe"), ParamSegment("id"))
                        view=|| view! { <ElementDetailPage level=Level::Task/> }
                    />
                    <Route path=(StaticSegment("projekte"), ParamSegment("id")) view=ProjectDetailPage/>
                    // ANF-027 Geländeplan eines Projekts (3D-Karte), Standort im Modal-Fenster (?standort=1)
                    <Route
                        path=(StaticSegment("projekte"), ParamSegment("id"), StaticSegment("gelaendeplan"))
                        view=SitePlanPage
                    />
                    // ANF-006 Aufgaben eines Benutzers und Aufgaben ohne Projekt (?benutzer=…, ?neu=1, ?bearbeiten=1)
                    <Route path=StaticSegment("aufgaben") view=TaskListPage/>
                    <Route path=(StaticSegment("aufgaben"), ParamSegment("id")) view=TaskDetailPage/>
                    // ANF-004 Einkauf – Eingabe im Modal-Fenster (?neu=…, ?bearbeiten=…, ?wareneingang=…)
                    <Route path=StaticSegment("einkauf") view=OrderListPage/>
                    <Route path=(StaticSegment("einkauf"), ParamSegment("id")) view=OrderDetailPage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
