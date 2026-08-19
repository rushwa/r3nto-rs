use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "RentoFingerprint"])]
    fn get() -> String;
}

#[derive(Clone, PartialEq)]
enum ViewState {
    Loading,
    Valid { video_url: String, expires_at_ms: f64 },
    Expired,
    WrongDevice,
    NotFound,
    Error(String),
}

#[component]
pub fn TourViewPage(token: String) -> Element {
    let mut state = use_signal(|| ViewState::Loading);
    let mut time_left_str = use_signal(|| "Loading...".to_string());
    let mut progress_pct = use_signal(|| 100.0_f64);

    // 1. Validate Session on Mount
    use_effect(move || {
        let token_clone = token.clone();
        let mut state_sig = state;

        spawn(async move {
            let fingerprint = get();
            let client = reqwest::Client::new();
            let resp = client
                .post(&format!("http://localhost:8000/api/tours/view/{}", token_clone))
                .json(&serde_json::json!({ "device_fingerprint": fingerprint }))
                .send()
                .await;

            match resp {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        if let Ok(data) = response.json::<serde_json::Value>().await {
                            let stream_url = format!(
                                "http://localhost:8000/api/tours/stream/{}?fp={}",
                                token_clone, fingerprint
                            );

                            let expires_at_str = data.get("expires_at").and_then(|v| v.as_str()).unwrap_or("");
                            // Parse backend ISO string to JS timestamp (milliseconds)
                            let expires_at_ms = js_sys::Date::parse(expires_at_str);

                            state_sig.set(ViewState::Valid { video_url: stream_url, expires_at_ms });
                        }
                    } else if status.as_u16() == 400 || status.as_u16() == 410 {
                        state_sig.set(ViewState::Expired);
                    } else if status == reqwest::StatusCode::FORBIDDEN {
                        state_sig.set(ViewState::WrongDevice);
                    } else if status == reqwest::StatusCode::NOT_FOUND {
                        state_sig.set(ViewState::NotFound);
                    } else {
                        let err = response.text().await.unwrap_or_default();
                        state_sig.set(ViewState::Error(err));
                    }
                }
                Err(e) => state_sig.set(ViewState::Error(format!("Network error: {}", e))),
            }
        });
    });

    // 2. Precise 1-Second Countdown Timer
    use_effect(move || {
        let mut time_sig = time_left_str;
        let mut prog_sig = progress_pct;
        let mut state_sig = state.clone();

        spawn(async move {
            // ✅ PHASE 1: Wait until state becomes Valid (poll every 500ms)
            loop {
                let current_state = state_sig.read().clone();
                match current_state {
                    ViewState::Valid { .. } => break, // Ready to start countdown
                    ViewState::Expired | ViewState::WrongDevice | ViewState::NotFound | ViewState::Error(_) => {
                        return; // Error state reached, no countdown needed
                    }
                    _ => {
                        // Still loading, wait and check again
                        gloo_timers::future::sleep(std::time::Duration::from_millis(500)).await;
                    }
                }
            }

            // ✅ PHASE 2: Run the countdown
            loop {
                let current_state = state_sig.read().clone();
                let expires_ms = match current_state {
                    ViewState::Valid { expires_at_ms, .. } => expires_at_ms,
                    _ => break, // State changed away from Valid, stop
                };

                let now_ms = js_sys::Date::now();
                let diff_ms = expires_ms - now_ms;

                if diff_ms <= 0.0 {
                    // ✅ Time's up! Auto-expire the view
                    state_sig.set(ViewState::Expired);
                    time_sig.set("Expired".to_string());
                    prog_sig.set(0.0);
                    break;
                }

                let total_seconds = (diff_ms / 1000.0).ceil() as i64;
                let hours = total_seconds / 3600;
                let minutes = (total_seconds % 3600) / 60;
                let seconds = total_seconds % 60;

                let time_str = if hours > 0 {
                    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
                } else {
                    format!("{:02}:{:02}", minutes, seconds)
                };
                time_sig.set(time_str);

                // Progress bar (120 mins = 7200 seconds)
                let pct = (diff_ms / (120.0 * 60.0 * 1000.0)) * 100.0;
                prog_sig.set(pct.max(0.0).min(100.0));

                gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;
            }
        });
    });
    let current_state = state.read().clone();
    let time_display = time_left_str.read().clone();
    let pct = *progress_pct.read();

    // Dynamic colors based on time remaining
    let timer_color = if pct > 50.0 { "text-green-400" }
    else if pct > 20.0 { "text-yellow-400" }
    else { "text-red-400" };
    let bar_color = if pct > 50.0 { "bg-green-500" }
    else if pct > 20.0 { "bg-yellow-500" }
    else { "bg-red-500" };

    rsx! {
        div { class: "min-h-screen bg-gray-900 flex items-center justify-center p-4",
            div { class: "max-w-4xl w-full",
                div { class: "text-center mb-6",
                    h1 { class: "text-3xl font-bold text-yellow-400", "R3NTO" }
                    p { class: "text-gray-400 mt-1", "Secure Virtual Property Tour" }
                }

                match current_state {
                    ViewState::Loading => rsx! {
                        div { class: "bg-gray-800 rounded-lg p-12 text-center border border-gray-700",
                            div { class: "text-4xl mb-4 animate-pulse", "🎬" }
                            p { class: "text-white text-lg", "Loading your tour..." }
                            p { class: "text-gray-400 text-sm mt-2", "Verifying device and access rights" }
                        }
                    },
                    ViewState::Valid { video_url, .. } => rsx! {
                        div {
                            // ⏱️ Countdown Banner & Progress Bar
                            div { class: "mb-4 bg-gray-800 rounded-lg p-4 border border-gray-700",
                                div { class: "flex items-center justify-between mb-3",
                                    div { class: "flex items-center gap-2",
                                        span { class: "text-2xl", "⏱️" }
                                        div {
                                            p { class: "text-white font-semibold text-sm", "Viewing Window Active" }
                                            p { class: "text-gray-400 text-xs", "Link locked to this device" }
                                        }
                                    }
                                    div { class: "text-right",
                                        p { class: "font-mono font-bold text-2xl {timer_color}", "{time_display}" }
                                    }
                                }
                                // Draining Progress Bar
                                div { class: "w-full bg-gray-700 rounded-full h-2.5",
                                    div {
                                        class: "{bar_color} h-2.5 rounded-full transition-all duration-1000 ease-linear",
                                        style: "width: {pct}%"
                                    }
                                }
                            }

                            // Video Player
                            div { class: "bg-black rounded-lg overflow-hidden border border-gray-700 shadow-2xl",
                                video {
                                    src: "{video_url}",
                                    class: "w-full aspect-video",
                                    controls: true,
                                    autoplay: true,
                                    playsinline: true,
                                }
                            }

                            div { class: "mt-4 bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-3",
                                p { class: "text-yellow-400 text-xs text-center",
                                    "🔒 This tour is locked to your device. The video will automatically expire and stop playing when the timer reaches zero."
                                }
                            }
                        }
                    },
                    ViewState::Expired => rsx! {
                        ErrorCard {
                            icon: "⏰",
                            title: "Viewing Window Expired",
                            message: "The 2-hour viewing window has ended. Please contact the agent to request a new link.",
                        }
                    },
                    ViewState::WrongDevice => rsx! {
                        ErrorCard {
                            icon: "🚫",
                            title: "Device Not Authorized",
                            message: "This tour link is locked to the first device that opened it. It cannot be shared.",
                        }
                    },
                    ViewState::NotFound => rsx! {
                        ErrorCard {
                            icon: "❓",
                            title: "Tour Not Found",
                            message: "This viewing link is invalid or the tour no longer exists.",
                        }
                    },
                    ViewState::Error(msg) => rsx! {
                        ErrorCard {
                            icon: "⚠️",
                            title: "Something Went Wrong",
                            message: msg.clone(),
                        }
                    },
                }
            }
        }
    }
}

#[component]
fn ErrorCard(icon: String, title: String, message: String) -> Element {
    rsx! {
        div { class: "bg-gray-800 rounded-lg p-12 text-center border border-gray-700",
            div { class: "text-5xl mb-4", "{icon}" }
            h2 { class: "text-white text-xl font-bold mb-2", "{title}" }
            p { class: "text-gray-400", "{message}" }
        }
    }
}