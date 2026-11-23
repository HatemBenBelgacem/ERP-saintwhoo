//! Startseite: alle Module als Karten (ANF-001, AK-12).

use leptos::prelude::*;
use leptos_meta::Title;

use crate::backend::system::server_fns::db_status;
use crate::icons::IconView;
use crate::navigation::{NavItem, modules};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <Title text="Startseite – uniqwork"/>
        <div class="space-y-4">
            <h1 class="text-2xl font-semibold">"Startseite"</h1>
            // Karten kommen aus der Hauptnavigation – neue Module erscheinen automatisch
            <div class="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
                {modules().map(|item| view! { <ModuleCard item/> }).collect_view()}
            </div>
            <DbStatus/>
        </div>
    }
}

/// Eine Modulkarte; die ganze Karte ist ein Verweis auf das Modul.
#[component]
fn ModuleCard(item: &'static NavItem) -> impl IntoView {
    view! {
        <a
            href=item.path
            class="card bg-base-100 shadow transition hover:-translate-y-0.5 hover:shadow-md focus-visible:outline-2 focus-visible:outline-primary"
        >
            <div class="card-body flex-row items-start gap-4">
                {item.icon.map(|icon| view! {
                    <span class="flex size-11 shrink-0 items-center justify-center rounded-field bg-primary/15 text-primary">
                        <IconView icon/>
                    </span>
                })}
                <div class="min-w-0">
                    <h2 class="card-title text-lg">{item.title}</h2>
                    <p class="text-base-content/70">{item.description}</p>
                </div>
            </div>
        </a>
    }
}

/// Kleine Statuszeile: Datenbank erreichbar?
#[component]
fn DbStatus() -> impl IntoView {
    let status = Resource::new(|| (), |_| db_status());
    view! {
        <Suspense fallback=|| ()>
            {move || Suspend::new(async move {
                match status.await {
                    Ok(version) => {
                        // Nur „PostgreSQL 17.x“ statt der ganzen Versionszeile
                        let short = version.split(" on ").next().unwrap_or(&version).to_string();
                        view! {
                            <p class="flex items-center gap-2 text-sm text-base-content/60" title=version>
                                <span class="status status-success"></span>
                                "Datenbank verbunden (" {short} ")"
                            </p>
                        }
                        .into_any()
                    }
                    Err(e) => view! {
                        <div role="alert" class="alert alert-error">"Datenbankfehler: " {e.to_string()}</div>
                    }
                    .into_any(),
                }
            })}
        </Suspense>
    }
}
