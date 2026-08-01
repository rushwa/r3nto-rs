use dioxus::prelude::*;
// use dioxus_router::prelude::*;

use crate::AdminRoute;

#[component]
pub fn NotFoundPage(segments: Vec<String>) -> Element {
    let nav = use_navigator();
    let path = segments.join("/");

    rsx! {
        div { class: "min-h-[60vh] flex items-center justify-center",
            div { class: "text-center",
                h1 { class: "text-6xl font-bold text-gray-700", "404" }
                p { class: "text-xl text-gray-400 mt-4", "Page not found" }
                p { class: "text-gray-500 mt-2 font-mono text-sm", "/{path}" }
                button {
                    class: "mt-6 px-6 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors",
                    onclick: move |_| { let _ = nav.push(AdminRoute::DashboardPage); },
                    "Go to Dashboard"
                }
            }
        }
    }
}
