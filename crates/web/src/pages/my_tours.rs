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
                    Ok(r) => { let _ = r.text().await; }
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
        div { class: "min-h-screen bg-gray-900",
            // Header
            div { class: "bg-gray-800 border-b border-gray-700",
                div { class: "max-w-6xl mx-auto px-4 py-8",
                    h1 { class: "text-3xl font-bold text-white mb-1", "🎬 My Virtual Tours" }
                    p { class: "text-gray-400", "Track your requested tours and watch your videos" }
                }
            }

            div { class: "max-w-6xl mx-auto px-4 py-8",
                // Stats
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-8",
                    div { class: "bg-gray-800 border border-gray-700 rounded-xl p-6",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-blue-600/20 rounded-lg flex items-center justify-center", span { class: "text-2xl", "🎬" } }
                            div {
                                p { class: "text-gray-400 text-sm", "Total Tours" }
                                p { class: "text-2xl font-bold text-white", "{total_tours}" }
                            }
                        }
                    }
                    div { class: "bg-gray-800 border border-gray-700 rounded-xl p-6",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-yellow-500/20 rounded-lg flex items-center justify-center", span { class: "text-2xl", "⏳" } }
                            div {
                                p { class: "text-gray-400 text-sm", "Pending" }
                                p { class: "text-2xl font-bold text-white", "{pending_count}" }
                            }
                        }
                    }
                    div { class: "bg-gray-800 border border-gray-700 rounded-xl p-6",
                        div { class: "flex items-center gap-4",
                            div { class: "w-12 h-12 bg-green-500/20 rounded-lg flex items-center justify-center", span { class: "text-2xl", "✅" } }
                            div {
                                p { class: "text-gray-400 text-sm", "Fulfilled" }
                                p { class: "text-2xl font-bold text-white", "{fulfilled_count}" }
                            }
                        }
                    }
                }

                if is_loading {
                    div { class: "flex items-center justify-center py-12",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 text-blue-400" }
                    }
                } else if has_error {
                    div { class: "bg-gray-800 border border-red-500/30 rounded-lg p-6 text-center",
                        p { class: "text-red-400 font-medium", "Failed to load tours" }
                        p { class: "text-red-400 text-sm mt-1", "{error_msg}" }
                    }
                } else if tours_list.is_empty() {
                    div { class: "bg-gray-800 border border-gray-700 rounded-xl p-12 text-center",
                        div { class: "text-6xl mb-4", "🎬" }
                        h2 { class: "text-2xl font-bold text-white mb-2", "No Tours Yet" }
                        p { class: "text-gray-400 mb-6 max-w-md mx-auto",
                            "You haven't requested any virtual tours yet. Browse properties and click \"Request Virtual Tour\" to get started."
                        }
                        Link {
                            to: Route::Properties {},
                            class: "inline-block bg-blue-600 hover:bg-blue-500 text-white font-bold py-3 px-6 rounded-lg",
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
        "pending" => ("bg-yellow-500/20 text-yellow-400 border-yellow-500/30", "⏳", "Pending"),
        "fulfilled" => ("bg-green-500/20 text-green-400 border-green-500/30", "✅", "Ready to Watch"),
        "expired" => ("bg-red-500/20 text-red-400 border-red-500/30", "⏰", "Expired"),
        "cancelled" => ("bg-gray-500/20 text-gray-400 border-gray-500/30", "❌", "Cancelled"),
        "property_delisted" => ("bg-orange-500/20 text-orange-400 border-orange-500/30", "🚫", "De-listed"),
        _ => ("bg-blue-500/20 text-blue-400 border-blue-500/30", "📋", tour.status.as_str()),
    };

    let location_display = tour.property_location.clone().unwrap_or_else(|| "Location not specified".to_string());

    let status_color_owned = status_color.to_string();
    let status_icon_owned = status_icon.to_string();
    let status_label_owned = status_label.to_string();

    rsx! {
        div { class: "bg-gray-800 border border-gray-700 rounded-xl overflow-hidden",
            div { class: "p-5 flex flex-col md:flex-row md:items-center justify-between gap-4",
                div { class: "flex-1 min-w-0",
                    div { class: "flex items-center gap-3 mb-2 flex-wrap",
                        h3 { class: "font-bold text-lg text-white truncate", "{tour.property_title}" }
                        span { class: "px-2.5 py-0.5 rounded-full text-xs font-semibold border {status_color_owned}",
                            "{status_icon_owned} {status_label_owned}"
                        }
                    }
                    p { class: "text-sm text-gray-400 mb-1", "📍 {location_display}" }
                    p { class: "text-xs text-gray-400", "Requested: {tour.created_at} • Fee: KES {tour.fee_amount}" }
                }

                div { class: "flex items-center gap-3",
                    if tour.status == "fulfilled" {
                        button {
                            class: if is_generating {
                                "px-5 py-2.5 bg-gray-600 text-white text-sm font-semibold rounded-lg cursor-not-allowed"
                            } else {
                                "px-5 py-2.5 bg-green-600 text-white text-sm font-semibold rounded-lg"
                            },
                            disabled: is_generating,
                            onclick: on_watch,
                            if is_generating { "Generating..." } else { "▶ Watch Tour" }
                        }
                    } else if tour.status == "pending" {
                        span { class: "px-4 py-2 bg-yellow-500/20 text-yellow-400 text-sm font-medium rounded-lg border border-yellow-500/30",
                            "⏳ Awaiting Agent"
                        }
                    }
                }
            }
        }
    }
}