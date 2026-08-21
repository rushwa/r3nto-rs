use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatusBadge};
use crate::context::admin_auth::use_admin_auth;

const API_BASE_URL: &str = "http://localhost:8000";

// ───────────────────────────────────────────
// Local data models (matching backend response)
// ───────────────────────────────────────────

#[derive(Clone, Debug, Default, PartialEq)]
struct PropertyDetailData {
    id: String,
    title: String,
    description: Option<String>,
    status: String,
    purpose: String,
    property_type: String,
    is_land: bool,
    plot_size: Option<String>,
    land_price: Option<f64>,
    unit_count: i64,
    min_unit_price: Option<f64>,
    max_unit_price: Option<f64>,
    display_location: String,
    county: Option<String>,
    constituency: Option<String>,
    ward: Option<String>,
    location: Option<String>,
    village: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
    map_address: Option<String>,
    images: Vec<String>,
    listing_date: String,
    views: u32,
    inquiries: u32,
    owner_name: String,
    owner_email: String,
    owner_role: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct UnitInfo {
    id: String,
    unit_number: String,
    unit_type: String,
    purpose: String,
    bedrooms: i32,
    bathrooms: i32,
    area_sqft: i32,
    price: String,
    status: String,
    floor_number: i32,
    description: Option<String>,
}

// ───────────────────────────────────────────
// Component
// ───────────────────────────────────────────

#[component]
pub fn PropertyDetailPage(id: String) -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let user_role = auth.read().user.as_ref()
        .map(|u| u.role.to_uppercase())
        .unwrap_or_default();

    let mut property = use_signal(|| Option::<PropertyDetailData>::None);
    let mut units = use_signal(|| Vec::<UnitInfo>::new());
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| Option::<String>::None);

    // Unit form state
    let mut show_unit_form = use_signal(|| false);
    let mut unit_saving = use_signal(|| false);
    let mut unit_error = use_signal(|| String::new());
    let mut unit_number = use_signal(|| String::new());
    let mut unit_type = use_signal(|| "one_bedroom".to_string());
    let mut unit_purpose = use_signal(|| "for_rent".to_string());
    let mut unit_bedrooms = use_signal(|| String::new());
    let mut unit_bathrooms = use_signal(|| String::new());
    let mut unit_area = use_signal(|| String::new());
    let mut unit_price = use_signal(|| String::new());
    let mut unit_floor = use_signal(|| String::new());
    let mut unit_desc = use_signal(|| String::new());

    let is_owner = user_role == "PROPERTY_OWNER";
    let property_id = id.clone();

    // ─── Fetch property detail + units ───
    let t1 = token.clone();
    let pid1 = property_id.clone();
    use_effect(move || {
        let t = t1.clone();
        let pid = pid1.clone();
        spawn(async move {
            // Fetch property detail
            match reqwest::Client::new()
                .get(format!("{}/admin/properties/{}", API_BASE_URL, pid))
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => {
                    if let Ok(data) = r.json::<serde_json::Value>().await {
                        let prop = PropertyDetailData {
                            id: data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            title: data.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            description: data.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            status: data.get("status").and_then(|v| v.as_str()).unwrap_or("available").to_string(),
                            purpose: data.get("purpose").and_then(|v| v.as_str()).unwrap_or("for_rent").to_string(),
                            property_type: data.get("property_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            is_land: data.get("is_land").and_then(|v| v.as_bool()).unwrap_or(false),
                            plot_size: data.get("plot_size").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            land_price: data.get("land_price").and_then(|v| v.as_f64()),
                            unit_count: data.get("unit_count").and_then(|v| v.as_i64()).unwrap_or(0),
                            min_unit_price: data.get("min_unit_price").and_then(|v| v.as_f64()),
                            max_unit_price: data.get("max_unit_price").and_then(|v| v.as_f64()),
                            display_location: data.get("display_location").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            county: data.get("county").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            constituency: data.get("constituency").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            ward: data.get("ward").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            location: data.get("location").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            village: data.get("village").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            latitude: data.get("latitude").and_then(|v| v.as_f64()),
                            longitude: data.get("longitude").and_then(|v| v.as_f64()),
                            map_address: data.get("map_address").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            images: data.get("images").and_then(|v| v.as_array())
                                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                                .unwrap_or_default(),
                            listing_date: data.get("listing_date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            views: data.get("views").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            inquiries: data.get("inquiries").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                            owner_name: data.get("owner").and_then(|o| o.get("name")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            owner_email: data.get("owner").and_then(|o| o.get("email")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                            owner_role: data.get("owner").and_then(|o| o.get("role")).and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        };
                        property.set(Some(prop));
                    }
                }
                Ok(r) => {
                    let err = r.text().await.unwrap_or_else(|_| "Failed to load property".to_string());
                    error.set(Some(err));
                }
                Err(e) => error.set(Some(format!("Network error: {}", e))),
            }

            // Fetch units
            match reqwest::Client::new()
                .get(format!("{}/admin/properties/{}/units", API_BASE_URL, pid))
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await
            {
                Ok(r) if r.status().is_success() => {
                    if let Ok(data) = r.json::<Vec<serde_json::Value>>().await {
                        let unit_list: Vec<UnitInfo> = data.into_iter().filter_map(|v| {
                            Some(UnitInfo {
                                id: v.get("id")?.as_str()?.to_string(),
                                unit_number: v.get("unit_number").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                unit_type: v.get("unit_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                purpose: v.get("purpose").and_then(|v| v.as_str()).unwrap_or("for_rent").to_string(),
                                bedrooms: v.get("bedrooms").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                bathrooms: v.get("bathrooms").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                area_sqft: v.get("area_sqft").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                price: v.get("price").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                status: v.get("status").and_then(|v| v.as_str()).unwrap_or("available").to_string(),
                                floor_number: v.get("floor_number").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                description: v.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            })
                        }).collect();
                        units.set(unit_list);
                    }
                }
                _ => {} // Units fetch failure is non-fatal
            }

            loading.set(false);
        });
    });

    // ─── Save unit handler ───
    let handle_save_unit = {
        let t = token.clone();
        let pid = property_id.clone();
        move |_: MouseEvent| {
            let t = t.clone();
            let pid = pid.clone();
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

                        // Reload units
                        let t2 = t.clone();
                        let pid2 = pid.clone();
                        spawn(async move {
                            if let Ok(resp) = reqwest::Client::new()
                                .get(format!("{}/admin/properties/{}/units", API_BASE_URL, pid2))
                                .header("Authorization", format!("Bearer {}", t2))
                                .send().await
                            {
                                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                                    let unit_list: Vec<UnitInfo> = data.into_iter().filter_map(|v| {
                                        Some(UnitInfo {
                                            id: v.get("id")?.as_str()?.to_string(),
                                            unit_number: v.get("unit_number").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                            unit_type: v.get("unit_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                            purpose: v.get("purpose").and_then(|v| v.as_str()).unwrap_or("for_rent").to_string(),
                                            bedrooms: v.get("bedrooms").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                            bathrooms: v.get("bathrooms").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                            area_sqft: v.get("area_sqft").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                            price: v.get("price").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                                            status: v.get("status").and_then(|v| v.as_str()).unwrap_or("available").to_string(),
                                            floor_number: v.get("floor_number").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
                                            description: v.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                                        })
                                    }).collect();
                                    units.set(unit_list);
                                }
                            }
                        });
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

    // ─── Pre-compute ALL values before rsx! ───
    let is_loading = *loading.read();
    let err_msg = error.read().clone();
    let has_error = err_msg.is_some();
    let prop_opt = property.read().clone();
    let unit_list = units.read().clone();
    let unit_form_visible = *show_unit_form.read();
    let unit_is_saving = *unit_saving.read();
    let unit_err = unit_error.read().clone();
    let has_unit_error = !unit_err.is_empty();
    let unit_save_label = if unit_is_saving { "Saving..." } else { "Save Unit" };

    let unum_val = unit_number.read().clone();
    let utype_val = unit_type.read().clone();
    let upurpose_val = unit_purpose.read().clone();
    let ubeds_val = unit_bedrooms.read().clone();
    let ubaths_val = unit_bathrooms.read().clone();
    let uarea_val = unit_area.read().clone();
    let uprice_val = unit_price.read().clone();
    let ufloor_val = unit_floor.read().clone();
    let udesc_val = unit_desc.read().clone();
    let unit_price_label = if upurpose_val == "for_rent" { "Rent (KES/month)" } else { "Sale Price (KES)" };

    rsx! {
        div { class: "space-y-6",
            // ─── Loading ───
            if is_loading {
                div { class: "text-center py-12",
                    p { class: "text-gray-400", "Loading property details..." }
                }
            }

            // ─── Error ───
            if !is_loading && has_error {
                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-6",
                    p { class: "text-red-400", "{err_msg:?}" }
                }
            }

            // ─── Property Content ───
            if !is_loading && !has_error {
                if let Some(prop) = prop_opt {
                    // Pre-compute display values
                    {
                        let purpose_display = prop.purpose.replace("_", " ");
                        let price_display = if prop.is_land {
                            format!("KES {:.0}", prop.land_price.unwrap_or(0.0))
                        } else if let (Some(min), Some(max)) = (prop.min_unit_price, prop.max_unit_price) {
                            if min == max {
                                format!("KES {:.0}", min)
                            } else {
                                format!("KES {:.0} – {:.0}", min, max)
                            }
                        } else if let Some(min) = prop.min_unit_price {
                            format!("From KES {:.0}", min)
                        } else {
                            "No units priced".to_string()
                        };
                        let unit_count_str = format!("{}", prop.unit_count);
                        let views_str = format!("{}", prop.views);
                        let inquiries_str = format!("{}", prop.inquiries);
                        let owner_initial = prop.owner_name.chars().next().unwrap_or('?').to_string();
                        let type_icon = if prop.is_land { "🌍" } else { "🏠" };

                        rsx! {
                            div { class: "space-y-6",
                                // Header
                                div { class: "flex items-start justify-between",
                                    div {
                                        PageHeader {
                                            title: prop.title.clone(),
                                            subtitle: format!("Listed on {}", prop.listing_date),
                                        }
                                    }
                                    StatusBadge { status: prop.status.clone() }
                                }

                                // Main Content Grid
                                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                                    // Left Column
                                    div { class: "lg:col-span-2 space-y-6",
                                        // Image Gallery
                                        div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                                            if prop.images.is_empty() {
                                                div { class: "h-72 bg-gradient-to-br from-blue-600/20 to-purple-600/20 flex items-center justify-center",
                                                    span { class: "text-8xl", "{type_icon}" }
                                                }
                                            } else {
                                                div { class: "h-72 bg-gray-900 flex items-center justify-center",
                                                    img {
                                                        src: "{prop.images[0]}",
                                                        class: "max-h-full object-contain",
                                                        alt: "{prop.title}"
                                                    }
                                                }
                                            }
                                        }

                                        // Description
                                        if prop.description.is_some() {
                                            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                                                h3 { class: "text-lg font-semibold text-white mb-3", "Description" }
                                                p { class: "text-gray-300 leading-relaxed",
                                                    "{prop.description.as_deref().unwrap_or_default()}"
                                                }
                                            }
                                        }

                                        // ─── Land Details ───
                                        if prop.is_land {
                                            div { class: "bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-6",
                                                h3 { class: "text-lg font-semibold text-yellow-400 mb-4", "🌍 Land Details" }
                                                div { class: "grid grid-cols-2 gap-4",
                                                    div {
                                                        p { class: "text-gray-400 text-sm", "Plot Size" }
                                                        p { class: "text-white font-semibold",
                                                            "{prop.plot_size.as_deref().unwrap_or(\"N/A\")}"
                                                        }
                                                    }
                                                    div {
                                                        p { class: "text-gray-400 text-sm", "Price" }
                                                        p { class: "text-white font-semibold", "{price_display}" }
                                                    }
                                                }
                                            }
                                        }

                                        // ─── Units Table (non-land) ───
                                        if !prop.is_land {
                                            div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                                                div { class: "p-6 border-b border-gray-700 flex items-center justify-between",
                                                    h3 { class: "text-lg font-semibold text-white", "Units ({unit_count_str})" }
                                                    if is_owner {
                                                        button {
                                                            class: "bg-purple-600 hover:bg-purple-500 text-white px-4 py-2 rounded-lg text-sm font-medium",
                                                            onclick: move |_| show_unit_form.set(true),
                                                            "+ Add Unit"
                                                        }
                                                    }
                                                }

                                                if unit_list.is_empty() {
                                                    div { class: "p-8 text-center",
                                                        p { class: "text-gray-400", "No units added yet." }
                                                        if is_owner {
                                                            button {
                                                                class: "mt-3 bg-purple-600/20 hover:bg-purple-600/40 text-purple-300 px-4 py-2 rounded-lg text-sm",
                                                                onclick: move |_| show_unit_form.set(true),
                                                                "+ Add First Unit"
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    div { class: "overflow-x-auto",
                                                        table { class: "min-w-full divide-y divide-gray-700",
                                                            thead { class: "bg-gray-900",
                                                                tr {
                                                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Unit" }
                                                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Type" }
                                                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Purpose" }
                                                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Beds/Baths" }
                                                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Price" }
                                                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Status" }
                                                                }
                                                            }
                                                            tbody { class: "divide-y divide-gray-700",
                                                                for unit in unit_list.iter() {
                                                                    // Pre-compute unit display values
                                                                    {
                                                                        let unit_type_display = unit.unit_type.replace("_", " ");
                                                                        let unit_purpose_display = unit.purpose.replace("_", " ");
                                                                        let beds_baths = format!("{}bd / {}ba", unit.bedrooms, unit.bathrooms);
                                                                        let unit_price_str = if unit.price.is_empty() {
                                                                            "—".to_string()
                                                                        } else {
                                                                            format!("KES {}", unit.price)
                                                                        };
                                                                        let unit_status_badge = match unit.status.as_str() {
                                                                            "available" => "bg-green-500/10 text-green-400",
                                                                            "occupied" => "bg-blue-500/10 text-blue-400",
                                                                            "reserved" => "bg-yellow-500/10 text-yellow-400",
                                                                            _ => "bg-gray-500/10 text-gray-400",
                                                                        };
                                                                        let unit_purpose_badge = if unit.purpose == "for_rent" {
                                                                            "bg-green-500/10 text-green-400"
                                                                        } else {
                                                                            "bg-blue-500/10 text-blue-400"
                                                                        };

                                                                        rsx! {
                                                                            tr { class: "hover:bg-gray-700/50",
                                                                                td { class: "px-4 py-3 text-white font-medium", "{unit.unit_number}" }
                                                                                td { class: "px-4 py-3 text-gray-300 capitalize", "{unit_type_display}" }
                                                                                td { class: "px-4 py-3",
                                                                                    span { class: "px-2 py-0.5 rounded text-xs {unit_purpose_badge}",
                                                                                        "{unit_purpose_display}"
                                                                                    }
                                                                                }
                                                                                td { class: "px-4 py-3 text-gray-300", "{beds_baths}" }
                                                                                td { class: "px-4 py-3 text-white font-medium", "{unit_price_str}" }
                                                                                td { class: "px-4 py-3",
                                                                                    span { class: "px-2 py-0.5 rounded text-xs {unit_status_badge}",
                                                                                        "{unit.status}"
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Right Column
                                    div { class: "space-y-6",
                                        // Price/Purpose Card
                                        if prop.is_land {
                                            div { class: "bg-gradient-to-br from-yellow-600 to-orange-600 rounded-lg p-6",
                                                p { class: "text-yellow-100 text-sm mb-1", "Land Price" }
                                                p { class: "text-3xl font-bold text-white", "{price_display}" }
                                                p { class: "text-yellow-100 text-sm mt-2 capitalize", "{purpose_display}" }
                                            }
                                        } else {
                                            div { class: "bg-gradient-to-br from-blue-600 to-purple-600 rounded-lg p-6",
                                                p { class: "text-blue-100 text-sm mb-1", "Purpose" }
                                                p { class: "text-2xl font-bold text-white capitalize", "{purpose_display}" }
                                                if prop.unit_count > 0 {
                                                    div { class: "mt-3 pt-3 border-t border-white/20",
                                                        p { class: "text-blue-100 text-sm", "{unit_count_str} unit(s)" }
                                                        p { class: "text-white font-semibold", "{price_display}" }
                                                    }
                                                }
                                            }
                                        }

                                        // Property Specs
                                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                                            h3 { class: "text-lg font-semibold text-white mb-4", "Property Details" }
                                            div { class: "space-y-3",
                                                div { class: "flex items-center justify-between",
                                                    span { class: "text-gray-400", "Type" }
                                                    span { class: "text-white font-medium capitalize", "{prop.property_type}" }
                                                }
                                                if prop.is_land {
                                                    div { class: "flex items-center justify-between",
                                                        span { class: "text-gray-400", "Plot Size" }
                                                        span { class: "text-white font-medium",
                                                            "{prop.plot_size.as_deref().unwrap_or(\"N/A\")}"
                                                        }
                                                    }
                                                }
                                                div { class: "flex items-center justify-between",
                                                    span { class: "text-gray-400", "Location" }
                                                    span { class: "text-white font-medium text-right max-w-[200px] truncate",
                                                        "{prop.display_location}"
                                                    }
                                                }
                                                if prop.map_address.is_some() {
                                                    div { class: "flex items-center justify-between",
                                                        span { class: "text-gray-400", "Address" }
                                                        span { class: "text-white font-medium text-right max-w-[200px] truncate",
                                                            "{prop.map_address.as_deref().unwrap_or_default()}"
                                                        }
                                                    }
                                                }
                                            }
                                        }

                                        // Owner Info
                                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                                            h3 { class: "text-lg font-semibold text-white mb-4", "Property Owner" }
                                            div { class: "space-y-2",
                                                div { class: "flex items-center gap-3",
                                                    div { class: "w-12 h-12 bg-gradient-to-br from-blue-500 to-purple-500 rounded-full flex items-center justify-center",
                                                        span { class: "text-white font-bold text-lg", "{owner_initial}" }
                                                    }
                                                    div {
                                                        p { class: "text-white font-medium", "{prop.owner_name}" }
                                                        p { class: "text-gray-400 text-sm", "{prop.owner_email}" }
                                                    }
                                                }
                                            }
                                        }

                                        // Stats
                                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                                            h3 { class: "text-lg font-semibold text-white mb-4", "Listing Stats" }
                                            div { class: "grid grid-cols-2 gap-4",
                                                div { class: "text-center",
                                                    p { class: "text-2xl font-bold text-blue-400", "{views_str}" }
                                                    p { class: "text-gray-400 text-sm", "Views" }
                                                }
                                                div { class: "text-center",
                                                    p { class: "text-2xl font-bold text-green-400", "{inquiries_str}" }
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
            }

            // ─── Add Unit Modal ───
            if unit_form_visible {
                div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4 overflow-y-auto",
                    div { class: "bg-gray-900 rounded-xl max-w-lg w-full my-8 max-h-[90vh] overflow-y-auto",
                        div { class: "p-6",
                            div { class: "flex items-center justify-between mb-2",
                                h2 { class: "text-white text-xl font-bold", "Add Unit" }
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