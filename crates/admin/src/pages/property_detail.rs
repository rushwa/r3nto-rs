use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatusBadge};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_property_detail, PropertyDetail};

#[component]
pub fn PropertyDetailPage(id: String) -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut property = use_signal(|| Option::<PropertyDetail>::None);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);

    let property_id = id.clone();
    let effect_token = token.clone();

    use_effect(move || {
        let t = effect_token.clone();
        let pid = property_id.clone();
        spawn(async move {
            match get_property_detail(&t, &pid).await {
                Ok(detail) => property.set(Some(detail)),
                Err(e) => error.set(Some(format!("Failed to load: {}", e))),
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "space-y-6",
            if *loading.read() {
                div { class: "text-center py-12",
                    p { class: "text-gray-400", "Loading property details..." }
                }
            } else if let Some(err) = error.read().as_ref() {
                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-6",
                    p { class: "text-red-400", "{err}" }
                }
            } else if let Some(prop) = property.read().as_ref() {
                // Header
                div { class: "flex items-start justify-between",
                    div {
                        PageHeader {
                            title: prop.title.clone(),
                            subtitle: format!("Listed on {}", prop.listing_date)
                        }
                    }
                    StatusBadge { status: prop.status.clone() }
                }

                // Main Content Grid
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                    // Left Column - Images and Details
                    div { class: "lg:col-span-2 space-y-6",
                        // Image Gallery
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                            if prop.images.is_empty() {
                                div { class: "h-96 bg-gradient-to-br from-blue-600/20 to-purple-600/20 flex items-center justify-center",
                                    span { class: "text-8xl", "🏠" }
                                }
                            } else {
                                div { class: "h-96 bg-gray-900 flex items-center justify-center",
                                    img {
                                        src: "{prop.images[0]}",
                                        class: "max-h-full object-contain",
                                        alt: "{prop.title}"
                                    }
                                }
                            }
                        }

                        // Description
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                            h3 { class: "text-lg font-semibold text-white mb-3", "Description" }
                            p { class: "text-gray-300 leading-relaxed", "{prop.description}" }
                        }

                        // Features
                        if !prop.features.is_empty() {
                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                                h3 { class: "text-lg font-semibold text-white mb-3", "Features" }
                                div { class: "flex flex-wrap gap-2",
                                    for feature in prop.features.iter() {
                                        span { class: "px-3 py-1 bg-blue-600/20 text-blue-400 rounded-full text-sm", "{feature}" }
                                    }
                                }
                            }
                        }
                    }

                    // Right Column - Key Info
                    div { class: "space-y-6",
                        // Price Card
                        div { class: "bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg p-6",
                            p { class: "text-blue-100 text-sm mb-1", "Asking Price" }
                            p { class: "text-4xl font-bold text-white", "KES {prop.price as i64}" }
                        }

                        // Property Specs
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                            h3 { class: "text-lg font-semibold text-white mb-4", "Property Details" }
                            div { class: "space-y-3",
                                div { class: "flex items-center justify-between",
                                    span { class: "text-gray-400", "Type" }
                                    span { class: "text-white font-medium", "{prop.property_type}" }
                                }
                                div { class: "flex items-center justify-between",
                                    span { class: "text-gray-400", "Bedrooms" }
                                    span { class: "text-white font-medium", "{prop.bedrooms}" }
                                }
                                div { class: "flex items-center justify-between",
                                    span { class: "text-gray-400", "Bathrooms" }
                                    span { class: "text-white font-medium", "{prop.bathrooms}" }
                                }
                                div { class: "flex items-center justify-between",
                                    span { class: "text-gray-400", "Area" }
                                    span { class: "text-white font-medium", "{prop.area_sqft} sqft" }
                                }
                                div { class: "flex items-center justify-between",
                                    span { class: "text-gray-400", "Location" }
                                    span { class: "text-white font-medium text-right", "{prop.location}" }
                                }
                            }
                        }

                        // Owner Info
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                            h3 { class: "text-lg font-semibold text-white mb-4", "Property Owner" }
                            div { class: "space-y-2",
                                div { class: "flex items-center gap-3",
                                    div { class: "w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-500 rounded-full flex items-center justify-center",
                                        span { class: "text-white font-bold text-lg",
                                            "{prop.owner.name.chars().next().unwrap_or('?')}"
                                        }
                                    }
                                    div {
                                        p { class: "text-white font-medium", "{prop.owner.name}" }
                                        p { class: "text-gray-400 text-sm", "{prop.owner.email}" }
                                    }
                                }
                            }
                        }

                        // Stats
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                            h3 { class: "text-lg font-semibold text-white mb-4", "Listing Stats" }
                            div { class: "grid grid-cols-2 gap-4",
                                div { class: "text-center",
                                    p { class: "text-2xl font-bold text-blue-400", "{prop.views}" }
                                    p { class: "text-gray-400 text-sm", "Views" }
                                }
                                div { class: "text-center",
                                    p { class: "text-2xl font-bold text-green-400", "{prop.inquiries}" }
                                    p { class: "text-gray-400 text-sm", "Inquiries" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}