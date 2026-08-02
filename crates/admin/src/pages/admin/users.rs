use dioxus::prelude::*;
#[component]
pub fn UsersPage() -> Element {
    rsx! { div { class: "space-y-6", h1 { class: "text-3xl font-bold", "User Management" }, p { class: "text-gray-400", "Coming soon..." } } }
}
