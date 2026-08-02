use dioxus::prelude::*;
use serde::Deserialize;
use crate::api::{api_get, api_post_status};

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct ClientUser {
    pub id: String,
    pub email: String,
    pub full_name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct UsersResponse {
    pub users: Vec<ClientUser>,
}

#[component]
pub fn ConversionPage() -> Element {
    let mut step = use_signal(|| 1u8);
    let mut selected_id = use_signal(String::new);
    let mut selected_email = use_signal(String::new);
    let mut selected_name = use_signal(String::new);
    let mut otp = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut message = use_signal(String::new);
    
    let clients = use_resource(move || async move {
        api_get::<UsersResponse>("/api/users?role=client").await
    });

    let initiate = move |id: String, email: String, name: String| {
        spawn(async move {
            loading.set(true);
            let body = serde_json::json!({ "user_id": id });
            if api_post_status("/api/conversion/initiate", &body).await {
                selected_id.set(id);
                selected_email.set(email);
                selected_name.set(name);
                step.set(2);
                message.set("OTP sent to client's email".to_string());
            } else {
                message.set("Failed to send OTP".to_string());
            }
            loading.set(false);
        });
    };

    let verify = move |_| {
        spawn(async move {
            loading.set(true);
            let body = serde_json::json!({
                "user_id": selected_id(),
                "otp": otp(),
                "referring_agent_id": "current_agent_id"
            });
            if api_post_status("/api/conversion/verify", &body).await {
                step.set(3);
                message.set("Client converted to Property Owner!".to_string());
            } else {
                message.set("Invalid or expired OTP".to_string());
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold", "Digital Handshake - Role Conversion" }
            p { class: "text-gray-400", "Convert a client to a property owner" }
            
            div { class: "flex gap-4 mb-6",
                StepIndicator { num: "1", label: "Select Client", active: step() >= 1 }
                StepIndicator { num: "2", label: "Verify OTP", active: step() >= 2 }
                StepIndicator { num: "3", label: "Complete", active: step() >= 3 }
            }
            
            if !message.read().is_empty() {
                div { class: "bg-blue-900 border border-blue-700 text-blue-300 px-4 py-3 rounded-lg", "{message}" }
            }
            
            match step() {
                1 => rsx! {
                    div { class: "bg-gray-800 rounded-lg p-6",
                        h2 { class: "text-xl font-bold mb-4", "Select Client to Convert" }
                        match &*clients.read() {
                            Some(Some(data)) => rsx! {
                                div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                                    for client in &data.users {
                                        div { class: "bg-gray-700 rounded-lg p-4", key: "{client.id}",
                                            h3 { class: "text-lg font-bold", "{client.full_name}" }
                                            p { class: "text-gray-400", "{client.email}" }
                                            button {
                                                class: "mt-3 bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg",
                                                disabled: loading(),
                                                onclick: {
                                                    let id = client.id.clone();
                                                    let email = client.email.clone();
                                                    let name = client.full_name.clone();
                                                    move |_| initiate(id.clone(), email.clone(), name.clone())
                                                },
                                                "Send OTP"
                                            }
                                        }
                                    }
                                }
                            },
                            _ => rsx! { p { class: "text-gray-400", "Loading clients..." } }
                        }
                    }
                },
                2 => rsx! {
                    div { class: "bg-gray-800 rounded-lg p-6",
                        h2 { class: "text-xl font-bold mb-4", "Enter OTP" }
                        p { class: "text-gray-400 mb-4", "OTP sent to {selected_email}" }
                        form { onsubmit: verify,
                            div { class: "mb-4",
                                label { class: "block text-sm text-gray-300 mb-1", "6-Digit OTP" }
                                input { class: "w-full bg-gray-700 px-3 py-2 rounded-lg", r#type: "text",
                                    maxlength: "6", required: true,
                                    oninput: move |e| otp.set(e.value()) }
                            }
                            button { r#type: "submit", class: "bg-blue-600 hover:bg-blue-700 px-6 py-2 rounded-lg",
                                disabled: loading(),
                                if loading() { "Verifying..." } else { "Verify & Convert" } }
                        }
                    }
                },
                3 => rsx! {
                    div { class: "bg-gray-800 rounded-lg p-6 text-center",
                        div { class: "text-6xl mb-4", "✅" }
                        h2 { class: "text-2xl font-bold mb-2", "Conversion Complete!" }
                        p { class: "text-gray-400 mb-6", "{selected_name} is now a Property Owner" }
                        button { class: "bg-blue-600 hover:bg-blue-700 px-6 py-2 rounded-lg",
                            onclick: move |_| { step.set(1); message.set(String::new()); },
                            "Convert Another" }
                    }
                },
                _ => rsx! {}
            }
        }
    }
}

#[component]
fn StepIndicator(num: String, label: String, active: bool) -> Element {
    let bg = if active { "bg-blue-600" } else { "bg-gray-700" };
    let text = if active { "text-white" } else { "text-gray-400" };
    rsx! {
        div { class: "flex-1 bg-gray-800 rounded-lg p-4",
            div { class: "flex items-center gap-2",
                div { class: "w-8 h-8 rounded-full {bg} flex items-center justify-center font-bold {text}", "{num}" }
                span { class: "{text}", "{label}" }
            }
        }
    }
}
