use dioxus::prelude::*;
use serde::Deserialize;
use crate::api::{api_get, api_post_status};

#[derive(Clone, Debug, Deserialize)]
pub struct LeadsResponse {
    pub leads: Vec<Lead>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Lead {
    pub id: String,
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
    pub status: String,
    pub claimed_by: Option<String>,
}

#[component]
pub fn LeadsPage() -> Element {
    let mut show_modal = use_signal(|| false);
    let mut needs_refresh = use_signal(|| 0u32);
    
    let leads = use_resource(move || {
        let _refresh = needs_refresh();
        async move {
            api_get::<LeadsResponse>("/api/leads").await
        }
    });

    let claim_lead = move |lead_id: String| {
        spawn(async move {
            let body = serde_json::json!({ "lead_id": lead_id });
            if api_post_status("/api/leads/claim", &body).await {
                needs_refresh += 1;
            }
        });
    };

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h1 { class: "text-3xl font-bold", "Lead Management" }
                button {
                    class: "bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg",
                    onclick: move |_| show_modal.set(true),
                    "+ Add New Lead"
                }
            }
            
            div { class: "bg-gray-800 rounded-lg overflow-hidden",
                table { class: "w-full",
                    thead { class: "bg-gray-700",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Name" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Email" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Phone" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Status" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Actions" }
                        }
                    }
                    tbody { class: "divide-y divide-gray-700",
                        match &*leads.read() {
                            Some(Some(data)) => rsx! {
                                for lead in &data.leads {
                                    LeadRow { lead: lead.clone(), on_claim: claim_lead.clone() }
                                }
                            },
                            _ => rsx! {
                                tr { td { colspan: "5", class: "px-6 py-4 text-center text-gray-400", "Loading..." } }
                            }
                        }
                    }
                }
            }
            
            if show_modal() {
                CreateLeadModal {
                    on_close: move |_| show_modal.set(false),
                    on_created: move |_| { show_modal.set(false); needs_refresh += 1; }
                }
            }
        }
    }
}

#[component]
fn LeadRow(lead: Lead, on_claim: EventHandler<String>) -> Element {
    let phone_text = lead.phone.as_deref().unwrap_or("-");
    let is_claimable = lead.claimed_by.is_none();
    
    rsx! {
        tr { class: "hover:bg-gray-750",
            td { class: "px-6 py-4", "{lead.full_name}" }
            td { class: "px-6 py-4 text-gray-300", "{lead.email}" }
            td { class: "px-6 py-4 text-gray-300", "{phone_text}" }
            td { class: "px-6 py-4",
                span { class: "px-2 py-1 text-xs rounded-full bg-blue-900 text-blue-300", "{lead.status}" }
            }
            td { class: "px-6 py-4",
                if is_claimable {
                    button {
                        class: "text-sm bg-green-600 hover:bg-green-700 px-3 py-1 rounded",
                        onclick: move |_| on_claim.call(lead.id.clone()),
                        "Claim"
                    }
                } else {
                    span { class: "text-sm text-gray-500", "Claimed" }
                }
            }
        }
    }
}

#[component]
fn CreateLeadModal(on_close: EventHandler<()>, on_created: EventHandler<()>) -> Element {
    let mut email = use_signal(String::new);
    let mut full_name = use_signal(String::new);
    let mut phone = use_signal(String::new);
    let mut loading = use_signal(|| false);

    let submit = move |_| {
        spawn(async move {
            loading.set(true);
            let body = serde_json::json!({
                "email": email(),
                "full_name": full_name(),
                "phone": if phone().is_empty() { None::<String> } else { Some(phone()) }
            });
            if api_post_status("/api/leads", &body).await {
                on_created.call(());
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
            onclick: move |_| on_close.call(()),
            div { class: "bg-gray-800 rounded-lg p-6 w-full max-w-md",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-xl font-bold mb-4", "Create New Lead" }
                form { onsubmit: submit,
                    div { class: "mb-4",
                        label { class: "block text-sm text-gray-300 mb-1", "Full Name *" }
                        input { class: "w-full bg-gray-700 px-3 py-2 rounded-lg", r#type: "text", required: true,
                            oninput: move |e| full_name.set(e.value()) }
                    }
                    div { class: "mb-4",
                        label { class: "block text-sm text-gray-300 mb-1", "Email *" }
                        input { class: "w-full bg-gray-700 px-3 py-2 rounded-lg", r#type: "email", required: true,
                            oninput: move |e| email.set(e.value()) }
                    }
                    div { class: "mb-4",
                        label { class: "block text-sm text-gray-300 mb-1", "Phone" }
                        input { class: "w-full bg-gray-700 px-3 py-2 rounded-lg", r#type: "tel",
                            oninput: move |e| phone.set(e.value()) }
                    }
                    div { class: "flex gap-3",
                        button { r#type: "button", class: "flex-1 bg-gray-700 hover:bg-gray-600 px-4 py-2 rounded-lg",
                            onclick: move |_| on_close.call(()), "Cancel" }
                        button { r#type: "submit", class: "flex-1 bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg",
                            disabled: loading(),
                            if loading() { "Creating..." } else { "Create Lead" } }
                    }
                }
            }
        }
    }
}
