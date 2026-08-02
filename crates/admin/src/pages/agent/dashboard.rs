use dioxus::prelude::*;
use serde::Deserialize;
use crate::api::api_get;

#[derive(Clone, Debug, Deserialize)]
pub struct AgentStats {
    pub total_leads: i32,
    pub converted_leads: i32,
    pub pending_leads: i32,
    pub total_commissions: String,
    pub active_properties: i32,
}

#[component]
pub fn AgentDashboard() -> Element {
    let stats = use_resource(move || async move {
        api_get::<AgentStats>("/api/agents/stats").await
    });

    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold", "Agent Dashboard" }
            p { class: "text-gray-400", "Welcome back! Here's your overview." }
            
            match &*stats.read() {
                Some(Some(s)) => rsx! {
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-6",
                        StatCard { icon: "👥", value: "{s.total_leads}", label: "Total Leads", color: "bg-gray-800" }
                        StatCard { icon: "✅", value: "{s.converted_leads}", label: "Converted", color: "bg-green-900" }
                        StatCard { icon: "⏳", value: "{s.pending_leads}", label: "Pending", color: "bg-yellow-900" }
                        StatCard { icon: "💰", value: "KES {s.total_commissions}", label: "Commissions", color: "bg-blue-900" }
                        StatCard { icon: "🏠", value: "{s.active_properties}", label: "Properties", color: "bg-purple-900" }
                    }
                },
                _ => rsx! { div { class: "text-gray-400", "Loading stats..." } }
            }
            
            div { class: "bg-gray-800 rounded-lg p-6",
                h2 { class: "text-xl font-bold mb-4", "Quick Actions" }
                div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
                    Link { to: crate::Route::Leads {}, class: "bg-blue-600 hover:bg-blue-700 px-4 py-3 rounded-lg text-center", "Manage Leads" }
                    Link { to: crate::Route::Conversion {}, class: "bg-green-600 hover:bg-green-700 px-4 py-3 rounded-lg text-center", "Convert Client" }
                    Link { to: crate::Route::Properties {}, class: "bg-purple-600 hover:bg-purple-700 px-4 py-3 rounded-lg text-center", "Verify Property" }
                    Link { to: crate::Route::Commissions {}, class: "bg-yellow-600 hover:bg-yellow-700 px-4 py-3 rounded-lg text-center", "Commissions" }
                }
            }
        }
    }
}

#[component]
fn StatCard(icon: String, value: String, label: String, color: String) -> Element {
    rsx! {
        div { class: "{color} rounded-lg p-6",
            div { class: "text-4xl mb-2", "{icon}" }
            h3 { class: "text-2xl font-bold", "{value}" }
            p { class: "text-gray-400", "{label}" }
        }
    }
}
