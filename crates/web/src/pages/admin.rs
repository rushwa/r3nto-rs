// crates/web/src/pages/admin.rs
use dioxus::prelude::*;

#[component]
pub fn Admin() -> Element {
    rsx! {
            div { class: "max-w-6xl mx-auto",
                h1 { class: "text-3xl font-bold text-gray-900 mb-8",
                    "Admin Dashboard"
                }
                div { class: "grid md:grid-cols-3 gap-6",
                    for _ in 0..3 {
                        div { class: "bg-white p-6 rounded-xl shadow-md",
                            p { class: "text-gray-600", "Admin widget - implement me" }
                        }
                    }
                }
            }

    }
}
