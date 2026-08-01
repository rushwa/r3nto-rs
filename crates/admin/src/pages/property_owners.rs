use dioxus::prelude::*;
use crate::api::admin::{get_users, User as AdminUser};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn PropertyOwnersPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut owners = use_signal(Vec::<AdminUser>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);

    use_hook({
        let t = token.clone();
        move || {
            spawn(async move {
                match get_users(&t).await {
                    Ok(users) => {
                        let filtered: Vec<_> = users.into_iter()
                            .filter(|u| u.role == "PROPERTY_OWNER")
                            .collect();
                        owners.set(filtered);
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        }
    });

    rsx! {
        div { class: "p-6",
            h1 { class: "text-2xl font-bold text-white mb-6", "Property Owners" }

            if let Some(msg) = error.read().as_ref() {
                div { class: "mb-4 p-4 bg-red-900/50 border border-red-700 rounded-lg text-red-200",
                    "{msg}"
                }
            }

            if loading.read().clone() {
                div { class: "flex justify-center py-12",
                    div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" }
                }
            } else if owners.read().is_empty() {
                div { class: "text-center py-12 text-gray-400",
                    "No property owners found."
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                    table { class: "w-full text-left",
                        thead { class: "bg-gray-900 text-gray-400 text-sm uppercase",
                            tr {
                                th { class: "px-6 py-3", "Name" }
                                th { class: "px-6 py-3", "Email" }
                                th { class: "px-6 py-3", "Status" }
                                th { class: "px-6 py-3", "Joined" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-700 text-gray-300",
                            for owner in owners.read().iter().cloned() {
                                tr { class: "hover:bg-gray-700/50",
                                    td { class: "px-6 py-4", "{owner.name}" }
                                    td { class: "px-6 py-4", "{owner.email}" }
                                    td { class: "px-6 py-4",
                                        span {
                                            class: if owner.status == "active" { "text-green-400" } else { "text-red-400" },
                                            "{owner.status}"
                                        }
                                    }
                                    td { class: "px-6 py-4", "{owner.created_at}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}