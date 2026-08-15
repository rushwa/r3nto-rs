use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "RentoFingerprint"])]
    fn get() -> String;
}

#[derive(Clone, PartialEq)]
enum ViewState {
    Loading,
    Valid { video_url: String, remaining_minutes: i64 },
    Expired,
    WrongDevice,
    NotFound,
    Error(String),
}

#[component]
pub fn TourViewPage(token: String) -> Element {
    let mut state = use_signal(|| ViewState::Loading);
    let mut remaining_minutes = use_signal(|| 120i64);

    // Validate session on mount
    use_effect(move || {
        let token_clone = token.clone();
        let mut state_sig = state.clone();

        spawn(async move {
            // Get device fingerprint
            let fingerprint = get();

            // Call backend to validate + lock device
            let client = reqwest::Client::new();
            let resp = client
                .post(&format!("http://localhost:8000/api/tours/view/{}", token_clone))
                .json(&serde_json::json!({
                    "device_fingerprint": fingerprint
                }))
                .send()
                .await;

            match resp {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        if let Ok(data) = response.json::<serde_json::Value>().await {
                            let remaining = data.get("remaining_minutes")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(120);

                            // Build secure stream URL with fingerprint
                            let stream_url = format!(
                                "http://localhost:8000/api/tours/stream/{}?fp={}",
                                token_clone, fingerprint
                            );

                            state_sig.set(ViewState::Valid {
                                video_url: stream_url,
                                remaining_minutes: remaining,
                            });
                        }
                    } else if status.as_u16() == 410 {
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
                Err(e) => {
                    state_sig.set(ViewState::Error(format!("Network error: {}", e)));
                }
            }
        });
    });

    // Countdown timer
    use_effect(move || {
        let mut rem_sig = remaining_minutes.clone();
        let state_sig = state.clone();

        spawn(async move {
            loop {
                gloo_timers::future::sleep(std::time::Duration::from_secs(60)).await;

                // Only count down if in Valid state
                let is_valid = matches!(*state_sig.read(), ViewState::Valid { .. });
                if !is_valid {
                    break;
                }

                let current = *rem_sig.read();
                if current > 0 {
                    rem_sig.set(current - 1);
                }
            }
        });
    });

    let current_state = state.read().clone();
    let mins = *remaining_minutes.read();

    rsx! {
        div { class: "min-h-screen bg-gray-900 flex items-center justify-center p-4",
            div { class: "max-w-4xl w-full",
                // Header
                div { class: "text-center mb-6",
                    h1 { class: "text-3xl font-bold text-yellow-400", "R3NTO" }
                    p { class: "text-gray-400 mt-1", "Virtual Property Tour" }
                }

                match current_state {
                    ViewState::Loading => rsx! {
                        div { class: "bg-gray-800 rounded-lg p-12 text-center",
                            div { class: "text-4xl mb-4 animate-pulse", "🎬" }
                            p { class: "text-white text-lg", "Loading your tour..." }
                            p { class: "text-gray-400 text-sm mt-2", "Verifying device and access" }
                        }
                    },
                    ViewState::Valid { video_url, .. } => rsx! {
                        div {
                            // Countdown banner
                            div { class: "mb-4 flex items-center justify-between bg-gray-800 rounded-lg p-4",
                                div { class: "flex items-center gap-2",
                                    span { class: "text-2xl", "⏱️" }
                                    div {
                                        p { class: "text-white font-semibold", "Viewing Window Active" }
                                        p { class: "text-gray-400 text-sm", "Link locked to this device" }
                                    }
                                }
                                div { class: "text-right",
                                    p { class: if mins < 15 { "text-red-400 font-bold text-xl" } else { "text-green-400 font-bold text-xl" },
                                        "{mins} min left"
                                    }
                                }
                            }

                            // Video player
                            div { class: "bg-black rounded-lg overflow-hidden",
                                video {
                                    src: "{video_url}",
                                    class: "w-full aspect-video",
                                    controls: true,
                                    autoplay: true,
                                    playsinline: true,
                                }
                            }

                            // Security notice
                            div { class: "mt-4 bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-3",
                                p { class: "text-yellow-400 text-xs",
                                    "🔒 This tour is locked to your device and expires when the timer runs out. Sharing this link will not work on other devices."
                                }
                            }
                        }
                    },
                    ViewState::Expired => rsx! {
                        ErrorCard {
                            icon: "⏰",
                            title: "Viewing Link Expired",
                            message: "This tour link has expired. Viewing links are valid for 2 hours from first access. Please request a new tour.",
                        }
                    },
                    ViewState::WrongDevice => rsx! {
                        ErrorCard {
                            icon: "🚫",
                            title: "Device Not Authorized",
                            message: "This tour link is locked to a different device. Tour links cannot be shared between devices.",
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
        div { class: "bg-gray-800 rounded-lg p-12 text-center",
            div { class: "text-5xl mb-4", "{icon}" }
            h2 { class: "text-white text-xl font-bold mb-2", "{title}" }
            p { class: "text-gray-400", "{message}" }
        }
    }
}