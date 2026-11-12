//! Grundgerüst: Kopfzeile, Seitenleiste und Inhaltsbereich (ANF-001).

use leptos::prelude::*;
use leptos_router::{components::Outlet, hooks::use_location};

use crate::icons::{Icon, IconView};
use crate::navigation::{
    MAIN_NAV, NavItem, SETTINGS_NAV, is_active, is_settings_path, settings_entry_path,
};

/// Id der Checkbox, die die Seitenleiste auf kleinen Bildschirmen öffnet/schliesst.
const DRAWER_ID: &str = "app-drawer";

/// Grundgerüst aller Seiten. Die jeweilige Seite erscheint im `<Outlet/>`.
#[component]
pub fn AppLayout() -> impl IntoView {
    let pathname = use_location().pathname;
    let drawer_open = RwSignal::new(false);

    // AK-11: Nach jedem Seitenwechsel die (mobile) Seitenleiste schliessen.
    Effect::new(move |_| {
        pathname.track();
        drawer_open.set(false);
    });

    view! {
        // AK-2: Nur der Inhaltsbereich scrollt, Header und Seitenleiste bleiben stehen.
        <div class="flex h-dvh flex-col overflow-hidden">
            <Header/>
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
fn Header() -> impl IntoView {
    view! {
        // Drei Spalten: Menü-Symbol | Leitspruch mittig | Platzhalter gleicher Breite (hält die Mitte exakt)
        <header class="z-10 grid min-h-16 grid-cols-[2.5rem_minmax(0,1fr)_2.5rem] items-center gap-2 bg-primary px-2 sm:px-4 py-2 text-primary-content">
            // AK-10: Menü-Symbol nur auf kleinen Bildschirmen
            <label for=DRAWER_ID aria-label="Seitenleiste öffnen" class="btn btn-square btn-ghost lg:invisible">
                <IconView icon=Icon::Menu/>
            </label>
            // Leitspruch in Tifinagh (Kabylisch), darunter die deutsche Übersetzung
            <div class="min-w-0 text-center leading-tight text-balance">
                <p
                    lang="kab-Tfng"
                    class="font-tifinagh text-lg tracking-wide sm:text-2xl"
                    title="Xas iqseḥ wedrar, ad t-yeɛfes uḍar."
                >
                    "ⵅⴰⵙ ⵉⵇⵙⴻⵃ ⵡⴻⴷⵔⴰⵔ, ⴰⴷ ⵜ-ⵢⴻⵄⴼⴻⵙ ⵓⴹⴰⵔ."
                </p>
                <p class="mt-0.5 text-xs opacity-90">
                    "So hart und steil der Berg auch sein mag, der Fuß wird ihn bezwingen."
                </p>
            </div>
            <span aria-hidden="true"></span>
        </header>
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
                        view! { <NavList items=SETTINGS_NAV pathname title="Einstellungen"/> }
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
