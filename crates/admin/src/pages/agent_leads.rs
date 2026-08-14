use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_agent_leads, update_lead_stage};

#[component]
pub fn LeadsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut leads = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            match get_agent_leads(&t).await {
                Ok(data) => leads.set(data),
                Err(e) => {
                    message.set(Some(format!("Failed to load leads: {}", e)));
                    is_error.set(true);
                }
            }
            loading.set(false);
        });
    });

    // ✅ FIX: Closure accepts tuple (String, String)
    let update_stage = {
        let token = token.clone();
        let mut leads_signal = leads.clone();
        let mut message_signal = message.clone();
        let mut is_error_signal = is_error.clone();

        move |(lead_id, new_stage): (String, String)| {
            let t = token.clone();
            let lid = lead_id.clone();
            let stage = new_stage.clone();

            spawn(async move {
                match update_lead_stage(&t, &lid, &stage).await {
                    Ok(_) => {
                        message_signal.set(Some(format!("✅ Lead moved to {}", stage.replace('_', " "))));
                        is_error_signal.set(false);

                        // Optimistic update
                        let mut list = leads_signal.write();
                        for item in list.iter_mut() {
                            if item.get("id").and_then(|v| v.as_str()) == Some(&lid) {
                                item["pipeline_stage"] = serde_json::Value::String(stage.clone());
                            }
                        }
                    }
                    Err(e) => {
                        message_signal.set(Some(format!("Failed: {}", e)));
                        is_error_signal.set(true);
                    }
                }
            });
        }
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading leads..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Lead Pipeline".to_string(),
                subtitle: "Manage and track your client conversions".to_string(),
            }

            if let Some(msg) = message.read().as_ref() {
                div {
                    class: if *is_error.read() { "bg-red-900/20 border border-red-500/30 rounded-lg p-3" } else { "bg-green-900/20 border border-green-500/30 rounded-lg p-3" },
                    p { class: if *is_error.read() { "text-red-400" } else { "text-green-400" }, "{msg}" }
                }
            }

            if leads.read().is_empty() {
                EmptyState {
                    icon: "👥".to_string(),
                    title: "No leads yet".to_string(),
                    message: "Use your digital handshake or referral link to start gathering leads.".to_string(),
                }
            } else {
                div { class: "space-y-4",
                    for lead in leads.read().iter() {
                        LeadCard {
                            lead: lead.clone(),
                            on_update_stage: update_stage.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn LeadCard(
    lead: serde_json::Value,
    on_update_stage: EventHandler<(String, String)>,
) -> Element {
    let id = lead.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = lead.get("client_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let email = lead.get("client_email").and_then(|v| v.as_str()).unwrap_or("—");
    let phone = lead.get("client_phone").and_then(|v| v.as_str()).unwrap_or("—");
    let property = lead.get("property_title").and_then(|v| v.as_str()).unwrap_or("General Lead");
    let stage = lead.get("pipeline_stage").and_then(|v| v.as_str()).unwrap_or("new");
    let lead_status = lead.get("lead_status").and_then(|v| v.as_str()).unwrap_or("");
    let created_at = lead.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let date_display = if created_at.len() > 10 { &created_at[..10] } else { created_at };

    let (stage_color, stage_label) = match stage {
        "new" => ("bg-blue-500/10 text-blue-400 border-blue-500/20", "🆕 New"),
        "contacted" => ("bg-yellow-500/10 text-yellow-400 border-yellow-500/20", "📞 Contacted"),
        "viewing_scheduled" => ("bg-purple-500/10 text-purple-400 border-purple-500/20", "📅 Viewing"),
        "negotiation" => ("bg-orange-500/10 text-orange-400 border-orange-500/20", "🤝 Negotiation"),
        "closed" => ("bg-green-500/10 text-green-400 border-green-500/20", "✅ Closed"),
        "lost" => ("bg-gray-500/10 text-gray-400 border-gray-500/20", "❌ Lost"),
        _ => ("bg-gray-500/10 text-gray-400 border-gray-500/20", "Unknown"),
    };

    // ✅ Extract boolean conditions BEFORE rsx!
    let show_contacted_btn = stage != "contacted";
    let show_viewing_btn = stage != "viewing_scheduled" && (stage == "new" || stage == "contacted");
    let show_negotiation_btn = stage != "negotiation" && stage == "viewing_scheduled";
    let show_closed_lost_btns = stage != "closed" && stage != "lost";

    let id_for_contacted = id.clone();
    let id_for_viewing = id.clone();
    let id_for_negotiation = id.clone();
    let id_for_closed = id.clone();
    let id_for_lost = id.clone();

    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5 hover:border-gray-600 transition-colors",
            div { class: "flex flex-col md:flex-row md:items-start justify-between gap-4",
                div { class: "flex-1",
                    div { class: "flex items-center gap-3 mb-2 flex-wrap",
                        h4 { class: "text-white font-semibold text-lg", "{name}" }
                        span { class: "px-2 py-0.5 rounded-full text-xs border {stage_color}", "{stage_label}" }
                        if !lead_status.is_empty() && lead_status != "pending" {
                            span { class: "px-2 py-0.5 rounded-full text-xs bg-gray-700 text-gray-300",
                                "Status: {lead_status}"
                            }
                        }
                    }
                    div { class: "flex flex-wrap gap-x-4 gap-y-1 text-sm text-gray-400 mb-3",
                        if email != "—" { span { "✉️ {email}" } }
                        if phone != "—" { span { "📱 {phone}" } }
                        span { "🏠 {property}" }
                        span { "📅 {date_display}" }
                    }
                }

                div { class: "flex flex-col items-end gap-2",
                    span { class: "text-gray-500 text-xs mb-1", "Move to stage:" }
                    div { class: "flex flex-wrap gap-2",
                        if show_contacted_btn {
                            button {
                                class: "px-3 py-1 text-xs rounded bg-yellow-600/20 text-yellow-400 hover:bg-yellow-600/40 transition-colors",
                                onclick: move |_| {
                                    on_update_stage.call((id_for_contacted.clone(), "contacted".to_string()));
                                },
                                "Contacted"
                            }
                        }
                        if show_viewing_btn {
                            button {
                                class: "px-3 py-1 text-xs rounded bg-purple-600/20 text-purple-400 hover:bg-purple-600/40 transition-colors",
                                onclick: move |_| {
                                    on_update_stage.call((id_for_viewing.clone(), "viewing_scheduled".to_string()));
                                },
                                "Viewing"
                            }
                        }
                        if show_negotiation_btn {
                            button {
                                class: "px-3 py-1 text-xs rounded bg-orange-600/20 text-orange-400 hover:bg-orange-600/40 transition-colors",
                                onclick: move |_| {
                                    on_update_stage.call((id_for_negotiation.clone(), "negotiation".to_string()));
                                },
                                "Negotiate"
                            }
                        }
                        if show_closed_lost_btns {
                            button {
                                class: "px-3 py-1 text-xs rounded bg-green-600/20 text-green-400 hover:bg-green-600/40 transition-colors",
                                onclick: move |_| {
                                    on_update_stage.call((id_for_closed.clone(), "closed".to_string()));
                                },
                                "Closed"
                            }
                            button {
                                class: "px-3 py-1 text-xs rounded bg-gray-600/20 text-gray-400 hover:bg-gray-600/40 transition-colors",
                                onclick: move |_| {
                                    on_update_stage.call((id_for_lost.clone(), "lost".to_string()));
                                },
                                "Lost"
                            }
                        }
                    }
                }
            }
        }
    }
}