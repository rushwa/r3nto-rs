use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;

const API_BASE_URL: &str = "http://localhost:8000";

#[component]
pub fn TourOversightPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut tours = use_signal(|| Vec::<serde_json::Value>::new());
    let mut stats = use_signal(|| Option::<serde_json::Value>::None);
    let mut loading = use_signal(|| true);
    let mut filter = use_signal(|| "all".to_string());

    let token_for_tours = token.clone();
    let token_for_stats = token.clone();
    let filter_for_effect = filter.read().clone();

    // Fetch tours
    use_effect(move || {
        let t = token_for_tours.clone();
        let f = filter_for_effect.clone();
        let mut tours_sig = tours;
        let mut loading_sig = loading;

        spawn(async move {
            let url = if f == "all" {
                format!("{}/admin/tours/all?limit=100", API_BASE_URL)
            } else {
                format!("{}/admin/tours/all?status={}&limit=100", API_BASE_URL, f)
            };

            if let Ok(resp) = reqwest::Client::new()
                .get(&url)
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                    tours_sig.set(data);
                }
            }
            loading_sig.set(false);
        });
    });

    // Fetch stats
    use_effect(move || {
        let t = token_for_stats.clone();
        let mut stats_sig = stats;

        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/tours/stats", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    stats_sig.set(Some(data));
                }
            }
        });
    });

    let filters = vec!["all", "pending", "fulfilled", "expired", "property_delisted"];
    let stats_data = stats.read().clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "🎬 Tour Oversight".to_string(),
                subtitle: "Full visibility into every virtual tour lifecycle".to_string(),
            }

            // Stats Cards
            if let Some(s) = &stats_data {
                div { class: "grid grid-cols-2 md:grid-cols-5 gap-4",
                    StatMini {
                        label: "Total Tours",
                        value: format!("{}", s.get("total_tours").and_then(|v| v.as_i64()).unwrap_or(0)),
                        color: "blue",
                    }
                    StatMini {
                        label: "Pending",
                        value: format!("{}", s.get("pending").and_then(|v| v.as_i64()).unwrap_or(0)),
                        color: "yellow",
                    }
                    StatMini {
                        label: "Fulfilled",
                        value: format!("{}", s.get("fulfilled").and_then(|v| v.as_i64()).unwrap_or(0)),
                        color: "green",
                    }
                    StatMini {
                        label: "Expired",
                        value: format!("{}", s.get("expired").and_then(|v| v.as_i64()).unwrap_or(0)),
                        color: "red",
                    }
                    StatMini {
                        label: "Revenue (KES)",
                        value: format!("{:.0}", s.get("total_revenue_kes").and_then(|v| v.as_f64()).unwrap_or(0.0)),
                        color: "purple",
                    }
                }
            }

            // Filters
            div { class: "flex gap-2 flex-wrap",
                for f in filters.iter() {
                    button {
                        class: if *f == *filter.read() {
                            "px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium"
                        } else {
                            "px-3 py-1.5 bg-gray-700 text-gray-300 rounded-lg text-sm hover:bg-gray-600"
                        },
                        onclick: {
                            let f = f.to_string();
                            move |_| filter.set(f.clone())
                        },
                        "{f}"
                    }
                }
            }

            // Tour Table
            if *loading.read() {
                div { class: "flex items-center justify-center py-12",
                    div { class: "text-white text-lg", "Loading tours..." }
                }
            } else if tours.read().is_empty() {
                EmptyState {
                    icon: "🎬".to_string(),
                    title: "No tours found".to_string(),
                    message: "No tours match the current filter.".to_string(),
                }
            } else {
                div { class: "space-y-3",
                    for tour in tours.read().iter() {
                        AdminTourCard { tour: tour.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn StatMini(label: &'static str, value: String, color: &'static str) -> Element {
    let border = match color {
        "blue" => "border-blue-500/30",
        "yellow" => "border-yellow-500/30",
        "green" => "border-green-500/30",
        "red" => "border-red-500/30",
        "purple" => "border-purple-500/30",
        _ => "border-gray-500/30",
    };

    rsx! {
        div { class: "bg-gray-800 border {border} rounded-lg p-4",
            p { class: "text-gray-400 text-xs mb-1", "{label}" }
            p { class: "text-white text-xl font-bold", "{value}" }
        }
    }
}

#[component]
fn AdminTourCard(tour: serde_json::Value) -> Element {
    let status = tour.get("status").and_then(|v| v.as_str()).unwrap_or("unknown");
    let sla_status = tour.get("sla_status").and_then(|v| v.as_str()).unwrap_or("");
    let property_title = tour.get("property_title").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let client_name = tour.get("client_name").and_then(|v| v.as_str()).unwrap_or("Anonymous");
    let client_email = tour.get("client_email").and_then(|v| v.as_str()).unwrap_or("");
    let agent_name = tour.get("agent_name").and_then(|v| v.as_str()).unwrap_or("Unassigned");
    let requested_at = tour.get("requested_at").and_then(|v| v.as_str()).unwrap_or("");
    let fulfilled_at = tour.get("fulfilled_at").and_then(|v| v.as_str());
    let fee_amount = tour.get("fee_amount").and_then(|v| v.as_str()).unwrap_or("20.00");
    let video_url = tour.get("video_url").and_then(|v| v.as_str());
    let viewing_count = tour.get("viewing_sessions_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let fulfillment_mins = tour.get("fulfillment_minutes").and_then(|v| v.as_i64());

    let (status_color, status_icon) = match status {
        "pending" => ("bg-yellow-500/20 text-yellow-400 border-yellow-500/30", "⏳"),
        "fulfilled" => ("bg-green-500/20 text-green-400 border-green-500/30", "✅"),
        "expired" => ("bg-red-500/20 text-red-400 border-red-500/30", "⏰"),
        "cancelled" => ("bg-gray-500/20 text-gray-400 border-gray-500/30", "❌"),
        "property_delisted" => ("bg-orange-500/20 text-orange-400 border-orange-500/30", "🚫"),
        _ => ("bg-blue-500/20 text-blue-400 border-blue-500/30", "📋"),
    };

    let sla_badge = match sla_status {
        "on_time" => "✓ On Time",
        "late" => "✗ Late",
        "overdue" => "⚠ Overdue",
        "expired" => "✗ Expired",
        _ => "",
    };

    rsx! {
        div { class: "bg-gray-800 border border-gray-700 rounded-lg p-4",
            // Row 1: Property + Status
            div { class: "flex items-center justify-between mb-3 flex-wrap gap-2",
                div { class: "flex items-center gap-3",
                    h4 { class: "text-white font-semibold", "🏠 {property_title}" }
                    span { class: "px-2 py-0.5 rounded-full text-xs border {status_color}",
                        "{status_icon} {status}"
                    }
                    if !sla_badge.is_empty() {
                        span { class: "text-xs text-gray-400", "{sla_badge}" }
                    }
                }
                span { class: "text-yellow-400 font-bold text-sm", "KES {fee_amount}" }
            }

            // Row 2: Lifecycle timeline
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-3 text-sm",
                div {
                    p { class: "text-gray-500 text-xs", "Client" }
                    p { class: "text-gray-300", "👤 {client_name}" }
                    p { class: "text-gray-500 text-xs", "{client_email}" }
                }
                div {
                    p { class: "text-gray-500 text-xs", "Agent" }
                    p { class: "text-gray-300", "🎥 {agent_name}" }
                }
                div {
                    p { class: "text-gray-500 text-xs", "Timeline" }
                    p { class: "text-gray-300 text-xs", "Requested: {requested_at}" }
                    if let Some(f) = fulfilled_at {
                        p { class: "text-green-400 text-xs", "Fulfilled: {f}" }
                    }
                    if let Some(mins) = fulfillment_mins {
                        p { class: "text-gray-400 text-xs", "⏱ Took {mins} min" }
                    }
                }
                div {
                    p { class: "text-gray-500 text-xs", "Viewing" }
                    p { class: "text-gray-300", "👁 {viewing_count} sessions" }
                    if let Some(url) = video_url {
                        a {
                            href: "{API_BASE_URL}{url}",
                            target: "_blank",
                            class: "text-blue-400 text-xs hover:underline",
                            "▶ Watch Video"
                        }
                    }
                }
            }
        }
    }
}