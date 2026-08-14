use dioxus::prelude::*;
use crate::components::sidebar::{PageHeader, EmptyState};
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_agent_referrals, record_referral};

#[component]
pub fn AgentReferralsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let agent_id = auth.read().user.as_ref().map(|u| u.id.clone()).unwrap_or_default();

    let mut referrals = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);
    let mut show_record_modal = use_signal(|| false);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            if let Ok(data) = get_agent_referrals(&t).await {
                referrals.set(data);
            }
            loading.set(false);
        });
    });

    // Build referral link
    let referral_link = format!("https://rento.com/register?ref={}", agent_id);

    let copy_link = {
        let mut message_signal = message.clone();
        let mut is_error_signal = is_error.clone();
        let link = referral_link.clone();
        move |_| {
            // In browser, this would use navigator.clipboard
            message_signal.set(Some(format!("📋 Referral link copied: {}", link)));
            is_error_signal.set(false);
        }
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading referrals..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "My Referral Links".to_string(),
                subtitle: "Share your unique link to earn bonuses for every successful signup".to_string(),
            }

            // Referral Link Card
            div { class: "bg-gradient-to-br from-blue-900/40 to-gray-800 rounded-lg border border-blue-500/30 p-6",
                h2 { class: "text-xl font-bold text-white mb-2", "🔗 Your Unique Referral Link" }
                p { class: "text-gray-300 text-sm mb-4",
                    "Share this link with potential property owners. When they sign up using your link, you'll be automatically credited."
                }

                div { class: "bg-gray-900 rounded-lg p-4 mb-4 flex items-center gap-3",
                    code { class: "flex-1 text-blue-400 font-mono text-sm break-all", "{referral_link}" }
                    button {
                        class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium",
                        onclick: copy_link,
                        "📋 Copy"
                    }
                }

                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mt-4",
                    div { class: "bg-gray-800/50 rounded-lg p-3",
                        p { class: "text-gray-400 text-xs", "Total Referrals" }
                        p { class: "text-2xl font-bold text-white", "{referrals.read().len()}" }
                    }
                    div { class: "bg-gray-800/50 rounded-lg p-3",
                        p { class: "text-gray-400 text-xs", "Signed Up" }
                        p { class: "text-2xl font-bold text-green-400",
                            "{referrals.read().iter().filter(|r| r.get(\"signup_completed\").and_then(|v| v.as_bool()).unwrap_or(false)).count()}"
                        }
                    }
                    div { class: "bg-gray-800/50 rounded-lg p-3",
                        p { class: "text-gray-400 text-xs", "Converted" }
                        p { class: "text-2xl font-bold text-blue-400",
                            "{referrals.read().iter().filter(|r| r.get(\"conversion_completed\").and_then(|v| v.as_bool()).unwrap_or(false)).count()}"
                        }
                    }
                }
            }

            if let Some(msg) = message.read().as_ref() {
                div {
                    class: if *is_error.read() { "bg-red-900/20 border border-red-500/30 rounded-lg p-3" } else { "bg-green-900/20 border border-green-500/30 rounded-lg p-3" },
                    p { class: if *is_error.read() { "text-red-400" } else { "text-green-400" }, "{msg}" }
                }
            }

            // How It Works
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                h2 { class: "text-xl font-bold text-white mb-4", "💡 How It Works" }
                div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                    div { class: "text-center",
                        div { class: "text-3xl mb-2", "🔗" }
                        p { class: "text-white font-semibold", "1. Share Link" }
                        p { class: "text-gray-400 text-sm", "Send your referral link to potential owners" }
                    }
                    div { class: "text-center",
                        div { class: "text-3xl mb-2", "📝" }
                        p { class: "text-white font-semibold", "2. They Sign Up" }
                        p { class: "text-gray-400 text-sm", "Client registers using your link" }
                    }
                    div { class: "text-center",
                        div { class: "text-3xl mb-2", "🤝" }
                        p { class: "text-white font-semibold", "3. Digital Handshake" }
                        p { class: "text-gray-400 text-sm", "Complete conversion via handshake" }
                    }
                    div { class: "text-center",
                        div { class: "text-3xl mb-2", "💰" }
                        p { class: "text-white font-semibold", "4. Earn Commission" }
                        p { class: "text-gray-400 text-sm", "Get 30% of registration fee" }
                    }
                }
            }

            // Referrals List
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                div { class: "flex items-center justify-between mb-4",
                    h2 { class: "text-xl font-bold text-white", "👥 Referral History" }
                    button {
                        class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg text-sm font-medium",
                        onclick: move |_| show_record_modal.set(true),
                        "+ Record Referral"
                    }
                }

                if referrals.read().is_empty() {
                    EmptyState {
                        icon: "🔗".to_string(),
                        title: "No referrals yet".to_string(),
                        message: "Share your referral link to start bringing in new property owners.".to_string(),
                    }
                } else {
                    div { class: "space-y-3",
                        for referral in referrals.read().iter() {
                            ReferralRow { referral: referral.clone() }
                        }
                    }
                }
            }

            if *show_record_modal.read() {
                RecordReferralModal {
                    token: token.clone(),
                    on_close: move |_| show_record_modal.set(false),
                    on_success: move |msg: String| {
                        show_record_modal.set(false);
                        message.set(Some(msg));
                        is_error.set(false);
                    },
                }
            }
        }
    }
}

#[component]
fn ReferralRow(referral: serde_json::Value) -> Element {
    let email = referral.get("referred_email").and_then(|v| v.as_str()).unwrap_or("—");
    let name = referral.get("referred_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let signup = referral.get("signup_completed").and_then(|v| v.as_bool()).unwrap_or(false);
    let converted = referral.get("conversion_completed").and_then(|v| v.as_bool()).unwrap_or(false);
    let created_at = referral.get("created_at").and_then(|v| v.as_str()).unwrap_or("");
    let date_display = if created_at.len() > 10 { &created_at[..10] } else { created_at };

    let status_badge = if converted {
        ("bg-green-500/10 text-green-400 border-green-500/20", "✅ Converted")
    } else if signup {
        ("bg-blue-500/10 text-blue-400 border-blue-500/20", "📝 Signed Up")
    } else {
        ("bg-gray-500/10 text-gray-400 border-gray-500/20", "⏳ Pending")
    };

    rsx! {
        div { class: "bg-gray-900 rounded-lg border border-gray-700 p-4 flex items-center justify-between",
            div {
                h4 { class: "text-white font-semibold", "{name}" }
                p { class: "text-gray-400 text-sm", "{email}" }
                p { class: "text-gray-500 text-xs mt-1", "Referred: {date_display}" }
            }
            span { class: "px-3 py-1 rounded-full text-xs border {status_badge.0}", "{status_badge.1}" }
        }
    }
}

#[component]
fn RecordReferralModal(
    token: String,
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
) -> Element {
    let mut email = use_signal(|| String::new());
    let mut name = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    let token_for_submit = token.clone();
    let handle_submit = move |_| {
        if email.read().is_empty() || !email.read().contains('@') {
            error_message.set(Some("Please enter a valid email".to_string()));
            return;
        }

        loading.set(true);
        error_message.set(None);

        let t = token_for_submit.clone();
        let em = email.read().clone();
        let nm = name.read().clone();
        let success_handler = on_success.clone();

        spawn(async move {
            let name_opt = if nm.is_empty() { None } else { Some(nm.as_str()) };
            match record_referral(&t, &em, name_opt).await {
                Ok(_) => {
                    success_handler.call(format!("✅ Referral for {} recorded!", em));
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4",
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-md w-full",
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-xl font-bold text-white", "Record Referral" }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Email *" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            r#type: "email",
                            placeholder: "client@example.com",
                            value: "{email}",
                            oninput: move |evt| email.set(evt.value()),
                        }
                    }
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Name (optional)" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                            placeholder: "John Doe",
                            value: "{name}",
                            oninput: move |evt| name.set(evt.value()),
                        }
                    }

                    if let Some(err) = error_message.read().as_ref() {
                        div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3",
                            p { class: "text-red-400 text-sm", "❌ {err}" }
                        }
                    }

                    div { class: "flex gap-2 pt-4 border-t border-gray-700",
                        button {
                            class: "flex-1 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium disabled:opacity-50",
                            disabled: *loading.read(),
                            onclick: handle_submit,
                            if *loading.read() { "Recording..." } else { "Record" }
                        }
                    }
                }
            }
        }
    }
}