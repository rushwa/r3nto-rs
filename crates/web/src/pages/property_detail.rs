use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::api::auth::get_access_token;
use crate::api::tours::{request_tour, confirm_payment, TourRequestPayload};
use crate::api::properties::PropertyDetail;
use crate::Route;

// ═══════════════════════════════════════════
// Helper: Format currency with thousands separators
// Converts 1500000.0 -> "1,500,000"
// ═══════════════════════════════════════════
fn format_currency(amount: f64) -> String {
    let s = format!("{:.0}", amount);
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();
    for (i, c) in chars.iter().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }
    result.chars().rev().collect()
}

#[component]
pub fn PropertyDetailPage(property_id: String) -> Element {
    let _auth = use_auth();

    // ✅ Get token from localStorage via get_access_token()
    let token: String = get_access_token().unwrap_or_default();

    let mut property: Signal<Option<PropertyDetail>> = use_signal(|| None);
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Tour request state
    let mut show_tour_modal: Signal<bool> = use_signal(|| false);
    let mut tour_client_name: Signal<String> = use_signal(|| String::new());
    let mut tour_client_email: Signal<String> = use_signal(|| String::new());
    let mut tour_client_phone: Signal<String> = use_signal(|| String::new());
    let mut tour_status: Signal<Option<String>> = use_signal(|| None);
    let mut tour_request_id: Signal<Option<String>> = use_signal(|| None);
    let mut tour_fee: Signal<f64> = use_signal(|| 20.0_f64);

    let property_id_clone = property_id.clone();
    let token_clone = token.clone();

    // Fetch property details
    use_effect(move || {
        let pid = property_id_clone.clone();
        let t = token_clone.clone();
        let mut prop_sig = property;
        let mut loading_sig = loading;
        let mut error_sig = error;

        spawn(async move {
            let client = reqwest::Client::new();
            let resp = client
                .get(&format!("http://localhost:8000/admin/properties/{}", pid))
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    match r.json::<PropertyDetail>().await {
                        Ok(data) => prop_sig.set(Some(data)),
                        Err(_) => error_sig.set(Some("Failed to parse property".to_string())),
                    }
                }
                Ok(r) => {
                    error_sig.set(Some(format!("Error: {}", r.status())));
                }
                Err(e) => {
                    error_sig.set(Some(format!("Network error: {}", e)));
                }
            }
            loading_sig.set(false);
        });
    });

    // Handle tour request submission
    let submit_tour_request = {
        let token = token.clone();
        move |_: MouseEvent| {
            let t = token.clone();
            let prop = property.read().clone();
            let name = tour_client_name.read().clone();
            let email = tour_client_email.read().clone();
            let phone = tour_client_phone.read().clone();
            let mut status_sig = tour_status;
            let mut req_id_sig = tour_request_id;
            let mut fee_sig = tour_fee;

            spawn(async move {
                status_sig.set(Some("requesting".to_string()));

                let payload = TourRequestPayload {
                    property_id: prop.as_ref().map(|p| p.id.clone()).unwrap_or_default(),
                    client_email: email,
                    client_name: if name.is_empty() { None } else { Some(name) },
                    client_phone: if phone.is_empty() { None } else { Some(phone) },
                };

                match request_tour(payload, &t).await {
                    Ok(resp) => {
                        req_id_sig.set(Some(resp.request_id));
                        fee_sig.set(resp.fee_amount);
                        status_sig.set(Some("payment".to_string()));
                    }
                    Err(e) => {
                        status_sig.set(Some(format!("error:{}", e)));
                    }
                }
            });
        }
    };

    // Simulate M-Pesa payment
    let simulate_payment = {
        let token = token.clone();
        move |_: MouseEvent| {
            let t = token.clone();
            let req_id = tour_request_id.read().clone();
            let mut status_sig = tour_status;

            spawn(async move {
                status_sig.set(Some("processing_payment".to_string()));
                gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await;

                if let Some(id) = req_id {
                    match confirm_payment(&id, &t).await {
                        Ok(_) => {
                            status_sig.set(Some("success".to_string()));
                        }
                        Err(e) => {
                            status_sig.set(Some(format!("error:{}", e)));
                        }
                    }
                }
            });
        }
    };

    let close_modal = move |_: MouseEvent| {
        show_tour_modal.set(false);
        tour_status.set(None);
        tour_client_name.set(String::new());
        tour_client_email.set(String::new());
        tour_client_phone.set(String::new());
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center min-h-screen",
                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
            }
        };
    }

    if let Some(err) = error.read().as_ref() {
        return rsx! {
            div { class: "min-h-screen flex items-center justify-center",
                div { class: "bg-red-50 border border-red-200 rounded-lg p-6 max-w-md",
                    h2 { class: "text-red-800 font-bold text-lg mb-2", "Error Loading Property" }
                    p { class: "text-red-600", "{err}" }
                }
            }
        };
    }

    let prop = match property.read().as_ref() {
        Some(p) => p.clone(),
        None => return rsx! { div { "Property not found" } },
    };

    let modal_open = *show_tour_modal.read();
    let current_status = tour_status.read().clone();

    // ═══════════════════════════════════════════
    // ✅ PRE-COMPUTE ALL VALUES BEFORE rsx! BLOCK
    // This avoids "Failed to parse formatted segment" errors
    // ═══════════════════════════════════════════

    // Price with thousands separators (e.g., "1,500,000")
    let price_display: String = format_currency(prop.price);

    // Tour account reference (first 8 chars of request ID)
    let account_ref: String = tour_request_id.read()
        .as_ref()
        .map(|s| {
            let end = 8.min(s.len());
            format!("TOUR-{}", &s[..end])
        })
        .unwrap_or_else(|| "TOUR-".to_string());

    // Error message (strip "error:" prefix)
    let error_message: String = current_status
        .as_deref()
        .and_then(|s| s.strip_prefix("error:"))
        .unwrap_or("Unknown error")
        .to_string();

    // Fee display
    let fee_display: f64 = *tour_fee.read();

    // Boolean flags
    let is_requesting: bool = current_status.as_deref() == Some("requesting");
    let email_empty: bool = tour_client_email.read().is_empty();

    // Input values (read once for controlled inputs)
    let client_name_val: String = tour_client_name.read().clone();
    let client_email_val: String = tour_client_email.read().clone();
    let client_phone_val: String = tour_client_phone.read().clone();

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            // Header
            div { class: "bg-white shadow-sm border-b",
                div { class: "max-w-6xl mx-auto px-4 py-4",
                    Link { to: Route::Properties {},
                        class: "text-blue-600 hover:text-blue-800 flex items-center gap-2",
                        "← Back to Properties"
                    }
                }
            }

            div { class: "max-w-6xl mx-auto px-4 py-8",
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-8",
                    // Main content
                    div { class: "lg:col-span-2 space-y-6",
                        // Property image placeholder
                        div { class: "bg-gradient-to-br from-blue-100 to-purple-100 rounded-xl h-96 flex items-center justify-center",
                            span { class: "text-6xl", "🏠" }
                        }

                        // Title and price
                        div {
                            h1 { class: "text-3xl font-bold text-gray-900", "{prop.title}" }
                            // ✅ FIXED: Uses format_currency() instead of {prop.price:,.0}
                            p { class: "text-2xl font-bold text-blue-600 mt-2",
                                "KES {price_display}"
                            }
                        }

                        // Details
                        div { class: "bg-white rounded-xl shadow-sm p-6",
                            h2 { class: "text-xl font-bold text-gray-900 mb-4", "Property Details" }
                            div { class: "grid grid-cols-2 gap-4",
                                div {
                                    p { class: "text-sm text-gray-500", "Type" }
                                    p { class: "font-semibold", "{prop.property_type}" }
                                }
                                div {
                                    p { class: "text-sm text-gray-500", "Status" }
                                    p { class: "font-semibold capitalize", "{prop.status}" }
                                }
                                div {
                                    p { class: "text-sm text-gray-500", "Location" }
                                    p { class: "font-semibold", "{prop.location}" }
                                }
                                div {
                                    p { class: "text-sm text-gray-500", "Listed" }
                                    p { class: "font-semibold", "{prop.listing_date}" }
                                }
                            }
                        }

                        // Description
                        if let Some(desc) = &prop.description {
                            if !desc.is_empty() {
                                div { class: "bg-white rounded-xl shadow-sm p-6",
                                    h2 { class: "text-xl font-bold text-gray-900 mb-4", "Description" }
                                    p { class: "text-gray-700 leading-relaxed", "{desc}" }
                                }
                            }
                        }
                    }

                    // Sidebar
                    div { class: "space-y-6",
                        // Owner info
                        div { class: "bg-white rounded-xl shadow-sm p-6",
                            h3 { class: "font-bold text-gray-900 mb-3", "Property Owner" }
                            div { class: "flex items-center gap-3",
                                div { class: "w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center",
                                    span { class: "text-xl", "👤" }
                                }
                                div {
                                    p { class: "font-semibold", "{prop.owner.name}" }
                                    p { class: "text-sm text-gray-500", "{prop.owner.role}" }
                                }
                            }
                        }

                        // Virtual Tour Request Card
                        div { class: "bg-gradient-to-br from-yellow-50 to-orange-50 rounded-xl shadow-sm p-6 border-2 border-yellow-200",
                            div { class: "flex items-center gap-2 mb-3",
                                span { class: "text-2xl", "🎬" }
                                h3 { class: "font-bold text-gray-900", "Virtual Tour" }
                            }
                            p { class: "text-gray-700 text-sm mb-4",
                                "Can't visit in person? Request a live, watermarked video tour recorded by our verified agent on-site."
                            }
                            div { class: "bg-white rounded-lg p-3 mb-4",
                                div { class: "flex justify-between items-center",
                                    span { class: "text-gray-600", "Tour Fee" }
                                    span { class: "font-bold text-lg", "KES 20" }
                                }
                                p { class: "text-xs text-gray-500 mt-1",
                                    "Paid via M-Pesa • 2-hour viewing window • Device-locked"
                                }
                            }
                            button {
                                class: "w-full bg-gradient-to-r from-yellow-500 to-orange-500 hover:from-yellow-600 hover:to-orange-600 text-white font-bold py-3 px-4 rounded-lg shadow-md transition-all",
                                onclick: move |_| show_tour_modal.set(true),
                                "🎥 Request Virtual Tour"
                            }
                        }
                    }
                }
            }

            // Tour Request Modal
            if modal_open {
                div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4",
                    div { class: "bg-white rounded-xl shadow-2xl max-w-md w-full p-6",
                        match current_status.as_deref() {
                            None | Some("requesting") => rsx! {
                                div {
                                    div { class: "flex justify-between items-center mb-4",
                                        h2 { class: "text-xl font-bold", "🎬 Request Virtual Tour" }
                                        button {
                                            class: "text-gray-400 hover:text-gray-600 text-2xl",
                                            onclick: close_modal,
                                            "×"
                                        }
                                    }
                                    p { class: "text-gray-600 text-sm mb-4",
                                        "Fill in your details to request a tour of {prop.title}"
                                    }
                                    div { class: "space-y-3",
                                        input {
                                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                            placeholder: "Your Name (optional)",
                                            value: "{client_name_val}",
                                            oninput: move |e| tour_client_name.set(e.value()),
                                        }
                                        input {
                                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                            placeholder: "Email Address *",
                                            r#type: "email",
                                            required: true,
                                            value: "{client_email_val}",
                                            oninput: move |e| tour_client_email.set(e.value()),
                                        }
                                        input {
                                            class: "w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                            placeholder: "Phone Number (optional)",
                                            value: "{client_phone_val}",
                                            oninput: move |e| tour_client_phone.set(e.value()),
                                        }
                                    }
                                    button {
                                        class: "w-full mt-4 bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-4 rounded-lg disabled:opacity-50",
                                        disabled: email_empty || is_requesting,
                                        onclick: submit_tour_request,
                                        if is_requesting { "Requesting..." } else { "Continue to Payment →" }
                                    }
                                }
                            },
                            Some("payment") => rsx! {
                                div {
                                    h2 { class: "text-xl font-bold mb-4", "💳 Simulate M-Pesa Payment" }
                                    div { class: "bg-green-50 border border-green-200 rounded-lg p-4 mb-4",
                                        p { class: "text-green-800 font-semibold", "✅ Tour Request Created!" }
                                        p { class: "text-green-700 text-sm mt-1",
                                            "Pay KES {fee_display:.2} to activate your tour"
                                        }
                                    }
                                    div { class: "bg-gray-50 rounded-lg p-4 mb-4",
                                        p { class: "text-sm text-gray-600 mb-2", "Simulated M-Pesa STK Push:" }
                                        div { class: "bg-white border rounded p-3 font-mono text-sm",
                                            p { "Lipa Na M-Pesa" }
                                            p { "Business: RENTOLINK" }
                                            p { "Amount: KES {fee_display:.2}" }
                                            p { "Account: {account_ref}" }
                                        }
                                    }
                                    button {
                                        class: "w-full bg-green-600 hover:bg-green-700 text-white font-bold py-3 px-4 rounded-lg",
                                        onclick: simulate_payment,
                                        "✅ Simulate Successful Payment"
                                    }
                                    button {
                                        class: "w-full mt-2 text-gray-600 hover:text-gray-800 py-2",
                                        onclick: close_modal,
                                        "Cancel"
                                    }
                                }
                            },
                            Some("processing_payment") => rsx! {
                                div { class: "text-center py-8",
                                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-green-600 mx-auto mb-4" }
                                    p { class: "text-gray-700 font-semibold", "Processing M-Pesa Payment..." }
                                    p { class: "text-gray-500 text-sm mt-2", "Please wait..." }
                                }
                            },
                            Some("success") => rsx! {
                                div { class: "text-center py-6",
                                    div { class: "text-6xl mb-4", "🎉" }
                                    h2 { class: "text-2xl font-bold text-green-600 mb-2", "Payment Successful!" }
                                    p { class: "text-gray-700 mb-4",
                                        "Your tour request has been confirmed. Our agent will record a fresh, watermarked video tour within 24 hours."
                                    }
                                    div { class: "bg-blue-50 border border-blue-200 rounded-lg p-4 mb-4 text-left",
                                        p { class: "text-sm text-blue-800",
                                            span { class: "font-semibold", "What happens next:" }
                                        }
                                        ul { class: "text-sm text-blue-700 mt-2 space-y-1 list-disc list-inside",
                                            li { "Agent receives notification to record tour" }
                                            li { "Video recorded on-site with R3NTO watermark" }
                                            li { "You receive an email with secure viewing link" }
                                            li { "Link valid for 2 hours, locked to your device" }
                                        }
                                    }
                                    button {
                                        class: "w-full bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-4 rounded-lg",
                                        onclick: close_modal,
                                        "Done"
                                    }
                                }
                            },
                            Some(_) => rsx! {
                                div {
                                    div { class: "text-center py-6",
                                        div { class: "text-5xl mb-4", "❌" }
                                        h2 { class: "text-xl font-bold text-red-600 mb-2", "Request Failed" }
                                        p { class: "text-gray-700 mb-4", "{error_message}" }
                                    }
                                    button {
                                        class: "w-full bg-gray-600 hover:bg-gray-700 text-white font-bold py-3 px-4 rounded-lg",
                                        onclick: close_modal,
                                        "Close"
                                    }
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}