use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_pending_payouts, approve_payout, reject_payout};

#[component]
pub fn PayoutsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut payouts = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut action_loading = use_signal(|| Option::<String>::None);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    let fetch_payouts = {
        let token = token.clone();
        move || {
            let t = token.clone();
            spawn(async move {
                loading.set(true);
                match get_pending_payouts(&t).await {
                    Ok(data) => payouts.set(data),
                    Err(e) => {
                        message.set(Some(format!("Failed to load payouts: {}", e)));
                        is_error.set(true);
                    }
                }
                loading.set(false);
            });
        }
    };

    use_effect(move || {
        fetch_payouts();
    });

    let handle_approve = {
        let token = token.clone();
        move |payout_id: String| {
            let t = token.clone();
            let pid = payout_id.clone();
            action_loading.set(Some(payout_id.clone()));
            spawn(async move {
                match approve_payout(&t, &pid).await {
                    Ok(_) => {
                        message.set(Some(format!("✅ Payout {} approved", &pid[..8])));
                        is_error.set(false);
                    }
                    Err(e) => {
                        message.set(Some(format!("Failed: {}", e)));
                        is_error.set(true);
                    }
                }
                action_loading.set(None);
                match get_pending_payouts(&t).await {
                    Ok(data) => payouts.set(data),
                    Err(_) => {}
                }
            });
        }
    };

    let handle_reject = {
        let token = token.clone();
        move |payout_id: String| {
            let t = token.clone();
            let pid = payout_id.clone();
            action_loading.set(Some(payout_id.clone()));
            spawn(async move {
                match reject_payout(&t, &pid).await {
                    Ok(_) => {
                        message.set(Some(format!("❌ Payout {} rejected, funds refunded", &pid[..8])));
                        is_error.set(false);
                    }
                    Err(e) => {
                        message.set(Some(format!("Failed: {}", e)));
                        is_error.set(true);
                    }
                }
                action_loading.set(None);
                match get_pending_payouts(&t).await {
                    Ok(data) => payouts.set(data),
                    Err(_) => {}
                }
            });
        }
    };

    let pending_count = payouts.read().iter()
        .filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("pending"))
        .count();
    let total_pending_amount: f64 = payouts.read().iter()
        .filter(|p| p.get("status").and_then(|s| s.as_str()) == Some("pending"))
        .filter_map(|p| p.get("amount").and_then(|v| v.as_f64()))
        .sum();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Agent Payouts".to_string(),
                subtitle: format!("{} pending requests totaling KES {:.2}", pending_count, total_pending_amount),
            }

            if *loading.read() {
                div { class: "flex items-center justify-center h-64",
                    div { class: "text-white", "Loading payouts..." }
                }
            } else if payouts.read().is_empty() {
                EmptyState {
                    icon: "💰".to_string(),
                    title: "No payout requests".to_string(),
                    message: "There are no pending payout requests from agents.".to_string(),
                }
            } else {
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                    div { class: "bg-gray-800 rounded-lg border border-gray-700 p-4",
                        p { class: "text-gray-400 text-sm", "Total Requests" }
                        p { class: "text-2xl font-bold text-white", "{payouts.read().len()}" }
                    }
                    div { class: "bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-4",
                        p { class: "text-yellow-400 text-sm", "Pending" }
                        p { class: "text-2xl font-bold text-yellow-400", "{pending_count}" }
                    }
                    div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4",
                        p { class: "text-blue-400 text-sm", "Total Pending Amount" }
                        p { class: "text-2xl font-bold text-blue-400", "KES {total_pending_amount as i32}" }
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
                            class: if *is_error.read() { "text-red-400 text-sm" } else { "text-green-400 text-sm" },
                            "{msg}"
                        }
                    }
                }

                div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                    table { class: "w-full",
                        thead { class: "bg-gray-900",
                            tr {
                                th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Agent" }
                                th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Amount" }
                                th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "M-Pesa Phone" }
                                th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Status" }
                                th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Requested" }
                                th { class: "px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase", "Actions" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-700",
                            for payout in payouts.read().iter() {
                                PayoutRow {
                                    payout: payout.clone(),
                                    action_loading: action_loading.clone(),
                                    on_approve: handle_approve.clone(),
                                    on_reject: handle_reject.clone(),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PayoutRow(
    payout: serde_json::Value,
    action_loading: Signal<Option<String>>,
    on_approve: EventHandler<String>,
    on_reject: EventHandler<String>,
) -> Element {
    let id = payout.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let agent_name = payout.get("agent_name").and_then(|v| v.as_str()).unwrap_or("—");
    let agent_email = payout.get("agent_email").and_then(|v| v.as_str()).unwrap_or("—");
    let amount = payout.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let phone = payout.get("mpesa_phone").and_then(|v| v.as_str()).unwrap_or("—");
    let status = payout.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    let created_at = payout.get("created_at").and_then(|v| v.as_str()).unwrap_or("—");
    let date_display = if created_at.len() > 10 { &created_at[..10] } else { created_at };

    let is_loading = action_loading.read().as_ref() == Some(&id);

    let status_badge = match status {
        "pending" => "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
        "approved" => "bg-green-500/10 text-green-400 border-green-500/20",
        "rejected" => "bg-red-500/10 text-red-400 border-red-500/20",
        _ => "bg-gray-500/10 text-gray-400 border-gray-500/20",
    };

    // ✅ FIX: Clone id for each closure
    let id_for_approve = id.clone();
    let id_for_reject = id.clone();

    rsx! {
        tr { class: "hover:bg-gray-700/30 transition-colors",
            td { class: "px-4 py-3",
                div { class: "text-white font-medium", "{agent_name}" }
                div { class: "text-gray-500 text-xs", "{agent_email}" }
            }
            td { class: "px-4 py-3 text-blue-400 font-bold", "KES {amount as i32}" }
            td { class: "px-4 py-3 text-gray-300 font-mono text-sm", "{phone}" }
            td { class: "px-4 py-3",
                span { class: "px-2 py-1 rounded-full text-xs border {status_badge}", "{status}" }
            }
            td { class: "px-4 py-3 text-gray-400 text-sm", "{date_display}" }
            td { class: "px-4 py-3 text-right",
                if status == "pending" {
                    div { class: "flex gap-2 justify-end",
                        button {
                            class: "px-3 py-1 bg-red-600/20 hover:bg-red-600/40 text-red-400 rounded text-sm disabled:opacity-50",
                            disabled: is_loading,
                            onclick: move |_| on_reject.call(id_for_reject.clone()),
                            if is_loading { "..." } else { "Reject" }
                        }
                        button {
                            class: "px-3 py-1 bg-green-600 hover:bg-green-500 text-white rounded text-sm disabled:opacity-50",
                            disabled: is_loading,
                            onclick: move |_| on_approve.call(id_for_approve.clone()),
                            if is_loading { "..." } else { "Approve" }
                        }
                    }
                } else {
                    span { class: "text-gray-500 text-sm", "—" }
                }
            }
        }
    }
}