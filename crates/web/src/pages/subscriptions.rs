// crates/web/src/pages/subscriptions.rs
use dioxus::prelude::*;

#[component]
pub fn Subscriptions() -> Element {
    rsx! {
            div { class: "max-w-6xl mx-auto",
                h1 { class: "text-3xl font-bold text-gray-900 mb-8",
                    "Subscriptions"
                }
                p { class: "text-gray-600", "Subscription management - implement me" }
            }

    }
}
