use dioxus::prelude::*;

use crate::api::admin::{get_commissions, Commission};
use crate::components::sidebar::{PageHeader, StatCard, StatusBadge, DataTable, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn CommissionsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_resource = token.clone();

    let commissions = use_resource(move || {
        let t = token_for_resource.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![]);
            }
            get_commissions(&t).await
        }
    });

    let comm_ref = commissions.read();
    let comm_data = match comm_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader { title: "Commissions".to_string(), subtitle: "Track and manage agent commissions".to_string() }

            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                StatCard { title: "Total Paid".to_string(), value: "$12,450".to_string(), icon: "💰".to_string(), change: "+8%".to_string(), change_positive: true }
                StatCard { title: "Pending".to_string(), value: "$3,210".to_string(), icon: "⏳".to_string(), change: "+2%".to_string(), change_positive: true }
                StatCard { title: "This Month".to_string(), value: "$4,890".to_string(), icon: "📅".to_string(), change: "+15%".to_string(), change_positive: true }
                StatCard { title: "Avg Commission".to_string(), value: "$285".to_string(), icon: "📊".to_string(), change: "-3%".to_string(), change_positive: false }
            }

            if let Some(data) = &comm_data {
                if data.is_empty() {
                    EmptyState { icon: "💰".to_string(), title: "No commissions found".to_string(), message: "Commissions will appear here when transactions occur.".to_string() }
                } else {
                    DataTable {
                        headers: vec!["ID".to_string(), "Agent".to_string(), "Property".to_string(), "Amount".to_string(), "Status".to_string(), "Date".to_string()],
                        for comm in data.iter() {
                            tr { class: "hover:bg-gray-700/30 transition-colors",
                                td { class: "px-4 py-3 text-gray-400 font-mono text-xs",
                                    {comm.id.chars().take(8).collect::<String>()}
                                }
                                td { class: "px-4 py-3 text-white text-sm", "{comm.agent}" }
                                td { class: "px-4 py-3 text-gray-300 text-sm", "{comm.property}" }
                                td { class: "px-4 py-3 text-white text-sm font-medium", "${comm.amount:.2}" }
                                td { class: "px-4 py-3",
                                    StatusBadge { status: comm.status.clone() }
                                }
                                td { class: "px-4 py-3 text-gray-400 text-sm", "{comm.date}" }
                            }
                        }
                    }
                }
            } else if comm_ref.as_ref().is_none() {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 animate-pulse",
                    for _ in 0..5 {
                        div { class: "px-4 py-3 flex items-center gap-4",
                            div { class: "flex-1 space-y-2",
                                div { class: "h-4 bg-gray-700 rounded w-1/6" }
                                div { class: "h-4 bg-gray-700 rounded w-1/4" }
                            }
                        }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load commissions" }
                }
            }
        }
    }
}
