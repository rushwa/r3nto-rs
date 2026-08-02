use dioxus::prelude::*;

#[component]
pub fn AdminDashboard() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold", "Admin Dashboard" }
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                div { class: "bg-gray-800 rounded-lg p-6",
                    h3 { class: "text-xl font-bold mb-2", "Total Users" }
                    p { class: "text-3xl text-blue-400", "1,234" }
                }
                div { class: "bg-gray-800 rounded-lg p-6",
                    h3 { class: "text-xl font-bold mb-2", "Active Properties" }
                    p { class: "text-3xl text-green-400", "567" }
                }
                div { class: "bg-gray-800 rounded-lg p-6",
                    h3 { class: "text-xl font-bold mb-2", "Revenue" }
                    p { class: "text-3xl text-yellow-400", "KES 2.5M" }
                }
            }
        }
    }
}
