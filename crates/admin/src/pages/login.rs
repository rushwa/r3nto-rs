use dioxus::prelude::*;
use crate::AdminRoute;
use crate::api::admin::admin_login;
use crate::context::admin_auth::{use_admin_auth, set_token, AdminAuthState};

#[component]
pub fn LoginPage() -> Element {
    let mut auth = use_admin_auth();
    let nav = use_navigator();
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error = use_signal(|| Option::<String>::None);

    if auth.read().token.is_some() {
        return rsx! {
            div { class: "min-h-screen flex items-center justify-center bg-gray-900",
                div { class: "text-center",
                    p { class: "text-white text-lg", "Already logged in" }
                    button {
                        class: "mt-4 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-500 transition-colors",
                        onclick: move |_| { let _ = nav.push(AdminRoute::DashboardPage); },
                        "Go to Dashboard"
                    }
                }
            }
        };
    }

    let submit = move |_| {
        if email.read().is_empty() || password.read().is_empty() {
            error.set(Some("Please fill in all fields".to_string()));
            return;
        }
        loading.set(true);
        error.set(None);

        let e = email.read().clone();
        let p = password.read().clone();
        let mut auth = auth.clone();

        spawn(async move {
            match admin_login(e, p).await {
                Ok(resp) => {
                    set_token(&resp.token);

                    // FIX: resp.user is ALREADY an AdminUser! No manual mapping needed.
                    // Serde handles the deserialization, including #[serde(default)] fields.
                    auth.set(AdminAuthState {
                        token: Some(resp.token),
                        user: Some(resp.user),
                    });

                    let _ = nav.push(AdminRoute::DashboardPage);
                }
                Err(e) => {
                    error.set(Some(e));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gray-900",
            div { class: "w-full max-w-md bg-gray-800 rounded-xl border border-gray-700 p-8",
                div { class: "text-center mb-8",
                    div { class: "w-12 h-12 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold text-xl mx-auto mb-4",
                        "R"
                    }
                    h1 { class: "text-2xl font-bold text-white", "Rento Admin" }
                    p { class: "text-gray-400 mt-2", "Sign in to your account" }
                }

                div { class: "space-y-4",
                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Email" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white placeholder-gray-600 focus:outline-none focus:border-blue-500",
                            r#type: "email",
                            placeholder: "admin@example.com",
                            value: "{email}",
                            oninput: move |evt| email.set(evt.value()),
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-400 mb-1", "Password" }
                        input {
                            class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white placeholder-gray-600 focus:outline-none focus:border-blue-500",
                            r#type: "password",
                            placeholder: "••••••••",
                            value: "{password}",
                            oninput: move |evt| password.set(evt.value()),
                        }
                    }

                    if let Some(msg) = error.read().as_ref() {
                        p { class: "text-red-400 text-sm", "{msg}" }
                    }

                    button {
                        class: "w-full py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50",
                        disabled: *loading.read(),
                        onclick: submit,
                        if *loading.read() { "Signing in..." } else { "Sign In" }
                    }
                }
            }
        }
    }
}