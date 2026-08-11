use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{
    get_all_payout_history, get_payout_stats, approve_payout, reject_payout,
};

#[component]
pub fn PayoutsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut payouts = use_signal(|| Vec::<serde_json::Value>::new());
    let mut stats = use_signal(|| Option::<serde_json::Value>::None);
    let mut loading = use_signal(|| true);
    let mut status_filter = use_signal(|| Option::<String>::None);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);
    let mut action_loading = use_signal(|| Option::<String>::None);
    let mut show_notes_modal = use_signal(|| Option::<(String, String)>::None); // (payout_id, action)

    // ✅ Clone before use_effect
    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        let filter = status_filter.read().clone();
        spawn(async move {
            loading.set(true);
            // Fetch stats
            if let Ok(s) = get_payout_stats(&t).await {
                stats.set(Some(s));
            }
            // Fetch history
            match get_all_payout_history(&t, filter.as_deref()).await {
                Ok(data) => payouts.set(data),
                Err(e) => {
                    message.set(Some(format!("Failed to load: {}", e)));
                    is_error.set(true);
                }
            }
            loading.set(false);
        });
    });

    let pending_count = stats.read().as_ref()
        .and_then(|s| s.get("pending_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let pending_amount = stats.read().as_ref()
        .and_then(|s| s.get("pending_amount"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let approved_amount = stats.read().as_ref()
        .and_then(|s| s.get("approved_amount"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let approved_count = stats.read().as_ref()
        .and_then(|s| s.get("approved_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let rejected_count = stats.read().as_ref()
        .and_then(|s| s.get("rejected_count"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    // ✅ FIX: Single action handler using tuple (payout_id, action)
    let handle_action = {
        let token = token.clone();
        let mut action_loading = action_loading.clone();
        let mut payouts_signal = payouts.clone();
        let mut message_signal = message.clone();
        let mut is_error_signal = is_error.clone();
        let status_filter_clone = status_filter.clone();
        move |(payout_id, action): (String, String), notes: Option<String>| {
            let t = token.clone();
            let pid = payout_id.clone();
            let act = action.clone();
            action_loading.set(Some(payout_id.clone()));
            spawn(async move {
                let result = if act == "approve" {
                    approve_payout(&t, &pid, notes.as_deref()).await
                } else {
                    reject_payout(&t, &pid, notes.as_deref()).await
                };

                match result {
                    Ok(_) => {
                        let action_word = if act == "approve" { "approved" } else { "rejected" };
                        let short_id = if pid.len() >= 8 { &pid[..8] } else { &pid };
                        message_signal.set(Some(format!("✅ Payout {} {}", short_id, action_word)));
                        is_error_signal.set(false);
                    }
                    Err(e) => {
                        message_signal.set(Some(format!("Failed: {}", e)));
                        is_error_signal.set(true);
                    }
                }
                action_loading.set(None);

                // Refetch
                let filter = status_filter_clone.read().clone();
                if let Ok(data) = get_all_payout_history(&t, filter.as_deref()).await {
                    payouts_signal.set(data);
                }
                // Also refetch stats
                if let Ok(s) = get_payout_stats(&t).await {
                    stats.set(Some(s));
                }
            });
        }
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white", "Loading payouts..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Agent Payouts".to_string(),
                subtitle: "Review and process agent payout requests".to_string(),
            }

            // Stats Grid
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-4",
                    p { class: "text-yellow-400 text-sm", "⏳ Pending Requests" }
                    p { class: "text-2xl font-bold text-white mt-1", "{pending_count}" }
                    p { class: "text-yellow-400/70 text-xs mt-1", "KES {pending_amount as i32}" }
                }
                div { class: "bg-green-900/20 border border-green-500/30 rounded-lg p-4",
                    p { class: "text-green-400 text-sm", "✅ Approved" }
                    p { class: "text-2xl font-bold text-white mt-1", "{approved_count}" }
                    p { class: "text-green-400/70 text-xs mt-1", "KES {approved_amount as i32}" }
                }
                div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-4",
                    p { class: "text-red-400 text-sm", "❌ Rejected" }
                    p { class: "text-2xl font-bold text-white mt-1", "{rejected_count}" }
                }
                div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4",
                    p { class: "text-blue-400 text-sm", "📊 Total Processed" }
                    p { class: "text-2xl font-bold text-white mt-1", "{approved_count + rejected_count}" }
                }
            }

            // Filter Tabs
            div { class: "flex gap-2 flex-wrap",
                FilterButton {
                    label: "All".to_string(),
                    is_active: status_filter.read().is_none(),
                    onclick: {
                        let mut filter = status_filter.clone();
                        move |_| filter.set(None)
                    },
                }
                FilterButton {
                    label: format!("Pending ({})", pending_count),
                    is_active: status_filter.read().as_deref() == Some("pending"),
                    onclick: {
                        let mut filter = status_filter.clone();
                        move |_| filter.set(Some("pending".to_string()))
                    },
                }
                FilterButton {
                    label: format!("Approved ({})", approved_count),
                    is_active: status_filter.read().as_deref() == Some("approved"),
                    onclick: {
                        let mut filter = status_filter.clone();
                        move |_| filter.set(Some("approved".to_string()))
                    },
                }
                FilterButton {
                    label: format!("Rejected ({})", rejected_count),
                    is_active: status_filter.read().as_deref() == Some("rejected"),
                    onclick: {
                        let mut filter = status_filter.clone();
                        move |_| filter.set(Some("rejected".to_string()))
                    },
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

            // Payouts Table
            div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                if payouts.read().is_empty() {
                    EmptyState {
                        icon: "💰".to_string(),
                        title: "No payouts found".to_string(),
                        message: "No payout requests match the current filter.".to_string(),
                    }
                } else {
                    div { class: "overflow-x-auto",
                        table { class: "w-full",
                            thead { class: "bg-gray-900",
                                tr {
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Agent" }
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Amount" }
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "M-Pesa" }
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Status" }
                                    th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Date" }
                                    th { class: "px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase", "Actions" }
                                }
                            }
                            tbody { class: "divide-y divide-gray-700",
                                for payout in payouts.read().iter() {
                                    PayoutRow {
                                        payout: payout.clone(),
                                        action_loading: action_loading.clone(),
                                        on_action: {
                                            let mut modal_state = show_notes_modal.clone();
                                            move |(payout_id, action): (String, String)| {
                                                modal_state.set(Some((payout_id, action)));
                                            }
                                        },
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Admin Notes Modal
            if let Some((payout_id, action)) = show_notes_modal.read().clone() {
                AdminNotesModal {
                    payout_id: payout_id.clone(),
                    action: action.clone(),
                    on_close: {
                        let mut modal_state = show_notes_modal.clone();
                        move |_| modal_state.set(None)
                    },
                    // ✅ AFTER (fixed):
                    on_submit: {
                        let mut modal_state = show_notes_modal.clone();
                        let mut handler = handle_action.clone();  // ✅ Added `mut`
                        let pid = payout_id.clone();
                        let act = action.clone();
                        move |notes: Option<String>| {
                            modal_state.set(None);
                            handler((pid.clone(), act.clone()), notes);  // ✅ Now works
                        }
                    },
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Filter Button
// ───────────────────────────────────────────
#[component]
fn FilterButton(label: String, is_active: bool, onclick: EventHandler<()>) -> Element {
    let btn_class = if is_active {
        "px-4 py-2 bg-blue-600 text-white rounded-lg font-medium text-sm"
    } else {
        "px-4 py-2 bg-gray-800 hover:bg-gray-700 text-gray-300 rounded-lg font-medium text-sm border border-gray-700"
    };

    rsx! {
        button {
            class: "{btn_class}",
            onclick: move |_| onclick.call(()),
            "{label}"
        }
    }
}

// ───────────────────────────────────────────
// Payout Row
// ───────────────────────────────────────────
#[component]
fn PayoutRow(
    payout: serde_json::Value,
    action_loading: Signal<Option<String>>,
    on_action: EventHandler<(String, String)>, // ✅ Single tuple-based handler
) -> Element {
    let id = payout.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let agent_name = payout.get("agent_name").and_then(|v| v.as_str()).unwrap_or("—");
    let agent_email = payout.get("agent_email").and_then(|v| v.as_str()).unwrap_or("—");
    let amount = payout.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let phone = payout.get("mpesa_phone").and_then(|v| v.as_str()).unwrap_or("—");
    let status = payout.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    let created_at = payout.get("created_at").and_then(|v| v.as_str()).unwrap_or("—");
    let admin_notes = payout.get("admin_notes").and_then(|v| v.as_str());
    let date_display = if created_at.len() > 10 { &created_at[..10] } else { created_at };
    let is_loading = action_loading.read().as_ref() == Some(&id);

    let status_badge = match status {
        "pending" => "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
        "approved" => "bg-green-500/10 text-green-400 border-green-500/20",
        "rejected" => "bg-red-500/10 text-red-400 border-red-500/20",
        _ => "bg-gray-500/10 text-gray-400 border-gray-500/20",
    };

    // ✅ FIX: Clone id for each closure separately
    let id_for_reject = id.clone();
    let id_for_approve = id.clone();

    rsx! {
        tr { class: "hover:bg-gray-700/30 transition-colors",
            td { class: "px-4 py-3",
                div { class: "text-white font-medium", "{agent_name}" }
                div { class: "text-gray-500 text-xs", "{agent_email}" }
            }
            td { class: "px-4 py-3 text-blue-400 font-bold", "KES {amount as i32}" }
            td { class: "px-4 py-3 text-gray-300 font-mono text-sm", "{phone}" }
            td { class: "px-4 py-3",
                div { class: "flex flex-col gap-1",
                    span { class: "px-2 py-1 rounded-full text-xs border {status_badge} inline-block w-fit", "{status}" }
                    if let Some(notes) = admin_notes {
                        if !notes.is_empty() {
                            span { class: "text-gray-500 text-xs italic", "💬 {notes}" }
                        }
                    }
                }
            }
            td { class: "px-4 py-3 text-gray-400 text-sm", "{date_display}" }
            td { class: "px-4 py-3 text-right",
                if status == "pending" {
                    div { class: "flex gap-2 justify-end",
                        button {
                            class: "px-3 py-1 bg-red-600/20 hover:bg-red-600/40 text-red-400 rounded text-sm disabled:opacity-50",
                            disabled: is_loading,
                            onclick: move |_| {
                                on_action.call((id_for_reject.clone(), "reject".to_string()));
                            },
                            if is_loading { "..." } else { "Reject" }
                        }
                        button {
                            class: "px-3 py-1 bg-green-600 hover:bg-green-500 text-white rounded text-sm disabled:opacity-50",
                            disabled: is_loading,
                            onclick: move |_| {
                                on_action.call((id_for_approve.clone(), "approve".to_string()));
                            },
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

// ───────────────────────────────────────────
// Admin Notes Modal
// ───────────────────────────────────────────
#[component]
fn AdminNotesModal(
    payout_id: String,
    action: String,
    on_close: EventHandler<()>,
    on_submit: EventHandler<Option<String>>,
) -> Element {
    let mut notes = use_signal(|| String::new());

    let is_approve = action == "approve";
    let title = if is_approve { "Approve Payout" } else { "Reject Payout" };
    let color = if is_approve { "green" } else { "red" };
    let short_id = if payout_id.len() >= 8 { &payout_id[..8] } else { &payout_id };

    let placeholder = if is_approve {
        "e.g., Verified agent identity, funds sent via M-Pesa"
    } else {
        "e.g., Reason for rejection (will be visible to agent)"
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4",
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-md w-full",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-xl font-bold text-white", "{title}" }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "bg-gray-900 rounded-lg p-3 mb-4",
                    p { class: "text-gray-400 text-xs mb-1", "Payout ID" }
                    p { class: "text-white font-mono text-sm", "{short_id}..." }
                }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1",
                            "Admin Notes (optional)"
                        }
                        textarea {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                            rows: "4",
                            placeholder: "{placeholder}",
                            value: "{notes}",
                            oninput: move |evt| notes.set(evt.value()),
                        }
                        p { class: "text-gray-500 text-xs mt-1",
                            if is_approve {
                                "These notes will be recorded for audit purposes."
                            } else {
                                "These notes will be visible to the agent."
                            }
                        }
                    }

                    if is_approve {
                        div { class: "bg-green-900/20 border border-green-500/30 rounded-lg p-3",
                            p { class: "text-green-400 text-sm",
                                "⚠️ Approving will mark the payout as complete. Ensure funds have been sent via M-Pesa."
                            }
                        }
                    } else {
                        div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3",
                            p { class: "text-red-400 text-sm",
                                "⚠️ Rejecting will refund the funds back to the agent's wallet."
                            }
                        }
                    }

                    div { class: "flex gap-2 pt-4 border-t border-gray-700",
                        button {
                            class: "flex-1 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 py-2.5 bg-{color}-600 hover:bg-{color}-500 text-white rounded-lg font-medium",
                            onclick: {
                                let notes_clone = notes.read().clone();
                                move |_| {
                                    let n = if notes_clone.is_empty() { None } else { Some(notes_clone.clone()) };
                                    on_submit.call(n);
                                }
                            },
                            if is_approve { "Confirm Approval" } else { "Confirm Rejection" }
                        }
                    }
                }
            }
        }
    }
}