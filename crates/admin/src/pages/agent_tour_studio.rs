use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::components::native_recorder::NativeRecorder;
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn AgentTourStudioPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let agent_id = auth.read().user.as_ref().map(|u| u.id.clone()).unwrap_or_default();

    let mut pending_tours = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    // ✅ CRITICAL: State to control modal visibility
    let mut show_recorder = use_signal(|| Option::<serde_json::Value>::None);

    // Fetch pending tours
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
                if resp.status().is_success() {
                    if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                        pending_tours.set(data);
                    }
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

    // Clone for modal
    let recorder_tour = show_recorder.read().clone();
    let token_clone = token.clone();
    let agent_id_clone = agent_id.clone();

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

            // Success/Error message
            if let Some(msg) = message.read().as_ref() {
                div {
                    class: if *is_error.read() {
                        "bg-red-900/20 border border-red-500/30 rounded-lg p-3"
                    } else {
                        "bg-green-900/20 border border-green-500/30 rounded-lg p-3"
                    },
                    p { class: if *is_error.read() { "text-red-400" } else { "text-green-400" }, "{msg}" }
                }
            }

            // Pending tours list
            if pending_tours.read().is_empty() {
                EmptyState {
                    icon: "🎬".to_string(),
                    title: "No pending tours".to_string(),
                    message: "You're all caught up! New tour requests will appear here.".to_string(),
                }
            } else {
                div { class: "space-y-4",
                    for tour in pending_tours.read().iter() {
                        PendingTourCard {
                            tour: tour.clone(),
                            // ✅ Wire up the on_record handler to update show_recorder state
                            on_record: {
                                let mut show_recorder = show_recorder.clone();
                                move |t: serde_json::Value| {
                                    show_recorder.set(Some(t));
                                }
                            },
                        }
                    }
                }
            }

            // ✅ CRITICAL: Conditionally render the NativeRecorder modal
            if let Some(tour_data) = recorder_tour {
                NativeRecorder {
                    tour_request_id: tour_data.get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    property_title: tour_data.get("property_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    agent_id: agent_id_clone,
                    auth_token: token_clone,
                    on_close: {
                        let mut show_recorder = show_recorder.clone();
                        move |_| {
                            show_recorder.set(None);
                        }
                    },
                    on_success: {
                        let mut show_recorder = show_recorder.clone();
                        let mut msg_signal = message.clone();
                        let mut err_signal = is_error.clone();
                        move |msg: String| {
                            show_recorder.set(None);
                            msg_signal.set(Some(msg));
                            err_signal.set(false);
                            // TODO: Refetch pending tours list here
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn PendingTourCard(
    tour: serde_json::Value,
    on_record: EventHandler<serde_json::Value>,
) -> Element {
    let id = tour.get("id").and_then(|v| v.as_str()).unwrap_or("");
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

    let tour_clone = tour.clone();

    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
            div { class: "flex items-start justify-between gap-4 flex-wrap",
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-3 mb-2 flex-wrap",
                        h4 { class: "text-white font-semibold text-lg", "{property_title}" }
                        span { class: "px-2 py-0.5 rounded-full text-xs border {urgency_color}",
                            "{urgency_icon} {hours}h {minutes}m left"
                        }
                    }
                    // ✅ FIXED: Direct string check instead of Option pattern matching
                    if !property_location.is_empty() {
                        p { class: "text-gray-400 text-sm", "📍 {property_location}" }
                    }
                    p { class: "text-gray-400 text-sm mt-1",
                        "👤 {client_name} • ✉️ {client_email}"
                    }
                    p { class: "text-gray-500 text-xs mt-1", "ID: {id}" }
                }
                button {
                    class: "px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium whitespace-nowrap",
                    onclick: move |_| {
                        on_record.call(tour_clone.clone());
                    },
                    "🎥 Record Tour"
                }
            }
        }
    }
}