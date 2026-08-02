use dioxus::prelude::*;
#[component]
pub fn AnalyticsPage() -> Element {
    rsx! { div { class: "space-y-6", h1 { class: "text-3xl font-bold", "Analytics" }, p { class: "text-gray-400", "Coming soon..." } } }
}
