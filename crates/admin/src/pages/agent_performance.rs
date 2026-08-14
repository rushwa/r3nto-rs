use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatCard};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::get_agent_performance;

#[component]
pub fn AgentPerformancePage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut performance = use_signal(|| Option::<serde_json::Value>::None);
    let mut loading = use_signal(|| true);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            if let Ok(data) = get_agent_performance(&t).await {
                performance.set(Some(data));
            }
            loading.set(false);
        });
    });

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading performance data..." }
            }
        };
    }

    let perf = performance.read().clone().unwrap_or(serde_json::json!({}));

    let total_leads = perf.get("total_leads").and_then(|v| v.as_i64()).unwrap_or(0);
    let converted = perf.get("converted_leads").and_then(|v| v.as_i64()).unwrap_or(0);
    let active = perf.get("active_leads").and_then(|v| v.as_i64()).unwrap_or(0);
    let conversion_rate = perf.get("conversion_rate").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total_earned = perf.get("total_earned").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let balance = perf.get("current_balance").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let month_commission = perf.get("commissions_this_month").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let properties_managed = perf.get("properties_managed").and_then(|v| v.as_i64()).unwrap_or(0);
    let owners_converted = perf.get("owners_converted").and_then(|v| v.as_i64()).unwrap_or(0);
    let referrals = perf.get("referrals_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let referrals_completed = perf.get("referrals_completed").and_then(|v| v.as_i64()).unwrap_or(0);
    let daily_activity = perf.get("daily_activity").and_then(|v| v.as_array()).cloned().unwrap_or_default();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "My Performance".to_string(),
                subtitle: "Track your earnings, conversions, and impact".to_string(),
            }

            // Key Metrics
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-gradient-to-br from-green-900/40 to-gray-800 rounded-lg border border-green-500/30 p-6",
                    p { class: "text-green-400 text-sm", "💰 Total Earned" }
                    p { class: "text-3xl font-bold text-white mt-2", "KES {total_earned as i32}" }
                    p { class: "text-gray-400 text-xs mt-1", "All-time earnings" }
                }
                div { class: "bg-gradient-to-br from-blue-900/40 to-gray-800 rounded-lg border border-blue-500/30 p-6",
                    p { class: "text-blue-400 text-sm", "📈 This Month" }
                    p { class: "text-3xl font-bold text-white mt-2", "KES {month_commission as i32}" }
                    p { class: "text-gray-400 text-xs mt-1", "Commissions earned" }
                }
                div { class: "bg-gradient-to-br from-purple-900/40 to-gray-800 rounded-lg border border-purple-500/30 p-6",
                    p { class: "text-purple-400 text-sm", "🎯 Conversion Rate" }
                    p { class: "text-3xl font-bold text-white mt-2", "{conversion_rate as i32}%" }
                    p { class: "text-gray-400 text-xs mt-1", "{converted}/{total_leads} leads converted" }
                }
                div { class: "bg-gradient-to-br from-orange-900/40 to-gray-800 rounded-lg border border-orange-500/30 p-6",
                    p { class: "text-orange-400 text-sm", "💼 Available Balance" }
                    p { class: "text-3xl font-bold text-white mt-2", "KES {balance as i32}" }
                    p { class: "text-gray-400 text-xs mt-1", "Ready for payout" }
                }
            }

            // Lead Pipeline Stats
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h2 { class: "text-xl font-bold text-white mb-4", "📊 Lead Pipeline" }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    StatCard {
                        title: "Total Leads".to_string(),
                        value: total_leads.to_string(),
                        icon: "👥".to_string(),
                        change: "All time".to_string(),
                        change_positive: true,
                    }
                    StatCard {
                        title: "Active Leads".to_string(),
                        value: active.to_string(),
                        icon: "🔥".to_string(),
                        change: "In pipeline".to_string(),
                        change_positive: true,
                    }
                    StatCard {
                        title: "Converted".to_string(),
                        value: converted.to_string(),
                        icon: "✅".to_string(),
                        change: "Closed deals".to_string(),
                        change_positive: true,
                    }
                }
            }

            // Impact Metrics
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h2 { class: "text-xl font-bold text-white mb-4", "🏆 Your Impact" }
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    StatCard {
                        title: "Property Owners Converted".to_string(),
                        value: owners_converted.to_string(),
                        icon: "🏘️".to_string(),
                        change: "New owners".to_string(),
                        change_positive: true,
                    }
                    StatCard {
                        title: "Properties Managed".to_string(),
                        value: properties_managed.to_string(),
                        icon: "🏠".to_string(),
                        change: "Active listings".to_string(),
                        change_positive: true,
                    }
                    StatCard {
                        title: "Referrals Brought".to_string(),
                        value: format!("{}/{}", referrals_completed, referrals),
                        icon: "🔗".to_string(),
                        change: "Signed up".to_string(),
                        change_positive: referrals_completed > 0,
                    }
                }
            }

            // Daily Activity Chart
            if !daily_activity.is_empty() {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                    h2 { class: "text-xl font-bold text-white mb-4", "📅 Last 7 Days Activity" }
                    div { class: "flex items-end gap-2 h-40",
                        for day in daily_activity.iter() {
                            DailyBar { day: day.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DailyBar(day: serde_json::Value) -> Element {
    let total = day.get("total").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let day_str = day.get("day").and_then(|v| v.as_str()).unwrap_or("");
    let display_day = if day_str.len() >= 10 { &day_str[8..10] } else { "?" };

    // Calculate height (max 100px)
    let max_value = 10000.0; // Adjust based on expected max
    let height_pct = ((total / max_value) * 100.0).min(100.0);

    rsx! {
        div { class: "flex-1 flex flex-col items-center gap-1",
            div {
                class: "w-full bg-gradient-to-t from-green-600 to-green-400 rounded-t transition-all",
                style: "height: {height_pct}%; min-height: 4px;",
                title: "KES {total as i32}"
            }
            span { class: "text-xs text-gray-400", "{display_day}" }
            span { class: "text-xs text-green-400 font-semibold", "KES {total as i32}" }
        }
    }
}