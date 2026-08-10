use dioxus::prelude::*;
use crate::components::sidebar::PageHeader;
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn OwnerProfilePage() -> Element {
    let auth = use_admin_auth();
    let user = auth.read().user.clone();

    let user_name = user.as_ref().map(|u| u.name.clone()).unwrap_or_default();
    let user_email = user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let user_id = user.as_ref().map(|u| u.id.clone()).unwrap_or_default();
    let user_role = user.as_ref().map(|u| u.role.clone()).unwrap_or_default();

    rsx! {
        div { class: "space-y-6 max-w-3xl",
            PageHeader {
                title: "My Profile".to_string(),
                subtitle: "Your account information".to_string(),
            }

            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8",
                div { class: "flex items-center gap-6 mb-8",
                    div { class: "w-20 h-20 bg-blue-600 rounded-full flex items-center justify-center text-white text-3xl font-bold",
                        {user_name.chars().next().unwrap_or('O').to_string()}
                    }
                    div {
                        h2 { class: "text-2xl font-bold text-white", "{user_name}" }
                        p { class: "text-gray-400", "{user_email}" }
                        span { class: "inline-block mt-2 px-3 py-1 bg-blue-600/20 text-blue-400 rounded-full text-sm",
                            "{user_role}"
                        }
                    }
                }

                div { class: "space-y-4",
                    div { class: "p-4 bg-gray-900 rounded-lg border border-gray-700",
                        p { class: "text-gray-400 text-sm mb-1", "User ID" }
                        p { class: "text-white font-mono text-sm break-all", "{user_id}" }
                        p { class: "text-gray-500 text-xs mt-2", "Share this ID with your Rento agent when needed." }
                    }
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "p-4 bg-gray-900 rounded-lg border border-gray-700",
                            p { class: "text-gray-400 text-sm mb-1", "Email" }
                            p { class: "text-white", "{user_email}" }
                        }
                        div { class: "p-4 bg-gray-900 rounded-lg border border-gray-700",
                            p { class: "text-gray-400 text-sm mb-1", "Role" }
                            p { class: "text-white capitalize", "{user_role.to_lowercase()}" }
                        }
                    }
                }
            }
        }
    }
}