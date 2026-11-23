//! Portal-Übersicht `/portal` (ANF-026 AK-4, AK-10, AK-19).

use leptos::prelude::*;
use leptos_meta::Title;

use crate::backend::auth::model::{CurrentUser, SessionTenant, TENANT_READY};
use crate::backend::auth::server_fns::get_current_user;
use crate::icons::{Icon, IconView};
use crate::layout::UserMenu;

#[component]
pub fn PortalPage() -> impl IntoView {
    let me = Resource::new(|| (), |_| get_current_user());

    view! {
        <Title text="Portal – uniqwork"/>
        <div class="flex min-h-dvh flex-col bg-base-200">
            // Kopfzeile wie im ERP, aber ohne Seitenleiste
            <header class="flex min-h-16 items-center gap-2 bg-neutral px-4 py-2 text-neutral-content">
                <span class="text-lg font-semibold">"uniqwork"</span>
                <Transition fallback=|| ()>
                    {move || Suspend::new(async move {
                        me.await.ok().flatten().map(|user| view! { <UserMenu user/> })
                    })}
                </Transition>
            </header>
            <main class="flex-1 space-y-4 p-4 sm:p-6">
                <Suspense fallback=|| view! { <span class="loading loading-spinner"></span> }>
                    {move || Suspend::new(async move {
                        match me.await {
                            Ok(Some(user)) => view! { <PortalContent user/> }.into_any(),
                            Ok(None) => view! {
                                <div role="alert" class="alert alert-warning">"Nicht angemeldet."</div>
                            }
                                .into_any(),
                            Err(e) => view! {
                                <div role="alert" class="alert alert-error">{crate::forms::error_message(&e)}</div>
                            }
                                .into_any(),
                        }
                    })}
                </Suspense>
            </main>
        </div>
    }
}

#[component]
fn PortalContent(user: CurrentUser) -> impl IntoView {
    view! {
        <h1 class="text-2xl font-semibold">"Willkommen, " {user.name.clone()}</h1>
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h2 class="card-title">
                    <IconView icon=Icon::Building/>
                    "Ihre Datenbank"
                </h2>
                {match user.mandant {
                    None => view! { <NoDatabase/> }.into_any(),
                    Some(tenant) => view! { <TenantInfo tenant/> }.into_any(),
                }}
            </div>
        </div>
    }
}

/// Noch keine Datenbank (ANF-026 AK-10). Das Erstellen folgt mit ANF-026 Schritt 3.
#[component]
fn NoDatabase() -> impl IntoView {
    view! {
        <p>"Sie haben noch keine Datenbank."</p>
        <div role="status" class="alert alert-info">
            "Das Erstellen einer eigenen Datenbank wird in Kürze freigeschaltet."
        </div>
    }
}

/// Datenbank vorhanden: Status, Mandant und Verweis ins ERP.
#[component]
fn TenantInfo(tenant: SessionTenant) -> impl IntoView {
    let usable = tenant.is_usable();
    let (label, badge) = match tenant.status.as_str() {
        TENANT_READY => ("Bereit", "badge badge-success"),
        "fehler" => ("Fehler", "badge badge-error"),
        _ => ("Wird erstellt", "badge badge-warning"),
    };
    view! {
        <p class="flex flex-wrap items-center gap-2">
            <span class="font-medium">
                {tenant.firmenname.clone().unwrap_or_else(|| "Mandant noch nicht definiert".into())}
            </span>
            <span class=badge>{label}</span>
        </p>
        {usable.then(|| view! {
            <div class="card-actions">
                <a href="/" class="btn btn-primary">"Zum ERP"</a>
            </div>
        })}
    }
}
