use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatCard, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn PropertyOwnerDashboard() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_effect = token.clone(); // ✅ FIX E0382: clone for use_effect

    // ───────────────────────────────────────────
    // State
    // ───────────────────────────────────────────
    let mut reg_fee_status = use_signal(|| None::<serde_json::Value>);
    let mut properties = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut show_payment_modal = use_signal(|| false);

    // Trigger signal: increment to refetch data
    let mut fetch_trigger = use_signal(|| 0u32);

    // ───────────────────────────────────────────
    // Single use_effect that fetches whenever fetch_trigger changes
    // ───────────────────────────────────────────
    use_effect(move || {
        // Subscribe to fetch_trigger changes
        let _trigger = *fetch_trigger.read();
        let t = token_for_effect.clone(); // ✅ FIX E0382: use cloned token

        loading.set(true);

        spawn(async move {
            // 1. Fetch registration fee status
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

            // 2. If fee paid, fetch properties
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

    // ───────────────────────────────────────────
    // Derived state (computed from signals)
    // ───────────────────────────────────────────
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

    // ───────────────────────────────────────────
    // Loading state
    // ───────────────────────────────────────────
    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading..." }
            }
        };
    }

    // ───────────────────────────────────────────
    // Main render
    // ───────────────────────────────────────────
    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Property Owner Dashboard".to_string(),
                subtitle: "Manage your properties and subscription".to_string(),
            }

            // Registration Fee Banner (if not paid)
            if !has_paid {
                div { class: "bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-6",
                    div { class: "flex items-start gap-4",
                        span { class: "text-4xl", "⚠️" }
                        div { class: "flex-1",
                            h3 { class: "text-yellow-400 font-semibold text-lg mb-2",
                                "Registration Fee Required"
                            }
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

            // Stats Cards (only if fee paid)
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

            // Properties Section
            if has_paid {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                    div { class: "flex items-center justify-between mb-6",
                        h2 { class: "text-xl font-bold text-white", "My Properties" }
                        button {
                            class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors",
                            onclick: move |_| {
                                tracing::info!("Navigate to create property form");
                            },
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
                    h3 { class: "text-xl font-bold text-white mt-4 mb-2",
                        "Property Management Locked"
                    }
                    p { class: "text-gray-400 max-w-md mx-auto",
                        "Complete the registration fee payment to unlock property management features."
                    }
                }
            }

            // Payment Modal
            if *show_payment_modal.read() {
                PaymentModal {
                    fee_amount,
                    token: token.clone(),
                    on_close: move |_| {
                        show_payment_modal.set(false);
                    },
                    on_success: move |_| {
                        show_payment_modal.set(false);
                        // Trigger a refetch by incrementing the trigger signal
                        fetch_trigger += 1;
                    },
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Property Card Component
// ───────────────────────────────────────────
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

// ───────────────────────────────────────────
// Payment Modal Component (Clean & Secure)
// Uses EventHandler<()> — Dioxus's idiomatic callback pattern
// ───────────────────────────────────────────
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

    // Validate phone format client-side (defense in depth; server also validates)
    let is_valid_phone = {
        let phone = phone_number.read().clone();
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        (digits.starts_with("254") && digits.len() == 12)
            || (digits.starts_with("0") && digits.len() == 10)
            || (digits.starts_with("7") && digits.len() == 9)
    };

    let handle_payment = move |_| {
        let phone = phone_number.read().clone();

        // Client-side validation
        if phone.is_empty() {
            error_message.set(Some("Please enter your M-Pesa phone number".to_string()));
            return;
        }
        if !is_valid_phone {
            error_message.set(Some("Invalid phone format. Use: 07XXXXXXXX or 2547XXXXXXXX".to_string()));
            return;
        }

        loading.set(true);
        error_message.set(None);
        success_message.set(None);

        // Clone values before moving into async block
        let t = token.clone();
        let fee = fee_amount;
        // EventHandler is Clone — this is the key pattern
        let success_handler = on_success.clone();
        let mut loading_signal = loading.clone();
        let mut error_signal = error_message.clone();
        let mut success_signal = success_message.clone();

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
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        let msg = json.get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Payment successful! Check your phone for the STK Push.");
                        success_signal.set(Some(msg.to_string()));
                        // Notify parent to refetch data
                        success_handler.call(());
                    }
                }
                Ok(resp) => {
                    let status = resp.status();
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        let err = json.get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Payment failed");
                        error_signal.set(Some(err.to_string()));
                    } else {
                        error_signal.set(Some(format!("Payment failed (HTTP {})", status)));
                    }
                }
                Err(e) => {
                    error_signal.set(Some(format!("Network error: {}", e)));
                }
            }
        });
    };

    rsx! {
        div {
            class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4",
            // ✅ FIX E0599: Click backdrop to close; stop_propagation on inner card
            onclick: move |_| on_close.call(()),
            div {
                class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-md w-full",
                onclick: move |evt| evt.stop_propagation(),
                // Header
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-xl font-bold text-white", "Pay Registration Fee" }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                // Amount info
                div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4 mb-4",
                    p { class: "text-blue-400 font-semibold text-lg", "Amount: KES {fee_amount as i32}" }
                    p { class: "text-gray-300 text-sm mt-1",
                        "One-time fee to activate your property listings."
                    }
                }

                // Form
                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1",
                            "M-Pesa Phone Number"
                        }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500 transition-colors",
                            r#type: "tel",
                            placeholder: "254712345678",
                            value: "{phone_number}",
                            oninput: move |evt| {
                                phone_number.set(evt.value());
                                error_message.set(None);
                            },
                        }
                        p { class: "text-gray-500 text-xs mt-1",
                            "Format: 07XXXXXXXX or 2547XXXXXXXX"
                        }
                    }

                    // Error message
                    if let Some(err) = error_message.read().as_ref() {
                        div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3",
                            p { class: "text-red-400 text-sm", "❌ {err}" }
                        }
                    }

                    // Success message
                    if let Some(msg) = success_message.read().as_ref() {
                        div { class: "bg-green-900/20 border border-green-500/30 rounded-lg p-3",
                            p { class: "text-green-400 text-sm", "✅ {msg}" }
                        }
                    }

                    // Submit button
                    button {
                        class: "w-full py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed",
                        disabled: *loading.read() || !is_valid_phone,
                        onclick: handle_payment,
                        if *loading.read() { "Processing..." } else { "Pay Now" }
                    }

                    p { class: "text-gray-500 text-xs text-center",
                        "You'll receive an STK Push on your phone. Enter your M-Pesa PIN to complete."
                    }
                }
            }
        }
    }
}