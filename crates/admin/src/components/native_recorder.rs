use dioxus::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

// ═══════════════════════════════════════════
// JS Bindings via wasm-bindgen
// ═══════════════════════════════════════════
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn startCamera(video_id: &str, facing_mode: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn stopCamera();

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn switchCamera(facing_mode: &str) -> js_sys::Promise;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn startRecording() -> bool;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn stopRecording() -> u32;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn getRecordedBlobUrl() -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn getRecordedSize() -> u32;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn getRecordedMimeType() -> String;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    async fn uploadVideo(tour_id: &str, auth_token: &str) -> JsValue;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn isSupported() -> bool;

    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn cleanup();

    // ✅ Watermark configuration
    #[wasm_bindgen(js_namespace = ["window", "RentoRecorder"])]
    fn setWatermark(config: JsValue);
}

// ═══════════════════════════════════════════
// Component
// ═══════════════════════════════════════════
#[component]
pub fn NativeRecorder(
    tour_request_id: String,
    property_title: String,
    agent_id: String,
    auth_token: String,
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
) -> Element {
    // State signals
    let mut status = use_signal(|| "initializing".to_string());
    let mut duration = use_signal(|| 0u32);
    let mut video_url = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);
    let mut facing_mode = use_signal(|| "environment".to_string());

    let video_id = use_signal(|| format!("video-{}", tour_request_id));

    // ✅ CRITICAL: Clone props BEFORE use_effect to avoid move errors
    let agent_for_watermark = agent_id.clone();
    let property_for_watermark = property_title.clone();

    // ═══════════════════════════════════════════
    // Initialize camera on mount
    // ═══════════════════════════════════════════
    use_effect(move || {
        let vid = video_id.read().clone();
        let mut status_sig = status.clone();
        let mut err_sig = error.clone();

        // ✅ Use the pre-cloned values
        let agent_for_wm = agent_for_watermark.clone();
        let property_for_wm = property_for_watermark.clone();

        spawn(async move {
            gloo_timers::future::sleep(std::time::Duration::from_millis(300)).await;

            if !isSupported() {
                status_sig.set("error".to_string());
                err_sig.set(Some(
                    "Your browser doesn't support video recording with watermarking. Please use Chrome, Firefox, or Edge.".to_string()
                ));
                return;
            }

            // ✅ Configure watermark BEFORE starting camera
            let watermark_config = serde_json::json!({
                "agentId": agent_for_wm,
                "propertyTitle": property_for_wm,
                "logoText": "R3NTO",
                "showTimestamp": true,
            });
            setWatermark(JsValue::from_str(&watermark_config.to_string()));

            let promise = startCamera(&vid, "environment");
            match JsFuture::from(promise).await {
                Ok(result) => {
                    if result.as_bool().unwrap_or(false) {
                        status_sig.set("ready".to_string());
                    } else {
                        status_sig.set("error".to_string());
                        err_sig.set(Some("Failed to access camera. Please check permissions.".to_string()));
                    }
                }
                Err(e) => {
                    status_sig.set("error".to_string());
                    err_sig.set(Some(format!("Camera error: {:?}", e)));
                }
            }
        });
    });

    // ═══════════════════════════════════════════
    // Recording duration timer
    // ═══════════════════════════════════════════
    use_effect(move || {
        if *status.read() != "recording" {
            return;
        }

        let mut dur_sig = duration.clone();
        let status_clone = status.clone();

        spawn(async move {
            let start = js_sys::Date::now();
            loop {
                gloo_timers::future::sleep(std::time::Duration::from_secs(1)).await;

                if *status_clone.read() != "recording" {
                    break;
                }

                let elapsed = ((js_sys::Date::now() - start) / 1000.0) as u32;
                dur_sig.set(elapsed);
            }
        });
    });

    // ═══════════════════════════════════════════
    // Handlers (all properly cloned for FnMut)
    // ═══════════════════════════════════════════

    // Start recording
    let start_recording_handler = {
        let status_sig = status.clone();
        let dur_sig = duration.clone();
        move |_| {
            let mut status_sig = status_sig.clone();
            let mut dur_sig = dur_sig.clone();

            if startRecording() {
                status_sig.set("recording".to_string());
                dur_sig.set(0);
            } else {
                status_sig.set("error".to_string());
            }
        }
    };

    // Stop recording
    let stop_recording_handler = {
        let status_sig = status.clone();
        let url_sig = video_url.clone();
        move |_| {
            let mut status_sig = status_sig.clone();
            let mut url_sig = url_sig.clone();

            let _duration = stopRecording();
            status_sig.set("stopped".to_string());

            let url_val = getRecordedBlobUrl();
            if let Some(url) = url_val.as_string() {
                url_sig.set(Some(url));
            }
        }
    };

    // Re-record
    let rerecord_handler = {
        let status_sig = status.clone();
        let url_sig = video_url.clone();
        let dur_sig = duration.clone();
        move |_| {
            let mut status_sig = status_sig.clone();
            let mut url_sig = url_sig.clone();
            let mut dur_sig = dur_sig.clone();

            url_sig.set(None);
            dur_sig.set(0);
            status_sig.set("ready".to_string());
        }
    };

    // Switch camera
    let switch_camera_handler = {
        let facing_sig = facing_mode.clone();
        move |_| {
            let mut facing_sig = facing_sig.clone();

            let new_mode = if *facing_sig.read() == "environment" {
                "user"
            } else {
                "environment"
            };

            spawn(async move {
                let promise = switchCamera(new_mode);
                if let Ok(result) = JsFuture::from(promise).await {
                    if result.as_bool().unwrap_or(false) {
                        facing_sig.set(new_mode.to_string());
                    }
                }
            });
        }
    };

    // Upload video (real file upload with progress)
    let upload_handler = {
        let token = auth_token.clone();
        let tour_id = tour_request_id.clone();
        let status_sig = status.clone();
        let success_handler = on_success.clone();
        let err_sig = error.clone();

        move |_| {
            let mut status_sig = status_sig.clone();
            let success_handler = success_handler.clone();
            let mut err_sig = err_sig.clone();
            let token = token.clone();
            let tour_id = tour_id.clone();

            status_sig.set("uploading".to_string());

            spawn(async move {
                // Call JS upload function
                let result = uploadVideo(&tour_id, &token).await;

                // Check if upload succeeded
                if result.is_undefined() || result.is_null() {
                    status_sig.set("stopped".to_string());
                    err_sig.set(Some("Upload failed: No response from server".to_string()));
                    return;
                }

                // Try to parse the response
                let result_str = js_sys::JSON::stringify(&result)
                    .map(|s| s.as_string().unwrap_or_default())
                    .unwrap_or_default();

                if result_str.contains("error") || result_str.contains("failed") {
                    status_sig.set("stopped".to_string());
                    err_sig.set(Some(format!("Upload failed: {}", result_str)));
                } else {
                    success_handler.call("✅ Tour uploaded with watermark!".to_string());
                }
            });
        }
    };

    // Close handler
    let close_handler = move |_| {
        cleanup();
        on_close.call(());
    };

    // ✅ Read values for RSX
    let video_id_str = video_id.read().clone();
    let current_status = status.read().clone();

    // ═══════════════════════════════════════════
    // Render
    // ═══════════════════════════════════════════
    rsx! {
        div { class: "fixed inset-0 bg-black/90 flex items-center justify-center z-50 p-4",
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-4xl w-full max-h-[90vh] overflow-y-auto",
                // Header
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-xl font-bold text-white", "🎥 Native Tour Recorder" }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: close_handler,
                        "×"
                    }
                }

                // ✅ Watermark banner
                div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4 mb-4",
                    div { class: "flex items-start gap-3",
                        span { class: "text-2xl", "🔒" }
                        div { class: "flex-1",
                            p { class: "text-blue-400 font-semibold text-sm mb-1",
                                "Automatic Watermarking Active"
                            }
                            p { class: "text-gray-300 text-xs",
                                "Every frame is stamped with: "
                                span { class: "text-yellow-400 font-semibold", "R3NTO" }
                                " logo • Agent ID: "
                                span { class: "text-yellow-400 font-mono", "{agent_id.get(..8).unwrap_or(&agent_id)}..." }
                                " • Live timestamp"
                            }
                            p { class: "text-gray-300 text-xs mt-1 font-medium",
                                "🏠 Property: {property_title}"
                            }
                        }
                    }
                }

                // Video preview area
                div { class: "bg-black rounded-lg aspect-video mb-4 relative flex items-center justify-center overflow-hidden",
                    video {
                        id: "{video_id_str}",
                        class: "w-full h-full object-cover",
                        autoplay: true,
                        muted: true,
                        playsinline: true,
                    }

                    // Recording indicator
                    if current_status == "recording" {
                        div { class: "absolute top-4 left-4 flex items-center gap-2 bg-red-600 px-3 py-1 rounded-full",
                            div { class: "w-3 h-3 bg-white rounded-full animate-pulse" }
                            span { class: "text-white text-sm font-semibold", "REC" }
                        }
                        div { class: "absolute top-4 right-4 bg-black/70 px-3 py-1 rounded-full",
                            span { class: "text-white text-sm font-mono",
                                "{format_duration(*duration.read())}"
                            }
                        }
                    }

                    // Switch camera button
                    if current_status == "ready" || current_status == "recording" {
                        button {
                            class: "absolute bottom-4 right-4 bg-black/70 hover:bg-black/90 text-white px-3 py-2 rounded-lg text-sm",
                            onclick: switch_camera_handler,
                            "🔄 Switch Camera"
                        }
                    }

                    // State overlays
                    if current_status == "initializing" {
                        div { class: "text-gray-500 text-center",
                            p { class: "text-4xl mb-2", "📹" }
                            p { class: "font-semibold", "Starting camera..." }
                        }
                    }
                    if current_status == "error" {
                        div { class: "text-red-400 text-center p-4",
                            p { class: "text-4xl mb-2", "⚠️" }
                            p { class: "font-semibold", "Camera Error" }
                        }
                    }
                }

                // Controls
                div { class: "flex gap-2 justify-center flex-wrap",
                    if current_status == "ready" {
                        button {
                            class: "px-6 py-2.5 bg-red-600 hover:bg-red-500 text-white rounded-lg font-medium",
                            onclick: start_recording_handler,
                            "⏺ Start Recording"
                        }
                    }
                    if current_status == "recording" {
                        button {
                            class: "px-6 py-2.5 bg-gray-600 hover:bg-gray-500 text-white rounded-lg font-medium",
                            onclick: stop_recording_handler,
                            "⏹ Stop Recording"
                        }
                    }
                    if current_status == "stopped" {
                        button {
                            class: "px-6 py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium",
                            onclick: upload_handler,
                            "✅ Upload with Watermark"
                        }
                        button {
                            class: "px-6 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium",
                            onclick: rerecord_handler,
                            "🔄 Re-record"
                        }
                    }
                    if current_status == "uploading" {
                        div { class: "text-center w-full",
                            p { class: "text-white mb-2", "Uploading & applying watermark..." }
                            div { class: "w-full bg-gray-700 rounded-full h-2",
                                div { class: "bg-blue-500 h-2 rounded-full animate-pulse", style: "width: 50%" }
                            }
                        }
                    }
                }

                // Error message
                if let Some(err) = error.read().as_ref() {
                    div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3 mt-4",
                        p { class: "text-red-400 text-sm", "❌ {err}" }
                    }
                }

                // Security notice
                div { class: "mt-4 bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-3",
                    p { class: "text-yellow-400 text-xs",
                        "⚠️ External video uploads are disabled. All tours must be recorded using this native recorder to ensure authenticity. Watermarks are burned into every video frame."
                    }
                }
            }
        }
    }
}

fn format_duration(seconds: u32) -> String {
    let mins = seconds / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}", mins, secs)
}