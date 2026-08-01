use dioxus::prelude::*;
// use dioxus_router::prelude::*;

use crate::AdminRoute;
use crate::api::admin::{get_properties, Property};
use crate::components::sidebar::{PageHeader, StatusBadge, FilterBar, DataTable, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn PropertiesPage() -> Element {
    let auth = use_admin_auth();
    let nav = use_navigator();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_resource = token.clone();
    let mut search = use_signal(|| String::new());
    let mut status_filter = use_signal(|| "all".to_string());
    let mut type_filter = use_signal(|| "all".to_string());

    let properties = use_resource(move || {
        let t = token_for_resource.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![]);
            }
            get_properties(&t).await
        }
    });

    let props_ref = properties.read();
    let props_data = match props_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    let filtered: Vec<Property> = props_data.clone().unwrap_or_default().into_iter().filter(|p| {
        let s = search.read().to_lowercase();
        let st = status_filter.read().clone();
        let ty = type_filter.read().clone();
        let matches_search = s.is_empty() || p.title.to_lowercase().contains(&s) || p.location.to_lowercase().contains(&s);
        let matches_status = st == "all" || p.status.to_lowercase() == st;
        let matches_type = ty == "all" || p.property_type.to_lowercase() == ty;
        matches_search && matches_status && matches_type
    }).collect();

    rsx! {
        div { class: "space-y-4",
            PageHeader { title: "Properties".to_string(), subtitle: "Manage real estate listings".to_string() }

            FilterBar {
                input {
                    class: "px-3 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-white placeholder-gray-500 focus:outline-none focus:border-blue-500",
                    placeholder: "Search properties...",
                    value: "{search}",
                    oninput: move |evt| search.set(evt.value()),
                }
                select {
                    class: "px-3 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-white focus:outline-none focus:border-blue-500",
                    value: "{status_filter}",
                    onchange: move |evt| status_filter.set(evt.value()),
                    option { value: "all", "All Status" }
                    option { value: "active", "Active" }
                    option { value: "pending", "Pending" }
                    option { value: "sold", "Sold" }
                    option { value: "inactive", "Inactive" }
                }
                select {
                    class: "px-3 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-white focus:outline-none focus:border-blue-500",
                    value: "{type_filter}",
                    onchange: move |evt| type_filter.set(evt.value()),
                    option { value: "all", "All Types" }
                    option { value: "house", "House" }
                    option { value: "apartment", "Apartment" }
                    option { value: "condo", "Condo" }
                    option { value: "commercial", "Commercial" }
                }
            }

            if let Some(data) = &props_data {
                if data.is_empty() {
                    EmptyState { icon: "🏠".to_string(), title: "No properties found".to_string(), message: "Properties will appear here once listed.".to_string() }
                } else {
                    DataTable {
                        headers: vec!["Property".to_string(), "Location".to_string(), "Type".to_string(), "Price".to_string(), "Status".to_string(), "Owner".to_string(), "Actions".to_string()],
                        if filtered.is_empty() {
                            tr {
                                td { colspan: "7", class: "px-4 py-8 text-center text-gray-400",
                                    "No properties match your filters"
                                }
                            }
                        } else {
                            for prop in filtered {
                                tr { class: "hover:bg-gray-700/30 transition-colors",
                                    td { class: "px-4 py-3",
                                        div { class: "flex items-center gap-3",
                                            div { class: "w-10 h-10 rounded-lg bg-gray-700 flex items-center justify-center text-lg",
                                                "🏠"
                                            }
                                            div {
                                                p { class: "text-white text-sm font-medium", "{prop.title}" }
                                                p { class: "text-gray-500 text-xs", "{prop.bedrooms} bed • {prop.bathrooms} bath • {prop.area_sqft} sqft" }
                                            }
                                        }
                                    }
                                    td { class: "px-4 py-3 text-gray-300 text-sm", "{prop.location}" }
                                    td { class: "px-4 py-3 text-gray-300 text-sm", "{prop.property_type}" }
                                    td { class: "px-4 py-3 text-white text-sm font-medium", "${prop.price:.0}" }
                                    td { class: "px-4 py-3",
                                        StatusBadge { status: prop.status.clone() }
                                    }
                                    td { class: "px-4 py-3 text-gray-300 text-sm", "{prop.owner}" }
                                    td { class: "px-4 py-3",
                                        div { class: "flex items-center gap-2",
                                            button {
                                                class: "text-xs px-2 py-1 rounded bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 border border-blue-500/20 transition-colors",
                                                onclick: {
                                                    let id = prop.id.clone();
                                                    move |_| { let _ = nav.push(AdminRoute::PropertyDetailPage { id: id.clone() }); }
                                                },
                                                "View"
                                            }
                                            button { class: "text-xs px-2 py-1 rounded bg-gray-700 text-gray-300 hover:bg-gray-600 transition-colors", "Edit" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if props_ref.as_ref().is_none() {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 animate-pulse",
                    for _ in 0..5 {
                        div { class: "px-4 py-3 flex items-center gap-4",
                            div { class: "w-10 h-10 bg-gray-700 rounded-lg" }
                            div { class: "flex-1 space-y-2",
                                div { class: "h-4 bg-gray-700 rounded w-1/4" }
                                div { class: "h-3 bg-gray-700 rounded w-1/3" }
                            }
                        }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load properties" }
                }
            }
        }
    }
}
