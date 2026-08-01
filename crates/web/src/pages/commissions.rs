// crates/web/src/pages/commissions.rs
use dioxus::prelude::*;

#[component]
pub fn Commissions() -> Element {
    rsx! {
            div { class: "max-w-6xl mx-auto",
                h1 { class: "text-3xl font-bold text-gray-900 mb-8",
                    "Commissions"
                }
                p { class: "text-gray-600", "Commission tracking - implement me" }
            }

    }
}
