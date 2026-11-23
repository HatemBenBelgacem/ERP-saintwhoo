//! Grundgerüst: Kopfzeile, Seitenleiste und Inhaltsbereich (ANF-001).

use leptos::prelude::*;
use leptos_router::{
    components::Outlet,
    hooks::{use_location, use_navigate},
};

use crate::backend::auth::model::{CurrentUser, login_url};
use crate::backend::auth::server_fns::{Logout, get_current_user};

use crate::icons::{Icon, IconView};
use crate::navigation::{
    MAIN_NAV, NavItem, SETTINGS_FINANCE_NAV, SETTINGS_NAV, is_active, is_settings_path,
    settings_entry_path,
};

/// Id der Checkbox, die die Seitenleiste auf kleinen Bildschirmen öffnet/schliesst.
const DRAWER_ID: &str = "app-drawer";

/// Grundgerüst aller Seiten. Die jeweilige Seite erscheint im `<Outlet/>`.
#[component]
pub fn AppLayout() -> impl IntoView {
    let location = use_location();
    let pathname = location.pathname;
    let drawer_open = RwSignal::new(false);

    // ANF-007: angemeldeter Benutzer für die Kopfzeile; bei jedem Seitenwechsel neu geprüft.
    // Ist die Sitzung inzwischen abgelaufen (AK-14), geht es zur Anmeldeseite (Seitenwechsel im
    // Browser laufen nicht über den Server, deshalb hier zusätzlich zur Middleware).
    let me = Resource::new(move || pathname.get(), |_| get_current_user());
    let navigate = use_navigate();
    Effect::new(move |_| match me.get() {
        Some(Ok(None)) => {
            let target = format!(
                "{}{}",
                location.pathname.get_untracked(),
                location.search.get_untracked()
            );
            navigate(&login_url(&target, true), Default::default());
        }
        // ANF-026 AK-19: ohne bereiten und definierten Mandanten nur das Portal
        Some(Ok(Some(user))) if !user.erp_access() => {
            navigate(&user.portal_target(), Default::default())
        }
        _ => {}
    });

    // AK-11: Nach jedem Seitenwechsel die (mobile) Seitenleiste schliessen.
    Effect::new(move |_| {
        pathname.track();
        drawer_open.set(false);
    });

    view! {
        // AK-2: Nur der Inhaltsbereich scrollt, Header und Seitenleiste bleiben stehen.
        <div class="flex h-dvh flex-col overflow-hidden">
            <Header me/>
            <div class="drawer min-h-0 flex-1 lg:drawer-open">
                <input
                    id=DRAWER_ID
                    type="checkbox"
                    class="drawer-toggle"
                    prop:checked=move || drawer_open.get()
                    on:change=move |ev| drawer_open.set(event_target_checked(&ev))
                />
                <main class="drawer-content min-w-0 overflow-y-auto bg-base-200 p-4 lg:p-6">
                    <Outlet/>
                </main>
                <div class="drawer-side z-20 lg:h-full">
                    <label
                        for=DRAWER_ID
                        aria-label="Seitenleiste schliessen"
                        class="drawer-overlay"
                    ></label>
                    <Sidebar pathname/>
                </div>
            </div>
        </div>
    }
}

#[component]
fn Header(me: Resource<Result<Option<CurrentUser>, ServerFnError>>) -> impl IntoView {
    view! {
        // Neutrale Kopfzeile in dunklem Anthrazit (daisyUI `neutral`)
        <header class="z-10 flex min-h-16 items-center gap-2 bg-neutral px-2 py-2 text-neutral-content sm:px-4">
            // AK-10: Menü-Symbol nur auf kleinen Bildschirmen
            <label for=DRAWER_ID aria-label="Seitenleiste öffnen" class="btn btn-square btn-ghost text-neutral-content lg:invisible">
                <IconView icon=Icon::Menu/>
            </label>
            // ANF-026 AK-20: Firmenname des Mandanten; ANF-007 AK-17: angemeldeter Benutzer und „Abmelden“ rechts
            <Transition fallback=|| ()>
                {move || Suspend::new(async move {
                    me.await.ok().flatten().map(|user| view! {
                        {user.company_name().map(|name| view! {
                            <span class="flex min-w-0 items-center gap-2 font-semibold" title="Mandant">
                                <IconView icon=Icon::Building/>
                                <span class="truncate">{name.to_string()}</span>
                            </span>
                        })}
                        <UserMenu user/>
                    })
                })}
            </Transition>
        </header>
    }
}

/// Name des angemeldeten Benutzers und „Abmelden“ (ANF-007 AK-17, AK-18). Auch auf der Portal-Übersicht.
#[component]
pub fn UserMenu(user: CurrentUser) -> impl IntoView {
    let logout = ServerAction::<Logout>::new();
    view! {
        <div class="ml-auto flex items-center gap-1 sm:gap-3">
            // ANF-026 AK-28: Name führt zum Portal (Datenbank, Zugang, Passwort ändern)
            <a href="/portal" class="btn btn-ghost btn-sm text-neutral-content" title=format!("Angemeldet als {} – Portal öffnen", user.email)>
                <IconView icon=Icon::User/>
                // Auf dem Mobiltelefon nur das Symbol
                <span class="hidden max-w-48 truncate sm:inline">{user.name.clone()}</span>
            </a>
            <ActionForm action=logout>
                <button type="submit" class="btn btn-ghost btn-sm text-neutral-content" aria-label="Abmelden" disabled=move || logout.pending().get()>
                    <IconView icon=Icon::Logout/>
                    <span class="hidden sm:inline">"Abmelden"</span>
                </button>
            </ActionForm>
        </div>
    }
}

#[component]
fn Sidebar(pathname: Memo<String>) -> impl IntoView {
    let in_settings = Memo::new(move |_| is_settings_path(&pathname.get()));

    view! {
        <aside class="flex h-full min-h-full w-64 flex-col border-r border-base-300 bg-base-100">
            <nav class="flex-1 overflow-y-auto" aria-label="Navigation">
                {move || {
                    if in_settings.get() {
                        view! {
                            <NavList items=SETTINGS_NAV pathname title="Einstellungen"/>
                            <NavList items=SETTINGS_FINANCE_NAV pathname title="Finanzen"/>
                        }
                            .into_any()
                    } else {
                        view! { <NavList items=MAIN_NAV pathname/> }.into_any()
                    }
                }}
            </nav>
            // AK-6/7/8: Fusszeile mit „Einstellungen“ bzw. „Zurück“
            <div class="border-t border-base-300">
                <ul class="menu w-full">
                    <li>
                        {move || {
                            if in_settings.get() {
                                view! {
                                    <a href="/">
                                        <IconView icon=Icon::Back/>
                                        "Zurück"
                                    </a>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <a href=settings_entry_path()>
                                        <IconView icon=Icon::Settings/>
                                        "Einstellungen"
                                    </a>
                                }
                                    .into_any()
                            }
                        }}
                    </li>
                </ul>
            </div>
        </aside>
    }
}

#[component]
fn NavList(
    items: &'static [NavItem],
    pathname: Memo<String>,
    #[prop(optional)] title: Option<&'static str>,
) -> impl IntoView {
    view! {
        <ul class="menu w-full gap-1">
            {title.map(|t| view! { <li class="menu-title">{t}</li> })}
            {items
                .iter()
                .map(|item| {
                    let path = item.path;
                    let active = move || is_active(path, &pathname.get());
                    view! {
                        <li>
                            <a
                                href=path
                                class:menu-active=active
                                aria-current=move || active().then_some("page")
                            >
                                {item.icon.map(|icon| view! { <IconView icon/> })}
                                {item.title}
                            </a>
                        </li>
                    }
                })
                .collect_view()}
        </ul>
    }
}
