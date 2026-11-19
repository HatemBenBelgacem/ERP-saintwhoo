//! Kleine Inline-SVG-Symbole für Navigation und Header.

use leptos::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Icon {
    Home,
    Settings,
    Sliders,
    Back,
    Menu,
    Addresses,
    Projects,
    Purchasing,
    Plus,
    Pencil,
    Trash,
    Search,
    Chevron,
    User,
    Tasks,
    Logout,
    Ledger,
    Building,
    Calendar,
    Percent,
}

impl Icon {
    fn path(self) -> &'static str {
        match self {
            Icon::Home => "M3 11.5 12 4l9 7.5M5.5 9.5V20h4.5v-6h4v6h4.5V9.5",
            Icon::Settings => {
                "M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6ZM12 2.5v3M12 18.5v3M2.5 12h3M18.5 12h3\
                 M5.3 5.3l2.1 2.1M16.6 16.6l2.1 2.1M5.3 18.7l2.1-2.1M16.6 7.4l2.1-2.1"
            }
            Icon::Sliders => "M4 6h9M17 6h3M4 12h3M11 12h9M4 18h11M19 18h1M15 4v4M9 10v4M17 16v4",
            Icon::Back => "M15 18l-6-6 6-6",
            Icon::Menu => "M4 6h16M4 12h16M4 18h16",
            Icon::Addresses => {
                "M15 19v-1a4 4 0 0 0-4-4H7a4 4 0 0 0-4 4v1M9 11a3.5 3.5 0 1 0 0-7 3.5 3.5 0 0 0 0 7Z\
                 M21 19v-1a4 4 0 0 0-3-3.87M15.5 4.13a3.5 3.5 0 0 1 0 6.74"
            }
            Icon::Projects => {
                "M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7ZM8 13h8M8 16h5"
            }
            Icon::Purchasing => {
                "M3 4h2l2.4 11.2a1 1 0 0 0 1 .8h9.2a1 1 0 0 0 1-.8L20.5 8H6.2\
                 M9 20.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2ZM17 20.5a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z"
            }
            Icon::Calendar => {
                "M7 3v3M17 3v3M4 9h16M5 5h14a1 1 0 0 1 1 1v13a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1Z"
            }
            Icon::Percent => "M19 5 5 19M7.5 7.5a1.5 1.5 0 1 0 0 .01M16.5 16.5a1.5 1.5 0 1 0 0 .01",
            Icon::Plus => "M12 5v14M5 12h14",
            Icon::Pencil => "M16.5 3.5l4 4L8 20H4v-4L16.5 3.5ZM14 6l4 4",
            Icon::Trash => "M4 7h16M10 11v6M14 11v6M9 7V4h6v3M6 7l1 13h10l1-13",
            Icon::Search => "M11 18a7 7 0 1 0 0-14 7 7 0 0 0 0 14ZM20 20l-4-4",
            Icon::Chevron => "M9 6l6 6-6 6",
            Icon::Logout => "M15 4h3a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-3M10 17l5-5-5-5M15 12H4",
            Icon::Building => {
                "M4 21V5a1 1 0 0 1 1-1h9a1 1 0 0 1 1 1v16M15 9h4a1 1 0 0 1 1 1v11M3 21h18M8 8h3M8 12h3M8 16h3"
            }
            Icon::Ledger => {
                "M5 4.5A1.5 1.5 0 0 1 6.5 3H19v15H6.5A1.5 1.5 0 0 0 5 19.5v-15ZM5 19.5A1.5 1.5 0 0 0 6.5 21H19M9 7h6M9 11h4"
            }
            Icon::Tasks => {
                "M10 6h10M10 12h10M10 18h10M3.5 6l1.5 1.5L7.5 5M3.5 12l1.5 1.5 2.5-2.5M3.5 18l1.5 1.5 2.5-2.5"
            }
            Icon::User => "M12 12a4 4 0 1 0 0-8 4 4 0 0 0 0 8ZM4.5 20a7.5 7.5 0 0 1 15 0",
        }
    }
}

#[component]
pub fn IconView(icon: Icon) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class="size-5 shrink-0"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
            stroke-width="1.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d=icon.path()></path>
        </svg>
    }
}
