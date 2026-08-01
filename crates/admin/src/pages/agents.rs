use dioxus::prelude::*;

use crate::api::admin::{get_agents, Agent};
use crate::components::sidebar::{PageHeader, StatusBadge, FilterBar, DataTable, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn AgentsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_resource = token.clone();
    let mut search = use_signal(|| String::new());
    let mut status_filter = use_signal(|| "all".to_string());

    let agents = use_resource(move || {
        let t = token_for_resource.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![]);
            }
            get_agents(&t).await
        }
    });

    let agents_ref = agents.read();
    let agents_data = match agents_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    let filtered: Vec<Agent> = agents_data.clone().unwrap_or_default().into_iter().filter(|a| {
        let s = search.read().to_lowercase();
        let st = status_filter.read().clone();
        let matches_search = s.is_empty() || a.name.to_lowercase().contains(&s) || a.email.to_lowercase().contains(&s);
        let matches_status = st == "all" || (st == "verified" && a.verified) || (st == "unverified" && !a.verified);
        matches_search && matches_status
    }).collect();

    rsx! {
        div { class: "space-y-4",
            PageHeader { title: "Agents".to_string(), subtitle: "Manage real estate agents".to_string() }

            FilterBar {
                input {
                    class: "px-3 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-white placeholder-gray-500 focus:outline-none focus:border-blue-500",
                    placeholder: "Search agents...",
                    value: "{search}",
                    oninput: move |evt| search.set(evt.value()),
                }
                select {
                    class: "px-3 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-white focus:outline-none focus:border-blue-500",
                    value: "{status_filter}",
                    onchange: move |evt| status_filter.set(evt.value()),
                    option { value: "all", "All Status" }
                    option { value: "verified", "Verified" }
                    option { value: "unverified", "Unverified" }
                }
            }

            if let Some(data) = &agents_data {
                if data.is_empty() {
                    EmptyState { icon: "🏢".to_string(), title: "No agents found".to_string(), message: "Agents will appear here once they register.".to_string() }
                } else {
                    DataTable {
                        headers: vec!["Agent".to_string(), "Status".to_string(), "Properties".to_string(), "Commission".to_string(), "Actions".to_string()],
                        if filtered.is_empty() {
                            tr {
                                td { colspan: "5", class: "px-4 py-8 text-center text-gray-400",
                                    "No agents match your filters"
                                }
                            }
                        } else {
                            for agent in filtered {
                                tr { class: "hover:bg-gray-700/30 transition-colors",
                                    td { class: "px-4 py-3",
                                        div { class: "flex items-center gap-3",
                                            div { class: "w-8 h-8 rounded-full bg-emerald-600 flex items-center justify-center text-white text-xs font-bold",
                                                {agent.name.chars().next().unwrap_or('?').to_string()}
                                            }
                                            div {
                                                p { class: "text-white text-sm font-medium", "{agent.name}" }
                                                p { class: "text-gray-500 text-xs", "{agent.email}" }
                                            }
                                        }
                                    }
                                    td { class: "px-4 py-3",
                                        StatusBadge { status: if agent.verified { "verified".to_string() } else { "unverified".to_string() } }
                                    }
                                    td { class: "px-4 py-3 text-gray-300 text-sm", "{agent.property_count}" }
                                    td { class: "px-4 py-3 text-gray-300 text-sm", "{agent.commission_rate}%" }
                                    td { class: "px-4 py-3",
                                        div { class: "flex items-center gap-2",
                                            button { class: "text-xs px-2 py-1 rounded bg-blue-500/10 text-blue-400 hover:bg-blue-500/20 border border-blue-500/20 transition-colors", "View" }
                                            button { class: "text-xs px-2 py-1 rounded bg-gray-700 text-gray-300 hover:bg-gray-600 transition-colors", "Edit" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if agents_ref.as_ref().is_none() {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 animate-pulse",
                    for _ in 0..5 {
                        div { class: "px-4 py-3 flex items-center gap-4",
                            div { class: "w-8 h-8 bg-gray-700 rounded-full" }
                            div { class: "flex-1 space-y-2",
                                div { class: "h-4 bg-gray-700 rounded w-1/4" }
                                div { class: "h-3 bg-gray-700 rounded w-1/3" }
                            }
                        }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load agents" }
                }
            }
        }
    }
}
