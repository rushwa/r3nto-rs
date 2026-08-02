use dioxus::prelude::*;

use crate::api::admin::{get_settings, update_settings, SystemSettings};
use crate::components::sidebar::PageHeader;
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn SettingsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_resource = token.clone();
    let mut saving = use_signal(|| false);
    let mut message = use_signal(|| None<String>);

    let settings = use_resource(move || {
        let t = token_for_resource.clone();
        async move {
            if t.is_empty() {
                return Ok(SystemSettings {
                    company_name: "Rento".to_string(),
                    commission_rate: 3.0,
                    maintenance_mode: false,
                    allow_registration: true,
                });
            }
            get_settings(&t).await
        }
    });

    let settings_ref = settings.read();
    let settings_data = match settings_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader { title: "Settings".to_string(), subtitle: "System configuration and preferences".to_string() }

            if let Some(s) = &settings_data {
                div { class: "space-y-6 max-w-2xl",
                    div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                        h3 { class: "text-white font-semibold mb-4", "General" }
                        div { class: "space-y-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-400 mb-1", "Company Name" }
                                input {
                                    class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded text-sm text-white focus:outline-none focus:border-blue-500",
                                    value: "{s.company_name}",
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-400 mb-1", "Commission Rate (%)" }
                                input {
                                    class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded text-sm text-white focus:outline-none focus:border-blue-500",
                                    r#type: "number",
                                    value: "{s.commission_rate}",
                                }
                            }
                        }
                    }

                    div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                        h3 { class: "text-white font-semibold mb-4", "System" }
                        div { class: "space-y-4",
                            div { class: "flex items-center justify-between",
                                div {
                                    p { class: "text-white text-sm", "Maintenance Mode" }
                                    p { class: "text-gray-500 text-xs", "Disable public access during maintenance" }
                                }
                                button {
                                    class: if s.maintenance_mode { "w-12 h-6 bg-blue-600 rounded-full relative" } else { "w-12 h-6 bg-gray-700 rounded-full relative" },
                                    div { class: if s.maintenance_mode { "absolute right-1 top-1 w-4 h-4 bg-white rounded-full" } else { "absolute left-1 top-1 w-4 h-4 bg-white rounded-full" } }
                                }
                            }
                            div { class: "flex items-center justify-between",
                                div {
                                    p { class: "text-white text-sm", "Allow Registration" }
                                    p { class: "text-gray-500 text-xs", "Let new users sign up" }
                                }
                                button {
                                    class: if s.allow_registration { "w-12 h-6 bg-blue-600 rounded-full relative" } else { "w-12 h-6 bg-gray-700 rounded-full relative" },
                                    div { class: if s.allow_registration { "absolute right-1 top-1 w-4 h-4 bg-white rounded-full" } else { "absolute left-1 top-1 w-4 h-4 bg-white rounded-full" } }
                                }
                            }
                        }
                    }

                    div { class: "bg-gray-800 rounded-lg border border-red-500/20 p-5",
                        h3 { class: "text-red-400 font-semibold mb-4", "Danger Zone" }
                        div { class: "space-y-3",
                            div { class: "flex items-center justify-between",
                                div {
                                    p { class: "text-white text-sm", "Clear All Data" }
                                    p { class: "text-gray-500 text-xs", "This action cannot be undone" }
                                }
                                button { class: "px-3 py-1.5 bg-red-500/10 text-red-400 border border-red-500/20 rounded text-sm hover:bg-red-500/20 transition-colors",
                                    "Clear"
                                }
                            }
                        }
                    }

                    if let Some(msg) = message.read().as_ref() {
                        p { class: "text-emerald-400 text-sm", "{msg}" }
                    }

                    button {
                        class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium transition-colors disabled:opacity-50",
                        disabled: saving.read().clone(),
                        onclick: move |_| {
                            saving.set(true);
                            message.set(Some("Settings saved".to_string()));
                            saving.set(false);
                        },
                        if saving.read().clone() { "Saving..." } else { "Save Changes" }
                    }
                }
            } else if settings_ref.as_ref().is_none() {
                div { class: "space-y-6 max-w-2xl animate-pulse",
                    for _ in 0..3 {
                        div { class: "h-48 bg-gray-800 rounded-lg border border-gray-700" }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load settings" }
                }
            }
        }
    }
}
