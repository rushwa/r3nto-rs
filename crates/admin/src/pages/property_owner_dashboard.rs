use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatCard, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn PropertyOwnerDashboard() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut reg_fee_status = use_signal(|| None::<serde_json::Value>);
    let mut properties = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut show_payment_modal = use_signal(|| false);
    let mut show_add_property_modal = use_signal(|| false);
    let mut fetch_trigger = use_signal(|| 0u32);

    // ✅ FIX: Clone token before use_effect
    let token_for_effect = token.clone();

    use_effect(move || {
        let _trigger = *fetch_trigger.read();
        let t = token_for_effect.clone();
        loading.set(true);
        spawn(async move {
            let status_res = reqwest::Client::new()
                .get("http://localhost:8000/admin/registration-fee/status")
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await;
            if let Ok(resp) = status_res {
                if let Ok(json) = resp.json::<serde_json::Value>().await {
                    reg_fee_status.set(Some(json));
                }
            }
            let has_paid = {
                let status = reg_fee_status.read();
                status.as_ref()
                    .and_then(|j| j.get("has_paid_registration_fee"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            };
            if has_paid {
                let props_res = reqwest::Client::new()
                    .get("http://localhost:8000/admin/properties")
                    .header("Authorization", format!("Bearer {}", t))
                    .send()
                    .await;
                if let Ok(resp) = props_res {
                    if let Ok(json) = resp.json::<Vec<serde_json::Value>>().await {
                        properties.set(json);
                    }
                }
            }
            loading.set(false);
        });
    });

    let has_paid = {
        let status = reg_fee_status.read();
        status.as_ref()
            .and_then(|j| j.get("has_paid_registration_fee"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };
    let fee_amount = {
        let status = reg_fee_status.read();
        status.as_ref()
            .and_then(|j| j.get("registration_fee_amount"))
            .and_then(|v| v.as_f64())
            .unwrap_or(1000.0)
    };
    let status_message = {
        let status = reg_fee_status.read();
        status.as_ref()
            .and_then(|j| j.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("Loading...")
            .to_string()
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Property Owner Dashboard".to_string(),
                subtitle: "Manage your properties and subscription".to_string(),
            }

            if !has_paid {
                div { class: "bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-6",
                    div { class: "flex items-start gap-4",
                        span { class: "text-4xl", "⚠️" }
                        div { class: "flex-1",
                            h3 { class: "text-yellow-400 font-semibold text-lg mb-2", "Registration Fee Required" }
                            p { class: "text-gray-300 mb-4", "{status_message}" }
                            button {
                                class: "px-6 py-2.5 bg-yellow-600 hover:bg-yellow-500 text-white rounded-lg font-medium transition-colors",
                                onclick: move |_| show_payment_modal.set(true),
                                "Pay Registration Fee (KES {fee_amount as i32}) →"
                            }
                        }
                    }
                }
            }

            if has_paid {
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    StatCard {
                        title: "Total Properties".to_string(),
                        value: properties.read().len().to_string(),
                        icon: "🏠".to_string(),
                        change: "Active".to_string(),
                        change_positive: true,
                    }
                    StatCard {
                        title: "Available Listings".to_string(),
                        value: properties.read().iter()
                            .filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("available"))
                            .count()
                            .to_string(),
                        icon: "✅".to_string(),
                        change: "Live".to_string(),
                        change_positive: true,
                    }
                    StatCard {
                        title: "Registration".to_string(),
                        value: "Verified".to_string(),
                        icon: "💳".to_string(),
                        change: "Paid".to_string(),
                        change_positive: true,
                    }
                }
            }

            if has_paid {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                    div { class: "flex items-center justify-between mb-6",
                        h2 { class: "text-xl font-bold text-white", "My Properties" }
                        button {
                            class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors",
                            onclick: move |_| show_add_property_modal.set(true),
                            "+ Add Property"
                        }
                    }

                    if properties.read().is_empty() {
                        EmptyState {
                            icon: "🏘️".to_string(),
                            title: "No properties yet".to_string(),
                            message: "Click 'Add Property' to list your first property.".to_string(),
                        }
                    } else {
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                            for property in properties.read().iter() {
                                PropertyCard { property: property.clone() }
                            }
                        }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-12 text-center",
                    span { class: "text-6xl", "🔒" }
                    h3 { class: "text-xl font-bold text-white mt-4 mb-2", "Property Management Locked" }
                    p { class: "text-gray-400 max-w-md mx-auto",
                        "Complete the registration fee payment to unlock property management features."
                    }
                }
            }

            // ✅ FIX: Clone token for each modal separately
            if *show_payment_modal.read() {
                PaymentModal {
                    fee_amount,
                    token: token.clone(),
                    on_close: move |_| show_payment_modal.set(false),
                    on_success: move |_| {
                        show_payment_modal.set(false);
                        fetch_trigger += 1;
                    },
                }
            }

            if *show_add_property_modal.read() {
                AddPropertyModal {
                    token: token.clone(),
                    on_close: move |_| show_add_property_modal.set(false),
                    on_success: move |_| {
                        show_add_property_modal.set(false);
                        fetch_trigger += 1;
                    },
                }
            }
        }
    }
}

#[component]
fn AddPropertyModal(
    token: String,
    on_close: EventHandler<()>,
    on_success: EventHandler<()>,
) -> Element {
    let mut title = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut price = use_signal(|| String::new());
    let mut property_type = use_signal(|| "apartment".to_string());
    let mut county = use_signal(|| String::new());
    let mut location = use_signal(|| String::new());
    let mut plot_number = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    // ✅ FIX: Clone token before moving into closure
    let token_for_submit = token.clone();
    let handle_submit = move |_| {
        if title.read().is_empty() {
            error_message.set(Some("Property title is required".to_string()));
            return;
        }
        if county.read().is_empty() {
            error_message.set(Some("County is required".to_string()));
            return;
        }

        loading.set(true);
        error_message.set(None);

        let t = token_for_submit.clone();
        let payload = serde_json::json!({
            "title": title.read().clone(),
            "description": description.read().clone(),
            "price": price.read().parse::<f64>().unwrap_or(0.0),
            "property_type": property_type.read().clone(),
            "county": county.read().clone(),
            "location": location.read().clone(),
            "plot_number": plot_number.read().clone(),
        });

        spawn(async move {
            let res = reqwest::Client::new()
                .post("http://localhost:8000/admin/properties")
                .header("Authorization", format!("Bearer {}", t))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;

            loading.set(false);

            match res {
                Ok(resp) if resp.status().is_success() => {
                    on_success.call(());
                }
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        let err = json.get("error").and_then(|v| v.as_str()).unwrap_or("Failed to create property");
                        error_message.set(Some(err.to_string()));
                    }
                }
                Err(e) => {
                    error_message.set(Some(format!("Network error: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4",
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-2xl w-full max-h-[90vh] overflow-y-auto",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-xl font-bold text-white", "Add New Property" }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Property Title *" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            placeholder: "e.g., Modern 2-Bedroom Apartment in Westlands",
                            value: "{title}",
                            oninput: move |evt| title.set(evt.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Description" }
                        textarea {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            rows: "3",
                            placeholder: "Describe your property...",
                            value: "{description}",
                            oninput: move |evt| description.set(evt.value()),
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Price (KES)" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                                r#type: "number",
                                placeholder: "50000",
                                value: "{price}",
                                oninput: move |evt| price.set(evt.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Property Type" }
                            select {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                                value: "{property_type}",
                                onchange: move |evt| property_type.set(evt.value()),
                                option { value: "apartment", "Apartment" }
                                option { value: "house", "House" }
                                option { value: "commercial", "Commercial" }
                                option { value: "land", "Land" }
                            }
                        }
                    }

                    div { class: "grid grid-cols-2 gap-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "County *" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                                placeholder: "Nairobi",
                                value: "{county}",
                                oninput: move |evt| county.set(evt.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "Location/Area" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                                placeholder: "Westlands",
                                value: "{location}",
                                oninput: move |evt| location.set(evt.value()),
                            }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Plot Number" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            placeholder: "LR No. 1234/567",
                            value: "{plot_number}",
                            oninput: move |evt| plot_number.set(evt.value()),
                        }
                    }

                    if let Some(err) = error_message.read().as_ref() {
                        div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3",
                            p { class: "text-red-400 text-sm", "❌ {err}" }
                        }
                    }

                    div { class: "flex gap-2 pt-4",
                        button {
                            class: "flex-1 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium disabled:opacity-50",
                            disabled: *loading.read(),
                            onclick: handle_submit,
                            if *loading.read() { "Creating..." } else { "Create Property" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PropertyCard(property: serde_json::Value) -> Element {
    let title = property.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
    let price = property.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let status = property.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
    let location = property.get("location").and_then(|v| v.as_str()).unwrap_or("Unknown location");
    let property_type = property.get("property_type").and_then(|v| v.as_str()).unwrap_or("property");

    let status_color = match status {
        "available" => "bg-green-500/10 text-green-400 border-green-500/20",
        "occupied" => "bg-red-500/10 text-red-400 border-red-500/20",
        "maintenance" => "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
        _ => "bg-gray-500/10 text-gray-400 border-gray-500/20",
    };

    rsx! {
        div { class: "bg-gray-900 rounded-lg border border-gray-700 p-4 hover:border-blue-500/50 transition-colors",
            div { class: "flex items-start justify-between mb-3",
                div {
                    h3 { class: "text-white font-semibold", "{title}" }
                    p { class: "text-gray-400 text-sm", "{location}" }
                }
                span { class: "px-2 py-1 rounded-full text-xs border {status_color}", "{status}" }
            }
            div { class: "flex items-center justify-between mt-4 pt-3 border-t border-gray-700",
                span { class: "text-gray-400 text-sm capitalize", "{property_type}" }
                span { class: "text-blue-400 font-bold", "KES {price as i32}" }
            }
        }
    }
}

#[component]
fn PaymentModal(
    fee_amount: f64,
    token: String,
    on_close: EventHandler<()>,
    on_success: EventHandler<()>,
) -> Element {
    let mut phone_number = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);
    let mut success_message = use_signal(|| Option::<String>::None);

    let is_valid_phone = {
        let phone = phone_number.read().clone();
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        (digits.starts_with("254") && digits.len() == 12)
            || (digits.starts_with("0") && digits.len() == 10)
            || (digits.starts_with("7") && digits.len() == 9)
    };

    // ✅ FIX: Clone token before moving into closure
    let token_for_payment = token.clone();
    let handle_payment = move |_| {
        let phone = phone_number.read().clone();
        if phone.is_empty() {
            error_message.set(Some("Please enter your M-Pesa phone number".to_string()));
            return;
        }
        if !is_valid_phone {
            error_message.set(Some("Invalid phone format".to_string()));
            return;
        }
        loading.set(true);
        error_message.set(None);
        success_message.set(None);

        let t = token_for_payment.clone();
        let fee = fee_amount;
        let success_handler = on_success.clone();
        let mut loading_signal = loading;
        let mut error_signal = error_message;
        let mut success_signal = success_message;

        spawn(async move {
            let payload = serde_json::json!({
                "phone_number": phone,
                "amount": fee as u32,
                "payment_type": "registration_fee",
                "account_reference": "RENTO-REG"
            });
            let res = reqwest::Client::new()
                .post("http://localhost:8000/api/payments/registration-fee")
                .header("Authorization", format!("Bearer {}", t))
                .header("Content-Type", "application/json")
                .json(&payload)
                .send()
                .await;
            loading_signal.set(false);
            match res {
                Ok(resp) if resp.status().is_success() => {
                    success_signal.set(Some("Payment successful!".to_string()));
                    success_handler.call(());
                }
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        let err = json.get("error").and_then(|v| v.as_str()).unwrap_or("Payment failed");
                        error_signal.set(Some(err.to_string()));
                    }
                }
                Err(e) => {
                    error_signal.set(Some(format!("Network error: {}", e)));
                }
            }
        });
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4",
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-md w-full",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-xl font-bold text-white", "Pay Registration Fee" }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4 mb-4",
                    p { class: "text-blue-400 font-semibold text-lg", "Amount: KES {fee_amount as i32}" }
                }
                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "M-Pesa Phone Number" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            r#type: "tel",
                            placeholder: "254712345678",
                            value: "{phone_number}",
                            oninput: move |evt| {
                                phone_number.set(evt.value());
                                error_message.set(None);
                            },
                        }
                    }
                    if let Some(err) = error_message.read().as_ref() {
                        div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3",
                            p { class: "text-red-400 text-sm", "❌ {err}" }
                        }
                    }
                    if let Some(msg) = success_message.read().as_ref() {
                        div { class: "bg-green-900/20 border border-green-500/30 rounded-lg p-3",
                            p { class: "text-green-400 text-sm", "✅ {msg}" }
                        }
                    }
                    button {
                        class: "w-full py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium disabled:opacity-50",
                        disabled: *loading.read() || !is_valid_phone,
                        onclick: handle_payment,
                        if *loading.read() { "Processing..." } else { "Pay Now" }
                    }
                }
            }
        }
    }
}