use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, DataTable, StatusBadge, EmptyState, FilterBar};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn PropertyOwnersPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut owners = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut search_query = use_signal(|| String::new());
    let mut filter_paid = use_signal(|| Option::<bool>::None);

    // Fetch data
    use_effect(move || {
        let t = token.clone();
        spawn(async move {
            let res = reqwest::Client::new()
                .get("http://localhost:8000/admin/property-owners")
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await;

            loading.set(false);

            if let Ok(resp) = res {
                if let Ok(json) = resp.json::<Vec<serde_json::Value>>().await {
                    owners.set(json);
                }
            }
        });
    });

    // Filtered list
    let filtered_owners = {
        let query = search_query.read().to_lowercase();
        let paid_filter = *filter_paid.read();

        owners.read().iter()
            .filter(|o| {
                let name = o.get("name").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                let email = o.get("email").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
                let matches_search = query.is_empty() || name.contains(&query) || email.contains(&query);

                let matches_paid = match paid_filter {
                    None => true,
                    Some(paid) => {
                        o.get("has_paid_registration_fee")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false) == paid
                    }
                };

                matches_search && matches_paid
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    let total_count = owners.read().len();
    let paid_count = owners.read().iter()
        .filter(|o| o.get("has_paid_registration_fee").and_then(|v| v.as_bool()).unwrap_or(false))
        .count();
    let unpaid_count = total_count - paid_count;

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading property owners..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Property Owners".to_string(),
                subtitle: format!("Manage {} property owners ({} paid, {} pending)", total_count, paid_count, unpaid_count),
            }

            // Stats Summary
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-4",
                    p { class: "text-gray-400 text-sm", "Total Owners" }
                    p { class: "text-2xl font-bold text-white", "{total_count}" }
                }
                div { class: "bg-gray-800 rounded-lg border border-green-500/30 p-4",
                    p { class: "text-green-400 text-sm", "✅ Registration Paid" }
                    p { class: "text-2xl font-bold text-green-400", "{paid_count}" }
                }
                div { class: "bg-gray-800 rounded-lg border border-yellow-500/30 p-4",
                    p { class: "text-yellow-400 text-sm", "⚠️ Pending Payment" }
                    p { class: "text-2xl font-bold text-yellow-400", "{unpaid_count}" }
                }
            }

            // Filters
            FilterBar {
                input {
                    class: "flex-1 px-4 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-blue-500",
                    r#type: "text",
                    placeholder: "Search by name or email...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                }

                button {
                    class: "px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors",
                    onclick: move |_| filter_paid.set(None),
                    if filter_paid.read().is_none() { "🔵 All" } else { "All" }
                }
                button {
                    class: "px-4 py-2 bg-green-700 hover:bg-green-600 text-white rounded-lg transition-colors",
                    onclick: move |_| filter_paid.set(Some(true)),
                    if *filter_paid.read() == Some(true) { "🟢 Paid" } else { "Paid" }
                }
                button {
                    class: "px-4 py-2 bg-yellow-700 hover:bg-yellow-600 text-white rounded-lg transition-colors",
                    onclick: move |_| filter_paid.set(Some(false)),
                    if *filter_paid.read() == Some(false) { "🟡 Unpaid" } else { "Unpaid" }
                }
            }

            // Table
            if filtered_owners.is_empty() {
                EmptyState {
                    icon: "👥".to_string(),
                    title: "No property owners found".to_string(),
                    message: "Try adjusting your search or filter.".to_string(),
                }
            } else {
                DataTable {
                    headers: vec![
                        "Name".to_string(),
                        "Email".to_string(),
                        "Phone".to_string(),
                        "Reg. Fee".to_string(),
                        "Properties".to_string(),
                        "Converted By".to_string(),
                        "Status".to_string(),
                        "Joined".to_string(),
                    ],
                    for owner in filtered_owners.iter() {
                        PropertyOwnerRow { owner: owner.clone() }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Property Owner Row
// ───────────────────────────────────────────
#[component]
fn PropertyOwnerRow(owner: serde_json::Value) -> Element {
    let name = owner.get("name").and_then(|v| v.as_str()).unwrap_or("—");
    let email = owner.get("email").and_then(|v| v.as_str()).unwrap_or("—");
    let phone = owner.get("phone").and_then(|v| v.as_str()).unwrap_or("—");
    let status = owner.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    let has_paid = owner.get("has_paid_registration_fee").and_then(|v| v.as_bool()).unwrap_or(false);
    let property_count = owner.get("property_count").and_then(|v| v.as_i64()).unwrap_or(0);
    let converted_by = owner.get("converted_by_agent").and_then(|v| v.as_str()).unwrap_or("—");
    let created_at = owner.get("created_at").and_then(|v| v.as_str()).unwrap_or("—");

    let joined_display = if created_at.len() > 10 {
        &created_at[..10]
    } else {
        created_at
    };

    rsx! {
        tr { class: "hover:bg-gray-700/30 transition-colors",
            td { class: "px-4 py-3",
                div { class: "text-white font-medium", "{name}" }
            }
            td { class: "px-4 py-3 text-gray-300", "{email}" }
            td { class: "px-4 py-3 text-gray-300", "{phone}" }
            td { class: "px-4 py-3",
                if has_paid {
                    span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs bg-green-500/10 text-green-400 border border-green-500/20",
                        span { "✅" }
                        span { "Paid" }
                    }
                } else {
                    span { class: "inline-flex items-center gap-1 px-2 py-1 rounded-full text-xs bg-yellow-500/10 text-yellow-400 border border-yellow-500/20",
                        span { "⚠️" }
                        span { "Unpaid" }
                    }
                }
            }
            td { class: "px-4 py-3 text-center",
                span { class: "text-white font-medium", "{property_count}" }
            }
            td { class: "px-4 py-3 text-gray-300", "{converted_by}" }
            td { class: "px-4 py-3",
                StatusBadge { status: status.to_string() }
            }
            td { class: "px-4 py-3 text-gray-400 text-sm", "{joined_display}" }
        }
    }
}