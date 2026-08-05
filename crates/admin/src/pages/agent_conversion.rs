use dioxus::prelude::*;
use crate::components::sidebar::PageHeader;
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn ConversionPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut target_email = use_signal(|| String::new());
    let mut otp_code = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    // Clone token into each closure separately so the outer `token` isn't moved
    let initiate_handshake = {
        let token = token.clone();
        move |_| {
            if target_email.read().is_empty() {
                message.set(Some("Please enter the user's email or ID".to_string()));
                is_error.set(true);
                return;
            }
            loading.set(true);
            message.set(None);

            let t = token.clone();
            let email = target_email.read().clone();

            spawn(async move {
                let req = serde_json::json!({ "target_user_id": email });

                let client = reqwest::Client::new();
                let res = client
                    .post("http://localhost:8000/admin/agents/handshake/initiate")
                    .header("Authorization", format!("Bearer {}", t))
                    .json(&req)
                    .send()
                    .await;

                loading.set(false);
                match res {
                    Ok(resp) if resp.status().is_success() => {
                        message.set(Some("OTP sent! Ask the owner for the 6-digit code.".to_string()));
                        is_error.set(false);
                    }
                    Ok(resp) => {
                        message.set(Some(format!("Failed: {}", resp.status())));
                        is_error.set(true);
                    }
                    Err(e) => {
                        message.set(Some(format!("Network error: {}", e)));
                        is_error.set(true);
                    }
                }
            });
        }
    };

    let verify_handshake = {
        let token = token.clone();
        move |_| {
            if otp_code.read().len() != 6 {
                message.set(Some("OTP must be 6 digits".to_string()));
                is_error.set(true);
                return;
            }
            loading.set(true);
            message.set(None);

            let t = token.clone();
            let email = target_email.read().clone();
            let code = otp_code.read().clone();

            spawn(async move {
                let req = serde_json::json!({
                    "target_user_id": email,
                    "otp_code": code
                });

                let client = reqwest::Client::new();
                let res = client
                    .post("http://localhost:8000/admin/agents/handshake/verify")
                    .header("Authorization", format!("Bearer {}", t))
                    .json(&req)
                    .send()
                    .await;

                loading.set(false);
                match res {
                    Ok(resp) if resp.status().is_success() => {
                        message.set(Some("✅ Digital Handshake Complete! User is now a Property Owner.".to_string()));
                        is_error.set(false);
                        target_email.set("".to_string());
                        otp_code.set("".to_string());
                    }
                    Ok(resp) => {
                        message.set(Some(format!("Verification Failed: {}", resp.status())));
                        is_error.set(true);
                    }
                    Err(e) => {
                        message.set(Some(format!("Network error: {}", e)));
                        is_error.set(true);
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "space-y-6 max-w-2xl",
            PageHeader {
                title: "Digital Handshake".to_string(),
                subtitle: "Convert clients to Property Owners securely".to_string()
            }

            div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4",
                div { class: "flex gap-3",
                    span { class: "text-2xl", "🛡️" }
                    div {
                        h4 { class: "text-blue-400 font-semibold", "The Handshake Protocol" }
                        p { class: "text-gray-300 text-sm mt-1",
                            "Explain to the owner that this code acts as their 'Digital Title Deed Protection'. "
                            "Once they provide the 6-digit code sent to their phone/email, you are legally authorized to manage their property."
                        }
                    }
                }
            }

            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 space-y-4",
                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-1", "Owner's Registered Email or ID" }
                    input {
                        class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                        r#type: "text",
                        placeholder: "owner@example.com",
                        value: "{target_email}",
                        oninput: move |evt| target_email.set(evt.value()),
                    }
                }

                button {
                    class: "w-full py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50",
                    disabled: *loading.read(),
                    onclick: initiate_handshake,
                    {if *loading.read() { "Sending..." } else { "Send Handshake OTP" }}
                }

                div { class: "border-t border-gray-700 my-4" }

                div {
                    label { class: "block text-sm font-medium text-gray-400 mb-1", "6-Digit Verification Code" }
                    input {
                        class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white text-center text-2xl tracking-widest focus:outline-none focus:border-green-500",
                        r#type: "text",
                        placeholder: "000000",
                        maxlength: "6",
                        value: "{otp_code}",
                        oninput: move |evt| {
                            let val = evt.value();
                            if val.chars().all(|c| c.is_ascii_digit()) && val.len() <= 6 {
                                otp_code.set(val);
                            }
                        },
                    }
                }

                button {
                    class: "w-full py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50",
                    disabled: *loading.read() || otp_code.read().len() != 6,
                    onclick: verify_handshake,
                    {if *loading.read() { "Verifying..." } else { "Complete Digital Handshake" }}
                }

                if let Some(msg) = message.read().as_ref() {
                    p {
                        class: if *is_error.read() { "text-red-400 text-sm text-center" } else { "text-green-400 text-sm text-center" },
                        "{msg}"
                    }
                }
            }
        }
    }
}