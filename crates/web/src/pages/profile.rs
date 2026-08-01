use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::Route;

#[component]
pub fn Profile() -> Element {
    let auth = use_auth();
    let nav = use_navigator();

    let auth_read = auth.read();
    if !auth_read.is_authenticated {
        nav.replace(Route::Login {});
        return rsx! {
            div { class: "flex items-center justify-center min-h-screen",
                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
            }
        };
    }
    drop(auth_read);

    let user = auth.read().user.clone();
    let user_name = user.as_ref().map(|u| {
        let name = format!("{} {}", u.first_name, u.last_name).trim().to_string();
        if name.is_empty() { u.username.clone() } else { name }
    }).unwrap_or_default();
    let user_email = user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let user_role = user.as_ref().map(|u| u.role.clone()).unwrap_or_default();
    let user_phone = user.as_ref().and_then(|u| u.phone_number.clone()).unwrap_or_default();

    rsx! {
        div { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-3xl mx-auto px-4 sm:px-6 lg:px-8",
                h1 { class: "text-3xl font-bold text-gray-900 mb-8", "My Profile" }

                div { class: "bg-white rounded-xl shadow-sm p-8",
                    div { class: "flex items-center mb-8",
                        div { class: "w-20 h-20 bg-blue-600 rounded-full flex items-center justify-center text-white text-2xl font-bold",
                            {user_name.chars().next().unwrap_or('U').to_string()}
                        }
                        div { class: "ml-6",
                            h2 { class: "text-2xl font-bold text-gray-900", "{user_name}" }
                            p { class: "text-gray-600", "{user_email}" }
                            span { class: "inline-block mt-2 px-3 py-1 bg-blue-100 text-blue-700 rounded-full text-sm capitalize",
                                "{user_role}"
                            }
                        }
                    }

                    div { class: "space-y-6",
                        div { class: "grid grid-cols-1 md:grid-cols-2 gap-6",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "First Name" }
                                input {
                                    class: "w-full px-4 py-2 border border-gray-300 rounded-lg bg-gray-50",
                                    value: user.as_ref().map(|u| u.first_name.clone()).unwrap_or_default(),
                                    readonly: true,
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1", "Last Name" }
                                input {
                                    class: "w-full px-4 py-2 border border-gray-300 rounded-lg bg-gray-50",
                                    value: user.as_ref().map(|u| u.last_name.clone()).unwrap_or_default(),
                                    readonly: true,
                                }
                            }
                        }

                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Email" }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg bg-gray-50",
                                r#type: "email",
                                value: user_email,
                                readonly: true,
                            }
                        }

                        div {
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Phone" }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg bg-gray-50",
                                r#type: "tel",
                                value: user_phone,
                                readonly: true,
                            }
                        }
                    }

                    div { class: "mt-8 pt-6 border-t border-gray-200",
                        p { class: "text-sm text-gray-500", "Profile editing coming soon." }
                    }
                }
            }
        }
    }
}