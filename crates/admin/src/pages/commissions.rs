use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatCard, EmptyState};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::get_my_commissions_summary;

#[component]
pub fn CommissionsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut summary = use_signal(|| None::<serde_json::Value>);
    let mut loading = use_signal(|| true);

    use_effect(move || {
        let t = token.clone();
        spawn(async move {
            match get_my_commissions_summary(&t).await {
                Ok(data) => summary.set(Some(data)),
                Err(e) => {
                    tracing::error!("Failed to load commissions: {}", e);
                }
            }
            loading.set(false);
        });
    });

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading commissions..." }
            }
        };
    }

    let wallet = summary.read().as_ref()
        .and_then(|s| s.get("wallet"))
        .cloned()
        .unwrap_or(serde_json::json!({
            "balance": 0.0,
            "total_earned": 0.0,
            "pending_balance": 0.0,
            "total_withdrawn": 0.0,
        }));

    let recent_commissions = summary.read().as_ref()
        .and_then(|s| s.get("recent_commissions"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let balance = wallet.get("balance").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total_earned = wallet.get("total_earned").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let pending = wallet.get("pending_balance").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let withdrawn = wallet.get("total_withdrawn").and_then(|v| v.as_f64()).unwrap_or(0.0);

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "My Commissions".to_string(),
                subtitle: "Track your earnings and wallet balance".to_string(),
            }

            // Wallet Stats
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-gradient-to-br from-green-900/40 to-gray-800 rounded-lg border border-green-500/30 p-6",
                    p { class: "text-green-400 text-sm", "💰 Available Balance" }
                    p { class: "text-3xl font-bold text-white mt-2", "KES {balance as i32}" }
                    p { class: "text-gray-400 text-xs mt-1", "Ready for payout" }
                }
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                    p { class: "text-gray-400 text-sm", "Total Earned" }
                    p { class: "text-2xl font-bold text-white mt-2", "KES {total_earned as i32}" }
                }
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                    p { class: "text-gray-400 text-sm", "Pending" }
                    p { class: "text-2xl font-bold text-yellow-400 mt-2", "KES {pending as i32}" }
                }
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                    p { class: "text-gray-400 text-sm", "Total Withdrawn" }
                    p { class: "text-2xl font-bold text-blue-400 mt-2", "KES {withdrawn as i32}" }
                }
            }

            // Recent Commissions
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h2 { class: "text-xl font-bold text-white mb-4", "Recent Commissions" }
                if recent_commissions.is_empty() {
                    EmptyState {
                        icon: "💸".to_string(),
                        title: "No commissions yet".to_string(),
                        message: "Complete property conversions to start earning commissions.".to_string(),
                    }
                } else {
                    div { class: "space-y-2",
                        for commission in recent_commissions.iter() {
                            CommissionRow { commission: commission.clone() }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CommissionRow(commission: serde_json::Value) -> Element {
    let commission_type = commission.get("type").and_then(|v| v.as_str()).unwrap_or("—");
    let amount = commission.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let gross = commission.get("gross_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let status = commission.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    let created_at = commission.get("created_at").and_then(|v| v.as_str()).unwrap_or("—");
    let date_display = if created_at.len() > 10 { &created_at[..10] } else { created_at };

    let type_display = match commission_type {
        "registration_30pct" => "Registration Fee (30%)",
        "renewal_10pct" => "Renewal (10%)",
        _ => commission_type,
    };

    rsx! {
        div { class: "flex items-center justify-between p-4 bg-gray-900 rounded-lg border border-gray-700",
            div { class: "flex-1",
                p { class: "text-white font-medium", "{type_display}" }
                p { class: "text-gray-400 text-sm", "On KES {gross as i32} payment • {date_display}" }
            }
            div { class: "text-right",
                p { class: "text-green-400 font-bold text-lg", "+KES {amount as i32}" }
                span { class: "text-xs text-gray-500 capitalize", "{status}" }
            }
        }
    }
}