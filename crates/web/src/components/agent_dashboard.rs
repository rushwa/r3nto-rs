use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentLead {
    pub id: Uuid,
    pub email: String,
    pub full_name: String,
    pub status: String,
}

#[component]
pub fn AgentDashboard() -> Element {
    let leads = use_resource(move || async move {
        let resp = reqwest::get("/api/leads")
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        resp
    });

    rsx! {
        div {
            class: "agent-dashboard",
            h1 { "Agent Dashboard" }
            
            div {
                class: "leads-section",
                h2 { "My Leads" }
                
                match leads.read().as_ref() {
                    Some(data) => {
                        if let Some(leads_array) = data.get("leads").and_then(|v| v.as_array()) {
                            ul {
                                for lead_data in leads_array {
                                    li {
                                        key: "{lead_data.get("id").unwrap()}",
                                        div {
                                            class: "lead-card",
                                            p { "Name: {lead_data.get("full_name").unwrap().as_str().unwrap()}" }
                                            p { "Email: {lead_data.get("email").unwrap().as_str().unwrap()}" }
                                            p { "Status: {lead_data.get("status").unwrap().as_str().unwrap()}" }
                                            button {
                                                class: "claim-btn",
                                                onclick: move |_| {
                                                    // TODO: Implement claim logic
                                                },
                                                "Claim Lead"
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            p { "No leads found" }
                        }
                    }
                    None => p { "Loading leads..." }
                }
            }
        }
    }
}
