use dioxus::prelude::*;
use crate::Route;

#[component]
pub fn Home() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "bg-blue-600 text-white py-20",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 text-center",
                    h1 { class: "text-5xl font-bold mb-6", "Welcome to Rento" }
                    p { class: "text-xl text-blue-100 mb-8 max-w-2xl mx-auto",
                        "Find your perfect rental property or list your property for rent. Easy, fast, and secure."
                    }
                    div { class: "flex justify-center gap-4",
                        Link {
                            to: Route::Register {},
                            class: "bg-white text-blue-600 px-8 py-3 rounded-lg font-medium hover:bg-blue-50 transition",
                            "Get Started"
                        }
                        Link {
                            to: Route::Properties {},
                            class: "bg-blue-700 text-white px-8 py-3 rounded-lg font-medium hover:bg-blue-800 transition",
                            "Browse Properties"
                        }
                    }
                }
            }

            div { class: "py-16",
                div { class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                    h2 { class: "text-3xl font-bold text-center text-gray-900 mb-12", "Why Choose Rento?" }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-8",
                        div { class: "bg-white p-6 rounded-xl shadow-sm",
                            div { class: "w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center mb-4",
                                svg { class: "w-6 h-6 text-blue-600", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" }
                                }
                            }
                            h3 { class: "text-xl font-semibold text-gray-900 mb-2", "Easy Search" }
                            p { class: "text-gray-600", "Find properties that match your needs with our powerful search and filtering." }
                        }

                        div { class: "bg-white p-6 rounded-xl shadow-sm",
                            div { class: "w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center mb-4",
                                svg { class: "w-6 h-6 text-green-600", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" }
                                }
                            }
                            h3 { class: "text-xl font-semibold text-gray-900 mb-2", "Secure Platform" }
                            p { class: "text-gray-600", "Your data and transactions are protected with enterprise-grade security." }
                        }

                        div { class: "bg-white p-6 rounded-xl shadow-sm",
                            div { class: "w-12 h-12 bg-purple-100 rounded-lg flex items-center justify-center mb-4",
                                svg { class: "w-6 h-6 text-purple-600", fill: "none", stroke: "currentColor", view_box: "0 0 24 24",
                                    path { stroke_linecap: "round", stroke_linejoin: "round", stroke_width: "2", d: "M13 10V3L4 14h7v7l9-11h-7z" }
                                }
                            }
                            h3 { class: "text-xl font-semibold text-gray-900 mb-2", "Fast Process" }
                            p { class: "text-gray-600", "From search to move-in, we streamline every step of the rental process." }
                        }
                    }
                }
            }
        }
    }
}