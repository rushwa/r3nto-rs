use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;

const API_BASE_URL: &str = "http://localhost:8000";
const MINIMUM_PAYOUT: f64 = 500.0;

#[component]
pub fn AgentPayoutsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    // ─── Data signals ───
    let mut wallet = use_signal(|| Option::<serde_json::Value>::None);
    let mut recent_commissions = use_signal(|| Vec::<serde_json::Value>::new());
    let mut payout_history = use_signal(|| Vec::<serde_json::Value>::new());
    let mut tour_stats = use_signal(|| Option::<serde_json::Value>::None);
    let mut loading = use_signal(|| true);

    // ─── Payout form signals ───
    let mut payout_amount = use_signal(|| String::new());
    let mut payout_phone = use_signal(|| String::new());
    let mut submitting = use_signal(|| false);
    let mut form_message = use_signal(|| Option::<(bool, String)>::None); // (success, message)

    let token_for_wallet = token.clone();
    let token_for_history = token.clone();
    let token_for_tours = token.clone();

    // ─── Fetch wallet + commissions summary ───
    use_effect(move || {
        let t = token_for_wallet.clone();
        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/commissions/my/summary", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    wallet.set(data.get("wallet").cloned());
                    if let Some(comms) = data.get("recent_commissions").and_then(|v| v.as_array()) {
                        recent_commissions.set(comms.clone());
                    }
                }
            }
            loading.set(false);
        });
    });

    // ─── Fetch payout history ───
    use_effect(move || {
        let t = token_for_history.clone();
        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/payouts/my-history", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<Vec<serde_json::Value>>().await {
                    payout_history.set(data);
                }
            }
        });
    });

    // ─── Fetch tour stats (tour fee earnings) ───
    use_effect(move || {
        let t = token_for_tours.clone();
        spawn(async move {
            if let Ok(resp) = reqwest::Client::new()
                .get(&format!("{}/admin/agents/sla-stats", API_BASE_URL))
                .header("Authorization", format!("Bearer {}", t))
                .send().await
            {
                if let Ok(data) = resp.json::<serde_json::Value>().await {
                    tour_stats.set(Some(data));
                }
            }
        });
    });

    // ─── Payout submit handler ───
    let submit_payout = {
        let token = token.clone();
        move |_: MouseEvent| {
            let t = token.clone();
            let amount_str = payout_amount.read().clone();
            let phone = payout_phone.read().clone();
            let mut submitting_sig = submitting;
            let mut msg_sig = form_message;
            let mut amount_sig = payout_amount;
            let mut phone_sig = payout_phone;

            spawn(async move {
                let amount: f64 = match amount_str.trim().parse() {
                    Ok(a) => a,
                    Err(_) => {
                        msg_sig.set(Some((false, "Please enter a valid amount".to_string())));
                        return;
                    }
                };

                if amount < MINIMUM_PAYOUT {
                    msg_sig.set(Some((false, format!("Minimum payout is KES {:.0}", MINIMUM_PAYOUT))));
                    return;
                }

                if phone.trim().is_empty() {
                    msg_sig.set(Some((false, "Please enter your M-Pesa phone number".to_string())));
                    return;
                }

                submitting_sig.set(true);
                msg_sig.set(None);

                let client = reqwest::Client::new();
                let resp = client
                    .post(&format!("{}/admin/payouts/request", API_BASE_URL))
                    .header("Authorization", format!("Bearer {}", t))
                    .json(&serde_json::json!({
                        "amount": amount,
                        "mpesa_phone": phone.trim(),
                    }))
                    .send()
                    .await;

                match resp {
                    Ok(r) if r.status().is_success() => {
                        msg_sig.set(Some((true, "✅ Payout request submitted! An admin will review it shortly.".to_string())));
                        amount_sig.set(String::new());
                        phone_sig.set(String::new());
                    }
                    Ok(r) => {
                        let err = r.text().await.unwrap_or_else(|_| "Request failed".to_string());
                        msg_sig.set(Some((false, err)));
                    }
                    Err(e) => {
                        msg_sig.set(Some((false, format!("Network error: {}", e))));
                    }
                }
                submitting_sig.set(false);
            });
        }
    };

    // ═══════════════════════════════════════════
    // Pre-compute values BEFORE rsx! block
    // ═══════════════════════════════════════════
    let wallet_data = wallet.read().clone();
    let balance = wallet_data.as_ref().and_then(|w| w.get("balance")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let total_earned = wallet_data.as_ref().and_then(|w| w.get("total_earned")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let pending = wallet_data.as_ref().and_then(|w| w.get("pending_balance")).and_then(|v| v.as_f64()).unwrap_or(0.0);
    let withdrawn = wallet_data.as_ref().and_then(|w| w.get("total_withdrawn")).and_then(|v| v.as_f64()).unwrap_or(0.0);

    // Tour fee earnings (from SLA stats)
    let tour_data = tour_stats.read().clone();
    let tour_revenue: f64 = tour_data.as_ref()
        .and_then(|t| t.get("total_revenue_kes"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let tours_fulfilled: i64 = tour_data.as_ref().map(|t| {
        t.get("tours_fulfilled_on_time").and_then(|v| v.as_i64()).unwrap_or(0)
            + t.get("tours_fulfilled_late").and_then(|v| v.as_i64()).unwrap_or(0)
    }).unwrap_or(0);

    // Commission earnings = total earned minus tour fees
    let commission_earned = (total_earned - tour_revenue).max(0.0);

    let can_payout = balance >= MINIMUM_PAYOUT;
    let progress_pct = ((balance / MINIMUM_PAYOUT) * 100.0).min(100.0);

    let is_loading = *loading.read();
    let is_submitting = *submitting.read();
    let amount_val = payout_amount.read().clone();
    let phone_val = payout_phone.read().clone();
    let msg = form_message.read().clone();
    let commissions_list = recent_commissions.read().clone();
    let history_list = payout_history.read().clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "💸 My Payouts".to_string(),
                subtitle: "Track earnings from commissions, virtual tours & bonuses, and withdraw to M-Pesa".to_string(),
            }

            // ─── Wallet Overview Cards ───
            div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                WalletCard { icon: "💰", label: "Available Balance", value: format!("KES {:.2}", balance), color: "green" }
                WalletCard { icon: "📈", label: "Total Earned", value: format!("KES {:.2}", total_earned), color: "blue" }
                WalletCard { icon: "⏳", label: "Pending", value: format!("KES {:.2}", pending), color: "yellow" }
                WalletCard { icon: "🏦", label: "Total Withdrawn", value: format!("KES {:.2}", withdrawn), color: "purple" }
            }

            // ─── Earnings Breakdown ───
            div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                // Tour Fee Earnings
                div { class: "bg-gray-800 border border-yellow-500/30 rounded-lg p-6",
                    div { class: "flex items-center justify-between mb-3",
                        div { class: "flex items-center gap-3",
                            span { class: "text-3xl", "🎬" }
                            div {
                                h3 { class: "text-white font-bold", "Virtual Tour Fees" }
                                p { class: "text-gray-400 text-sm", "KES 20 per fulfilled tour" }
                            }
                        }
                        span { class: "text-yellow-400 font-bold text-xl", "KES {tour_revenue:.2}" }
                    }
                    div { class: "flex items-center justify-between text-sm",
                        span { class: "text-gray-400", "Tours fulfilled" }
                        span { class: "text-white font-semibold", "{tours_fulfilled}" }
                    }
                }

                // Commission Earnings
                div { class: "bg-gray-800 border border-blue-500/30 rounded-lg p-6",
                    div { class: "flex items-center justify-between mb-3",
                        div { class: "flex items-center gap-3",
                            span { class: "text-3xl", "🤝" }
                            div {
                                h3 { class: "text-white font-bold", "Commissions & Bonuses" }
                                p { class: "text-gray-400 text-sm", "Handshake, subscription & referrals" }
                            }
                        }
                        span { class: "text-blue-400 font-bold text-xl", "KES {commission_earned:.2}" }
                    }
                    div { class: "flex items-center justify-between text-sm",
                        span { class: "text-gray-400", "Recent transactions" }
                        span { class: "text-white font-semibold", "{commissions_list.len()}" }
                    }
                }
            }

            // ─── Payout Request Section ───
            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                // Form
                div { class: "lg:col-span-2 bg-gray-800 border border-gray-700 rounded-lg p-6",
                    h3 { class: "text-white font-bold text-lg mb-4", "💸 Request a Payout" }

                    // Progress toward minimum
                    div { class: "mb-5",
                        div { class: "flex justify-between mb-2 text-sm",
                            span { class: "text-gray-400", "Progress to minimum payout" }
                            span { class: if can_payout { "text-green-400 font-semibold" } else { "text-yellow-400 font-semibold" },
                                "KES {balance:.2} / KES {MINIMUM_PAYOUT:.0}"
                            }
                        }
                        div { class: "w-full bg-gray-700 rounded-full h-3",
                            div {
                                class: if can_payout { "bg-green-500 h-3 rounded-full transition-all" } else { "bg-yellow-500 h-3 rounded-full transition-all" },
                                style: "width: {progress_pct}%"
                            }
                        }
                        if !can_payout {
                            p { class: "text-gray-500 text-xs mt-2",
                                "You need KES {(MINIMUM_PAYOUT - balance).max(0.0):.2} more to request a payout. Complete more tours and conversions!"
                            }
                        }
                    }

                    // Form message
                    if let Some((success, message)) = msg.as_ref() {
                        div {
                            class: if *success {
                                "bg-green-900/20 border border-green-500/30 rounded-lg p-3 mb-4"
                            } else {
                                "bg-red-900/20 border border-red-500/30 rounded-lg p-3 mb-4"
                            },
                            p { class: if *success { "text-green-400 text-sm" } else { "text-red-400 text-sm" }, "{message}" }
                        }
                    }

                    // Inputs
                    div { class: "space-y-4",
                        div {
                            label { class: "block text-gray-400 text-sm mb-1", "Amount (KES)" }
                            input {
                                class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500",
                                r#type: "number",
                                placeholder: "e.g. 500",
                                value: "{amount_val}",
                                oninput: move |e| payout_amount.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-gray-400 text-sm mb-1", "M-Pesa Phone Number" }
                            input {
                                class: "w-full px-3 py-2 bg-gray-700 text-white border border-gray-700 rounded-lg focus:ring-2 focus:ring-blue-500",
                                r#type: "tel",
                                placeholder: "e.g. 0712345678",
                                value: "{phone_val}",
                                oninput: move |e| payout_phone.set(e.value()),
                            }
                        }
                        button {
                            class: if can_payout && !is_submitting {
                                "w-full bg-green-600 hover:bg-green-500 text-white font-bold py-3 px-4 rounded-lg transition-colors"
                            } else {
                                "w-full bg-gray-600 text-gray-400 font-bold py-3 px-4 rounded-lg cursor-not-allowed"
                            },
                            disabled: !can_payout || is_submitting,
                            onclick: submit_payout,
                            if is_submitting { "Submitting..." }
                            else if can_payout { "Submit Payout Request" }
                            else { "Minimum KES 500 Required" }
                        }
                    }
                }

                // Info panel
                div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-6",
                    h3 { class: "text-blue-400 font-bold mb-3", "ℹ️ How Payouts Work" }
                    ul { class: "text-gray-300 text-sm space-y-2 list-disc list-inside",
                        li { "Earn KES 20 for every virtual tour you fulfill" }
                        li { "Earn commissions from handshake & subscriptions" }
                        li { "Minimum withdrawal is KES 500" }
                        li { "Payouts are sent via M-Pesa B2C" }
                        li { "An admin reviews each request before sending" }
                    }
                }
            }

            // ─── Payout History ───
            div { class: "bg-gray-800 border border-gray-700 rounded-lg p-6",
                h3 { class: "text-white font-bold text-lg mb-4", "📜 Payout History" }

                if is_loading {
                    div { class: "flex items-center justify-center py-8",
                        div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" }
                    }
                } else if history_list.is_empty() {
                    EmptyState {
                        icon: "💸".to_string(),
                        title: "No payout requests yet".to_string(),
                        message: "Once you reach KES 500, request your first payout here.".to_string(),
                    }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "min-w-full divide-y divide-gray-700",
                            thead {
                                tr {
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Amount" }
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Phone" }
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Status" }
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Requested" }
                                }
                            }
                            tbody { class: "divide-y divide-gray-700",
                                for payout in history_list.iter() {
                                    PayoutRow { payout: payout.clone() }
                                }
                            }
                        }
                    }
                }
            }

            // ─── Recent Earnings ───
            div { class: "bg-gray-800 border border-gray-700 rounded-lg p-6",
                h3 { class: "text-white font-bold text-lg mb-4", "💰 Recent Earnings" }

                if commissions_list.is_empty() {
                    EmptyState {
                        icon: "💰".to_string(),
                        title: "No earnings yet".to_string(),
                        message: "Fulfill tours and convert owners to start earning.".to_string(),
                    }
                } else {
                    div { class: "space-y-3",
                        for comm in commissions_list.iter() {
                            EarningRow { commission: comm.clone() }
                        }
                    }
                }
            }
        }
    }
}

// ═══════════════════════════════════════════
// Wallet Card
// ═══════════════════════════════════════════
#[component]
fn WalletCard(icon: String, label: String, value: String, color: String) -> Element {
    let border_color = match color.as_str() {
        "green" => "border-green-500/30",
        "blue" => "border-blue-500/30",
        "yellow" => "border-yellow-500/30",
        "purple" => "border-purple-500/30",
        _ => "border-gray-500/30",
    };

    rsx! {
        div { class: "bg-gray-800 border {border_color} rounded-lg p-5",
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

// ═══════════════════════════════════════════
// Payout History Row
// ═══════════════════════════════════════════
#[component]
fn PayoutRow(payout: serde_json::Value) -> Element {
    let amount = payout.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let phone = payout.get("mpesa_phone").and_then(|v| v.as_str()).unwrap_or("");
    let status = payout.get("status").and_then(|v| v.as_str()).unwrap_or("pending");
    let created_at = payout.get("created_at").and_then(|v| v.as_str()).unwrap_or("");

    let (status_color, status_icon) = match status {
        "pending" => ("bg-yellow-500/20 text-yellow-400 border-yellow-500/30", "⏳"),
        "approved" => ("bg-green-500/20 text-green-400 border-green-500/30", "✅"),
        "rejected" => ("bg-red-500/20 text-red-400 border-red-500/30", "❌"),
        _ => ("bg-gray-500/20 text-gray-400 border-gray-500/30", "📋"),
    };

    // Show only first 16 chars of timestamp for readability
    let date_display: String = created_at.chars().take(16).collect();

    rsx! {
        tr { class: "hover:bg-gray-700/50",
            td { class: "px-4 py-3 text-white font-semibold", "KES {amount:.2}" }
            td { class: "px-4 py-3 text-gray-400", "{phone}" }
            td { class: "px-4 py-3",
                span { class: "px-2 py-1 rounded-full text-xs border {status_color}",
                    "{status_icon} {status}"
                }
            }
            td { class: "px-4 py-3 text-gray-400 text-sm", "{date_display}" }
        }
    }
}

// ═══════════════════════════════════════════
// Recent Earnings Row
// ═══════════════════════════════════════════
#[component]
fn EarningRow(commission: serde_json::Value) -> Element {
    let amount = commission.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let comm_type = commission.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let created_at = commission.get("created_at").and_then(|v| v.as_str()).unwrap_or("");

    let (label, icon) = match comm_type {
        "tour_fee" => ("Virtual Tour Fee", "🎬"),
        "handshake_30pct" => ("Registration Fee Commission (30%)", "🤝"),
        "subscription_10pct" => ("Subscription Commission (10%)", "⭐"),
        "referral_bonus" => ("Referral Bonus", "🏆"),
        _ => (comm_type, "💰"),
    };

    let date_display: String = created_at.chars().take(16).collect();

    rsx! {
        div { class: "flex items-center justify-between p-3 bg-gray-700/30 rounded-lg",
            div { class: "flex items-center gap-3",
                span { class: "text-2xl", "{icon}" }
                div {
                    p { class: "text-white text-sm font-medium", "{label}" }
                    p { class: "text-gray-500 text-xs", "{date_display}" }
                }
            }
            span { class: "text-green-400 font-bold", "+KES {amount:.2}" }
        }
    }
}