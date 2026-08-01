use dioxus::prelude::*;

use crate::api::admin::{get_inquiries, update_inquiry_status, Inquiry};
use crate::components::sidebar::{PageHeader, StatusBadge, FilterBar, DataTable, EmptyState};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn InquiriesPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_resource = token.clone();
    let mut status_filter = use_signal(|| "all".to_string());
    let mut action_loading = use_signal(|| false);

    let inquiries = use_resource(move || {
        let t = token_for_resource.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![]);
            }
            get_inquiries(&t).await
        }
    });

    let inq_ref = inquiries.read();
    let inq_data = match inq_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    let filtered: Vec<Inquiry> = inq_data.clone().unwrap_or_default().into_iter().filter(|i| {
        let st = status_filter.read().clone();
        st == "all" || i.status.to_lowercase() == st
    }).collect();

    let status_options = vec![
        ("new", "New"),
        ("contacted", "Contacted"),
        ("viewing", "Viewing"),
        ("negotiating", "Negotiating"),
        ("closed", "Closed"),
    ];

    rsx! {
        div { class: "space-y-4",
            PageHeader { title: "Inquiries".to_string(), subtitle: "Manage property inquiries and leads".to_string() }

            FilterBar {
                select {
                    class: "px-3 py-1.5 bg-gray-900 border border-gray-700 rounded text-sm text-white focus:outline-none focus:border-blue-500",
                    value: "{status_filter}",
                    onchange: move |evt| status_filter.set(evt.value()),
                    option { value: "all", "All Status" }
                    for (value, label) in status_options.iter() {
                        option { value: "{value}", "{label}" }
                    }
                }
            }

            if let Some(data) = &inq_data {
                if data.is_empty() {
                    EmptyState { icon: "📨".to_string(), title: "No inquiries found".to_string(), message: "Inquiries will appear here when leads come in.".to_string() }
                } else {
                    DataTable {
                        headers: vec!["Lead".to_string(), "Property".to_string(), "Status".to_string(), "Date".to_string(), "Assigned".to_string(), "Actions".to_string()],
                        if filtered.is_empty() {
                            tr {
                                td { colspan: "6", class: "px-4 py-8 text-center text-gray-400",
                                    "No inquiries match your filters"
                                }
                            }
                        } else {
                            for inquiry in filtered {
                                tr { class: "hover:bg-gray-700/30 transition-colors",
                                    td { class: "px-4 py-3",
                                        div {
                                            p { class: "text-white text-sm font-medium", "{inquiry.name}" }
                                            p { class: "text-gray-500 text-xs", "{inquiry.email}" }
                                            p { class: "text-gray-500 text-xs", "{inquiry.phone}" }
                                        }
                                    }
                                    td { class: "px-4 py-3 text-gray-300 text-sm", "{inquiry.property_title}" }
                                    td { class: "px-4 py-3",
                                        StatusBadge { status: inquiry.status.clone() }
                                    }
                                    td { class: "px-4 py-3 text-gray-400 text-sm", "{inquiry.created_at}" }
                                    td { class: "px-4 py-3 text-gray-300 text-sm",
                                        {inquiry.assigned_to.as_ref().map(|s| s.as_str()).unwrap_or("Unassigned")}
                                    }
                                    td { class: "px-4 py-3",
                                        div { class: "flex items-center gap-1 flex-wrap",
                                            for (next_status, label) in status_options.iter() {
                                                if inquiry.status != *next_status {
                                                    button {
                                                        class: "text-xs px-2 py-1 rounded bg-gray-700 text-gray-300 hover:bg-gray-600 transition-colors",
                                                        disabled: action_loading.read().clone(),
                                                        onclick: {
                                                            let iid = inquiry.id.clone();
                                                            let ns = next_status.to_string();
                                                            let t = token.clone();
                                                            let mut inq_res = inquiries.clone();
                                                            move |_| {
                                                                action_loading.set(true);
                                                                spawn({
                                                                    let iid = iid.clone();
                                                                    let ns = ns.clone();
                                                                    let t = t.clone();
                                                                    let mut inq_res = inq_res.clone();
                                                                    async move {
                                                                        let _ = update_inquiry_status(&t, &iid, &ns, None).await;
                                                                        inq_res.restart();
                                                                        action_loading.set(false);
                                                                    }
                                                                });
                                                            }
                                                        },
                                                        "{label}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if inq_ref.as_ref().is_none() {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 animate-pulse",
                    for _ in 0..5 {
                        div { class: "px-4 py-3 flex items-center gap-4",
                            div { class: "flex-1 space-y-2",
                                div { class: "h-4 bg-gray-700 rounded w-1/4" }
                                div { class: "h-3 bg-gray-700 rounded w-1/3" }
                            }
                        }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load inquiries" }
                }
            }
        }
    }
}
