use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::api::auth::get_access_token;
use crate::api::tours::{request_tour, confirm_payment, TourRequestPayload};
use crate::api::properties::PropertyDetail;
use crate::Route;

const API_BASE: &str = "http://localhost:8000";

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
    let auth = use_auth();
    let nav = use_navigator();

    let token: String = get_access_token().unwrap_or_default();

    let mut property: Signal<Option<PropertyDetail>> = use_signal(|| None);
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let mut show_tour_modal: Signal<bool> = use_signal(|| false);
    let mut tour_client_name: Signal<String> = use_signal(|| String::new());
    let mut tour_client_email: Signal<String> = use_signal(|| String::new());
    let mut tour_client_phone: Signal<String> = use_signal(|| String::new());
    let mut tour_status: Signal<Option<String>> = use_signal(|| None);
    let mut tour_request_id: Signal<Option<String>> = use_signal(|| None);
    let mut tour_fee: Signal<f64> = use_signal(|| 20.0_f64);

    let property_id_clone = property_id.clone();

    use_effect(move || {
        let pid = property_id_clone.clone();
        let mut prop_sig = property;
        let mut loading_sig = loading;
        let mut error_sig = error;

        spawn(async move {
            let client = reqwest::Client::new();
            let resp = client
                .get(&format!("{}/api/public/properties/{}", API_BASE, pid))
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
                    let status = r.status();
                    let _ = r.text().await;
                    error_sig.set(Some(format!("Error: {}", status)));
                }
                Err(e) => {
                    error_sig.set(Some(format!("Network error: {}", e)));
                }
            }
            loading_sig.set(false);
        });
    });

    // ✅ AUTH-GATED BUTTON
    let handle_request_tour_click = {
        let auth = auth.clone();
        let nav = nav.clone();
        move |_: MouseEvent| {
            if auth.read().is_authenticated {
                show_tour_modal.set(true);
            } else {
                nav.push(Route::Login {});
            }
        }
    };

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
                        Ok(_) => status_sig.set(Some("success".to_string())),
                        Err(e) => status_sig.set(Some(format!("error:{}", e))),
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
            div { class: "flex items-center justify-center min-h-screen bg-gray-900",
                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 text-blue-400" }
            }
        };
    }

    if let Some(err) = error.read().as_ref() {
        return rsx! {
            div { class: "min-h-screen flex items-center justify-center bg-gray-900",
                div { class: "bg-gray-800 border border-red-500/30 rounded-xl p-8 max-w-md text-center",
                    div { class: "text-5xl mb-4", "😕" }
                    h2 { class: "text-xl font-bold text-white mb-2", "Error Loading Property" }
                    p { class: "text-red-400 mb-6", "{err}" }
                    Link {
                        to: Route::Properties {},
                        class: "inline-block bg-blue-600 hover:bg-blue-500 text-white font-bold py-2 px-6 rounded-lg",
                        "← Back to Properties"
                    }
                }
            }
        };
    }

    let prop = match property.read().as_ref() {
        Some(p) => p.clone(),
        None => return rsx! { div { class: "bg-gray-900 text-white min-h-screen p-8", "Property not found" } },
    };

    let modal_open = *show_tour_modal.read();
    let current_status = tour_status.read().clone();

    // ✅ Pre-compute BEFORE rsx!
    let price_display: String = format_currency(prop.price);
    let is_authenticated: bool = auth.read().is_authenticated;
    let user_name: String = auth.read().user.as_ref()
        .map(|u| format!("{} {}", u.first_name, u.last_name).trim().to_string())
        .unwrap_or_default();

    let account_ref: String = tour_request_id.read()
        .as_ref()
        .map(|s| {
            let end = 8.min(s.len());
            format!("TOUR-{}", &s[..end])
        })
        .unwrap_or_else(|| "TOUR-".to_string());

    let error_message: String = current_status
        .as_deref()
        .and_then(|s| s.strip_prefix("error:"))
        .unwrap_or("Unknown error")
        .to_string();

    let fee_display: f64 = *tour_fee.read();
    let is_requesting: bool = current_status.as_deref() == Some("requesting");
    let email_empty: bool = tour_client_email.read().is_empty();

    let client_name_val: String = tour_client_name.read().clone();
    let client_email_val: String = tour_client_email.read().clone();
    let client_phone_val: String = tour_client_phone.read().clone();

    rsx! {
        div { class: "min-h-screen bg-gray-900",
            // Header
            div { class: "bg-gray-800 border-b border-gray-700",
                div { class: "max-w-6xl mx-auto px-4 py-4 flex justify-between items-center",
                    Link {
                        to: Route::Properties {},
                        class: "text-blue-400 flex items-center gap-2",
                        "← Back to Properties"
                    }
                    if is_authenticated {
                        Link { to: Route::MyToursPage {}, class: "text-sm text-gray-400", "My Tours 🎬" }
                    } else {
                        Link { to: Route::Login {}, class: "text-sm text-blue-400", "Sign In" }
                    }
                }
            }

            div { class: "max-w-6xl mx-auto px-4 py-8",
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-8",
                    // Main content
                    div { class: "lg:col-span-2 space-y-6",
                        div { class: "bg-gray-800 border border-gray-700 rounded-xl h-96 flex items-center justify-center",
                            span { class: "text-6xl", "🏠" }
                        }

                        div {
                            h1 { class: "text-3xl font-bold text-white", "{prop.title}" }
                            p { class: "text-2xl font-bold text-yellow-400 mt-2", "KES {price_display}" }
                        }

                        div { class: "bg-gray-800 border border-gray-700 rounded-xl p-6",
                            h2 { class: "text-xl font-bold text-white mb-4", "Property Details" }
                            div { class: "grid grid-cols-2 gap-4",
                                div {
                                    p { class: "text-sm text-gray-400", "Type" }
                                    p { class: "font-semibold text-white", "{prop.property_type}" }
                                }
                                div {
                                    p { class: "text-sm text-gray-400", "Status" }
                                    p { class: "font-semibold text-white capitalize", "{prop.status}" }
                                }
                                div {
                                    p { class: "text-sm text-gray-400", "Location" }
                                    p { class: "font-semibold text-white", "{prop.location}" }
                                }
                                div {
                                    p { class: "text-sm text-gray-400", "Listed" }
                                    p { class: "font-semibold text-white", "{prop.listing_date}" }
                                }
                            }
                        }

                        if let Some(desc) = &prop.description {
                            if !desc.is_empty() {
                                div { class: "bg-gray-800 border border-gray-700 rounded-xl p-6",
                                    h2 { class: "text-xl font-bold text-white mb-4", "Description" }
                                    p { class: "text-gray-400 leading-relaxed", "{desc}" }
                                }
                            }
                        }
                    }

                    // Sidebar
                    div { class: "space-y-6",
                        div { class: "bg-gray-800 border border-gray-700 rounded-xl p-6",
                            h3 { class: "font-bold text-white mb-3", "Property Owner" }
                            div { class: "flex items-center gap-3",
                                div { class: "w-12 h-12 bg-blue-600/20 rounded-full flex items-center justify-center",
                                    span { class: "text-xl", "👤" }
                                }
                                div {
                                    p { class: "font-semibold text-white", "{prop.owner.name}" }
                                    p { class: "text-sm text-gray-400", "{prop.owner.role}" }
                                }
                            }
                        }

                        // 🎬 Virtual Tour Card
                        div { class: "bg-gray-800 border-2 border-yellow-500/30 rounded-xl p-6",
                            div { class: "flex items-center gap-2 mb-3",
                                span { class: "text-2xl", "🎬" }
                                h3 { class: "font-bold text-white", "Virtual Tour" }
                            }
                            p { class: "text-gray-400 text-sm mb-4",
                                "Can't visit in person? Request a live, watermarked video tour recorded by our verified agent on-site."
                            }
                            div { class: "bg-gray-900 rounded-lg p-3 mb-4",
                                div { class: "flex justify-between items-center",
                                    span { class: "text-gray-400", "Tour Fee" }
                                    span { class: "font-bold text-lg text-yellow-400", "KES 20" }
                                }
                                p { class: "text-xs text-gray-400 mt-1",
                                    "Paid via M-Pesa • 2-hour viewing window • Device-locked"
                                }
                            }

                            button {
                                class: "w-full bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 px-4 rounded-lg",
                                onclick: handle_request_tour_click,
                                if is_authenticated { "🎥 Request Virtual Tour" } else { "🔐 Sign In to Request Tour" }
                            }

                            if !is_authenticated {
                                p { class: "text-xs text-gray-400 text-center mt-2",
                                    "You'll need an account to request a tour"
                                }
                            } else {
                                p { class: "text-xs text-green-400 text-center mt-2", "✓ Signed in as {user_name}" }
                            }
                        }
                    }
                }
            }

            // Modal
            if modal_open {
                div {
                    class: "fixed inset-0 flex items-center justify-center z-50 p-4",
                    style: "background: rgba(0,0,0,0.7);",
                    div { class: "bg-gray-800 border border-gray-700 rounded-xl max-w-md w-full p-6",
                        match current_status.as_deref() {
                            None | Some("requesting") => rsx! {
                                div {
                                    div { class: "flex justify-between items-center mb-4",
                                        h2 { class: "text-xl font-bold text-white", "🎬 Request Virtual Tour" }
                                        button { class: "text-gray-400 text-2xl", onclick: close_modal, "×" }
                                    }
                                    p { class: "text-gray-400 text-sm mb-4",
                                        "Fill in your details to request a tour of {prop.title}"
                                    }
                                    div { class: "space-y-3",
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-700 rounded-lg",
                                            placeholder: "Your Name (optional)",
                                            value: "{client_name_val}",
                                            oninput: move |e| tour_client_name.set(e.value()),
                                        }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-700 rounded-lg",
                                            placeholder: "Email Address *",
                                            r#type: "email",
                                            value: "{client_email_val}",
                                            oninput: move |e| tour_client_email.set(e.value()),
                                        }
                                        input {
                                            class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-700 rounded-lg",
                                            placeholder: "Phone Number (optional)",
                                            value: "{client_phone_val}",
                                            oninput: move |e| tour_client_phone.set(e.value()),
                                        }
                                    }
                                    button {
                                        class: "w-full mt-4 bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 px-4 rounded-lg",
                                        disabled: email_empty || is_requesting,
                                        onclick: submit_tour_request,
                                        if is_requesting { "Requesting..." } else { "Continue to Payment →" }
                                    }
                                }
                            },
                            Some("payment") => rsx! {
                                div {
                                    h2 { class: "text-xl font-bold text-white mb-4", "💳 Simulate M-Pesa Payment" }
                                    div { class: "bg-green-500/20 border border-green-500/30 rounded-lg p-4 mb-4",
                                        p { class: "text-green-400 font-semibold", "✅ Tour Request Created!" }
                                        p { class: "text-green-400 text-sm mt-1", "Pay KES {fee_display:.2} to activate your tour" }
                                    }
                                    div { class: "bg-gray-900 border border-gray-700 rounded-lg p-4 mb-4",
                                        p { class: "text-sm text-gray-400 mb-2", "Simulated M-Pesa STK Push:" }
                                        div { class: "bg-gray-800 border border-gray-700 rounded p-3 font-mono text-sm text-gray-300",
                                            p { "Lipa Na M-Pesa" }
                                            p { "Business: RENTOLINK" }
                                            p { "Amount: KES {fee_display:.2}" }
                                            p { "Account: {account_ref}" }
                                        }
                                    }
                                    button {
                                        class: "w-full bg-green-600 text-white font-bold py-3 px-4 rounded-lg",
                                        onclick: simulate_payment,
                                        "✅ Simulate Successful Payment"
                                    }
                                    button {
                                        class: "w-full mt-2 text-gray-400 py-2",
                                        onclick: close_modal,
                                        "Cancel"
                                    }
                                }
                            },
                            Some("processing_payment") => rsx! {
                                div { class: "text-center py-8",
                                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 text-green-400 mx-auto mb-4" }
                                    p { class: "text-white font-semibold", "Processing M-Pesa Payment..." }
                                    p { class: "text-gray-400 text-sm mt-2", "Please wait..." }
                                }
                            },
                            Some("success") => rsx! {
                                div { class: "text-center py-6",
                                    div { class: "text-6xl mb-4", "🎉" }
                                    h2 { class: "text-2xl font-bold text-green-400 mb-2", "Payment Successful!" }
                                    p { class: "text-gray-400 mb-4",
                                        "Your tour request has been confirmed. Our agent will record a fresh, watermarked video tour within 24 hours."
                                    }
                                    div { class: "bg-blue-600/20 border border-blue-500/30 rounded-lg p-4 mb-4 text-left",
                                        p { class: "text-sm text-blue-400 font-semibold", "What happens next:" }
                                        ul { class: "text-sm text-blue-400 mt-2 space-y-1 list-disc list-inside",
                                            li { "Agent receives notification to record tour" }
                                            li { "Video recorded on-site with R3NTO watermark" }
                                            li { "You receive an email with secure viewing link" }
                                            li { "Link valid for 2 hours, locked to your device" }
                                        }
                                    }
                                    div { class: "flex gap-2",
                                        button {
                                            class: "flex-1 bg-blue-600 text-white font-bold py-3 px-4 rounded-lg",
                                            onclick: close_modal,
                                            "Done"
                                        }
                                        Link {
                                            to: Route::MyToursPage {},
                                            class: "flex-1 bg-gray-700 text-white font-bold py-3 px-4 rounded-lg text-center",
                                            "View My Tours"
                                        }
                                    }
                                }
                            },
                            Some(_) => rsx! {
                                div {
                                    div { class: "text-center py-6",
                                        div { class: "text-5xl mb-4", "❌" }
                                        h2 { class: "text-xl font-bold text-red-400 mb-2", "Request Failed" }
                                        p { class: "text-gray-400 mb-4", "{error_message}" }
                                    }
                                    button {
                                        class: "w-full bg-gray-600 text-white font-bold py-3 px-4 rounded-lg",
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