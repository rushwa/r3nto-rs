use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState, StatCard};
use crate::context::admin_auth::use_admin_auth;

const API_BASE_URL: &str = "http://localhost:8000";

#[derive(Clone, Debug, Default, PartialEq)]
struct PropertyInfo {
    id: String,
    title: String,
    price: f64,
    status: String,
    owner_name: String,
    location: String,
    property_type: String,
    bedrooms: i32,
    bathrooms: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct UnitFormData {
    unit_number: String,
    unit_type: String,
    bedrooms: String,
    bathrooms: String,
    area_sqft: String,
    price: String,
    floor_number: String,
    description: String,
}

#[component]
pub fn PropertiesPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let user_role = auth.read().user.as_ref()
        .map(|u| u.role.to_uppercase())
        .unwrap_or_default();

    let mut properties = use_signal(|| Vec::<PropertyInfo>::new());
    let mut loading = use_signal(|| true);
    let mut error_msg = use_signal(|| String::new());
    let mut show_form = use_signal(|| false);
    let mut saving = use_signal(|| false);

    // Unit form state
    let mut show_unit_form = use_signal(|| false);
    let mut unit_prop_id = use_signal(|| String::new());
    let mut unit_prop_title = use_signal(|| String::new());
    let mut unit_saving = use_signal(|| false);
    let mut unit_error = use_signal(|| String::new());

    // Unit form fields
    let mut unit_number = use_signal(|| String::new());
    let mut unit_type = use_signal(|| "apartment".to_string());
    let mut unit_bedrooms = use_signal(|| String::new());
    let mut unit_bathrooms = use_signal(|| String::new());
    let mut unit_area = use_signal(|| String::new());
    let mut unit_price = use_signal(|| String::new());
    let mut unit_floor = use_signal(|| String::new());
    let mut unit_desc = use_signal(|| String::new());

    // Property form fields
    let mut form_title = use_signal(|| String::new());
    let mut form_price = use_signal(|| String::new());
    let mut form_type = use_signal(|| "apartment".to_string());
    let mut form_status = use_signal(|| "available".to_string());
    let mut form_bedrooms = use_signal(|| String::new());
    let mut form_bathrooms = use_signal(|| String::new());
    let mut form_area_sqft = use_signal(|| String::new());
    let mut form_description = use_signal(|| String::new());
    let mut form_county = use_signal(|| String::new());
    let mut form_constituency = use_signal(|| String::new());
    let mut form_ward = use_signal(|| String::new());
    let mut form_location = use_signal(|| String::new());
    let mut form_village = use_signal(|| String::new());
    let mut form_latitude = use_signal(|| String::new());
    let mut form_longitude = use_signal(|| String::new());
    let mut form_map_address = use_signal(|| String::new());

    let is_owner = user_role == "PROPERTY_OWNER";

    // ─── Fetch properties ───
    let t1 = token.clone();
    use_effect(move || {
        let t = t1.clone();
        spawn(async move {
            match reqwest::Client::new()
                .get(format!("{}/admin/properties", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => {
                    if let Ok(data) = r.json::<Vec<serde_json::Value>>().await {
                        let props: Vec<PropertyInfo> = data.into_iter().filter_map(|v| {
                            Some(PropertyInfo {
                                id: v.get("id")?.as_str()?.to_string(),
                                title: v.get("title")?.as_str()?.to_string(),
                                price: v.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0),
                                status: v.get("status").and_then(|s| s.as_str()).unwrap_or("available").to_string(),
                                owner_name: v.get("owner_name").and_then(|o| o.as_str()).unwrap_or("").to_string(),
                                location: v.get("location").and_then(|l| l.as_str()).unwrap_or("").to_string(),
                                property_type: v.get("property_type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                                bedrooms: v.get("bedrooms").and_then(|b| b.as_i64()).unwrap_or(0) as i32,
                                bathrooms: v.get("bathrooms").and_then(|b| b.as_i64()).unwrap_or(0) as i32,
                            })
                        }).collect();
                        properties.set(props);
                    }
                }
                Ok(r) => {
                    let err = r.text().await.unwrap_or_else(|_| "Request failed".to_string());
                    error_msg.set(err);
                }
                Err(e) => error_msg.set(format!("Network error: {}", e)),
            }
            loading.set(false);
        });
    });

    // ─── Save property handler ───
    let handle_save = {
        let t = token.clone();
        move |_: MouseEvent| {
            let t = t.clone();
            let title = form_title.read().clone();
            let price_str = form_price.read().clone();
            let ptype = form_type.read().clone();
            let status = form_status.read().clone();
            let beds = form_bedrooms.read().clone();
            let baths = form_bathrooms.read().clone();
            let area = form_area_sqft.read().clone();
            let desc = form_description.read().clone();
            let county = form_county.read().clone();
            let constituency = form_constituency.read().clone();
            let ward = form_ward.read().clone();
            let loc = form_location.read().clone();
            let village = form_village.read().clone();
            let lat = form_latitude.read().clone();
            let lng = form_longitude.read().clone();
            let map_addr = form_map_address.read().clone();

            if title.is_empty() {
                error_msg.set("Property title is required".to_string());
                return;
            }

            saving.set(true);
            error_msg.set(String::new());

            spawn(async move {
                let price: Option<f64> = price_str.parse().ok();
                let bedrooms: i32 = beds.parse().unwrap_or(0);
                let bathrooms: i32 = baths.parse().unwrap_or(0);
                let area_sqft: Option<i32> = area.parse().ok();
                let latitude: Option<f64> = lat.parse().ok();
                let longitude: Option<f64> = lng.parse().ok();

                let body = serde_json::json!({
                    "title": title,
                    "description": if desc.is_empty() { serde_json::Value::Null } else { serde_json::json!(desc) },
                    "price": price,
                    "property_type": ptype,
                    "status": status,
                    "bedrooms": bedrooms,
                    "bathrooms": bathrooms,
                    "area_sqft": area_sqft,
                    "county": if county.is_empty() { serde_json::Value::Null } else { serde_json::json!(county) },
                    "constituency": if constituency.is_empty() { serde_json::Value::Null } else { serde_json::json!(constituency) },
                    "ward": if ward.is_empty() { serde_json::Value::Null } else { serde_json::json!(ward) },
                    "location": if loc.is_empty() { serde_json::Value::Null } else { serde_json::json!(loc) },
                    "village": if village.is_empty() { serde_json::Value::Null } else { serde_json::json!(village) },
                    "latitude": latitude,
                    "longitude": longitude,
                    "map_address": if map_addr.is_empty() { serde_json::Value::Null } else { serde_json::json!(map_addr) },
                });

                match reqwest::Client::new()
                    .post(format!("{}/admin/properties/create", API_BASE_URL))
                    .header("Authorization", format!("Bearer {}", t))
                    .json(&body)
                    .send()
                    .await
                {
                    Ok(r) if r.status().is_success() => {
                        show_form.set(false);
                        form_title.set(String::new());
                        form_price.set(String::new());
                        form_bedrooms.set(String::new());
                        form_bathrooms.set(String::new());
                        form_area_sqft.set(String::new());
                        form_description.set(String::new());
                        form_county.set(String::new());
                        form_constituency.set(String::new());
                        form_ward.set(String::new());
                        form_location.set(String::new());
                        form_village.set(String::new());
                        form_latitude.set(String::new());
                        form_longitude.set(String::new());
                        form_map_address.set(String::new());

                        let t2 = t.clone();
                        spawn(async move {
                            if let Ok(resp) = reqwest::Client::new()
                                .get(format!("{}/admin/properties", API_BASE_URL))
                                .header("Authorization", format!("Bearer {}", t2))
                                .send().await
                            {
                                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                                    let props: Vec<PropertyInfo> = data.into_iter().filter_map(|v| {
                                        Some(PropertyInfo {
                                            id: v.get("id")?.as_str()?.to_string(),
                                            title: v.get("title")?.as_str()?.to_string(),
                                            price: v.get("price").and_then(|p| p.as_f64()).unwrap_or(0.0),
                                            status: v.get("status").and_then(|s| s.as_str()).unwrap_or("available").to_string(),
                                            owner_name: v.get("owner_name").and_then(|o| o.as_str()).unwrap_or("").to_string(),
                                            location: v.get("location").and_then(|l| l.as_str()).unwrap_or("").to_string(),
                                            property_type: v.get("property_type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                                            bedrooms: v.get("bedrooms").and_then(|b| b.as_i64()).unwrap_or(0) as i32,
                                            bathrooms: v.get("bathrooms").and_then(|b| b.as_i64()).unwrap_or(0) as i32,
                                        })
                                    }).collect();
                                    properties.set(props);
                                }
                            }
                        });
                    }
                    Ok(r) => {
                        let err = r.text().await.unwrap_or_else(|_| "Failed to save".to_string());
                        error_msg.set(err);
                    }
                    Err(e) => error_msg.set(format!("Network error: {}", e)),
                }
                saving.set(false);
            });
        }
    };

    // ─── Save unit handler ───
    let handle_save_unit = {
        let t = token.clone();
        move |_: MouseEvent| {
            let t = t.clone();
            let pid = unit_prop_id.read().clone();
            let unum = unit_number.read().clone();
            let utype = unit_type.read().clone();
            let ubeds = unit_bedrooms.read().clone();
            let ubaths = unit_bathrooms.read().clone();
            let uarea = unit_area.read().clone();
            let uprice = unit_price.read().clone();
            let ufloor = unit_floor.read().clone();
            let udesc = unit_desc.read().clone();

            if unum.is_empty() {
                unit_error.set("Unit number is required".to_string());
                return;
            }

            unit_saving.set(true);
            unit_error.set(String::new());

            spawn(async move {
                let body = serde_json::json!({
                    "property_id": pid,
                    "unit": {
                        "unit_number": unum,
                        "unit_type": utype,
                        "bedrooms": ubeds.parse::<i32>().unwrap_or(0),
                        "bathrooms": ubaths.parse::<i32>().unwrap_or(0),
                        "area_sqft": uarea.parse::<i32>().ok(),
                        "price": uprice.parse::<f64>().ok(),
                        "floor_number": ufloor.parse::<i32>().unwrap_or(0),
                        "description": if udesc.is_empty() { serde_json::Value::Null } else { serde_json::json!(udesc) },
                        "features": {},
                    }
                });

                match reqwest::Client::new()
                    .post(format!("{}/admin/properties/units", API_BASE_URL))
                    .header("Authorization", format!("Bearer {}", t))
                    .json(&body)
                    .send()
                    .await
                {
                    Ok(r) if r.status().is_success() => {
                        show_unit_form.set(false);
                        unit_number.set(String::new());
                        unit_bedrooms.set(String::new());
                        unit_bathrooms.set(String::new());
                        unit_area.set(String::new());
                        unit_price.set(String::new());
                        unit_floor.set(String::new());
                        unit_desc.set(String::new());
                    }
                    Ok(r) => {
                        let err = r.text().await.unwrap_or_else(|_| "Failed to save unit".to_string());
                        unit_error.set(err);
                    }
                    Err(e) => unit_error.set(format!("Network error: {}", e)),
                }
                unit_saving.set(false);
            });
        }
    };

    // ─── Pre-compute values ───
    let is_loading = *loading.read();
    let err = error_msg.read().clone();
    let has_error = !err.is_empty();
    let props_list = properties.read().clone();
    let total_props = props_list.len();
    let available_count = props_list.iter().filter(|p| p.status == "available").count();
    let occupied_count = props_list.iter().filter(|p| p.status == "occupied").count();
    let form_visible = *show_form.read();
    let is_saving = *saving.read();
    let unit_form_visible = *show_unit_form.read();
    let unit_is_saving = *unit_saving.read();
    let unit_err = unit_error.read().clone();
    let has_unit_error = !unit_err.is_empty();
    let unit_pid_display = unit_prop_title.read().clone();

    let save_label = if is_saving { "Saving..." } else { "Save Property" };
    let unit_save_label = if unit_is_saving { "Saving..." } else { "Save Unit" };

    let title_val = form_title.read().clone();
    let price_val = form_price.read().clone();
    let type_val = form_type.read().clone();
    let status_val = form_status.read().clone();
    let beds_val = form_bedrooms.read().clone();
    let baths_val = form_bathrooms.read().clone();
    let area_val = form_area_sqft.read().clone();
    let desc_val = form_description.read().clone();
    let county_val = form_county.read().clone();
    let constituency_val = form_constituency.read().clone();
    let ward_val = form_ward.read().clone();
    let loc_val = form_location.read().clone();
    let village_val = form_village.read().clone();
    let lat_val = form_latitude.read().clone();
    let lng_val = form_longitude.read().clone();
    let map_addr_val = form_map_address.read().clone();

    let unum_val = unit_number.read().clone();
    let utype_val = unit_type.read().clone();
    let ubeds_val = unit_bedrooms.read().clone();
    let ubaths_val = unit_bathrooms.read().clone();
    let uarea_val = unit_area.read().clone();
    let uprice_val = unit_price.read().clone();
    let ufloor_val = unit_floor.read().clone();
    let udesc_val = unit_desc.read().clone();

    rsx! {
        div { class: "space-y-6",
            // ─── Header ───
            div { class: "flex items-center justify-between",
                PageHeader {
                    title: "My Properties".to_string(),
                    subtitle: "Manage property listings, units, and locations".to_string(),
                }
                if is_owner {
                    button {
                        class: "bg-blue-600 hover:bg-blue-500 text-white px-6 py-3 rounded-lg font-medium",
                        onclick: move |_| show_form.set(true),
                        "+ Add Property"
                    }
                }
            }

            // ─── Stats ───
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard {
                    title: "Total Properties".to_string(),
                    value: format!("{}", total_props),
                    icon: "🏘️".to_string(),
                    change: "".to_string(),
                    change_positive: true,
                }
                StatCard {
                    title: "Available".to_string(),
                    value: format!("{}", available_count),
                    icon: "✅".to_string(),
                    change: "".to_string(),
                    change_positive: true,
                }
                StatCard {
                    title: "Occupied".to_string(),
                    value: format!("{}", occupied_count),
                    icon: "🔑".to_string(),
                    change: "".to_string(),
                    change_positive: false,
                }
            }

            // ─── Error ───
            if has_error {
                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-4",
                    p { class: "text-red-400", "{err}" }
                }
            }

            // ─── Loading ───
            if is_loading {
                div { class: "flex items-center justify-center py-16",
                    div { class: "animate-spin rounded-full h-10 w-10 border-b-2 border-blue-500" }
                }
            }

            // ─── Empty state ───
            if !is_loading && props_list.is_empty() {
                EmptyState {
                    icon: "🏘️".to_string(),
                    title: "No properties found".to_string(),
                    message: "Add a property to get started.".to_string(),
                }
            }

            // ─── Property grid ───
            if !is_loading && !props_list.is_empty() {
                div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                    for prop in props_list.iter() {
                        // ✅ WRAPPED IN LINK — makes card clickable
                        Link {
                            key: "{prop.id}",
                            to: crate::AdminRoute::PropertyDetailPage { id: prop.id.clone() },
                            class: "bg-gray-800 rounded-lg border border-gray-700 p-5 hover:border-blue-500/50 hover:shadow-lg transition-all cursor-pointer block",
                            div { class: "flex items-start justify-between mb-3",
                                h3 { class: "text-white font-semibold text-lg", "{prop.title}" }
                                span { class: "px-2 py-1 rounded-full text-xs bg-gray-700 text-gray-300", "{prop.status}" }
                            }
                            p { class: "text-gray-400 text-sm mb-3", "📍 {prop.location}" }
                            div { class: "grid grid-cols-3 gap-3 mb-3",
                                div { class: "text-center",
                                    p { class: "text-gray-500 text-xs", "Price" }
                                    p { class: "text-white font-semibold text-sm", "KES {prop.price}" }
                                }
                                div { class: "text-center",
                                    p { class: "text-gray-500 text-xs", "Type" }
                                    p { class: "text-white font-semibold text-sm capitalize", "{prop.property_type}" }
                                }
                                div { class: "text-center",
                                    p { class: "text-gray-500 text-xs", "Beds/Baths" }
                                    p { class: "text-white font-semibold text-sm", "{prop.bedrooms}/{prop.bathrooms}" }
                                }
                            }
                            if !prop.owner_name.is_empty() {
                                p { class: "text-gray-500 text-xs mb-2", "👤 {prop.owner_name}" }
                            }
                            // ✅ ADD UNIT BUTTON (stop propagation so it doesn't navigate)
                            div { class: "flex gap-2 mt-3",
                                button {
                                    class: "flex-1 bg-purple-600/20 hover:bg-purple-600/40 text-purple-300 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
                                    onclick: {
                                        let pid = prop.id.clone();
                                        let ptitle = prop.title.clone();
                                        move |e: MouseEvent| {
                                            e.stop_propagation();
                                            unit_prop_id.set(pid.clone());
                                            unit_prop_title.set(ptitle.clone());
                                            show_unit_form.set(true);
                                        }
                                    },
                                    "🏢 Add Unit"
                                }
                            }
                        }
                    }
                }
            }

            // ─── Create Property Modal ───
            if form_visible {
                div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4 overflow-y-auto",
                    div { class: "bg-gray-900 rounded-xl max-w-3xl w-full my-8 max-h-[90vh] overflow-y-auto",
                        div { class: "p-6",
                            div { class: "flex items-center justify-between mb-6",
                                h2 { class: "text-white text-xl font-bold", "Create New Property" }
                                button {
                                    class: "text-gray-400 hover:text-white text-2xl",
                                    onclick: move |_| show_form.set(false),
                                    "x"
                                }
                            }

                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5 mb-4",
                                h3 { class: "text-white font-bold mb-4", "Basic Information" }
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div { class: "md:col-span-2",
                                        label { class: "block text-gray-400 text-sm mb-1", "Property Title *" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., Modern 3BR Apartment",
                                            value: "{title_val}",
                                            oninput: move |e: Event<FormData>| form_title.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Property Type" }
                                        select {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{type_val}",
                                            onchange: move |e: Event<FormData>| form_type.set(e.value()),
                                            option { value: "apartment", "Apartment" }
                                            option { value: "house", "House" }
                                            option { value: "maisonette", "Maisonette" }
                                            option { value: "bungalow", "Bungalow" }
                                            option { value: "commercial", "Commercial" }
                                            option { value: "land", "Land" }
                                            option { value: "office", "Office" }
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Status" }
                                        select {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{status_val}",
                                            onchange: move |e: Event<FormData>| form_status.set(e.value()),
                                            option { value: "available", "Available" }
                                            option { value: "occupied", "Occupied" }
                                            option { value: "reserved", "Reserved" }
                                            option { value: "maintenance", "Maintenance" }
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Price (KES)" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{price_val}",
                                            oninput: move |e: Event<FormData>| form_price.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Area (sq ft)" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{area_val}",
                                            oninput: move |e: Event<FormData>| form_area_sqft.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Bedrooms" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{beds_val}",
                                            oninput: move |e: Event<FormData>| form_bedrooms.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Bathrooms" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{baths_val}",
                                            oninput: move |e: Event<FormData>| form_bathrooms.set(e.value()),
                                        }
                                    }
                                    div { class: "md:col-span-2",
                                        label { class: "block text-gray-400 text-sm mb-1", "Description" }
                                        textarea {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            rows: "3",
                                            value: "{desc_val}",
                                            oninput: move |e: Event<FormData>| form_description.set(e.value()),
                                        }
                                    }
                                }
                            }

                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5 mb-4",
                                h3 { class: "text-white font-bold mb-4", "Location" }
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "County" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{county_val}",
                                            oninput: move |e: Event<FormData>| form_county.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Constituency" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{constituency_val}",
                                            oninput: move |e: Event<FormData>| form_constituency.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Ward" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{ward_val}",
                                            oninput: move |e: Event<FormData>| form_ward.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Location / Area" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{loc_val}",
                                            oninput: move |e: Event<FormData>| form_location.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Village / Estate" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{village_val}",
                                            oninput: move |e: Event<FormData>| form_village.set(e.value()),
                                        }
                                    }
                                }
                            }

                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5 mb-4",
                                h3 { class: "text-white font-bold mb-4", "Geolocation" }
                                div { class: "grid grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Latitude" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{lat_val}",
                                            oninput: move |e: Event<FormData>| form_latitude.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Longitude" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{lng_val}",
                                            oninput: move |e: Event<FormData>| form_longitude.set(e.value()),
                                        }
                                    }
                                }
                                div { class: "mt-4",
                                    label { class: "block text-gray-400 text-sm mb-1", "Map Address" }
                                    input {
                                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                        value: "{map_addr_val}",
                                        oninput: move |e: Event<FormData>| form_map_address.set(e.value()),
                                    }
                                }
                            }

                            div { class: "flex gap-3",
                                button {
                                    class: "flex-1 bg-gray-600 hover:bg-gray-500 text-white font-bold py-3 px-4 rounded-lg",
                                    onclick: move |_| show_form.set(false),
                                    "Cancel"
                                }
                                button {
                                    class: "flex-1 bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 px-4 rounded-lg",
                                    disabled: is_saving,
                                    onclick: handle_save,
                                    "{save_label}"
                                }
                            }
                        }
                    }
                }
            }

            // ─── Add Unit Modal ───
            if unit_form_visible {
                div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4 overflow-y-auto",
                    div { class: "bg-gray-900 rounded-xl max-w-lg w-full my-8 max-h-[90vh] overflow-y-auto",
                        div { class: "p-6",
                            div { class: "flex items-center justify-between mb-6",
                                div {
                                    h2 { class: "text-white text-xl font-bold", "Add Unit" }
                                    p { class: "text-gray-400 text-sm mt-1", "Property: {unit_pid_display}" }
                                }
                                button {
                                    class: "text-gray-400 hover:text-white text-2xl",
                                    onclick: move |_| show_unit_form.set(false),
                                    "x"
                                }
                            }

                            if has_unit_error {
                                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3 mb-4",
                                    p { class: "text-red-400 text-sm", "{unit_err}" }
                                }
                            }

                            div { class: "space-y-4",
                                div { class: "grid grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Unit Number *" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., A1, G-01",
                                            value: "{unum_val}",
                                            oninput: move |e: Event<FormData>| unit_number.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Unit Type" }
                                        select {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{utype_val}",
                                            onchange: move |e: Event<FormData>| unit_type.set(e.value()),
                                            option { value: "apartment", "Apartment" }
                                            option { value: "bedsitter", "Bedsitter" }
                                            option { value: "single", "Single Room" }
                                            option { value: "studio", "Studio" }
                                            option { value: "maisonette", "Maisonette" }
                                            option { value: "commercial", "Commercial" }
                                            option { value: "office", "Office" }
                                        }
                                    }
                                }
                                div { class: "grid grid-cols-3 gap-4",
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Bedrooms" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{ubeds_val}",
                                            oninput: move |e: Event<FormData>| unit_bedrooms.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Bathrooms" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{ubaths_val}",
                                            oninput: move |e: Event<FormData>| unit_bathrooms.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Floor" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            placeholder: "0",
                                            value: "{ufloor_val}",
                                            oninput: move |e: Event<FormData>| unit_floor.set(e.value()),
                                        }
                                    }
                                }
                                div { class: "grid grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Area (sq ft)" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{uarea_val}",
                                            oninput: move |e: Event<FormData>| unit_area.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Price (KES)" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            value: "{uprice_val}",
                                            oninput: move |e: Event<FormData>| unit_price.set(e.value()),
                                        }
                                    }
                                }
                                div {
                                    label { class: "block text-gray-400 text-sm mb-1", "Description" }
                                    textarea {
                                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                        rows: "2",
                                        value: "{udesc_val}",
                                        oninput: move |e: Event<FormData>| unit_desc.set(e.value()),
                                    }
                                }
                            }

                            div { class: "flex gap-3 mt-6",
                                button {
                                    class: "flex-1 bg-gray-600 hover:bg-gray-500 text-white font-bold py-3 px-4 rounded-lg",
                                    onclick: move |_| show_unit_form.set(false),
                                    "Cancel"
                                }
                                button {
                                    class: "flex-1 bg-purple-600 hover:bg-purple-500 text-white font-bold py-3 px-4 rounded-lg",
                                    disabled: unit_is_saving,
                                    onclick: handle_save_unit,
                                    "{unit_save_label}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}