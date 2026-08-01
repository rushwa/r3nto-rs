// crates/web/src/pages/activate.rs
use dioxus::prelude::*;

#[component]
pub fn ActivateAccount() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-7xl mx-auto py-12 px-4 sm:px-6 lg:px-8",
                div { class: "bg-white shadow rounded-lg p-6",
                    h1 { class: "text-2xl font-bold text-gray-900 mb-4",
                    "Activating Your Account..."
                }
                p { class: "text-gray-600",
                    "Please wait while we verify your activation link."
                }
                }
            }
        }
    }
}
