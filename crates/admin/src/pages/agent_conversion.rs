use dioxus::prelude::*;
use crate::components::sidebar::PageHeader;

#[component]
pub fn ConversionPage() -> Element {
    rsx! {
        div { class: "space-y-4 p-6",
            PageHeader { title: "Role Conversion".to_string(), subtitle: "Convert clients to property owners".to_string() }
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                p { class: "text-gray-400 text-lg", "Digital Handshake conversion flow coming soon..." }
            }
        }
    }
}
