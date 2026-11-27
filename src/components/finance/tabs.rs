//! Reiter des Bereichs Finanzen (ANF-010 AK-2). Noch nicht umgesetzte Reiter sind ausgegraut.

use leptos::prelude::*;

use crate::backend::finance::debtors::model::DEBTORS_URL;
use crate::backend::finance::journal::server_fns::JOURNAL_URL;

/// `active` ist der Schlüssel des aktiven Reiters: `journal`, `debitoren` oder `auswertungen`.
#[component]
pub fn FinanceTabs(active: &'static str) -> impl IntoView {
    view! {
        <nav class="tabs tabs-border overflow-x-auto" aria-label="Bereiche der Finanzen">
            <span role="tab" class="tab tab-disabled whitespace-nowrap" aria-disabled="true" title="Folgt mit ANF-010 AK-4">
                "Übersicht"
            </span>
            <a
                role="tab"
                href=JOURNAL_URL
                class="tab whitespace-nowrap"
                class:tab-active=active == "journal"
                aria-current=if active == "journal" { "page" } else { "false" }
            >
                "Journal"
            </a>
            <a
                role="tab"
                href=DEBTORS_URL
                class="tab whitespace-nowrap"
                class:tab-active=active == "debitoren"
                aria-current=if active == "debitoren" { "page" } else { "false" }
            >
                "Debitoren"
            </a>
            <span role="tab" class="tab tab-disabled whitespace-nowrap" aria-disabled="true" title="Folgt mit ANF-019">
                "Kreditoren"
            </span>
            <span role="tab" class="tab tab-disabled whitespace-nowrap" aria-disabled="true" title="Folgt mit ANF-020">
                "Zahlungen"
            </span>
            <a
                role="tab"
                href="/finanzen/auswertungen"
                class="tab whitespace-nowrap"
                class:tab-active=active == "auswertungen"
                aria-current=if active == "auswertungen" { "page" } else { "false" }
            >
                "Auswertungen"
            </a>
        </nav>
    }
}
