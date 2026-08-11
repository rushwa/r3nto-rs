use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatCard, EmptyState};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{
    get_my_commissions_summary, get_my_payout_history, request_payout,
};

#[component]
pub fn AgentPayoutsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut wallet_info = use_signal(|| Option::<serde_json::Value>::None);
    let mut payout_history = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut show_request_modal = use_signal(|| false);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);
    let mut fetch_trigger = use_signal(|| 0u32);

    let token_for_effect = token.clone();

    use_effect(move || {
        let _trigger = *fetch_trigger.read();
        let t = token_for_effect.clone();
        spawn(async move {
            // Fetch wallet info
            if let Ok(summary) = get_my_commissions_summary(&t).await {
                wallet_info.set(summary.get("wallet").cloned());
            }
            // Fetch payout history
            if let Ok(history) = get_my_payout_history(&t).await {
                payout_history.set(history);
            }
            loading.set(false);
        });
    });

    let balance = wallet_info.read().as_ref()
        .and_then(|w| w.get("balance"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let total_earned = wallet_info.read().as_ref()
        .and_then(|w| w.get("total_earned"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let pending_balance = wallet_info.read().as_ref()
        .and_then(|w| w.get("pending_balance"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let total_withdrawn = wallet_info.read().as_ref()
        .and_then(|w| w.get("total_withdrawn"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Count payout statuses
    let pending_count = payout_history.read().iter()
        .filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("pending"))
        .count();
    let approved_count = payout_history.read().iter()
        .filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("approved"))
        .count();

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "My Wallet & Payouts".to_string(),
                subtitle: "Manage your earnings and request payouts".to_string(),
            }

            // Wallet Stats
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-gradient-to-br from-green-900/40 to-gray-800 rounded-lg border border-green-500/30 p-6",
                    p { class: "text-green-400 text-sm", "💰 Available Balance" }
                    p { class: "text-3xl font-bold text-white mt-2", "KES {balance as i32}" }
                    p { class: "text-gray-400 text-xs mt-1", "Ready for payout" }
                }
                StatCard {
                    title: "Total Earned".to_string(),
                    value: format!("KES {}", total_earned as i32),
                    icon: "📈".to_string(),
                    change: "All time".to_string(),
                    change_positive: true,
                }
                StatCard {
                    title: "Pending Payout".to_string(),
                    value: format!("KES {}", pending_balance as i32),
                    icon: "⏳".to_string(),
                    change: format!("{} request{}", pending_count, if pending_count == 1 { "" } else { "s" }),
                    change_positive: false,
                }
                StatCard {
                    title: "Total Withdrawn".to_string(),
                    value: format!("KES {}", total_withdrawn as i32),
                    icon: "💸".to_string(),
                    change: format!("{} approved", approved_count),
                    change_positive: true,
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

            // Request Payout Button
            div { class: "flex items-center justify-between bg-gray-800 rounded-lg border border-gray-700 p-6",
                div {
                    h2 { class: "text-xl font-bold text-white", "Request a Payout" }
                    p { class: "text-gray-400 text-sm mt-1",
                        "Minimum payout: KES 500. Funds will be sent to your M-Pesa."
                    }
                }
                button {
                    class: "px-6 py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50",
                    disabled: balance < 500.0 || pending_count > 0,
                    onclick: move |_| show_request_modal.set(true),
                    if pending_count > 0 {
                        "Payout Pending"
                    } else if balance < 500.0 {
                        "Min. KES 500 Required"
                    } else {
                        "Request Payout →"
                    }
                }
            }

            // Payout History
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h2 { class: "text-xl font-bold text-white mb-4", "Payout History" }

                if payout_history.read().is_empty() {
                    EmptyState {
                        icon: "💸".to_string(),
                        title: "No payouts yet".to_string(),
                        message: "Your payout history will appear here once you request your first payout.".to_string(),
                    }
                } else {
                    div { class: "space-y-3",
                        for payout in payout_history.read().iter() {
                            PayoutHistoryRow { payout: payout.clone() }
                        }
                    }
                }
            }

            // Request Payout Modal
            if *show_request_modal.read() {
                RequestPayoutModal {
                    available_balance: balance,
                    token: token.clone(),
                    on_close: move |_| show_request_modal.set(false),
                    on_success: move |msg: String| {
                        show_request_modal.set(false);
                        message.set(Some(msg));
                        is_error.set(false);
                        fetch_trigger += 1;
                    },
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Payout History Row
// ───────────────────────────────────────────
#[component]
fn PayoutHistoryRow(payout: serde_json::Value) -> Element {
    let amount = payout.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let status = payout.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    let phone = payout.get("mpesa_phone").and_then(|v| v.as_str()).unwrap_or("—");
    let created_at = payout.get("created_at").and_then(|v| v.as_str()).unwrap_or("—");
    let processed_at = payout.get("processed_at").and_then(|v| v.as_str());
    let admin_notes = payout.get("admin_notes").and_then(|v| v.as_str());

    let date_display = if created_at.len() > 10 { &created_at[..10] } else { created_at };
    let processed_display = processed_at
        .map(|d| if d.len() > 10 { &d[..10] } else { d })
        .unwrap_or("—");

    let (status_badge, status_text, row_border) = match status {
        "pending" => (
            "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
            "⏳ Pending Review",
            "border-yellow-500/20",
        ),
        "approved" => (
            "bg-green-500/10 text-green-400 border-green-500/20",
            "✅ Approved & Sent",
            "border-green-500/20",
        ),
        "rejected" => (
            "bg-red-500/10 text-red-400 border-red-500/20",
            "❌ Rejected (Refunded)",
            "border-red-500/20",
        ),
        _ => (
            "bg-gray-500/10 text-gray-400 border-gray-500/20",
            "Unknown",
            "border-gray-700",
        ),
    };

    rsx! {
        div { class: "bg-gray-900 rounded-lg border {row_border} p-4",
            div { class: "flex items-start justify-between gap-4",
                div { class: "flex-1",
                    div { class: "flex items-center gap-3 mb-2",
                        p { class: "text-xl font-bold text-white", "KES {amount as i32}" }
                        span { class: "px-2 py-0.5 rounded-full text-xs border {status_badge}",
                            "{status_text}"
                        }
                    }
                    div { class: "flex flex-wrap gap-x-4 gap-y-1 text-xs text-gray-400",
                        span { "📱 {phone}" }
                        span { "📅 Requested: {date_display}" }
                        span { "✓ Processed: {processed_display}" }
                    }
                    if let Some(notes) = admin_notes {
                        if !notes.is_empty() {
                            p { class: "text-gray-500 text-xs mt-2 italic", "💬 {notes}" }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Request Payout Modal
// ───────────────────────────────────────────
#[component]
fn RequestPayoutModal(
    available_balance: f64,
    token: String,
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
) -> Element {
    let mut amount = use_signal(|| String::new());
    let mut phone_number = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    let is_valid_phone = {
        let phone = phone_number.read().clone();
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        (digits.starts_with("254") && digits.len() == 12)
            || (digits.starts_with("0") && digits.len() == 10)
            || (digits.starts_with("7") && digits.len() == 9)
    };

    let amount_value = amount.read().parse::<f64>().unwrap_or(0.0);
    let is_valid_amount = amount_value >= 500.0 && amount_value <= available_balance;

    let token_for_submit = token.clone();
    let handle_submit = move |_| {
        if !is_valid_amount {
            error_message.set(Some("Amount must be between KES 500 and your available balance".to_string()));
            return;
        }
        if !is_valid_phone {
            error_message.set(Some("Invalid M-Pesa phone number".to_string()));
            return;
        }

        loading.set(true);
        error_message.set(None);

        let t = token_for_submit.clone();
        let amt = amount_value;
        let phone = phone_number.read().clone();
        let success_handler = on_success.clone();

        spawn(async move {
            match request_payout(&t, amt, &phone).await {
                Ok(result) => {
                    let msg = result.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Payout request submitted");
                    success_handler.call(format!("✅ {}", msg));
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4",
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-md w-full",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-xl font-bold text-white", "Request Payout" }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "bg-green-900/20 border border-green-500/30 rounded-lg p-4 mb-4",
                    p { class: "text-green-400 text-sm", "Available Balance" }
                    p { class: "text-2xl font-bold text-white", "KES {available_balance as i32}" }
                }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Amount (KES)" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            r#type: "number",
                            placeholder: "500",
                            min: "500",
                            max: "{available_balance as i32}",
                            value: "{amount}",
                            oninput: move |evt| {
                                amount.set(evt.value());
                                error_message.set(None);
                            },
                        }
                        p { class: "text-gray-500 text-xs mt-1",
                            "Minimum: KES 500 • Maximum: KES {available_balance as i32}"
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "M-Pesa Phone Number" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            r#type: "tel",
                            placeholder: "254712345678",
                            value: "{phone_number}",
                            oninput: move |evt| {
                                phone_number.set(evt.value());
                                error_message.set(None);
                            },
                        }
                        p { class: "text-gray-500 text-xs mt-1", "Funds will be sent to this number" }
                    }

                    if let Some(err) = error_message.read().as_ref() {
                        div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3",
                            p { class: "text-red-400 text-sm", "❌ {err}" }
                        }
                    }

                    div { class: "flex gap-2 pt-4 border-t border-gray-700",
                        button {
                            class: "flex-1 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium disabled:opacity-50",
                            disabled: *loading.read() || !is_valid_amount || !is_valid_phone,
                            onclick: handle_submit,
                            if *loading.read() { "Submitting..." } else { "Submit Request" }
                        }
                    }
                }
            }
        }
    }
}