use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatCard, EmptyState};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_payment_history, get_payment_summary};

#[component]
pub fn PaymentHistoryPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut payments = use_signal(|| Vec::<serde_json::Value>::new());
    let mut summary = use_signal(|| Option::<serde_json::Value>::None);
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            match get_payment_summary(&t).await {
                Ok(data) => summary.set(Some(data)),
                Err(e) => {
                    message.set(Some(format!("Failed to load summary: {}", e)));
                    is_error.set(true);
                }
            }
            match get_payment_history(&t).await {
                Ok(data) => payments.set(data),
                Err(e) => {
                    message.set(Some(format!("Failed to load payments: {}", e)));
                    is_error.set(true);
                }
            }
            loading.set(false);
        });
    });

    let total_paid = summary.read().as_ref()
        .and_then(|s| s.get("total_paid"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let total_payments = summary.read().as_ref()
        .and_then(|s| s.get("total_payments"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let has_paid_reg_fee = summary.read().as_ref()
        .and_then(|s| s.get("has_paid_registration_fee"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let active_subs = summary.read().as_ref()
        .and_then(|s| s.get("active_subscriptions"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading payment history..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Payment History".to_string(),
                subtitle: "View all your payments, receipts, and subscription status".to_string(),
            }

            // Summary Stats
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-gradient-to-br from-green-900/40 to-gray-800 rounded-lg border border-green-500/30 p-4",
                    p { class: "text-green-400 text-sm", "💰 Total Paid" }
                    p { class: "text-3xl font-bold text-white mt-1", "KES {total_paid as i32}" }
                }
                StatCard {
                    title: "Total Payments".to_string(),
                    value: total_payments.to_string(),
                    icon: "🧾".to_string(),
                    change: "Completed".to_string(),
                    change_positive: true,
                }
                StatCard {
                    title: "Registration Fee".to_string(),
                    value: if has_paid_reg_fee { "Paid".to_string() } else { "Unpaid".to_string() },
                    icon: if has_paid_reg_fee { "✅".to_string() } else { "⚠️".to_string() },
                    change: if has_paid_reg_fee { "Verified".to_string() } else { "Required".to_string() },
                    change_positive: has_paid_reg_fee,
                }
                StatCard {
                    title: "Active Subscriptions".to_string(),
                    value: active_subs.to_string(),
                    icon: "⭐".to_string(),
                    change: "Properties".to_string(),
                    change_positive: active_subs > 0,
                }
            }

            if let Some(msg) = message.read().as_ref() {
                div {
                    class: if *is_error.read() {
                        "bg-red-900/20 border border-red-500/30 rounded-lg p-3"
                    } else {
                        "bg-green-900/20 border border-green-500/30 rounded-lg p-3"
                    },
                    p {
                        class: if *is_error.read() { "text-red-400" } else { "text-green-400" },
                        "{msg}"
                    }
                }
            }

            // Payments List
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h2 { class: "text-xl font-bold text-white mb-4", "All Transactions" }

                if payments.read().is_empty() {
                    EmptyState {
                        icon: "🧾".to_string(),
                        title: "No payments yet".to_string(),
                        message: "Your payment history will appear here once you make your first payment.".to_string(),
                    }
                } else {
                    div { class: "space-y-3",
                        for payment in payments.read().iter() {
                            PaymentRow { payment: payment.clone() }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Payment Row Component
// ───────────────────────────────────────────
#[component]
fn PaymentRow(payment: serde_json::Value) -> Element {
    let description = payment.get("description")
        .and_then(|v| v.as_str()).unwrap_or("Payment");
    let amount = payment.get("amount")
        .and_then(|v| v.as_f64()).unwrap_or(0.0);
    let status = payment.get("status")
        .and_then(|v| v.as_str()).unwrap_or("unknown");
    let payment_type = payment.get("payment_type")
        .and_then(|v| v.as_str()).unwrap_or("payment");
    let receipt = payment.get("receipt_number")
        .and_then(|v| v.as_str());
    let phone = payment.get("phone_number")
        .and_then(|v| v.as_str());
    let paid_at = payment.get("paid_at")
        .and_then(|v| v.as_str());
    let created_at = payment.get("created_at")
        .and_then(|v| v.as_str()).unwrap_or("");

    // Format date
    let date_display = paid_at
        .or(Some(created_at))
        .map(|d| if d.len() > 10 { &d[..10] } else { d })
        .unwrap_or("—");

    // Icon based on payment type
    let icon = match payment_type {
        "registration_fee" => "🏠",
        "subscription" => "⭐",
        "renewal" => "🔄",
        _ => "💳",
    };

    // Status badge styling
    let (status_badge, status_text) = match status {
        "completed" => ("bg-green-500/10 text-green-400 border-green-500/20", "✅ Completed"),
        "pending" => ("bg-yellow-500/10 text-yellow-400 border-yellow-500/20", "⏳ Pending"),
        "failed" => ("bg-red-500/10 text-red-400 border-red-500/20", "❌ Failed"),
        "refunded" => ("bg-blue-500/10 text-blue-400 border-blue-500/20", "💸 Refunded"),
        _ => ("bg-gray-500/10 text-gray-400 border-gray-500/20", "Unknown"),
    };

    rsx! {
        div { class: "bg-gray-900 rounded-lg border border-gray-700 p-4 hover:border-gray-600 transition-colors",
            div { class: "flex items-start justify-between gap-4",
                // Left side: Icon + Details
                div { class: "flex items-start gap-3 flex-1 min-w-0",
                    div { class: "text-2xl flex-shrink-0", "{icon}" }
                    div { class: "flex-1 min-w-0",
                        h4 { class: "text-white font-semibold truncate", "{description}" }
                        div { class: "flex flex-wrap items-center gap-x-3 gap-y-1 mt-1 text-xs text-gray-400",
                            span { "📅 {date_display}" }
                            if let Some(r) = receipt {
                                span { class: "font-mono", "🧾 {r}" }
                            }
                            if let Some(p) = phone {
                                span { "📱 {p}" }
                            }
                        }
                    }
                }

                // Right side: Amount + Status
                div { class: "text-right flex-shrink-0",
                    p { class: "text-xl font-bold text-white", "KES {amount as i32}" }
                    span { class: "inline-block mt-1 px-2 py-0.5 rounded-full text-xs border {status_badge}",
                        "{status_text}"
                    }
                }
            }
        }
    }
}