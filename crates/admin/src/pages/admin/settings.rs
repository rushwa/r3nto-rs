use dioxus::prelude::*;
#[component]
pub fn SettingsPage() -> Element {
    rsx! { div { class: "space-y-6", h1 { class: "text-3xl font-bold", "Settings" }, p { class: "text-gray-400", "Coming soon..." } } }
}
