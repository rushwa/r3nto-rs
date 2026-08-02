use dioxus::prelude::*;

#[component]
pub fn Navbar() -> Element {
    rsx! {
        nav { class: "bg-gray-800 border-b border-gray-700 px-8 py-4",
            div { class: "flex justify-between items-center",
                h2 { class: "text-xl font-bold", "Welcome to Rento" }
                div { class: "flex items-center gap-4",
                    button { class: "text-gray-400 hover:text-white", "🔔" }
                }
            }
        }
    }
}
