use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn OwnerInquiriesPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut inquiries = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            let res = reqwest::Client::new()
                .get("http://localhost:8000/admin/owner-inquiries")
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await;

            if let Ok(resp) = res {
                if let Ok(json) = resp.json::<Vec<serde_json::Value>>().await {
                    inquiries.set(json);
                }
            }
            loading.set(false);
        });
    });

    // ✅ FIX: Closure now accepts a tuple (String, String)
    let update_status = {
        let token = token.clone();
        let mut inquiries_signal = inquiries.clone();
        let mut message_signal = message.clone();
        let mut is_error_signal = is_error.clone();

        move |(inquiry_id, new_status): (String, String)| {
            let t = token.clone();
            let iid = inquiry_id.clone();
            let status = new_status.clone();

            spawn(async move {
                let payload = serde_json::json!({ "status": status.clone() });
                let res = reqwest::Client::new()
                    .post(format!("http://localhost:8000/admin/owner-inquiries/{}/status", iid))
                    .header("Authorization", format!("Bearer {}", t))
                    .header("Content-Type", "application/json")
                    .json(&payload)
                    .send()
                    .await;

                if let Ok(resp) = res {
                    if resp.status().is_success() {
                        message_signal.set(Some(format!("✅ Status updated to {}", status)));
                        is_error_signal.set(false);

                        // Optimistic update
                        let mut list = inquiries_signal.write();
                        for item in list.iter_mut() {
                            if item.get("id").and_then(|v| v.as_str()) == Some(&iid) {
                                item["status"] = serde_json::Value::String(status.clone());
                            }
                        }
                    } else {
                        message_signal.set(Some("Failed to update status".to_string()));
                        is_error_signal.set(true);
                    }
                }
            });
        }
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading inquiries..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "My Property Inquiries".to_string(),
                subtitle: "Manage messages and leads from interested tenants/buyers".to_string(),
            }

            if let Some(msg) = message.read().as_ref() {
                div {
                    class: if *is_error.read() { "bg-red-900/20 border border-red-500/30 rounded-lg p-3" } else { "bg-green-900/20 border border-green-500/30 rounded-lg p-3" },
                    p { class: if *is_error.read() { "text-red-400" } else { "text-green-400" }, "{msg}" }
                }
            }

            if inquiries.read().is_empty() {
                EmptyState {
                    icon: "✉️".to_string(),
                    title: "No inquiries yet".to_string(),
                    message: "When someone is interested in your property, their message will appear here.".to_string(),
                }
            } else {
                div { class: "space-y-4",
                    for inquiry in inquiries.read().iter() {
                        InquiryCard {
                            inquiry: inquiry.clone(),
                            on_update_status: update_status.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InquiryCard(inquiry: serde_json::Value, on_update_status: EventHandler<(String, String)>) -> Element {
    let id = inquiry.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = inquiry.get("inquirer_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let email = inquiry.get("inquirer_email").and_then(|v| v.as_str()).unwrap_or("—");
    let phone = inquiry.get("inquirer_phone").and_then(|v| v.as_str()).unwrap_or("—");
    let message_text = inquiry.get("message").and_then(|v| v.as_str()).unwrap_or("No message");
    let status = inquiry.get("status").and_then(|v| v.as_str()).unwrap_or("new");
    let property_title = inquiry.get("property_title").and_then(|v| v.as_str()).unwrap_or("Unknown Property");
    let created_at = inquiry.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let date_display = if created_at.len() > 10 { &created_at[..10] } else { created_at };

    let status_color = match status {
        "new" => "bg-blue-500/10 text-blue-400 border-blue-500/20",
        "contacted" => "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
        "viewing_scheduled" => "bg-purple-500/10 text-purple-400 border-purple-500/20",
        "closed" => "bg-gray-500/10 text-gray-400 border-gray-500/20",
        _ => "bg-gray-500/10 text-gray-400 border-gray-500/20",
    };

    let status_display = status.replace('_', " ");

    // ✅ FIX: Clone IDs for each closure to avoid move errors
    let id_for_contacted = id.clone();
    let id_for_viewing = id.clone();
    let id_for_closed = id.clone();

    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
            div { class: "flex items-start justify-between mb-3",
                div {
                    h4 { class: "text-white font-semibold", "{name}" }
                    p { class: "text-gray-400 text-sm", "{email} • {phone}" }
                    p { class: "text-blue-400 text-xs mt-1", "🏠 {property_title}" }
                }
                div { class: "text-right",
                    span { class: "px-2 py-1 rounded-full text-xs border {status_color} capitalize", "{status_display}" }
                    p { class: "text-gray-500 text-xs mt-1", "{date_display}" }
                }
            }

            div { class: "bg-gray-900 rounded p-3 mb-4 text-gray-300 text-sm",
                "{message_text}"
            }

            div { class: "flex items-center gap-2 flex-wrap",
                span { class: "text-gray-400 text-sm", "Update Status:" }
                button {
                    class: "px-3 py-1 text-xs rounded bg-blue-600/20 text-blue-400 hover:bg-blue-600/40",
                    onclick: move |_| on_update_status.call((id_for_contacted.clone(), "contacted".to_string())),
                    "Contacted"
                }
                button {
                    class: "px-3 py-1 text-xs rounded bg-purple-600/20 text-purple-400 hover:bg-purple-600/40",
                    onclick: move |_| on_update_status.call((id_for_viewing.clone(), "viewing_scheduled".to_string())),
                    "Viewing Scheduled"
                }
                button {
                    class: "px-3 py-1 text-xs rounded bg-gray-600/20 text-gray-400 hover:bg-gray-600/40",
                    onclick: move |_| on_update_status.call((id_for_closed.clone(), "closed".to_string())),
                    "Closed"
                }
            }
        }
    }
}