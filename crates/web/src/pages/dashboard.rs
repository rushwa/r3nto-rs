use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::Route;

#[component]
pub fn Dashboard() -> Element {
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
    }).unwrap_or_else(|| "User".to_string());
    let user_email = user.as_ref().map(|u| u.email.clone()).unwrap_or_default();
    let user_role = user.as_ref().map(|u| u.role.clone()).unwrap_or_else(|| "user".to_string());

    rsx! {
        div { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "bg-white rounded-xl shadow-sm p-6 mb-6",
                    h1 { class: "text-3xl font-bold text-gray-900", "Welcome back, {user_name}!" }
                    p { class: "text-gray-600 mt-1", "Here's what's happening with your account." }
                }

                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6 mb-6",
                    div { class: "bg-white rounded-xl shadow-sm p-6",
                        div { class: "flex items-center",
                            div { class: "p-3 rounded-full bg-blue-100 text-blue-600",
                                svg { class: "w-6 h-6", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" }
                                }
                            }
                            div { class: "ml-4",
                                p { class: "text-sm text-gray-600", "Properties" }
                                p { class: "text-2xl font-bold text-gray-900", "0" }
                            }
                        }
                    }

                    div { class: "bg-white rounded-xl shadow-sm p-6",
                        div { class: "flex items-center",
                            div { class: "p-3 rounded-full bg-green-100 text-green-600",
                                svg { class: "w-6 h-6", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" }
                                }
                            }
                            div { class: "ml-4",
                                p { class: "text-sm text-gray-600", "Active Listings" }
                                p { class: "text-2xl font-bold text-gray-900", "0" }
                            }
                        }
                    }

                    div { class: "bg-white rounded-xl shadow-sm p-6",
                        div { class: "flex items-center",
                            div { class: "p-3 rounded-full bg-purple-100 text-purple-600",
                                svg { class: "w-6 h-6", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" }
                                }
                            }
                            div { class: "ml-4",
                                p { class: "text-sm text-gray-600", "Revenue" }
                                p { class: "text-2xl font-bold text-gray-900", "$0" }
                            }
                        }
                    }
                }

                div { class: "bg-white rounded-xl shadow-sm p-6",
                    h2 { class: "text-xl font-bold text-gray-900 mb-4", "Profile Information" }
                    div { class: "grid grid-cols-1 md:grid-cols-2 gap-4",
                        div {
                            p { class: "text-sm text-gray-600", "Full Name" }
                            p { class: "text-lg font-medium text-gray-900", "{user_name}" }
                        }
                        div {
                            p { class: "text-sm text-gray-600", "Email" }
                            p { class: "text-lg font-medium text-gray-900", "{user_email}" }
                        }
                        div {
                            p { class: "text-sm text-gray-600", "Role" }
                            p { class: "text-lg font-medium text-gray-900 capitalize", "{user_role}" }
                        }
                    }

                    div { class: "mt-6",
                        Link {
                            to: Route::Profile {},
                            class: "inline-flex items-center text-blue-600 hover:text-blue-500 font-medium",
                            "Edit Profile "
                            svg { class: "w-4 h-4 ml-1", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M9 5l7 7-7 7" }
                            }
                        }
                    }
                }
            }
        }
    }
}