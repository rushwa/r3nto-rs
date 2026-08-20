use dioxus::prelude::*;
use crate::context::admin_auth::use_admin_auth;

const API_BASE_URL: &str = "http://localhost:8000";

#[derive(Clone, Debug, Default)]
pub struct UnitFormData {
    pub unit_number: String,
    pub unit_type: String,
    pub bedrooms: i32,
    pub bathrooms: i32,
    pub area_sqft: i32,
    pub price: String,
    pub floor_number: i32,
    pub description: String,
    pub features: Vec<String>,
}

#[component]
pub fn UnitForm(
    property_id: String,
    on_saved: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut form = use_signal(|| UnitFormData {
        unit_type: "apartment".to_string(),
        ..Default::default()
    });
    let mut features_map = use_signal(|| std::collections::HashMap::<String, Vec<(String, String, String)>>::new());
    let mut saving = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);

    // Load available features
    let token_features = token.clone();
    use_effect(move || {
        let t = token_features.clone();
        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/unit-features", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    if let Some(obj) = data.as_object() {
                        let mut map = std::collections::HashMap::new();
                        for (category, features) in obj {
                            if let Some(arr) = features.as_array() {
                                let items: Vec<(String, String, String)> = arr.iter().filter_map(|f| {
                                    let name = f.get("name")?.as_str()?.to_string();
                                    let icon = f.get("icon").and_then(|v| v.as_str()).unwrap_or("✓").to_string();
                                    let desc = f.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                    Some((name, icon, desc))
                                }).collect();
                                map.insert(category.clone(), items);
                            }
                        }
                        features_map.set(map);
                    }
                }
            }
        });
    });

    let save_unit = {
        let token = token.clone();
        let prop_id = property_id.clone();
        move |_: MouseEvent| {
            let t = token.clone();
            let pid = prop_id.clone();
            let f = form.read().clone();
            let mut saving_sig = saving;
            let mut error_sig = error;
            let mut on_saved = on_saved.clone();

            if f.unit_number.is_empty() {
                error_sig.set(Some("Unit number is required".to_string()));
                return;
            }

            spawn(async move {
                saving_sig.set(true);
                error_sig.set(None);

                let features_json: serde_json::Value = {
                    let mut map = serde_json::Map::new();
                    for feature_name in &f.features {
                        map.insert(feature_name.clone(), serde_json::Value::Bool(true));
                    }
                    serde_json::Value::Object(map)
                };

                let price: Option<f64> = f.price.parse().ok();

                let body = serde_json::json!({
                    "property_id": pid,
                    "unit": {
                        "unit_number": f.unit_number,
                        "unit_type": f.unit_type,
                        "bedrooms": f.bedrooms,
                        "bathrooms": f.bathrooms,
                        "area_sqft": if f.area_sqft > 0 { Some(f.area_sqft) } else { None },
                        "price": price,
                        "floor_number": f.floor_number,
                        "description": if f.description.is_empty() { None } else { Some(f.description.clone()) },
                        "features": features_json,
                    }
                });

                let resp = reqwest::Client::new()
                    .post(&format!("{}/admin/properties/units", API_BASE_URL))
                    .header("Authorization", format!("Bearer {}", t))
                    .json(&body)
                    .send()
                    .await;

                match resp {
                    Ok(r) if r.status().is_success() => {
                        on_saved.call(());
                    }
                    Ok(r) => {
                        let err = r.text().await.unwrap_or_else(|_| "Failed to save unit".to_string());
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

    let current_form = form.read().clone();
    let is_saving = *saving.read();

    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
            div { class: "flex items-center justify-between mb-4",
                h3 { class: "text-white font-bold text-lg", "➕ Add Unit" }
                button {
                    class: "text-gray-400 hover:text-white text-xl",
                    onclick: move |_| on_cancel.call(()),
                    "×"
                }
            }

            if let Some(err) = error.read().as_ref() {
                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3 mb-4",
                    p { class: "text-red-400 text-sm", "❌ {err}" }
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                // Unit Number
                div {
                    label { class: "block text-gray-400 text-sm mb-1", "Unit Number *" }
                    input {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                        placeholder: "e.g., A1, G-01, Penthouse",
                        value: "{current_form.unit_number}",
                        oninput: move |e: Event<FormData>| {
                            let mut f = form.write();
                            f.unit_number = e.value();
                        },
                    }
                }

                // Unit Type
                div {
                    label { class: "block text-gray-400 text-sm mb-1", "Unit Type" }
                    select {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                        value: "{current_form.unit_type}",
                        onchange: move |e: Event<FormData>| {
                            let mut f = form.write();
                            f.unit_type = e.value();
                        },
                        option { value: "apartment", "Apartment" }
                        option { value: "bedsitter", "Bedsitter" }
                        option { value: "single", "Single Room" }
                        option { value: "studio", "Studio" }
                        option { value: "maisonette", "Maisonette" }
                        option { value: "bungalow", "Bungalow" }
                        option { value: "commercial", "Commercial Space" }
                        option { value: "office", "Office" }
                        option { value: "shop", "Shop" }
                        option { value: "warehouse", "Warehouse" }
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

                // Area
                div {
                    label { class: "block text-gray-400 text-sm mb-1", "📐 Area (sq ft)" }
                    input {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                        r#type: "number",
                        min: "0",
                        placeholder: "e.g., 1200",
                        value: if current_form.area_sqft > 0 { current_form.area_sqft.to_string() } else { String::new() },
                        oninput: move |e: Event<FormData>| {
                            let mut f = form.write();
                            f.area_sqft = e.value().parse().unwrap_or(0);
                        },
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

                // Floor
                div {
                    label { class: "block text-gray-400 text-sm mb-1", "🏢 Floor" }
                    input {
                        class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                        r#type: "number",
                        placeholder: "0 = Ground",
                        value: "{current_form.floor_number}",
                        oninput: move |e: Event<FormData>| {
                            let mut f = form.write();
                            f.floor_number = e.value().parse().unwrap_or(0);
                        },
                    }
                }
            }

            // Description
            div { class: "mt-4",
                label { class: "block text-gray-400 text-sm mb-1", "Description" }
                textarea {
                    class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-600 rounded-lg",
                    rows: "3",
                    placeholder: "Describe this unit...",
                    value: "{current_form.description}",
                    oninput: move |e: Event<FormData>| {
                        let mut f = form.write();
                        f.description = e.value();
                    },
                }
            }

            // Features (checkboxes grouped by category)
            div { class: "mt-4",
                label { class: "block text-gray-400 text-sm mb-3", "✨ Features & Amenities" }
                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                    for (category, features) in features_map.read().iter() {
                        div { class: "bg-gray-700/50 rounded-lg p-3",
                            p { class: "text-gray-300 text-sm font-semibold mb-2 uppercase", "{category}" }
                            div { class: "space-y-1.5",
                                for (name, icon, _desc) in features.iter() {
                                    label { class: "flex items-center gap-2 cursor-pointer hover:bg-gray-600/30 rounded px-2 py-1",
                                        input {
                                            r#type: "checkbox",
                                            class: "rounded border-gray-600 text-blue-500",
                                            checked: current_form.features.contains(name),
                                            onchange: {
                                                let name = name.clone();
                                                move |e: Event<FormData>| {
                                                    let mut f = form.write();
                                                    if e.value() == "true" || e.value() == "on" {
                                                        if !f.features.contains(&name) {
                                                            f.features.push(name.clone());
                                                        }
                                                    } else {
                                                        f.features.retain(|x| x != &name);
                                                    }
                                                }
                                            },
                                        }
                                        span { class: "text-sm", "{icon}" }
                                        span { class: "text-gray-300 text-sm", "{name}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Buttons
            div { class: "flex gap-3 mt-6",
                button {
                    class: "flex-1 bg-gray-600 hover:bg-gray-500 text-white font-bold py-2.5 px-4 rounded-lg",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    class: if is_saving {
                        "flex-1 bg-gray-600 text-gray-400 font-bold py-2.5 px-4 rounded-lg cursor-not-allowed"
                    } else {
                        "flex-1 bg-blue-600 hover:bg-blue-500 text-white font-bold py-2.5 px-4 rounded-lg"
                    },
                    disabled: is_saving,
                    onclick: save_unit,
                    if is_saving { "Saving..." } else { "💾 Save Unit" }
                }
            }
        }
    }
}