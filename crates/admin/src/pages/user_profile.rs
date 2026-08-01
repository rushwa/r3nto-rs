use dioxus::prelude::*;
use crate::api::admin::{get_user_profile, UserProfile};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn UserProfilePage(id: String) -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut profile = use_signal(|| None::<UserProfile>);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);

    use_hook({
        let t = token.clone();
        let uid = id.clone();
        move || {
            spawn(async move {
                match get_user_profile(&t, &uid).await {
                    Ok(data) => profile.set(Some(data)),
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        }
    });

    rsx! {
        div { class: "p-6",
            if loading.read().clone() {
                div { class: "flex justify-center py-12",
                    div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" }
                }
            } else if let Some(p) = profile.read().as_ref() {
                div { class: "max-w-2xl",
                    div { class: "flex items-center gap-4 mb-6",
                        div { class: "w-16 h-16 rounded-full bg-blue-600 flex items-center justify-center text-2xl font-bold text-white",
                            {p.first_name.chars().next().unwrap_or('?').to_string()}
                        }
                        div {
                            h1 { class: "text-2xl font-bold text-white", "{p.first_name} {p.last_name}" }
                            p { class: "text-gray-400", "{p.email}" }
                        }
                    }

                    div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 space-y-4",
                        div { class: "grid grid-cols-2 gap-4",
                            div {
                                label { class: "text-sm text-gray-500", "Username" }
                                p { class: "text-white", "{p.username}" }
                            }
                            div {
                                label { class: "text-sm text-gray-500", "Role" }
                                span {
                                    class: match p.role.as_str() {
                                        "ADMIN" => "px-2 py-1 bg-purple-900 text-purple-200 rounded text-xs font-medium",
                                        "AGENT" => "px-2 py-1 bg-blue-900 text-blue-200 rounded text-xs font-medium",
                                        "PROPERTY_OWNER" => "px-2 py-1 bg-green-900 text-green-200 rounded text-xs font-medium",
                                        _ => "px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs font-medium",
                                    },
                                    "{p.role}"
                                }
                            }
                            div {
                                label { class: "text-sm text-gray-500", "Phone" }
                                p { class: "text-white", "{p.phone_number.as_deref().unwrap_or(\"N/A\")}" }
                            }
                            div {
                                label { class: "text-sm text-gray-500", "Status" }
                                span {
                                    class: if p.is_active { "text-green-400" } else { "text-red-400" },
                                    if p.is_active { "Active" } else { "Disabled" }
                                }
                            }
                            div {
                                label { class: "text-sm text-gray-500", "Superuser" }
                                p { class: "text-white", if p.is_superuser { "Yes" } else { "No" } }
                            }
                            div {
                                label { class: "text-sm text-gray-500", "Staff" }
                                p { class: "text-white", if p.is_staff { "Yes" } else { "No" } }
                            }
                            div {
                                label { class: "text-sm text-gray-500", "Phone Verified" }
                                p { class: "text-white", if p.phone_verified { "Yes" } else { "No" } }
                            }
                            div {
                                label { class: "text-sm text-gray-500", "Subscribed" }
                                p { class: "text-white", if p.subscribed { "Yes" } else { "No" } }
                            }
                        }

                        if let Some(ref id_no) = p.identification_no {
                            div {
                                label { class: "text-sm text-gray-500", "ID Number" }
                                p { class: "text-white", "{id_no}" }
                            }
                        }

                        if let Some(ref county) = p.county {
                            div {
                                label { class: "text-sm text-gray-500", "Location" }
                                p { class: "text-white", "{county}, {p.constituency.as_deref().unwrap_or(\"\")}, {p.ward.as_deref().unwrap_or(\"\")}" }
                            }
                        }

                        div {
                            label { class: "text-sm text-gray-500", "Date Joined" }
                            p { class: "text-white", "{p.date_joined}" }
                        }

                        if let Some(ref last) = p.last_login {
                            div {
                                label { class: "text-sm text-gray-500", "Last Login" }
                                p { class: "text-white", "{last}" }
                            }
                        }
                    }
                }
            } else if let Some(msg) = error.read().as_ref() {
                div { class: "p-4 bg-red-900/50 border border-red-700 rounded-lg text-red-200",
                    "{msg}"
                }
            }
        }
    }
}