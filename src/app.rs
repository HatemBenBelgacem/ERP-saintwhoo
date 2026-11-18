use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    ParamSegment, SsrMode, StaticSegment,
    components::{ParentRoute, Redirect, Route, Router, Routes},
};

use crate::backend::projects::model::Level;
use crate::components::{
    addresses::{AddressDetailPage, AddressListPage},
    auth::LoginPage,
    home::HomePage,
    projects::{ElementDetailPage, ProjectDetailPage, ProjectListPage},
    purchase_orders::{OrderDetailPage, OrderListPage},
    settings::{AccountsPage, CompanyPage, GeneralSettingsPage},
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
        <Stylesheet id="leptos" href="/pkg/erp-saintwhoo.css"/>
        <Title text="ERP-saintwhoo"/>
        <Router>
            <Routes fallback=|| view! { <p class="p-8">"Seite nicht gefunden."</p> }>
                // ANF-007: Anmeldeseite ohne Grundgerüst (öffentlich, siehe auth::middleware)
                <Route path=StaticSegment("anmelden") view=LoginPage/>
                <ParentRoute path=StaticSegment("") view=AppLayout>
                    <Route path=StaticSegment("") view=HomePage/>
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
