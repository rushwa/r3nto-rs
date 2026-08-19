use dioxus::prelude::*;
use crate::context::auth::{use_auth, clear_auth};
use crate::Route;

#[component]
pub fn Navbar() -> Element {
    let auth = use_auth();
    let nav = use_navigator();
    let mut mobile_menu_open = use_signal(|| false);

    let is_authenticated = auth.read().is_authenticated;
    let user_name = auth.read()
        .user
        .as_ref()
        .map(|u| {
            let name = format!("{} {}", u.first_name, u.last_name).trim().to_string();
            if name.is_empty() { u.username.clone() } else { name }
        })
        .unwrap_or_else(|| "User".to_string());

    let handle_logout = move |_| {
        clear_auth();
        nav.push(Route::Home {});
    };

    let toggle_mobile = move |_| {
        let current = *mobile_menu_open.read();
        mobile_menu_open.set(!current);
    };

    rsx! {
        nav { class: "bg-white shadow-md sticky top-0 z-50",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "flex justify-between h-16",
                    div { class: "flex items-center",
                        Link { to: Route::Home {}, class: "text-2xl font-bold text-blue-600",
                            "Rento"
                        }
                    }

                    div { class: "hidden md:flex items-center space-x-4",
                        Link { to: Route::Home {}, class: "text-gray-700 hover:text-blue-600 px-3 py-2",
                            "Home"
                        }

                        if is_authenticated {
                            Link {
                                to: Route::MyToursPage {},
                                class: "nav-link text-gray-700 hover:text-blue-600 px-3 py-2 rounded-md text-sm font-medium",
                                "🎬 My Tours"
                            }
                            Link { to: Route::Dashboard {}, class: "text-gray-700 hover:text-blue-600 px-3 py-2",
                                "Dashboard"
                            }
                            Link { to: Route::Profile {}, class: "text-gray-700 hover:text-blue-600 px-3 py-2",
                                "Profile"
                            }
                            // In your navbar
                            Link { to: Route::Properties {}, class: "nav-link", "Properties" }
                            Link { to: Route::MyToursPage {}, class: "nav-link", "My Tours" }
                            span { class: "text-gray-600 px-3 py-2",
                                "Welcome, {user_name}"
                            }
                            button {
                                class: "bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg transition",
                                onclick: handle_logout,
                                "Logout"
                            }
                        } else {
                            Link { to: Route::Login {}, class: "text-gray-700 hover:text-blue-600 px-3 py-2",
                                "Login"
                            }
                            Link {
                                to: Route::Register {},
                                class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition",
                                "Register"
                            }
                        }
                    }

                    div { class: "flex md:hidden items-center",
                        button {
                            class: "text-gray-700 hover:text-blue-600 p-2",
                            onclick: toggle_mobile,
                            if *mobile_menu_open.read() {
                                svg { class: "w-6 h-6", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M6 18L18 6M6 6l12 12" }
                                }
                            } else {
                                svg { class: "w-6 h-6", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M4 6h16M4 12h16M4 18h16" }
                                }
                            }
                        }
                    }
                }

                if *mobile_menu_open.read() {
                    div { class: "md:hidden pb-4",
                        Link { to: Route::Home {}, class: "block text-gray-700 hover:text-blue-600 px-3 py-2",
                            "Home"
                        }

                        if is_authenticated {
                            Link { to: Route::Dashboard {}, class: "block text-gray-700 hover:text-blue-600 px-3 py-2",
                                "Dashboard"
                            }
                            Link { to: Route::Profile {}, class: "block text-gray-700 hover:text-blue-600 px-3 py-2",
                                "Profile"
                            }
                            Link { to: Route::Properties {}, class: "block text-gray-700 hover:text-blue-600 px-3 py-2",
                                "Properties"
                            }
                            span { class: "block text-gray-600 px-3 py-2",
                                "Welcome, {user_name}"
                            }
                            button {
                                class: "block w-full text-left bg-red-500 hover:bg-red-600 text-white px-4 py-2 rounded-lg transition mt-2",
                                onclick: handle_logout,
                                "Logout"
                            }
                        } else {
                            Link { to: Route::Login {}, class: "block text-gray-700 hover:text-blue-600 px-3 py-2",
                                "Login"
                            }
                            Link {
                                to: Route::Register {},
                                class: "block bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition mt-2",
                                "Register"
                            }
                        }
                    }
                }
            }
        }
        Outlet::<Route> {}
    }
}