use dioxus::prelude::*;
use serde::Deserialize;
use crate::Route;

const API_BASE: &str = "http://localhost:8000";

#[derive(Debug, Clone, Deserialize,PartialEq)]
pub struct Property {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub price: f64,
    #[serde(default)]
    pub status: String,
    // ✅ FIX: API sends "owner", but we keep the semantic name "owner_name"
    #[serde(rename = "owner", default)]
    pub owner_name: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub property_type: String,
    #[serde(default)]
    pub bedrooms: u32,
    #[serde(default)]
    pub bathrooms: u32,
    #[serde(default)]
    pub area_sqft: u32,
    #[serde(default)]
    pub created_at: String,
}
/// Helper: Format currency with thousands separators (e.g., 1500000 -> "1,500,000")
fn format_currency(amount: f64) -> String {
    let s = format!("{:.0}", amount);
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result.chars().rev().collect()
}

#[component]
pub fn Properties() -> Element {
    let mut properties: Signal<Vec<Property>> = use_signal(|| Vec::new());
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // ✅ Fetch from PUBLIC endpoint - no auth header needed
    use_effect(move || {
        let mut props_sig = properties;
        let mut loading_sig = loading;
        let mut error_sig = error;

        spawn(async move {
            let client = reqwest::Client::new();
            let resp = client
                .get(&format!("{}/api/public/properties", API_BASE))
                // ❌ NO Authorization header - this is a public endpoint
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    match r.json::<Vec<Property>>().await {
                        Ok(data) => props_sig.set(data),
                        Err(_) => error_sig.set(Some("Failed to parse properties".to_string())),
                    }
                }
                Ok(r) => {
                    error_sig.set(Some(format!("Error: {}", r.status())));
                }
                Err(e) => {
                    error_sig.set(Some(format!("Network error: {}", e)));
                }
            }
            loading_sig.set(false);
        });
    });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            // Hero header
            div { class: "bg-gradient-to-r from-blue-600 to-indigo-700 text-white",
                div { class: "max-w-6xl mx-auto px-4 py-12",
                    h1 { class: "text-4xl font-bold mb-2", "🏠 Browse Properties" }
                    p { class: "text-blue-100 text-lg",
                        "Discover your next home. Request a virtual tour to see it in person — recorded live by our verified agents."
                    }
                }
            }

            // Content
            div { class: "max-w-6xl mx-auto px-4 py-8",
                if *loading.read() {
                    div { class: "flex items-center justify-center py-12",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
                    }
                } else if let Some(err) = error.read().as_ref() {
                    div { class: "bg-red-50 border border-red-200 rounded-lg p-6 text-center",
                        p { class: "text-red-600", "{err}" }
                    }
                } else if properties.read().is_empty() {
                    div { class: "bg-white rounded-xl shadow-sm p-12 text-center",
                        div { class: "text-6xl mb-4", "🏘️" }
                        h2 { class: "text-xl font-bold text-gray-900 mb-2", "No Properties Available" }
                        p { class: "text-gray-600", "Check back soon for new listings." }
                    }
                } else {
                    div {
                        p { class: "text-gray-600 mb-6",
                            "{properties.read().len()} properties available"
                        }
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for property in properties.read().iter() {
                                PropertyCard { property: property.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PropertyCard(property: Property) -> Element {
    // ✅ Pre-compute formatted values before rsx! block
    let price_display: String = format_currency(property.price);
    let location_display: String = if property.location.is_empty() {
        "Location not specified".to_string()
    } else {
        property.location.clone()
    };
    let type_display: String = if property.property_type.is_empty() {
        "Property".to_string()
    } else {
        let mut chars = property.property_type.chars();
        match chars.next() {
            None => String::new(),
            Some(f) => f.to_uppercase().to_string() + chars.as_str(),
        }
    };

    rsx! {
        Link {
            to: Route::PropertyDetailPage { property_id: property.id.clone() },
            class: "group bg-white rounded-xl shadow-sm hover:shadow-xl transition-all duration-300 overflow-hidden border border-gray-100 hover:border-blue-300 cursor-pointer block",

            // Image placeholder
            div { class: "h-48 bg-gradient-to-br from-blue-100 to-purple-100 flex items-center justify-center group-hover:from-blue-200 group-hover:to-purple-200 transition-all",
                span { class: "text-5xl", "🏠" }
            }

            // Content
            div { class: "p-5",
                div { class: "flex justify-between items-start mb-2",
                    h3 { class: "font-bold text-lg text-gray-900 group-hover:text-blue-600 transition-colors line-clamp-1",
                        "{property.title}"
                    }
                    span { class: "px-2 py-1 bg-green-100 text-green-700 text-xs rounded-full font-medium capitalize whitespace-nowrap",
                        "{property.status}"
                    }
                }

                // Location
                p { class: "text-gray-500 text-sm mb-3 flex items-center gap-1",
                    span { "📍" }
                    span { class: "line-clamp-1", "{location_display}" }
                }

                // Stats
                div { class: "flex gap-3 text-sm text-gray-600 mb-4",
                    if property.bedrooms > 0 {
                        span { "🛏️ {property.bedrooms}" }
                    }
                    if property.bathrooms > 0 {
                        span { "🚿 {property.bathrooms}" }
                    }
                    if property.area_sqft > 0 {
                        span { "📐 {property.area_sqft} sqft" }
                    }
                    span { class: "text-gray-400", "• {type_display}" }
                }

                // Price & CTA
                div { class: "flex justify-between items-center pt-3 border-t border-gray-100",
                    p { class: "font-bold text-blue-600 text-lg",
                        "KES {price_display}"
                    }
                    span { class: "text-blue-600 text-sm font-medium group-hover:translate-x-1 transition-transform",
                        "View →"
                    }
                }
            }
        }
    }
}