use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState, StatusBadge};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_properties, create_property, CreatePropertyRequest, Property};

#[component]
pub fn PropertiesPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let user_role = auth.read().user.as_ref()
        .map(|u| u.role.to_uppercase())
        .unwrap_or_default();

    let mut properties = use_signal(|| Vec::<Property>::new());
    let mut loading = use_signal(|| true);
    let mut show_form = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);

    // Form state
    let mut title = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut price = use_signal(|| String::new());
    let mut property_type = use_signal(|| "apartment".to_string());
    let mut location = use_signal(|| String::new());
    let mut county = use_signal(|| String::new());
    let mut submitting = use_signal(|| false);

    // Only Property Owners and Admins can add properties
    let can_add_property = user_role == "PROPERTY_OWNER"
        || user_role == "ADMIN"
        || user_role == "SUPERUSER";

    // FIX 1: Clone token for the effect, then clone AGAIN inside the closure for the async block
    let effect_token = token.clone();
    use_effect(move || {
        let t = effect_token.clone(); // Clone inside FnMut to avoid move error
        spawn(async move {
            match get_properties(&t).await {
                Ok(props) => properties.set(props),
                Err(e) => error.set(Some(format!("Failed to load: {}", e))),
            }
            loading.set(false);
        });
    });

    // FIX 2: Clone token for the submit handler
    let submit_token = token.clone();
    let handle_submit = move |_| {
        if title.read().is_empty() || price.read().is_empty() || location.read().is_empty() {
            error.set(Some("Please fill in all required fields".to_string()));
            return;
        }

        submitting.set(true);
        error.set(None);

        // Clone everything needed for the async block
        let t = submit_token.clone();
        let req = CreatePropertyRequest {
            title: title.read().clone(),
            description: if description.read().is_empty() {
                None
            } else {
                Some(description.read().clone())
            },
            price: price.read().parse().unwrap_or(0.0),
            property_type: property_type.read().clone(),
            location: location.read().clone(),
            county: if county.read().is_empty() {
                None
            } else {
                Some(county.read().clone())
            },
        };

        spawn(async move {
            let result = create_property(&t, &req).await;
            match result {
                Ok(_) => {
                    // Refresh the list
                    if let Ok(props) = get_properties(&t).await {
                        properties.set(props);
                    }
                    show_form.set(false);
                    title.set(String::new());
                    description.set(String::new());
                    price.set(String::new());
                    location.set(String::new());
                    county.set(String::new());
                    property_type.set("apartment".to_string());
                }
                Err(e) => {
                    error.set(Some(format!("Failed to create: {}", e)));
                }
            }
            submitting.set(false);
        });
    };

    rsx! {
        div { class: "space-y-6",
            div { class: "flex items-center justify-between",
                PageHeader {
                    title: "Properties".to_string(),
                    subtitle: format!("{} total listings", properties.read().len())
                }
                if can_add_property {
                    button {
                        class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors",
                        onclick: move |_| {
                            // FIX 3: Read into a local variable first to avoid mutable/immutable borrow conflict
                            let is_open = *show_form.read();
                            show_form.set(!is_open);
                            error.set(None);
                        },
                        if *show_form.read() { "Cancel" } else { "+ Add Property" }
                    }
                }
            }

            // Add Property Form
            if *show_form.read() && can_add_property {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 space-y-4",
                    h3 { class: "text-lg font-semibold text-white mb-4", "Add New Property" }

                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Title *" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                placeholder: "e.g., Modern 2BR Apartment",
                                value: "{title}",
                                oninput: move |evt| title.set(evt.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Price (KES) *" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                r#type: "number",
                                placeholder: "50000",
                                value: "{price}",
                                oninput: move |evt| price.set(evt.value()),
                            }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Description" }
                        textarea {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                            rows: "3",
                            placeholder: "Describe the property...",
                            value: "{description}",
                            oninput: move |evt| description.set(evt.value()),
                        }
                    }

                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Property Type" }
                            select {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                value: "{property_type}",
                                onchange: move |evt| property_type.set(evt.value()),
                                option { value: "apartment", "Apartment" }
                                option { value: "house", "House" }
                                option { value: "villa", "Villa" }
                                option { value: "studio", "Studio" }
                                option { value: "commercial", "Commercial" }
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Location *" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                placeholder: "e.g., Westlands",
                                value: "{location}",
                                oninput: move |evt| location.set(evt.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "County" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                placeholder: "e.g., Nairobi",
                                value: "{county}",
                                oninput: move |evt| county.set(evt.value()),
                            }
                        }
                    }

                    if let Some(err) = error.read().as_ref() {
                        p { class: "text-red-400 text-sm", "{err}" }
                    }

                    div { class: "flex gap-3",
                        button {
                            class: "px-6 py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50",
                            disabled: *submitting.read(),
                            onclick: handle_submit,
                            if *submitting.read() { "Creating..." } else { "Create Property" }
                        }
                        button {
                            class: "px-6 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium transition-colors",
                            onclick: move |_| {
                                show_form.set(false);
                                error.set(None);
                            },
                            "Cancel"
                        }
                    }
                }
            }

            // Properties List
            if *loading.read() {
                div { class: "text-center py-12",
                    p { class: "text-gray-400", "Loading properties..." }
                }
            } else if properties.read().is_empty() {
                EmptyState {
                    icon: "🏠".to_string(),
                    title: "No Properties Yet".to_string(),
                    message: if can_add_property {
                        "Click 'Add Property' to create your first listing.".to_string()
                    } else {
                        "No properties have been added by your converted owners yet.".to_string()
                    },
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                    for property in properties.read().iter() {
                        PropertyCard { property: property.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn PropertyCard(property: Property) -> Element {
    rsx! {
        Link {
            to: crate::AdminRoute::PropertyDetailPage { id: property.id.clone() },
            class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden hover:border-blue-500 transition-colors",
            div { class: "h-40 bg-gradient-to-br from-blue-600/20 to-purple-600/20 flex items-center justify-center",
                span { class: "text-5xl", "🏠" }
            }
            div { class: "p-5",
                div { class: "flex items-start justify-between mb-2",
                    h3 { class: "text-lg font-semibold text-white line-clamp-1 flex-1", "{property.title}" }
                    StatusBadge { status: property.status.clone() }
                }
                p { class: "text-2xl font-bold text-blue-400 mb-3",
                    "KES {property.price as i64}"
                }
                div { class: "space-y-2 text-sm text-gray-400",
                    div { class: "flex items-center gap-2",
                        span { "📍" }
                        span { class: "line-clamp-1", "{property.location}" }
                    }
                    div { class: "flex items-center gap-2",
                        span { "🏷️" }
                        span { "{property.property_type}" }
                    }
                    div { class: "flex items-center gap-2",
                        span { "👤" }
                        span { class: "line-clamp-1", "{property.owner}" }
                    }
                }
            }
        }
    }
}