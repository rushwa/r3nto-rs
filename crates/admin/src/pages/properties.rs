use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState, StatCard};
use crate::context::admin_auth::use_admin_auth;

const API_BASE_URL: &str = "http://localhost:8000";

#[derive(Clone, Debug, Default, PartialEq)]
struct PropertyInfo {
    id: String,
    title: String,
    description: Option<String>,
    status: String,
    owner_name: String,
    location: String,
    property_type: String,
    purpose: String,
    is_land: bool,
    plot_size: Option<String>,
    land_price: Option<f64>,
    unit_count: i64,
    min_unit_price: Option<f64>,
    max_unit_price: Option<f64>,
}

#[component]
pub fn PropertiesPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let user_role = auth.read().user.as_ref()
        .map(|u| u.role.to_uppercase())
        .unwrap_or_default();
    let nav = use_navigator();

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
    let mut unit_type = use_signal(|| "one_bedroom".to_string());
    let mut unit_purpose = use_signal(|| "for_rent".to_string());
    let mut unit_bedrooms = use_signal(|| String::new());
    let mut unit_bathrooms = use_signal(|| String::new());
    let mut unit_area = use_signal(|| String::new());
    let mut unit_price = use_signal(|| String::new());
    let mut unit_floor = use_signal(|| String::new());
    let mut unit_desc = use_signal(|| String::new());

    // Property form fields
    let mut form_title = use_signal(|| String::new());
    let mut form_purpose = use_signal(|| "for_rent".to_string());
    let mut form_type = use_signal(|| "apartment".to_string());
    let mut form_status = use_signal(|| "available".to_string());
    let mut form_description = use_signal(|| String::new());
    let mut form_plot_size = use_signal(|| String::new());
    let mut form_land_price = use_signal(|| String::new());
    let mut form_county = use_signal(|| String::new());
    let mut form_constituency = use_signal(|| String::new());
    let mut form_ward = use_signal(|| String::new());
    let mut form_location = use_signal(|| String::new());
    let mut form_village = use_signal(|| String::new());
    let mut form_latitude = use_signal(|| String::new());
    let mut form_longitude = use_signal(|| String::new());
    let mut form_map_address = use_signal(|| String::new());

    let is_owner = user_role == "PROPERTY_OWNER";

    // ─── Helper: parse property from JSON ───
    fn parse_prop(v: &serde_json::Value) -> Option<PropertyInfo> {
        Some(PropertyInfo {
            id: v.get("id")?.as_str()?.to_string(),
            title: v.get("title")?.as_str()?.to_string(),
            description: v.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
            status: v.get("status").and_then(|s| s.as_str()).unwrap_or("available").to_string(),
            owner_name: v.get("owner_name").and_then(|o| o.as_str()).unwrap_or("").to_string(),
            location: v.get("location").and_then(|l| l.as_str()).unwrap_or("").to_string(),
            property_type: v.get("property_type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
            purpose: v.get("purpose").and_then(|p| p.as_str()).unwrap_or("for_rent").to_string(),
            is_land: v.get("is_land").and_then(|l| l.as_bool()).unwrap_or(false),
            plot_size: v.get("plot_size").and_then(|p| p.as_str()).map(|s| s.to_string()),
            land_price: v.get("land_price").and_then(|p| p.as_f64()),
            unit_count: v.get("unit_count").and_then(|u| u.as_i64()).unwrap_or(0),
            min_unit_price: v.get("min_unit_price").and_then(|p| p.as_f64()),
            max_unit_price: v.get("max_unit_price").and_then(|p| p.as_f64()),
        })
    }

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
                        let props: Vec<PropertyInfo> = data.iter().filter_map(parse_prop).collect();
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

    // ─── Reload helper ───
    let reload_properties = {
        let t = token.clone();
        move || {
            let t2 = t.clone();
            spawn(async move {
                if let Ok(resp) = reqwest::Client::new()
                    .get(format!("{}/admin/properties", API_BASE_URL))
                    .header("Authorization", format!("Bearer {}", t2))
                    .send().await
                {
                    if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                        let props: Vec<PropertyInfo> = data.iter().filter_map(parse_prop).collect();
                        properties.set(props);
                    }
                }
            });
        }
    };

    // ─── Save property handler ───
    let handle_save = {
        let t = token.clone();
        let reload = reload_properties.clone();
        move |_: MouseEvent| {
            let t = t.clone();
            let reload = reload.clone();
            let title = form_title.read().clone();
            let purpose = form_purpose.read().clone();
            let ptype = form_type.read().clone();
            let status = form_status.read().clone();
            let desc = form_description.read().clone();
            let plot_size = form_plot_size.read().clone();
            let land_price = form_land_price.read().clone();
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

            let is_land = ptype.to_lowercase() == "land";
            if is_land && land_price.is_empty() {
                error_msg.set("Land must have a price".to_string());
                return;
            }

            saving.set(true);
            error_msg.set(String::new());

            spawn(async move {
                let latitude: Option<f64> = lat.parse().ok();
                let longitude: Option<f64> = lng.parse().ok();
                let land_price_val: Option<f64> = if is_land { land_price.parse().ok() } else { None };

                let body = serde_json::json!({
                    "title": title,
                    "description": if desc.is_empty() { serde_json::Value::Null } else { serde_json::json!(desc) },
                    "purpose": purpose,
                    "property_type": ptype,
                    "status": status,
                    "is_land": is_land,
                    "plot_size": if plot_size.is_empty() { serde_json::Value::Null } else { serde_json::json!(plot_size) },
                    "plot_dimensions": serde_json::Value::Null,
                    "land_price": land_price_val,
                    "county": if county.is_empty() { serde_json::Value::Null } else { serde_json::json!(county) },
                    "constituency": if constituency.is_empty() { serde_json::Value::Null } else { serde_json::json!(constituency) },
                    "ward": if ward.is_empty() { serde_json::Value::Null } else { serde_json::json!(ward) },
                    "location": if loc.is_empty() { serde_json::Value::Null } else { serde_json::json!(loc) },
                    "village": if village.is_empty() { serde_json::Value::Null } else { serde_json::json!(village) },
                    "latitude": latitude,
                    "longitude": longitude,
                    "map_address": if map_addr.is_empty() { serde_json::Value::Null } else { serde_json::json!(map_addr) },
                    "images": []
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
                        form_purpose.set("for_rent".to_string());
                        form_type.set("apartment".to_string());
                        form_status.set("available".to_string());
                        form_description.set(String::new());
                        form_plot_size.set(String::new());
                        form_land_price.set(String::new());
                        form_county.set(String::new());
                        form_constituency.set(String::new());
                        form_ward.set(String::new());
                        form_location.set(String::new());
                        form_village.set(String::new());
                        form_latitude.set(String::new());
                        form_longitude.set(String::new());
                        form_map_address.set(String::new());
                        reload();
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
        let reload = reload_properties.clone();
        move |_: MouseEvent| {
            let t = t.clone();
            let reload = reload.clone();
            let pid = unit_prop_id.read().clone();
            let unum = unit_number.read().clone();
            let utype = unit_type.read().clone();
            let upurpose = unit_purpose.read().clone();
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
                        "purpose": upurpose,
                        "bedrooms": ubeds.parse::<i32>().unwrap_or(0),
                        "bathrooms": ubaths.parse::<i32>().unwrap_or(0),
                        "area_sqft": uarea.parse::<i32>().ok(),
                        "price": uprice.parse::<f64>().ok(),
                        "status": "available",
                        "floor_number": ufloor.parse::<i32>().unwrap_or(0),
                        "description": if udesc.is_empty() { serde_json::Value::Null } else { serde_json::json!(udesc) },
                        "features": {}
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
                        unit_type.set("one_bedroom".to_string());
                        unit_purpose.set("for_rent".to_string());
                        unit_bedrooms.set(String::new());
                        unit_bathrooms.set(String::new());
                        unit_area.set(String::new());
                        unit_price.set(String::new());
                        unit_floor.set(String::new());
                        unit_desc.set(String::new());
                        reload();
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

    // ─── Pre-compute all display values BEFORE rsx! ───
    let is_loading = *loading.read();
    let err = error_msg.read().clone();
    let has_error = !err.is_empty();
    let props_list = properties.read().clone();
    let total_props = props_list.len();
    let total_units: i64 = props_list.iter().map(|p| p.unit_count).sum();
    let land_count = props_list.iter().filter(|p| p.is_land).count();
    let available_count = props_list.iter().filter(|p| p.status == "available").count();
    let form_visible = *show_form.read();
    let is_saving = *saving.read();
    let unit_form_visible = *show_unit_form.read();
    let unit_is_saving = *unit_saving.read();
    let unit_err = unit_error.read().clone();
    let has_unit_error = !unit_err.is_empty();
    let unit_pid_display = unit_prop_title.read().clone();
    let save_label = if is_saving { "Saving..." } else { "Save Property" };
    let unit_save_label = if unit_is_saving { "Saving..." } else { "Save Unit" };
    let is_land_selected = form_type.read().to_lowercase() == "land";

    let title_val = form_title.read().clone();
    let purpose_val = form_purpose.read().clone();
    let type_val = form_type.read().clone();
    let status_val = form_status.read().clone();
    let desc_val = form_description.read().clone();
    let plot_size_val = form_plot_size.read().clone();
    let land_price_val = form_land_price.read().clone();
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
    let upurpose_val = unit_purpose.read().clone();
    let ubeds_val = unit_bedrooms.read().clone();
    let ubaths_val = unit_bathrooms.read().clone();
    let uarea_val = unit_area.read().clone();
    let uprice_val = unit_price.read().clone();
    let ufloor_val = unit_floor.read().clone();
    let udesc_val = unit_desc.read().clone();
    let unit_price_label = if upurpose_val == "for_rent" { "Rent (KES/month) *" } else { "Sale Price (KES) *" };

    rsx! {
        div { class: "space-y-6",
            // ─── Header ───
            div { class: "flex items-center justify-between",
                PageHeader {
                    title: "My Properties".to_string(),
                    subtitle: "Manage buildings, units, and land listings".to_string(),
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
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                StatCard {
                    title: "Properties".to_string(),
                    value: format!("{}", total_props),
                    icon: "🏘️".to_string(),
                    change: "".to_string(),
                    change_positive: true,
                }
                StatCard {
                    title: "Total Units".to_string(),
                    value: format!("{}", total_units),
                    icon: "🏢".to_string(),
                    change: "".to_string(),
                    change_positive: true,
                }
                StatCard {
                    title: "Land Plots".to_string(),
                    value: format!("{}", land_count),
                    icon: "🌍".to_string(),
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
                        PropertyCard {
                            key: "{prop.id}",
                            property: prop.clone(),
                            is_owner: is_owner,
                            on_navigate: {
                                let pid = prop.id.clone();
                                let nav_clone = nav.clone();
                                move |_| {
                                    nav_clone.push(crate::AdminRoute::PropertyDetailPage { id: pid.clone() });
                                }
                            },
                            on_add_unit: {
                                let pid = prop.id.clone();
                                let ptitle = prop.title.clone();
                                move |_| {
                                    unit_prop_id.set(pid.clone());
                                    unit_prop_title.set(ptitle.clone());
                                    show_unit_form.set(true);
                                }
                            },
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

                            // Basic Information
                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5 mb-4",
                                h3 { class: "text-white font-bold mb-4", "Basic Information" }
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div { class: "md:col-span-2",
                                        label { class: "block text-gray-400 text-sm mb-1", "Property Title *" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., Greenwood Gardens Apartments",
                                            value: "{title_val}",
                                            oninput: move |e: Event<FormData>| form_title.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Purpose *" }
                                        select {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{purpose_val}",
                                            onchange: move |e: Event<FormData>| form_purpose.set(e.value()),
                                            option { value: "for_rent", "For Rent" }
                                            option { value: "for_sale", "For Sale" }
                                            option { value: "for_rent_and_sale", "For Rent & Sale" }
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Property Type *" }
                                        select {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{type_val}",
                                            onchange: move |e: Event<FormData>| form_type.set(e.value()),
                                            option { value: "apartment", "Apartment / Flat" }
                                            option { value: "maisonette", "Maisonette" }
                                            option { value: "bungalow", "Bungalow" }
                                            option { value: "villa", "Villa" }
                                            option { value: "townhouse", "Townhouse" }
                                            option { value: "commercial", "Commercial Building" }
                                            option { value: "office", "Office Space" }
                                            option { value: "retail", "Retail / Shop" }
                                            option { value: "warehouse", "Warehouse" }
                                            option { value: "land", "Land / Plot" }
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

                                    // Land-specific fields (conditional)
                                    if is_land_selected {
                                        div { class: "md:col-span-2 bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-4",
                                            p { class: "text-yellow-400 text-sm mb-3", "Land Details (no units)" }
                                            div { class: "grid grid-cols-2 gap-4",
                                                div {
                                                    label { class: "block text-gray-400 text-sm mb-1", "Plot Size *" }
                                                    select {
                                                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                                        value: "{plot_size_val}",
                                                        onchange: move |e: Event<FormData>| form_plot_size.set(e.value()),
                                                        option { value: "", "Select plot size" }
                                                        option { value: "50x100", "50x100 (Standard)" }
                                                        option { value: "1/8 acre", "1/8 Acre" }
                                                        option { value: "1/4 acre", "1/4 Acre" }
                                                        option { value: "1/2 acre", "1/2 Acre" }
                                                        option { value: "1 acre", "1 Acre" }
                                                        option { value: "2 acres", "2 Acres" }
                                                        option { value: "5 acres", "5 Acres" }
                                                        option { value: "10+ acres", "10+ Acres" }
                                                    }
                                                }
                                                div {
                                                    label { class: "block text-gray-400 text-sm mb-1", "Land Price (KES) *" }
                                                    input {
                                                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                                        r#type: "number",
                                                        placeholder: "5000000",
                                                        value: "{land_price_val}",
                                                        oninput: move |e: Event<FormData>| form_land_price.set(e.value()),
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div { class: "md:col-span-2",
                                        label { class: "block text-gray-400 text-sm mb-1", "Description" }
                                        textarea {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            rows: "3",
                                            placeholder: "Describe the property, amenities, shared facilities...",
                                            value: "{desc_val}",
                                            oninput: move |e: Event<FormData>| form_description.set(e.value()),
                                        }
                                    }
                                }
                            }

                            // Location
                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5 mb-4",
                                h3 { class: "text-white font-bold mb-4", "Location" }
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "County" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., Nairobi",
                                            value: "{county_val}",
                                            oninput: move |e: Event<FormData>| form_county.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Constituency" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., Westlands",
                                            value: "{constituency_val}",
                                            oninput: move |e: Event<FormData>| form_constituency.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Ward" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., Parklands/Highridge",
                                            value: "{ward_val}",
                                            oninput: move |e: Event<FormData>| form_ward.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Location / Area" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., Kilimani",
                                            value: "{loc_val}",
                                            oninput: move |e: Event<FormData>| form_location.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Village / Estate" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "e.g., Spring Valley",
                                            value: "{village_val}",
                                            oninput: move |e: Event<FormData>| form_village.set(e.value()),
                                        }
                                    }
                                }
                            }

                            // Geolocation
                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5 mb-4",
                                h3 { class: "text-white font-bold mb-4", "Geolocation" }
                                div { class: "grid grid-cols-2 gap-4",
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Latitude" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "-1.2921",
                                            value: "{lat_val}",
                                            oninput: move |e: Event<FormData>| form_latitude.set(e.value()),
                                        }
                                    }
                                    div {
                                        label { class: "block text-gray-400 text-sm mb-1", "Longitude" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            placeholder: "36.8219",
                                            value: "{lng_val}",
                                            oninput: move |e: Event<FormData>| form_longitude.set(e.value()),
                                        }
                                    }
                                }
                                div { class: "mt-4",
                                    label { class: "block text-gray-400 text-sm mb-1", "Map Address" }
                                    input {
                                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                        placeholder: "e.g., Off Waiyaki Way, Near Westlands",
                                        value: "{map_addr_val}",
                                        oninput: move |e: Event<FormData>| form_map_address.set(e.value()),
                                    }
                                }
                            }

                            // Buttons
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
                            div { class: "flex items-center justify-between mb-2",
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
                                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3 mb-4 mt-4",
                                    p { class: "text-red-400 text-sm", "{unit_err}" }
                                }
                            }

                            div { class: "space-y-4 mt-4",
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
                                        label { class: "block text-gray-400 text-sm mb-1", "Unit Type *" }
                                        select {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            value: "{utype_val}",
                                            onchange: move |e: Event<FormData>| unit_type.set(e.value()),
                                            option { value: "single_room", "Single Room" }
                                            option { value: "bedsitter", "Bedsitter" }
                                            option { value: "studio", "Studio" }
                                            option { value: "one_bedroom", "One Bedroom" }
                                            option { value: "two_bedroom", "Two Bedroom" }
                                            option { value: "three_bedroom", "Three Bedroom" }
                                            option { value: "four_bedroom", "Four Bedroom+" }
                                            option { value: "maisonette", "Maisonette" }
                                            option { value: "bungalow", "Bungalow" }
                                            option { value: "penthouse", "Penthouse" }
                                            option { value: "commercial_space", "Commercial Space" }
                                            option { value: "office", "Office" }
                                            option { value: "shop", "Shop / Retail" }
                                            option { value: "warehouse", "Warehouse" }
                                        }
                                    }
                                }
                                div {
                                    label { class: "block text-gray-400 text-sm mb-1", "Purpose" }
                                    select {
                                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                        value: "{upurpose_val}",
                                        onchange: move |e: Event<FormData>| unit_purpose.set(e.value()),
                                        option { value: "for_rent", "For Rent" }
                                        option { value: "for_sale", "For Sale" }
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
                                        label { class: "block text-gray-400 text-sm mb-1", "{unit_price_label}" }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                                            r#type: "number",
                                            placeholder: "25000",
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

// ═══════════════════════════════════════════
// PROPERTY CARD COMPONENT
// Separated to avoid rsx! macro parsing issues with nested Link + buttons
// ═══════════════════════════════════════════
#[component]
fn PropertyCard(
    property: PropertyInfo,
    is_owner: bool,
    on_navigate: EventHandler<MouseEvent>,
    on_add_unit: EventHandler<MouseEvent>,
) -> Element {
    let title = property.title.clone();
    let status = property.status.clone();
    let location = property.location.clone();
    let prop_type = property.property_type.clone();
    let purpose = property.purpose.replace("_", " ");
    let is_land = property.is_land;
    let unit_count = property.unit_count;
    let plot_size = property.plot_size.clone().unwrap_or_default();
    let land_price = property.land_price.unwrap_or(0.0);
    let min_price = property.min_unit_price.unwrap_or(0.0);
    let max_price = property.max_unit_price.unwrap_or(0.0);
    let owner_name = property.owner_name.clone();

    let purpose_badge = match purpose.as_str() {
        "for rent" => "bg-green-500/10 text-green-400 border-green-500/30",
        "for sale" => "bg-blue-500/10 text-blue-400 border-blue-500/30",
        _ => "bg-purple-500/10 text-purple-400 border-purple-500/30",
    };

    let price_display = if is_land {
        format!("KES {:.0}", land_price)
    } else if min_price > 0.0 && max_price > min_price {
        format!("KES {:.0} - {:.0}", min_price, max_price)
    } else if min_price > 0.0 {
        format!("KES {:.0}", min_price)
    } else {
        "No units priced".to_string()
    };

    rsx! {
        div {
            class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden hover:border-blue-500/50 transition-all",
            // Clickable card body (navigates to detail)
            div {
                class: "p-5 cursor-pointer",
                onclick: on_navigate,
                div { class: "flex items-start justify-between mb-3",
                    h3 { class: "text-white font-semibold text-lg", "{title}" }
                    span { class: "px-2 py-1 rounded-full text-xs bg-gray-700 text-gray-300", "{status}" }
                }
                p { class: "text-gray-400 text-sm mb-3", "{location}" }

                div { class: "flex items-center gap-2 mb-3",
                    span { class: "px-2 py-1 rounded-full text-xs border {purpose_badge}", "{purpose}" }
                    span { class: "px-2 py-1 rounded-full text-xs bg-gray-700 text-gray-300 capitalize", "{prop_type}" }
                }

                if is_land {
                    div { class: "bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-3 mb-3",
                        div { class: "flex justify-between text-sm",
                            span { class: "text-gray-400", "Plot Size" }
                            span { class: "text-white font-semibold", "{plot_size}" }
                        }
                        div { class: "flex justify-between text-sm mt-1",
                            span { class: "text-gray-400", "Price" }
                            span { class: "text-white font-semibold", "{price_display}" }
                        }
                    }
                } else {
                    div { class: "grid grid-cols-2 gap-3 mb-3",
                        div { class: "text-center",
                            p { class: "text-gray-500 text-xs", "Units" }
                            p { class: "text-white font-semibold text-sm", "{unit_count}" }
                        }
                        div { class: "text-center",
                            p { class: "text-gray-500 text-xs", "Price Range" }
                            p { class: "text-white font-semibold text-xs", "{price_display}" }
                        }
                    }
                }

                if !owner_name.is_empty() {
                    p { class: "text-gray-500 text-xs", "Owner: {owner_name}" }
                }
            }

            // Action buttons (NOT inside the clickable area)
            if is_owner && !is_land {
                div { class: "px-5 pb-4",
                    button {
                        class: "w-full bg-purple-600/20 hover:bg-purple-600/40 text-purple-300 px-3 py-2 rounded-lg text-sm font-medium transition-colors",
                        onclick: on_add_unit,
                        "Add Unit"
                    }
                }
            }
        }
    }
}