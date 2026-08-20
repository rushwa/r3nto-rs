use dioxus::prelude::*;
use crate::context::admin_auth::use_admin_auth;
use crate::components::location_selector::{LocationSelector, LocationSelection};
use crate::components::map_picker::{MapPicker, GeoLocation};
use crate::components::unit_form::UnitForm;

const API_BASE_URL: &str = "http://localhost:8000";

#[derive(Clone, Debug, Default,PartialEq)]
pub struct PropertyFormData {
    pub id: Option<String>,
    pub title: String,
    pub description: String,
    pub price: String,
    pub property_type: String,
    pub status: String,
    pub bedrooms: i32,
    pub bathrooms: i32,
    pub area_sqft: i32,
    pub location: LocationSelection,
    pub geolocation: GeoLocation,
    pub images: Vec<String>,
}

#[component]
pub fn PropertyForm(
    initial_data: Option<PropertyFormData>,
    on_saved: EventHandler<String>,  // Returns property ID
    on_cancel: EventHandler<()>,
) -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut form = use_signal(|| initial_data.clone().unwrap_or_default());
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);
    let mut show_unit_form = use_signal(|| false);
    let mut units = use_signal(|| Vec::<serde_json::Value>::new());

    // Load existing units if editing
    let token_units = token.clone();
    let prop_id = initial_data.as_ref().and_then(|d| d.id.clone());
    use_effect(move || {
        let t = token_units.clone();
        let pid = prop_id.clone();
        let mut units_sig = units;

        if let Some(id) = pid {
            spawn(async move {
                if let Ok(resp) = reqwest::Client::new()
                    .get(&format!("{}/admin/properties/{}/units", API_BASE_URL, id))
                    .header("Authorization", format!("Bearer {}", t))
                    .send().await
                {
                    if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                        units_sig.set(data);
                    }
                }
            });
        }
    });

    let save_property = {
        let token = token.clone();
        move |_: MouseEvent| {
            let t = token.clone();
            let f = form.read().clone();
            let mut saving_sig = saving;
            let mut error_sig = error;
            let mut on_saved = on_saved.clone();

            if f.title.is_empty() {
                error_sig.set(Some("Property title is required".to_string()));
                return;
            }

            if f.location.country_id.is_none() || f.location.county_id.is_none() {
                error_sig.set(Some("Please select country and county".to_string()));
                return;
            }

            spawn(async move {
                saving_sig.set(true);
                error_sig.set(None);

                let price: Option<f64> = f.price.parse().ok();

                let body = serde_json::json!({
                    "id": f.id,
                    "title": f.title,
                    "description": if f.description.is_empty() { None } else { Some(f.description.clone()) },
                    "price": price,
                    "property_type": f.property_type,
                    "status": f.status,
                    "bedrooms": f.bedrooms,
                    "bathrooms": f.bathrooms,
                    "area_sqft": if f.area_sqft > 0 { Some(f.area_sqft) } else { None },
                    "country_id": f.location.country_id,
                    "county_id": f.location.county_id,
                    "constituency_id": f.location.constituency_id,
                    "ward_id": f.location.ward_id,
                    "location_id": f.location.location_id,
                    "village": if f.location.village.is_empty() { None } else { Some(f.location.village.clone()) },
                    "latitude": if f.geolocation.latitude != 0.0 { Some(f.geolocation.latitude) } else { None },
                    "longitude": if f.geolocation.longitude != 0.0 { Some(f.geolocation.longitude) } else { None },
                    "map_address": if f.geolocation.map_address.is_empty() { None } else { Some(f.geolocation.map_address.clone()) },
                    "images": f.images,
                });

                let url = if f.id.is_some() {
                    format!("{}/admin/properties/update", API_BASE_URL)
                } else {
                    format!("{}/admin/properties/create", API_BASE_URL)
                };

                let resp = reqwest::Client::new()
                    .post(&url)
                    .header("Authorization", format!("Bearer {}", t))
                    .json(&body)
                    .send()
                    .await;

                match resp {
                    Ok(r) if r.status().is_success() => {
                        if let Ok(data) = r.json::<serde_json::Value>().await {
                            let prop_id = data.get("property_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string();
                            on_saved.call(prop_id);
                        }
                    }
                    Ok(r) => {
                        let err = r.text().await.unwrap_or_else(|_| "Failed to save property".to_string());
                        error_sig.set(Some(err));
                    }
                    Err(e) => {
                        error_sig.set(Some(format!("Network error: {}", e)));
                    }
                }
                saving_sig.set(false);
            });
        }
    };

    let on_location_change = {
        move |new_loc: LocationSelection| {
            let mut f = form.write();
            f.location = new_loc;
        }
    };

    let on_geolocation_change = {
        move |new_geo: GeoLocation| {
            let mut f = form.write();
            f.geolocation = new_geo;
        }
    };

    let on_unit_saved = {
        let prop_id = initial_data.as_ref().and_then(|d| d.id.clone());
        move |_: ()| {
            show_unit_form.set(false);
            // Reload units
            if let Some(pid) = prop_id.clone() {
                let token = token.clone();
                let mut units_sig = units;
                spawn(async move {
                    if let Ok(resp) = reqwest::Client::new()
                        .get(&format!("{}/admin/properties/{}/units", API_BASE_URL, pid))
                        .header("Authorization", format!("Bearer {}", token))
                        .send().await
                    {
                        if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                            units_sig.set(data);
                        }
                    }
                });
            }
        }
    };

    let current_form = form.read().clone();
    let is_saving = *saving.read();
    let is_editing = current_form.id.is_some();
    let units_list = units.read().clone();

    rsx! {
        div { class: "space-y-6",
            // Error message
            if let Some(err) = error.read().as_ref() {
                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-4",
                    p { class: "text-red-400", "❌ {err}" }
                }
            }

            // ─── Basic Information ───
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h3 { class: "text-white font-bold text-lg mb-4", "📋 Basic Information" }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    // Title
                    div { class: "md:col-span-2",
                        label { class: "block text-gray-400 text-sm mb-1", "Property Title *" }
                        input {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            placeholder: "e.g., Modern 3BR Apartment in Kilimani",
                            value: "{current_form.title}",
                            oninput: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.title = e.value();
                            },
                        }
                    }

                    // Property Type
                    div {
                        label { class: "block text-gray-400 text-sm mb-1", "Property Type" }
                        select {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            value: "{current_form.property_type}",
                            onchange: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.property_type = e.value();
                            },
                            option { value: "apartment", "Apartment" }
                            option { value: "house", "House" }
                            option { value: "maisonette", "Maisonette" }
                            option { value: "bungalow", "Bungalow" }
                            option { value: "commercial", "Commercial" }
                            option { value: "land", "Land" }
                            option { value: "office", "Office" }
                        }
                    }

                    // Status
                    div {
                        label { class: "block text-gray-400 text-sm mb-1", "Status" }
                        select {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            value: "{current_form.status}",
                            onchange: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.status = e.value();
                            },
                            option { value: "available", "Available" }
                            option { value: "occupied", "Occupied" }
                            option { value: "reserved", "Reserved" }
                            option { value: "maintenance", "Under Maintenance" }
                        }
                    }

                    // Price
                    div {
                        label { class: "block text-gray-400 text-sm mb-1", "💰 Price (KES)" }
                        input {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            r#type: "number",
                            placeholder: "e.g., 25000",
                            value: "{current_form.price}",
                            oninput: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.price = e.value();
                            },
                        }
                    }

                    // Area
                    div {
                        label { class: "block text-gray-400 text-sm mb-1", "📐 Area (sq ft)" }
                        input {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            r#type: "number",
                            placeholder: "e.g., 1200",
                            value: if current_form.area_sqft > 0 { current_form.area_sqft.to_string() } else { String::new() },
                            oninput: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.area_sqft = e.value().parse().unwrap_or(0);
                            },
                        }
                    }

                    // Bedrooms
                    div {
                        label { class: "block text-gray-400 text-sm mb-1", "🛏️ Bedrooms" }
                        input {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            r#type: "number",
                            min: "0",
                            value: "{current_form.bedrooms}",
                            oninput: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.bedrooms = e.value().parse().unwrap_or(0);
                            },
                        }
                    }

                    // Bathrooms
                    div {
                        label { class: "block text-gray-400 text-sm mb-1", "🚿 Bathrooms" }
                        input {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            r#type: "number",
                            min: "0",
                            value: "{current_form.bathrooms}",
                            oninput: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.bathrooms = e.value().parse().unwrap_or(0);
                            },
                        }
                    }

                    // Description
                    div { class: "md:col-span-2",
                        label { class: "block text-gray-400 text-sm mb-1", "Description" }
                        textarea {
                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                            rows: "4",
                            placeholder: "Describe your property...",
                            value: "{current_form.description}",
                            oninput: move |e: Event<FormData>| {
                                let mut f = form.write();
                                f.description = e.value();
                            },
                        }
                    }
                }
            }

            // ─── Location ───
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h3 { class: "text-white font-bold text-lg mb-4", "📍 Location" }
                LocationSelector {
                    selection: current_form.location.clone(),
                    on_change: on_location_change,
                }
            }

            // ─── Geolocation ───
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h3 { class: "text-white font-bold text-lg mb-4", "🗺️ Geolocation" }
                MapPicker {
                    location: current_form.geolocation.clone(),
                    on_change: on_geolocation_change,
                }
            }

            // ─── Units (only for existing properties) ───
            if is_editing {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                    div { class: "flex items-center justify-between mb-4",
                        h3 { class: "text-white font-bold text-lg", "🏢 Property Units" }
                        button {
                            class: "bg-blue-600 hover:bg-blue-500 text-white px-4 py-2 rounded-lg text-sm font-medium",
                            onclick: move |_| show_unit_form.set(true),
                            "➕ Add Unit"
                        }
                    }

                    if *show_unit_form.read() {
                        if let Some(pid) = current_form.id.clone() {
                            UnitForm {
                                property_id: pid,
                                on_saved: on_unit_saved,
                                on_cancel: move |_| show_unit_form.set(false),
                            }
                        }
                    }

                    if units_list.is_empty() && !*show_unit_form.read() {
                        div { class: "text-center py-8",
                            p { class: "text-gray-400", "No units added yet. Click \"Add Unit\" to create apartments, rooms, or spaces within this property." }
                        }
                    } else {
                        div { class: "space-y-3 mt-4",
                            for unit in units_list.iter() {
                                UnitCard { unit: unit.clone() }
                            }
                        }
                    }
                }
            }

            // ─── Action Buttons ───
            div { class: "flex gap-3",
                button {
                    class: "flex-1 bg-gray-600 hover:bg-gray-500 text-white font-bold py-3 px-4 rounded-lg",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    class: if is_saving {
                        "flex-1 bg-gray-600 text-gray-400 font-bold py-3 px-4 rounded-lg cursor-not-allowed"
                    } else {
                        "flex-1 bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 px-4 rounded-lg"
                    },
                    disabled: is_saving,
                    onclick: save_property,
                    if is_saving { "Saving..." } else if is_editing { "💾 Update Property" } else { "💾 Create Property" }
                }
            }
        }
    }
}

#[component]
fn UnitCard(unit: serde_json::Value) -> Element {
    let unit_number = unit.get("unit_number").and_then(|v| v.as_str()).unwrap_or("");
    let unit_type = unit.get("unit_type").and_then(|v| v.as_str()).unwrap_or("");
    let bedrooms = unit.get("bedrooms").and_then(|v| v.as_i64()).unwrap_or(0);
    let bathrooms = unit.get("bathrooms").and_then(|v| v.as_i64()).unwrap_or(0);
    let price = unit.get("price").and_then(|v| v.as_str()).unwrap_or("0");
    let status = unit.get("status").and_then(|v| v.as_str()).unwrap_or("available");

    let status_color = match status {
        "available" => "bg-green-500/20 text-green-400",
        "occupied" => "bg-blue-500/20 text-blue-400",
        "reserved" => "bg-yellow-500/20 text-yellow-400",
        _ => "bg-gray-500/20 text-gray-400",
    };

    rsx! {
        div { class: "bg-gray-700/50 rounded-lg p-4 border border-gray-600",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-3",
                    div { class: "w-10 h-10 bg-blue-600/20 rounded-lg flex items-center justify-center",
                        span { class: "text-blue-400 font-bold", "{unit_number.chars().next().unwrap_or('U')}" }
                    }
                    div {
                        p { class: "text-white font-semibold", "Unit {unit_number}" }
                        p { class: "text-gray-400 text-sm capitalize", "{unit_type}" }
                    }
                }
                span { class: "px-2 py-1 rounded-full text-xs {status_color}", "{status}" }
            }
            div { class: "grid grid-cols-3 gap-4 mt-3 text-sm",
                div {
                    p { class: "text-gray-500 text-xs", "Bedrooms" }
                    p { class: "text-white", "{bedrooms}" }
                }
                div {
                    p { class: "text-gray-500 text-xs", "Bathrooms" }
                    p { class: "text-white", "{bathrooms}" }
                }
                div {
                    p { class: "text-gray-500 text-xs", "Price" }
                    p { class: "text-white", "KES {price}" }
                }
            }
        }
    }
}