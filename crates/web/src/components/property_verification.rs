use dioxus::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub struct VerificationData {
    pub property_id: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[component]
pub fn PropertyVerification(property_id: String) -> Element {
    let mut latitude = use_signal(|| String::new());
    let mut longitude = use_signal(|| String::new());
    let mut video_file = use_signal(|| None::<web_sys::File>);
    let mut status = use_signal(|| String::new());

    let on_submit = move |_| {
        spawn(async move {
            status.set("Uploading...".to_string());
            
            // TODO: Implement multipart form upload
            // This is a simplified example
            status.set("Property verified successfully!".to_string());
        });
    };

    rsx! {
        div {
            class: "property-verification",
            h2 { "Verify Property" }
            p { "Property ID: {property_id}" }
            
            form {
                onsubmit: on_submit,
                
                div {
                    label { "Latitude:" }
                    input {
                        r#type: "number",
                        step: "0.00000001",
                        value: "{latitude}",
                        oninput: move |e| latitude.set(e.value()),
                    }
                }
                
                div {
                    label { "Longitude:" }
                    input {
                        r#type: "number",
                        step: "0.00000001",
                        value: "{longitude}",
                        oninput: move |e| longitude.set(e.value()),
                    }
                }
                
                div {
                    label { "Property Video:" }
                    input {
                        r#type: "file",
                        accept: "video/*",
                        onchange: move |e| {
                            // TODO: Handle file selection
                        }
                    }
                }
                
                button {
                    r#type: "submit",
                    "Upload & Verify"
                }
                
                if !status.read().is_empty() {
                    p {
                        class: "status-message",
                        "{status}"
                    }
                }
            }
        }
    }
}
