use dioxus::prelude::*;
use serde::Deserialize;
use crate::api::api_get;

#[derive(Clone, Debug, Deserialize)]
pub struct PropertiesResponse {
    pub properties: Vec<Property>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Property {
    pub id: String,
    pub title: String,
    pub location: Option<String>,
    pub verified_at: Option<String>,
}

#[component]
pub fn PropertiesPage() -> Element {
    let properties = use_resource(move || async move {
        api_get::<PropertiesResponse>("/api/properties").await
    });

    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold", "Property Management" }
            match &*properties.read() {
                Some(Some(data)) => rsx! {
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                        for prop in &data.properties {
                            PropertyCard { property: prop.clone() }
                        }
                    }
                },
                _ => rsx! { p { class: "text-gray-400", "Loading properties..." } }
            }
        }
    }
}

#[component]
fn PropertyCard(property: Property) -> Element {
    let location_text = property.location.as_deref().unwrap_or("No location");
    let is_verified = property.verified_at.is_some();
    
    rsx! {
        div { class: "bg-gray-800 rounded-lg p-6",
            div { class: "flex justify-between items-start mb-3",
                h3 { class: "text-lg font-bold", "{property.title}" }
                if is_verified {
                    span { class: "px-2 py-1 text-xs rounded-full bg-green-900 text-green-300", "✅ Verified" }
                }
            }
            p { class: "text-gray-400 mb-4", "{location_text}" }
            if !is_verified {
                button { class: "w-full bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg text-sm", "Verify Property" }
            } else {
                button { class: "w-full bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded-lg text-sm", "Generate Viewing Link" }
            }
        }
    }
}
