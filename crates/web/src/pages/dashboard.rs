use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::api::auth::get_access_token;
use crate::Route;

const API_BASE: &str = "http://localhost:8000";

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
struct TourRequest {
    pub id: String,
    pub client_name: Option<String>,
    pub status: String,
    pub fee_amount: String,
    pub fee_paid: bool,
    pub created_at: String,
    pub fulfilled_at: Option<String>,
    pub property_title: String,
    pub property_location: Option<String>,
    pub video_url: Option<String>,
    pub duration_seconds: Option<i32>,
}

#[component]
pub fn Dashboard() -> Element {
    let auth = use_auth();
    let token: String = get_access_token().unwrap_or_default();

    let mut tours: Signal<Vec<TourRequest>> = use_signal(|| Vec::new());
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    let token_clone = token.clone();

    // Fetch user's tours
    use_effect(move || {
        let t = token_clone.clone();
        let mut tours_sig = tours;
        let mut loading_sig = loading;
        let mut error_sig = error;

        spawn(async move {
            let client = reqwest::Client::new();
            let resp = client
                .get(&format!("{}/api/tours/my-tours", API_BASE))
                .header("Authorization", format!("Bearer {}", t))
                .send()
                .await;

            match resp {
                Ok(r) if r.status().is_success() => {
                    match r.json::<Vec<TourRequest>>().await {
                        Ok(data) => tours_sig.set(data),
                        Err(e) => error_sig.set(Some(format!("Parse error: {}", e))),
                    }
                }
                Ok(r) => {
                    error_sig.set(Some(format!("Error: {}", r.status())));
                }
                Err(e) => {
                    error_sig.set(Some(format!("Network error: {}", e)));
                }
            }
            loading_sig.set(false);
        });
    });

    // ✅ Pre-compute user info before rsx!
    let user_name: String = auth.read().user.as_ref()
        .map(|u| {
            let full = format!("{} {}", u.first_name, u.last_name).trim().to_string();
            if full.is_empty() { u.username.clone() } else { full }
        })
        .unwrap_or_else(|| "User".to_string());

    let user_email: String = auth.read().user.as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_default();

    let user_role: String = auth.read().user.as_ref()
        .map(|u| u.role.clone())
        .unwrap_or_else(|| "CLIENT".to_string());

    let is_loading = *loading.read();
    let tours_list = tours.read().clone();
    let has_error = error.read().is_some();
    let error_msg = error.read().clone().unwrap_or_default();

    // Count stats
    let total_tours = tours_list.len();
    let pending_count = tours_list.iter().filter(|t| t.status == "pending").count();
    let fulfilled_count = tours_list.iter().filter(|t| t.status == "fulfilled").count();

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            // ─── Header ───
            div { class: "bg-gradient-to-r from-blue-600 to-indigo-700 text-white",
                div { class: "max-w-6xl mx-auto px-4 py-10",
                    div { class: "flex items-center justify-between",
                        div {
                            h1 { class: "text-3xl font-bold mb-1", "Welcome back, {user_name}! 👋" }
                            p { class: "text-blue-100", "{user_email}" }
                            span { class: "inline-block mt-2 px-3 py-1 bg-white/20 rounded-full text-sm",
                                "{user_role}"
                            }
                        }
                        Link {
                            to: Route::Properties {},
                            class: "bg-white text-blue-600 font-bold py-3 px-6 rounded-lg hover:bg-blue-50 transition-colors shadow-lg",
                            "🏠 Browse Properties"
                        }
                    }
                }
            }

            div { class: "max-w-6xl mx-auto px-4 py-8",
                // ─── Quick Stats ───
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-8",
                    div { class: "bg-white rounded-xl shadow-sm p-6 border border-gray-100",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center",
                                span { class: "text-2xl", "🎬" }
                            }
                            div {
                                p { class: "text-gray-500 text-sm", "Total Tours" }
                                p { class: "text-2xl font-bold text-gray-900", "{total_tours}" }
                            }
                        }
                    }
                    div { class: "bg-white rounded-xl shadow-sm p-6 border border-gray-100",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-yellow-100 rounded-lg flex items-center justify-center",
                                span { class: "text-2xl", "⏳" }
                            }
                            div {
                                p { class: "text-gray-500 text-sm", "Pending" }
                                p { class: "text-2xl font-bold text-gray-900", "{pending_count}" }
                            }
                        }
                    }
                    div { class: "bg-white rounded-xl shadow-sm p-6 border border-gray-100",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center",
                                span { class: "text-2xl", "✅" }
                            }
                            div {
                                p { class: "text-gray-500 text-sm", "Fulfilled" }
                                p { class: "text-2xl font-bold text-gray-900", "{fulfilled_count}" }
                            }
                        }
                    }
                }

                // ─── Quick Actions ───
                div { class: "bg-white rounded-xl shadow-sm p-6 mb-8 border border-gray-100",
                    h2 { class: "text-xl font-bold text-gray-900 mb-4", "Quick Actions" }
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        Link {
                            to: Route::MyToursPage {},
                            class: "flex items-center gap-4 p-4 bg-yellow-50 hover:bg-yellow-100 rounded-lg border border-yellow-200 transition-colors",
                            div { class: "text-3xl", "🎬" }
                            div {
                                p { class: "font-semibold text-gray-900", "My Virtual Tours" }
                                p { class: "text-sm text-gray-500", "Track requests & watch videos" }
                            }
                        }
                        Link {
                            to: Route::Properties {},
                            class: "flex items-center gap-4 p-4 bg-blue-50 hover:bg-blue-100 rounded-lg border border-blue-200 transition-colors",
                            div { class: "text-3xl", "🏠" }
                            div {
                                p { class: "font-semibold text-gray-900", "Browse Properties" }
                                p { class: "text-sm text-gray-500", "Find your next home" }
                            }
                        }
                        Link {
                            to: Route::Profile {},
                            class: "flex items-center gap-4 p-4 bg-gray-50 hover:bg-gray-100 rounded-lg border border-gray-200 transition-colors",
                            div { class: "text-3xl", "👤" }
                            div {
                                p { class: "font-semibold text-gray-900", "My Profile" }
                                p { class: "text-sm text-gray-500", "Manage your account" }
                            }
                        }
                    }
                }

                // ─── Recent Tours ───
                div { class: "bg-white rounded-xl shadow-sm p-6 border border-gray-100",
                    div { class: "flex items-center justify-between mb-4",
                        h2 { class: "text-xl font-bold text-gray-900", "Recent Tour Requests" }
                        Link {
                            to: Route::MyToursPage {},
                            class: "text-blue-600 hover:text-blue-800 text-sm font-medium",
                            "View All →"
                        }
                    }

                    if is_loading {
                        div { class: "flex items-center justify-center py-8",
                            div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" }
                        }
                    } else if has_error {
                        div { class: "bg-red-50 border border-red-200 rounded-lg p-4 text-center",
                            p { class: "text-red-600", "{error_msg}" }
                        }
                    } else if tours_list.is_empty() {
                        div { class: "text-center py-8",
                            div { class: "text-5xl mb-3", "🎬" }
                            p { class: "text-gray-600 font-medium mb-2", "No tour requests yet" }
                            p { class: "text-gray-500 text-sm mb-4",
                                "Browse properties and request a virtual tour to get started."
                            }
                            Link {
                                to: Route::Properties {},
                                class: "inline-block bg-blue-600 hover:bg-blue-700 text-white font-bold py-2 px-6 rounded-lg",
                                "Browse Properties"
                            }
                        }
                    } else {
                        div { class: "space-y-3",
                            for tour in tours_list.iter().take(5) {
                                TourCard { tour: tour.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Tour Card Component
// ───────────────────────────────────────────
#[component]
fn TourCard(tour: TourRequest) -> Element {
    // ✅ Pre-compute display values
    let (status_color, status_icon, status_label) = match tour.status.as_str() {
        "pending" => ("bg-yellow-100 text-yellow-700 border-yellow-300", "⏳", "Pending"),
        "fulfilled" => ("bg-green-100 text-green-700 border-green-300", "✅", "Ready to Watch"),
        "expired" => ("bg-red-100 text-red-700 border-red-300", "⏰", "Expired"),
        "cancelled" => ("bg-gray-100 text-gray-600 border-gray-300", "❌", "Cancelled"),
        "property_delisted" => ("bg-orange-100 text-orange-700 border-orange-300", "🚫", "Property De-listed"),
        _ => ("bg-blue-100 text-blue-700 border-blue-300", "📋", tour.status.as_str()),
    };

    let location_display: String = tour.property_location.clone()
        .unwrap_or_else(|| "Location not specified".to_string());

    let duration_display: String = tour.duration_seconds
        .map(|s| format!("{}m {}s", s / 60, s % 60))
        .unwrap_or_else(|| "--".to_string());

    let status_color_owned = status_color.to_string();
    let status_icon_owned = status_icon.to_string();
    let status_label_owned = status_label.to_string();

    rsx! {
        div { class: "flex items-center justify-between p-4 bg-gray-50 rounded-lg border border-gray-100 hover:border-blue-200 transition-colors",
            div { class: "flex-1 min-w-0",
                div { class: "flex items-center gap-3 mb-1 flex-wrap",
                    h3 { class: "font-semibold text-gray-900 truncate", "{tour.property_title}" }
                    span { class: "px-2 py-0.5 rounded-full text-xs font-medium border {status_color_owned}",
                        "{status_icon_owned} {status_label_owned}"
                    }
                }
                p { class: "text-sm text-gray-500", "📍 {location_display}" }
                p { class: "text-xs text-gray-400 mt-1",
                    "Requested: {tour.created_at} • Duration: {duration_display}"
                }
            }

            // Action button based on status
            if tour.status == "fulfilled" {
                if let Some(_url) = &tour.video_url {
                    Link {
                        to: Route::MyToursPage {},
                        class: "ml-4 px-4 py-2 bg-green-600 hover:bg-green-700 text-white text-sm font-medium rounded-lg whitespace-nowrap",
                        "▶ Watch"
                    }
                }
            } else if tour.status == "pending" {
                span { class: "ml-4 px-4 py-2 bg-yellow-100 text-yellow-700 text-sm font-medium rounded-lg whitespace-nowrap",
                    "⏳ Awaiting Agent"
                }
            }
        }
    }
}