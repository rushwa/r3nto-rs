use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::components::native_recorder::NativeRecorder;
use crate::context::admin_auth::use_admin_auth;

// ═══════════════════════════════════════════
// JS Bindings (clipboard for sharing links)
// ═══════════════════════════════════════════
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["navigator", "clipboard"])]
    fn writeText(text: &str) -> js_sys::Promise;
}

// ✅ Base URL where rento-web (client-facing) is served.
// Adjust the port to match your rento-web dev server.
// In production: "https://rento.co.ke"
const CLIENT_BASE_URL: &str = "http://localhost:3001";

// API base URL
const API_BASE_URL: &str = "http://localhost:8000";

#[derive(Clone, Copy, PartialEq)]
enum TourTab {
    Pending,
    History,
    Performance,
}

#[component]
pub fn AgentTourStudioPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let agent_id = auth.read().user.as_ref().map(|u| u.id.clone()).unwrap_or_default();

    // Tab state
    let mut active_tab = use_signal(|| TourTab::Pending);

    // Data signals
    let mut pending_tours = use_signal(|| Vec::<serde_json::Value>::new());
    let mut tour_history = use_signal(|| Vec::<serde_json::Value>::new());
    let mut sla_stats = use_signal(|| Option::<serde_json::Value>::None);
    let mut loading = use_signal(|| true);
    let mut show_recorder = use_signal(|| Option::<serde_json::Value>::None);
    let mut message = use_signal(|| Option::<String>::None);
    let mut history_filter = use_signal(|| "all".to_string());

    let token_for_pending = token.clone();
    let token_for_history = token.clone();
    let token_for_stats = token.clone();
    let filter_for_effect = history_filter.read().clone();

    // Fetch pending tours
    use_effect(move || {
        let t = token_for_pending.clone();
        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/agents/pending-tours", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                    pending_tours.set(data);
                }
            }
            loading.set(false);
        });
    });

    // Fetch tour history (with filter)
    use_effect(move || {
        let t = token_for_history.clone();
        let filter = filter_for_effect.clone();
        spawn(async move {
            let url = if filter == "all" {
                format!("{}/admin/agents/tour-history?limit=50", API_BASE_URL)
            } else {
                format!("{}/admin/agents/tour-history?status={}&limit=50", API_BASE_URL, filter)
            };
            if let Ok(resp) = reqwest::Client::new()
                .get(&url)
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                    tour_history.set(data);
                }
            }
        });
    });

    // Fetch SLA stats
    use_effect(move || {
        let t = token_for_stats.clone();
        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/agents/sla-stats", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    sla_stats.set(Some(data));
                }
            }
        });
    });

    let current_tab = *active_tab.read();
    let recorder_tour = show_recorder.read().clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "🎬 Tour Recording Studio".to_string(),
                subtitle: "Manage virtual tours, track SLA performance".to_string(),
            }

            // Tab Navigation
            div { class: "flex gap-2 border-b border-gray-700 pb-0",
                TabButton {
                    label: "📋 Pending",
                    count: pending_tours.read().len(),
                    active: current_tab == TourTab::Pending,
                    onclick: {
                        let mut tab = active_tab.clone();
                        move |_| tab.set(TourTab::Pending)
                    },
                }
                TabButton {
                    label: "📜 History",
                    count: tour_history.read().len(),
                    active: current_tab == TourTab::History,
                    onclick: {
                        let mut tab = active_tab.clone();
                        move |_| tab.set(TourTab::History)
                    },
                }
                TabButton {
                    label: "📊 Performance",
                    count: 0,
                    active: current_tab == TourTab::Performance,
                    onclick: {
                        let mut tab = active_tab.clone();
                        move |_| tab.set(TourTab::Performance)
                    },
                }
            }

            // Success message
            if let Some(msg) = message.read().as_ref() {
                div { class: "bg-green-900/20 border border-green-500/30 rounded-lg p-3",
                    p { class: "text-green-400", "{msg}" }
                }
            }

            // Tab Content
            match current_tab {
                TourTab::Pending => rsx! {
                    PendingToursView {
                        pending_tours: pending_tours.read().clone(),
                        loading: *loading.read(),
                        on_record: {
                            let mut show_recorder = show_recorder.clone();
                            move |t: serde_json::Value| show_recorder.set(Some(t))
                        },
                    }
                },
                TourTab::History => rsx! {
                    HistoryView {
                        history: tour_history.read().clone(),
                        filter: history_filter.read().clone(),
                        auth_token: token.clone(),  // ✅ Phase 4: pass token
                        on_filter_change: {
                            let mut filter = history_filter.clone();
                            move |f: String| filter.set(f)
                        },
                    }
                },
                TourTab::Performance => rsx! {
                    PerformanceView {
                        stats: sla_stats.read().clone(),
                    }
                },
            }

            // Recorder modal
            if let Some(tour_data) = recorder_tour {
                NativeRecorder {
                    tour_request_id: tour_data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    property_title: tour_data.get("property_title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                    agent_id: agent_id.clone(),
                    auth_token: token.clone(),
                    on_close: {
                        let mut show_recorder = show_recorder.clone();
                        move |_| show_recorder.set(None)
                    },
                    on_success: {
                        let mut show_recorder = show_recorder.clone();
                        let mut msg_signal = message.clone();
                        move |msg: String| {
                            show_recorder.set(None);
                            msg_signal.set(Some(msg));
                        }
                    },
                }
            }
        }
    }
}

// ═══════════════════════════════════════════
// Tab Button Component
// ═══════════════════════════════════════════
#[component]
fn TabButton(
    label: String,
    count: usize,
    active: bool,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        button {
            class: if active {
                "px-4 py-2 font-medium text-blue-400 border-b-2 border-blue-400"
            } else {
                "px-4 py-2 font-medium text-gray-400 hover:text-white"
            },
            onclick: move |_| onclick.call(()),
            "{label}"
            if count > 0 && label != "📊 Performance" {
                span { class: "ml-2 px-2 py-0.5 bg-blue-600/20 text-blue-300 rounded-full text-xs",
                    "{count}"
                }
            }
        }
    }
}

// ═══════════════════════════════════════════
// Pending Tours View
// ═══════════════════════════════════════════
#[component]
fn PendingToursView(
    pending_tours: Vec<serde_json::Value>,
    loading: bool,
    on_record: EventHandler<serde_json::Value>,
) -> Element {
    if loading {
        return rsx! {
            div { class: "flex items-center justify-center h-64",
                div { class: "text-white text-lg", "Loading pending tours..." }
            }
        };
    }

    if pending_tours.is_empty() {
        return rsx! {
            EmptyState {
                icon: "🎬".to_string(),
                title: "No pending tours".to_string(),
                message: "You're all caught up! New tour requests will appear here.".to_string(),
            }
        };
    }

    rsx! {
        div { class: "space-y-4",
            for tour in pending_tours.iter() {
                PendingTourCard {
                    tour: tour.clone(),
                    on_record: {
                        let on_record = on_record.clone();
                        move |t: serde_json::Value| on_record.call(t)
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
    let client_name = tour.get("client_name").and_then(|v| v.as_str()).unwrap_or("Anonymous");
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
                    if !property_location.is_empty() {
                        p { class: "text-gray-400 text-sm", "📍 {property_location}" }
                    }
                    p { class: "text-gray-400 text-sm mt-1", "👤 {client_name}" }
                }
                button {
                    class: "px-4 py-2 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium whitespace-nowrap",
                    onclick: move |_| on_record.call(tour_clone.clone()),
                    "🎥 Record Tour"
                }
            }
        }
    }
}

// ═══════════════════════════════════════════
// History View
// ═══════════════════════════════════════════
#[component]
fn HistoryView(
    history: Vec<serde_json::Value>,
    filter: String,
    auth_token: String,  // ✅ Phase 4
    on_filter_change: EventHandler<String>,
) -> Element {
    let filters = vec!["all", "fulfilled", "expired", "cancelled", "property_delisted"];

    rsx! {
        div { class: "space-y-4",
            // Filter buttons
            div { class: "flex gap-2 flex-wrap",
                for f in filters.iter() {
                    button {
                        class: if *f == filter {
                            "px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium"
                        } else {
                            "px-3 py-1.5 bg-gray-700 text-gray-300 rounded-lg text-sm hover:bg-gray-600"
                        },
                        onclick: {
                            let f = f.to_string();
                            move |_| on_filter_change.call(f.clone())
                        },
                        "{f}"
                    }
                }
            }

            // History list
            if history.is_empty() {
                EmptyState {
                    icon: "📜".to_string(),
                    title: "No tour history".to_string(),
                    message: "Your completed and expired tours will appear here.".to_string(),
                }
            } else {
                div { class: "space-y-3",
                    for item in history.iter() {
                        HistoryCard {
                            item: item.clone(),
                            auth_token: auth_token.clone(),  // ✅ Phase 4
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn HistoryCard(
    item: serde_json::Value,
    auth_token: String,  // ✅ Phase 4
) -> Element {
    let client_name = item.get("client_name").and_then(|v| v.as_str()).unwrap_or("Anonymous");
    let property_title = item.get("property_title").and_then(|v| v.as_str()).unwrap_or("");
    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
    let fee = item.get("fee_amount").and_then(|v| v.as_str()).unwrap_or("20.00");
    let video_url = item.get("video_url").and_then(|v| v.as_str());
    let met_sla = item.get("met_sla").and_then(|v| v.as_bool());
    let duration = item.get("duration_seconds").and_then(|v| v.as_i64()).unwrap_or(0);
    let tour_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let (status_color, status_icon, status_label) = match status {
        "fulfilled" => ("bg-green-500/20 text-green-400 border-green-500/30", "✅", "Fulfilled"),
        "expired" => ("bg-red-500/20 text-red-400 border-red-500/30", "⏰", "Expired"),
        "cancelled" => ("bg-gray-500/20 text-gray-400 border-gray-500/30", "❌", "Cancelled"),
        "property_delisted" => ("bg-orange-500/20 text-orange-400 border-orange-500/30", "🚫", "De-listed"),
        _ => ("bg-blue-500/20 text-blue-400 border-blue-500/30", "⏳", status),
    };

    // ✅ Phase 4: copied state for this card
    let mut copied = use_signal(|| false);
    let is_fulfilled = status == "fulfilled";

    // ✅ Phase 4: share handler
    let share_handler = {
        let tid = tour_id.clone();
        let token = auth_token.clone();
        move |_| {
            let tid = tid.clone();
            let token = token.clone();
            let mut copied_sig = copied;
            spawn(async move {
                // Call backend to generate viewing link
                let resp = reqwest::Client::new()
                    .post(&format!("{}/api/tours/{}/viewing-link", API_BASE_URL, tid))
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                if let Ok(response) = resp {
                    if response.status().is_success() {
                        if let Ok(data) = response.json::<serde_json::Value>().await {
                            if let Some(viewing_url) = data.get("viewing_url").and_then(|v| v.as_str()) {
                                let full_url = format!("{}{}", CLIENT_BASE_URL, viewing_url);
                                // Copy to clipboard
                                let _ = JsFuture::from(writeText(&full_url)).await;
                                copied_sig.set(true);
                                // Revert after 3 seconds
                                gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                                copied_sig.set(false);
                            }
                        }
                    }
                }
            });
        }
    };

    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-4",
            div { class: "flex items-start justify-between gap-4 flex-wrap",
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-3 mb-2 flex-wrap",
                        h4 { class: "text-white font-semibold", "{property_title}" }
                        span { class: "px-2 py-0.5 rounded-full text-xs border {status_color}",
                            "{status_icon} {status_label}"
                        }
                        if let Some(met) = met_sla {
                            if met {
                                span { class: "px-2 py-0.5 rounded-full text-xs bg-green-600/20 text-green-400",
                                    "✓ On Time"
                                }
                            } else {
                                span { class: "px-2 py-0.5 rounded-full text-xs bg-red-600/20 text-red-400",
                                    "✗ Late"
                                }
                            }
                        }
                    }
                    p { class: "text-gray-400 text-sm", "👤 {client_name}" }
                    if duration > 0 {
                        p { class: "text-gray-500 text-xs mt-1",
                            "🎥 Duration: {duration}s • 💰 Fee: KES {fee}"
                        }
                    }
                }

                // ✅ Action buttons
                div { class: "flex gap-2 flex-wrap",
                    // Phase 4: Share client link (fulfilled tours only)
                    if is_fulfilled {
                        button {
                            class: if *copied.read() {
                                "px-4 py-2 bg-green-600 text-white rounded-lg font-medium whitespace-nowrap"
                            } else {
                                "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium whitespace-nowrap"
                            },
                            onclick: share_handler,
                            if *copied.read() { "✅ Copied!" } else { "🔗 Share Client Link" }
                        }
                    }

                    // Agent preview (watch own video)
                    if let Some(url) = video_url {
                        a {
                            href: "{API_BASE_URL}{url}",
                            target: "_blank",
                            class: "px-4 py-2 bg-gray-600 hover:bg-gray-500 text-white rounded-lg font-medium whitespace-nowrap",
                            "▶ Watch"
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════
// Performance View
// ═══════════════════════════════════════════
#[component]
fn PerformanceView(stats: Option<serde_json::Value>) -> Element {
    let stats = match stats {
        Some(s) => s,
        None => return rsx! {
            div { class: "flex items-center justify-center h-64",
                div { class: "text-white text-lg", "Loading performance data..." }
            }
        },
    };

    let total = stats.get("total_tours_assigned").and_then(|v| v.as_i64()).unwrap_or(0);
    let on_time = stats.get("tours_fulfilled_on_time").and_then(|v| v.as_i64()).unwrap_or(0);
    let late = stats.get("tours_fulfilled_late").and_then(|v| v.as_i64()).unwrap_or(0);
    let expired = stats.get("tours_expired").and_then(|v| v.as_i64()).unwrap_or(0);
    let avg_minutes = stats.get("average_fulfillment_minutes").and_then(|v| v.as_i64()).unwrap_or(0);
    let on_time_rate = stats.get("on_time_rate_percent").and_then(|v| v.as_i64()).unwrap_or(0);
    let revenue = stats.get("total_revenue_kes").and_then(|v| v.as_str()).unwrap_or("0.00");

    rsx! {
        div { class: "space-y-6",
            div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                StatCard {
                    icon: "🎬".to_string(),
                    label: "Total Tours".to_string(),
                    value: format!("{}", total),
                    color: "blue".to_string(),
                }
                StatCard {
                    icon: "💰".to_string(),
                    label: "Revenue".to_string(),
                    value: format!("KES {}", revenue),
                    color: "green".to_string(),
                }
                StatCard {
                    icon: "⚡".to_string(),
                    label: "Avg Fulfillment".to_string(),
                    value: format!("{} min", avg_minutes),
                    color: "yellow".to_string(),
                }
                StatCard {
                    icon: "🎯".to_string(),
                    label: "On-Time Rate".to_string(),
                    value: format!("{}%", on_time_rate),
                    color: "purple".to_string(),
                }
            }

            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h3 { class: "text-white font-bold text-lg mb-4", "📈 SLA Compliance" }

                div { class: "mb-4",
                    div { class: "flex justify-between mb-2",
                        span { class: "text-gray-300", "On-time fulfillment rate" }
                        span { class: "text-white font-bold", "{on_time_rate}%" }
                    }
                    div { class: "w-full bg-gray-700 rounded-full h-3",
                        div {
                            class: if on_time_rate >= 80 { "bg-green-500 h-3 rounded-full transition-all" }
                                else if on_time_rate >= 50 { "bg-yellow-500 h-3 rounded-full transition-all" }
                                else { "bg-red-500 h-3 rounded-full transition-all" },
                            style: "width: {on_time_rate}%"
                        }
                    }
                }

                div { class: "grid grid-cols-3 gap-4 mt-6",
                    div { class: "text-center",
                        p { class: "text-green-400 text-2xl font-bold", "{on_time}" }
                        p { class: "text-gray-400 text-sm", "On Time" }
                    }
                    div { class: "text-center",
                        p { class: "text-yellow-400 text-2xl font-bold", "{late}" }
                        p { class: "text-gray-400 text-sm", "Late" }
                    }
                    div { class: "text-center",
                        p { class: "text-red-400 text-2xl font-bold", "{expired}" }
                        p { class: "text-gray-400 text-sm", "Expired" }
                    }
                }
            }

            div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4",
                p { class: "text-blue-400 font-semibold text-sm mb-2", "💡 Tips to maintain high SLA compliance" }
                ul { class: "text-gray-300 text-sm space-y-1 list-disc list-inside",
                    li { "Check Tour Studio daily for new requests" }
                    li { "Record tours within 12 hours when possible" }
                    li { "If a property is no longer available, de-list it immediately" }
                    li { "Aim for 80%+ on-time rate to maintain top leaderboard ranking" }
                }
            }
        }
    }
}

#[component]
fn StatCard(icon: String, label: String, value: String, color: String) -> Element {
    let border_color = match color.as_str() {
        "blue" => "border-blue-500/30",
        "green" => "border-green-500/30",
        "yellow" => "border-yellow-500/30",
        "purple" => "border-purple-500/30",
        _ => "border-gray-500/30",
    };

    rsx! {
        div { class: "bg-gray-800 rounded-lg border {border_color} p-5",
            div { class: "flex items-center gap-3",
                span { class: "text-3xl", "{icon}" }
                div {
                    p { class: "text-gray-400 text-sm", "{label}" }
                    p { class: "text-white text-xl font-bold", "{value}" }
                }
            }
        }
    }
}