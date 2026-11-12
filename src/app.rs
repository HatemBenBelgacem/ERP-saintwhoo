use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    ParamSegment, SsrMode, StaticSegment,
    components::{ParentRoute, Redirect, Route, Router, Routes},
};

use crate::layout::AppLayout;
use crate::navigation::settings_entry_path;
use crate::pages::{
    addresses::{AddressDetailPage, AddressListPage},
    home::HomePage,
    projects::{ElementDetailPage, ProjectDetailPage, ProjectListPage, model::Level},
    purchase_orders::{OrderDetailPage, OrderListPage},
    settings::GeneralSettingsPage,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="de" data-theme="light">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                // Schrift für den Leitspruch in Tifinagh (Berberschrift)
                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin=""/>
                <link
                    rel="stylesheet"
                    href="https://fonts.googleapis.com/css2?family=Noto+Sans+Tifinagh&display=swap"
                />
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
                    // ANF-004 Einkauf – Eingabe im Modal-Fenster (?neu=…, ?bearbeiten=…, ?wareneingang=…)
                    <Route path=StaticSegment("einkauf") view=OrderListPage/>
                    <Route path=(StaticSegment("einkauf"), ParamSegment("id")) view=OrderDetailPage/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
