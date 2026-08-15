use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn AgentTourStudioPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut pending_tours = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            let res = reqwest::Client::new()
                .get("http://localhost:8000/admin/agents/pending-tours")
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await;

            if let Ok(resp) = res {
                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                    pending_tours.set(data);
                }
            }
            loading.set(false);
        });
    });

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading pending tours..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "🎬 Tour Recording Studio".to_string(),
                subtitle: "Record and upload property tours (24-hour SLA)".to_string(),
            }

            // SLA Info Banner
            div { class: "bg-gradient-to-r from-blue-900/40 to-purple-900/40 rounded-lg border border-blue-500/30 p-5",
                div { class: "flex items-center gap-4",
                    span { class: "text-4xl", "⏰" }
                    div {
                        h3 { class: "text-white font-bold text-lg", "24-Hour SLA Active" }
                        p { class: "text-gray-300 text-sm mt-1",
                            "All tours must be fulfilled within 24 hours. Videos are automatically watermarked with your Agent ID + timestamp."
                        }
                    }
                }
            }

            if pending_tours.read().is_empty() {
                EmptyState {
                    icon: "🎬".to_string(),
                    title: "No pending tours".to_string(),
                    message: "You're all caught up! New tour requests will appear here.".to_string(),
                }
            } else {
                div { class: "space-y-4",
                    for tour in pending_tours.read().iter() {
                        PendingTourCard { tour: tour.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn PendingTourCard(tour: serde_json::Value) -> Element {
    let client_name = tour.get("client_name").and_then(|v| v.as_str()).unwrap_or("Anonymous");
    let client_email = tour.get("client_email").and_then(|v| v.as_str()).unwrap_or("");
    let property_title = tour.get("property_title").and_then(|v| v.as_str()).unwrap_or("");
    let property_location = tour.get("property_location").and_then(|v| v.as_str()).unwrap_or("");
    let seconds_remaining = tour.get("seconds_remaining").and_then(|v| v.as_i64()).unwrap_or(0);
    let urgency = tour.get("urgency").and_then(|v| v.as_str()).unwrap_or("normal");

    let hours = seconds_remaining / 3600;
    let minutes = (seconds_remaining % 3600) / 60;

    let (urgency_color, urgency_icon) = match urgency {
        "critical" => ("bg-red-500/20 text-red-400 border-red-500/30", "🚨"),
        "urgent" => ("bg-orange-500/20 text-orange-400 border-orange-500/30", "⚠️"),
        "normal" => ("bg-yellow-500/20 text-yellow-400 border-yellow-500/30", "⏰"),
        _ => ("bg-green-500/20 text-green-400 border-green-500/30", "✅"),
    };

    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
            div { class: "flex items-start justify-between gap-4",
                div { class: "flex-1",
                    div { class: "flex items-center gap-3 mb-2 flex-wrap",
                        h4 { class: "text-white font-semibold text-lg", "{property_title}" }
                        span { class: "px-2 py-0.5 rounded-full text-xs border {urgency_color}",
                            "{urgency_icon} {hours}h {minutes}m left"
                        }
                    }
                    p { class: "text-gray-400 text-sm", "📍 {property_location}" }
                    p { class: "text-gray-400 text-sm mt-1",
                        "👤 {client_name} • ✉️ {client_email}"
                    }
                }
                button {
                    class: "px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium",
                    onclick: move |_| {
                        // TODO: Open recorder modal (Milestone 1)
                    },
                    "🎥 Record Tour"
                }
            }
        }
    }
}