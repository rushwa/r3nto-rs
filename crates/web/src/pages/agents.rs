// crates/web/src/pages/agents.rs
use dioxus::prelude::*;

#[component]
pub fn Agents() -> Element {
    rsx! {
            div { class: "max-w-6xl mx-auto",
                h1 { class: "text-3xl font-bold text-gray-900 mb-8",
                    "Agents"
                }
                p { class: "text-gray-600", "Agent management - implement me" }
            }

    }
}
