use dioxus::prelude::*;

use crate::api::admin::{get_property_detail, PropertyDetail};
use crate::components::sidebar::{PageHeader, StatusBadge};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn PropertyDetailPage(id: String) -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let prop_id = id.clone();

    let property = use_resource(move || {
        let t = token.clone();
        let pid = prop_id.clone();
        async move {
            if t.is_empty() {
                return Ok(None::<PropertyDetail>);
            }
            match get_property_detail(&t, &pid).await {
                Ok(p) => Ok(Some(p)),
                Err(e) => Err(e),
            }
        }
    });

    let prop_ref = property.read();
    let prop_data = match prop_ref.as_ref() {
        Some(Ok(Some(d))) => Some(d.clone()),
        _ => None,
    };

    rsx! {
        div { class: "space-y-6",
            if let Some(p) = &prop_data {
                PageHeader { title: p.title.clone(), subtitle: format!("{} • Listed {}", p.location, p.listing_date) }

                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                    div { class: "lg:col-span-2 space-y-6",
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                            div { class: "h-64 bg-gray-700 flex items-center justify-center text-6xl",
                                "🏠"
                            }
                            if !p.images.is_empty() {
                                div { class: "p-3 flex gap-2 overflow-x-auto",
                                    for _img in p.images.iter() {
                                        div { class: "w-20 h-20 bg-gray-700 rounded flex-shrink-0 flex items-center justify-center text-2xl",
                                            "🏠"
                                        }
                                    }
                                }
                            }
                        }

                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                            h3 { class: "text-white font-semibold mb-3", "Property Details" }
                            div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                                div { class: "text-center p-3 bg-gray-900 rounded-lg",
                                    p { class: "text-2xl font-bold text-white", "{p.bedrooms}" }
                                    p { class: "text-gray-500 text-xs", "Bedrooms" }
                                }
                                div { class: "text-center p-3 bg-gray-900 rounded-lg",
                                    p { class: "text-2xl font-bold text-white", "{p.bathrooms}" }
                                    p { class: "text-gray-500 text-xs", "Bathrooms" }
                                }
                                div { class: "text-center p-3 bg-gray-900 rounded-lg",
                                    p { class: "text-2xl font-bold text-white", "{p.area_sqft}" }
                                    p { class: "text-gray-500 text-xs", "Sq Ft" }
                                }
                                div { class: "text-center p-3 bg-gray-900 rounded-lg",
                                    p { class: "text-2xl font-bold text-white", "${p.price:.0}" }
                                    p { class: "text-gray-500 text-xs", "Price" }
                                }
                            }
                            div { class: "mt-4",
                                h4 { class: "text-sm font-medium text-gray-400 mb-2", "Description" }
                                p { class: "text-gray-300 text-sm leading-relaxed", "{p.description}" }
                            }
                            if !p.features.is_empty() {
                                div { class: "mt-4",
                                    h4 { class: "text-sm font-medium text-gray-400 mb-2", "Features" }
                                    div { class: "flex flex-wrap gap-2",
                                        for feature in p.features.iter() {
                                            span { class: "px-2.5 py-1 bg-gray-900 rounded text-xs text-gray-300 border border-gray-700",
                                                "{feature}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    div { class: "space-y-6",
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                            h3 { class: "text-white font-semibold mb-3", "Status" }
                            div { class: "flex items-center justify-between",
                                StatusBadge { status: p.status.clone() }
                                span { class: "text-gray-400 text-sm", "{p.views} views" }
                            }
                            div { class: "mt-3 pt-3 border-t border-gray-700",
                                div { class: "flex justify-between text-sm",
                                    span { class: "text-gray-400", "Inquiries" }
                                    span { class: "text-white font-medium", "{p.inquiries}" }
                                }
                            }
                        }

                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                            h3 { class: "text-white font-semibold mb-3", "Owner" }
                            div { class: "flex items-center gap-3",
                                div { class: "w-10 h-10 rounded-full bg-gray-700 flex items-center justify-center text-white font-bold",
                                    {p.owner.name.chars().next().unwrap_or('?').to_string()}
                                }
                                div {
                                    p { class: "text-white text-sm font-medium", "{p.owner.name}" }
                                    p { class: "text-gray-500 text-xs", "{p.owner.email}" }
                                    p { class: "text-gray-500 text-xs", "{p.owner.role}" }
                                }
                            }
                        }

                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                            h3 { class: "text-white font-semibold mb-3", "Actions" }
                            div { class: "space-y-2",
                                button { class: "w-full py-2 bg-blue-600 hover:bg-blue-500 text-white rounded text-sm font-medium transition-colors", "Edit Property" }
                                button { class: "w-full py-2 bg-gray-700 hover:bg-gray-600 text-white rounded text-sm transition-colors", "Mark as Sold" }
                                button { class: "w-full py-2 bg-red-500/10 hover:bg-red-500/20 text-red-400 border border-red-500/20 rounded text-sm transition-colors", "Delete Property" }
                            }
                        }
                    }
                }
            } else if prop_ref.as_ref().is_none() || matches!(prop_ref.as_ref(), Some(Ok(None))) {
                div { class: "flex items-center justify-center py-20",
                    div { class: "animate-pulse flex flex-col items-center",
                        div { class: "w-12 h-12 bg-gray-700 rounded-lg mb-4" }
                        div { class: "h-4 bg-gray-700 rounded w-32" }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load property details" }
                }
            }
        }
    }
}
