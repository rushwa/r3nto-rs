use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::Route;

#[component]
pub fn Properties() -> Element {
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

    rsx! {
        div { class: "min-h-screen bg-gray-50 py-8",
            div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div { class: "flex justify-between items-center mb-8",
                    h1 { class: "text-3xl font-bold text-gray-900", "Properties" }
                    button {
                        class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition",
                        "Add Property"
                    }
                }

                div { class: "bg-white rounded-xl shadow-sm p-4 mb-6",
                    div { class: "flex gap-4",
                        input {
                            class: "flex-1 px-4 py-2 border border-gray-300 rounded-lg",
                            placeholder: "Search properties...",
                        }
                        select { class: "px-4 py-2 border border-gray-300 rounded-lg",
                            option { value: "", "All Types" }
                            option { value: "apartment", "Apartment" }
                            option { value: "house", "House" }
                            option { value: "condo", "Condo" }
                        }
                        select { class: "px-4 py-2 border border-gray-300 rounded-lg",
                            option { value: "", "Any Price" }
                            option { value: "low", "Under $1000" }
                            option { value: "mid", "$1000 - $2000" }
                            option { value: "high", "$2000+" }
                        }
                    }
                }

                div { class: "bg-white rounded-xl shadow-sm p-12 text-center",
                    div { class: "w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4",
                        svg { class: "w-8 h-8 text-gray-400", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                            path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" }
                        }
                    }
                    h3 { class: "text-lg font-medium text-gray-900 mb-2", "No properties yet" }
                    p { class: "text-gray-600 mb-4", "Start by adding your first property listing." }
                    button {
                        class: "bg-blue-600 hover:bg-blue-700 text-white px-4 py-2 rounded-lg transition",
                        "Add Your First Property"
                    }
                }
            }
        }
    }
}