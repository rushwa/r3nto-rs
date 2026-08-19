use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::api::auth::get_access_token;
use crate::Route;

const API_BASE: &str = "http://localhost:8000";

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
struct Tour {
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
pub fn MyToursPage() -> Element {
    let _auth = use_auth();
    let nav = use_navigator();

    let token: String = get_access_token().unwrap_or_default();

    let mut tours: Signal<Vec<Tour>> = use_signal(|| Vec::new());
    let mut loading: Signal<bool> = use_signal(|| true);
    let mut error: Signal<Option<String>> = use_signal(|| None);
    let mut generating_link: Signal<Option<String>> = use_signal(|| None);

    let token_clone = token.clone();

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
                    match r.json::<Vec<Tour>>().await {
                        Ok(data) => tours_sig.set(data),
                        Err(e) => error_sig.set(Some(format!("Parse error: {}", e))),
                    }
                }
                Ok(r) => {
                    // ✅ FIX: Capture status BEFORE consuming `r` with `.text()`
                    let status = r.status();
                    let err_text = r.text().await.unwrap_or_default();
                    error_sig.set(Some(format!("Error {}: {}", status, err_text)));
                }
                Err(e) => {
                    error_sig.set(Some(format!("Network error: {}", e)));
                }
            }
            loading_sig.set(false);
        });
    });

    let handle_watch_tour = {
        let token = token.clone();
        let nav = nav.clone();
        move |tour_id: String| {
            let t = token.clone();
            let tid = tour_id.clone();
            let mut gen_sig = generating_link;
            let nav_clone = nav.clone();

            spawn(async move {
                gen_sig.set(Some(tid.clone()));

                let client = reqwest::Client::new();
                let resp = client
                    .post(&format!("{}/api/tours/{}/viewing-link", API_BASE, tid))
                    .header("Authorization", format!("Bearer {}", t))
                    .send()
                    .await;

                match resp {
                    Ok(r) if r.status().is_success() => {
                        if let Ok(data) = r.json::<serde_json::Value>().await {
                            if let Some(viewing_url) = data.get("viewing_url").and_then(|v| v.as_str()) {
                                let token_part = viewing_url.trim_start_matches("/tour/view/");
                                nav_clone.push(Route::TourViewPage { token: token_part.to_string() });
                            }
                        }
                    }
                    Ok(r) => {
                        let _ = r.text().await; // Consume to avoid unused warning
                    }
                    Err(_) => {}
                }

                gen_sig.set(None);
            });
        }
    };

    let is_loading = *loading.read();
    let has_error = error.read().is_some();
    let error_msg = error.read().clone().unwrap_or_default();
    let tours_list = tours.read().clone();
    let current_generating = generating_link.read().clone();

    let total_tours = tours_list.len();
    let pending_count = tours_list.iter().filter(|t| t.status == "pending").count();
    let fulfilled_count = tours_list.iter().filter(|t| t.status == "fulfilled").count();

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "bg-gradient-to-r from-purple-600 to-indigo-700 text-white",
                div { class: "max-w-6xl mx-auto px-4 py-10",
                    h1 { class: "text-3xl font-bold mb-2", "🎬 My Virtual Tours" }
                    p { class: "text-purple-100", "Track your requested tours and watch your videos" }
                }
            }

            div { class: "max-w-6xl mx-auto px-4 py-8",
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-8",
                    div { class: "bg-white rounded-xl shadow-sm p-6 border border-gray-100",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-blue-100 rounded-lg flex items-center justify-center", span { class: "text-2xl", "🎬" } }
                            div {
                                p { class: "text-gray-500 text-sm", "Total Tours" }
                                p { class: "text-2xl font-bold text-gray-900", "{total_tours}" }
                            }
                        }
                    }
                    div { class: "bg-white rounded-xl shadow-sm p-6 border border-gray-100",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-yellow-100 rounded-lg flex items-center justify-center", span { class: "text-2xl", "⏳" } }
                            div {
                                p { class: "text-gray-500 text-sm", "Pending" }
                                p { class: "text-2xl font-bold text-gray-900", "{pending_count}" }
                            }
                        }
                    }
                    div { class: "bg-white rounded-xl shadow-sm p-6 border border-gray-100",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-green-100 rounded-lg flex items-center justify-center", span { class: "text-2xl", "✅" } }
                            div {
                                p { class: "text-gray-500 text-sm", "Fulfilled" }
                                p { class: "text-2xl font-bold text-gray-900", "{fulfilled_count}" }
                            }
                        }
                    }
                }

                if is_loading {
                    div { class: "flex items-center justify-center py-12",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600" }
                    }
                } else if has_error {
                    div { class: "bg-red-50 border border-red-200 rounded-lg p-6 text-center",
                        p { class: "text-red-600 font-medium", "Failed to load tours" }
                        p { class: "text-red-500 text-sm mt-1", "{error_msg}" }
                    }
                } else if tours_list.is_empty() {
                    div { class: "bg-white rounded-xl shadow-sm p-12 text-center",
                        div { class: "text-6xl mb-4", "🎬" }
                        h2 { class: "text-2xl font-bold text-gray-900 mb-2", "No Tours Yet" }
                        p { class: "text-gray-600 mb-6 max-w-md mx-auto",
                            "You haven't requested any virtual tours yet. Browse properties and click \"Request Virtual Tour\" to get started."
                        }
                        Link {
                            to: Route::Properties {},
                            class: "inline-block bg-purple-600 hover:bg-purple-700 text-white font-bold py-3 px-6 rounded-lg",
                            "Browse Properties"
                        }
                    }
                } else {
                    div { class: "space-y-4",
                        for tour in tours_list.iter() {
                            TourCard {
                                tour: tour.clone(),
                                is_generating: current_generating.as_deref() == Some(&tour.id),
                                on_watch: {
                                    let id = tour.id.clone();
                                    let handler = handle_watch_tour.clone();
                                    move |_| handler(id.clone())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TourCard(
    tour: Tour,
    is_generating: bool,
    on_watch: EventHandler<MouseEvent>,
) -> Element {
    let (status_color, status_icon, status_label) = match tour.status.as_str() {
        "pending" => ("bg-yellow-100 text-yellow-800 border-yellow-300", "⏳", "Pending"),
        "fulfilled" => ("bg-green-100 text-green-800 border-green-300", "✅", "Ready to Watch"),
        "expired" => ("bg-red-100 text-red-800 border-red-300", "⏰", "Expired"),
        "cancelled" => ("bg-gray-100 text-gray-800 border-gray-300", "❌", "Cancelled"),
        "property_delisted" => ("bg-orange-100 text-orange-800 border-orange-300", "🚫", "De-listed"),
        _ => ("bg-blue-100 text-blue-800 border-blue-300", "📋", tour.status.as_str()),
    };

    let location_display = tour.property_location.clone().unwrap_or_else(|| "Location not specified".to_string());

    let status_color_owned = status_color.to_string();
    let status_icon_owned = status_icon.to_string();
    let status_label_owned = status_label.to_string();

    rsx! {
        div { class: "bg-white rounded-xl shadow-sm border border-gray-100 hover:border-purple-200 transition-all overflow-hidden",
            div { class: "p-5 flex flex-col md:flex-row md:items-center justify-between gap-4",
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-3 mb-2 flex-wrap",
                        h3 { class: "font-bold text-lg text-gray-900 truncate", "{tour.property_title}" }
                        span { class: "px-2.5 py-0.5 rounded-full text-xs font-semibold border {status_color_owned}",
                            "{status_icon_owned} {status_label_owned}"
                        }
                    }
                    p { class: "text-sm text-gray-500 mb-1", "📍 {location_display}" }
                    p { class: "text-xs text-gray-400",
                        "Requested: {tour.created_at} • Fee: KES {tour.fee_amount}"
                    }
                }

                div { class: "flex items-center gap-3",
                    if tour.status == "fulfilled" {
                        button {
                            class: if is_generating {
                                "px-5 py-2.5 bg-gray-400 text-white text-sm font-semibold rounded-lg cursor-not-allowed flex items-center gap-2"
                            } else {
                                "px-5 py-2.5 bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-700 hover:to-indigo-700 text-white text-sm font-semibold rounded-lg shadow-md transition-all flex items-center gap-2"
                            },
                            disabled: is_generating,
                            onclick: on_watch,
                            if is_generating {
                                div { class: "animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full" }
                                "Generating..."
                            } else {
                                "▶ Watch Tour"
                            }
                        }
                    } else if tour.status == "pending" {
                        span { class: "px-4 py-2 bg-yellow-50 text-yellow-700 text-sm font-medium rounded-lg border border-yellow-200",
                            "⏳ Awaiting Agent"
                        }
                    }
                }
            }
        }
    }
}