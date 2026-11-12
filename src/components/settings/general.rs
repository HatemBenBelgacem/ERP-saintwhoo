use leptos::prelude::*;

/// Einstellungen › Allgemein (Platzhalter, Inhalte folgen in eigener Anforderung).
#[component]
pub fn GeneralSettingsPage() -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h1 class="card-title text-2xl">"Allgemein"</h1>
                <p class="text-base-content/70">"Hier stehen später die allgemeinen Einstellungen."</p>
            </div>
        </div>
    }
}
