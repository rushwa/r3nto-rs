use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, StatusBadge, DataTable, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[derive(Clone, Debug, PartialEq)]
pub struct Lead {
    pub id: String,
    pub name: String,
    pub email: String,
    pub status: String,
    pub claimed_by: Option<String>,
}

#[component]
pub fn LeadsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_resource = token.clone();

    let leads: Resource<Result<Vec<Lead>, String>> = use_resource(move || {  // <-- annotated
        let t = token_for_resource.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![
                    Lead {
                        id: "1".to_string(),
                        name: "John Doe".to_string(),
                        email: "john@example.com".to_string(),
                        status: "pending".to_string(),
                        claimed_by: None
                    },
                ]);
            }
            Ok(vec![])
        }
    });

    let leads_ref = leads.read();
    let leads_data: Option<Vec<Lead>> = match leads_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Agent Leads".to_string(),
                subtitle: "Manage and claim potential property owners".to_string()
            }

            if let Some(data) = &leads_data {
                if data.is_empty() {
                    EmptyState {
                        icon: "👥".to_string(),
                        title: "No leads found".to_string(),
                        message: "Leads will appear here once generated.".to_string()
                    }
                } else {
                    DataTable {
                        headers: vec!["Name".to_string(), "Email".to_string(), "Status".to_string(), "Actions".to_string()],
                        for lead in data.iter() {
                            tr { class: "hover:bg-gray-700/30 transition-colors",
                                td { class: "px-4 py-3 text-white text-sm font-medium", "{lead.name}" }
                                td { class: "px-4 py-3 text-gray-300 text-sm", "{lead.email}" }
                                td { class: "px-4 py-3",
                                    StatusBadge { status: lead.status.clone() }
                                }
                                td { class: "px-4 py-3",
                                    if lead.claimed_by.is_none() {
                                        button {
                                            class: "text-xs px-3 py-1.5 rounded bg-green-500/10 text-green-400 hover:bg-green-500/20 border border-green-500/20 transition-colors",
                                            "Claim"
                                        }
                                    } else {
                                        span { class: "text-xs text-gray-500", "Claimed" }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if leads_ref.as_ref().is_none() {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 animate-pulse p-8",
                    p { class: "text-gray-400 text-center", "Loading leads..." }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load leads" }
                }
            }
        }
    }
}