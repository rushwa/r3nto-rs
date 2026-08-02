use dioxus::prelude::*;
use serde::Deserialize;
use crate::api::api_get;

#[derive(Clone, Debug, Deserialize)]
pub struct CommissionsResponse {
    pub commissions: Vec<Commission>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Commission {
    pub id: String,
    pub amount: String,
    pub commission_rate: String,
    pub transaction_ref: String,
    pub status: String,
    pub created_at: String,
}

#[component]
pub fn CommissionsPage() -> Element {
    let commissions = use_resource(move || async move {
        api_get::<CommissionsResponse>("/api/commissions").await
    });

    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-3xl font-bold", "Commission History" }
            div { class: "bg-gray-800 rounded-lg overflow-hidden",
                table { class: "w-full",
                    thead { class: "bg-gray-700",
                        tr {
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Ref" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Amount" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Rate" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Status" }
                            th { class: "px-6 py-3 text-left text-xs font-medium text-gray-300 uppercase", "Date" }
                        }
                    }
                    tbody { class: "divide-y divide-gray-700",
                        match &*commissions.read() {
                            Some(Some(data)) => rsx! {
                                for comm in &data.commissions {
                                    tr { class: "hover:bg-gray-750", key: "{comm.id}",
                                        td { class: "px-6 py-4", "{comm.transaction_ref}" }
                                        td { class: "px-6 py-4", "KES {comm.amount}" }
                                        td { class: "px-6 py-4 text-gray-300", "{comm.commission_rate}%" }
                                        td { class: "px-6 py-4",
                                            span { class: "px-2 py-1 text-xs rounded-full bg-green-900 text-green-300", "{comm.status}" }
                                        }
                                        td { class: "px-6 py-4 text-gray-300", "{comm.created_at}" }
                                    }
                                }
                            },
                            _ => rsx! {
                                tr { td { colspan: "5", class: "px-6 py-4 text-center text-gray-400", "Loading..." } }
                            }
                        }
                    }
                }
            }
        }
    }
}
