use dioxus::prelude::*;
use crate::api::auth::{login as login_api, store_tokens};
use crate::context::auth::{use_auth, set_authenticated};
use crate::Route;

#[derive(Debug, Clone)]
struct LoginForm {
    email: String,
    password: String,
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

#[component]
pub fn Login() -> Element {
    let mut form = use_signal(|| LoginForm {
        email: String::new(),
        password: String::new(),
    });
    let mut error = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let auth = use_auth();
    let nav = use_navigator();

    let auth_read = auth.read();
    if auth_read.is_authenticated {
        nav.replace(Route::Dashboard {});
        return rsx! {
            div { class: "flex items-center justify-center min-h-screen",
                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
            }
        };
    }
    drop(auth_read);

    let handle_submit = move |event: FormEvent| {
        event.prevent_default();

        let email = form.read().email.trim().to_string();
        let password = form.read().password.clone();

        // Validation
        if email.is_empty() {
            error.set(Some("Email is required".to_string()));
            return;
        }
        if !is_valid_email(&email) {
            error.set(Some("Please enter a valid email address".to_string()));
            return;
        }
        if password.is_empty() {
            error.set(Some("Password is required".to_string()));
            return;
        }
        if password.len() < 6 {
            error.set(Some("Password must be at least 6 characters".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        spawn(async move {
            match login_api(&email, &password).await {
                Ok(response) => {
                    store_tokens(&response.access_token, &response.refresh_token);
                    set_authenticated(response.user);
                    nav.replace(Route::Dashboard {});
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        });
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4",
            div { class: "max-w-md w-full bg-white rounded-xl shadow-lg p-8",
                h2 { class: "text-3xl font-bold text-center text-gray-900 mb-8", "Sign In" }

                if let Some(err) = error.read().as_ref() {
                    div { class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4",
                        "{err}"
                    }
                }

                form { onsubmit: handle_submit,
                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Email" }
                        input {
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            r#type: "email",
                            placeholder: "you@example.com",
                            value: "{form.read().email}",
                            oninput: move |e| form.write().email = e.value(),
                        }
                    }

                    div { class: "mb-6",
                        label { class: "block text-sm font-medium text-gray-700 mb-1", "Password" }
                        input {
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500",
                            r#type: "password",
                            placeholder: "Enter your password",
                            value: "{form.read().password}",
                            oninput: move |e| form.write().password = e.value(),
                        }
                    }

                    button {
                        class: "w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition disabled:opacity-50",
                        disabled: loading.read().clone(),
                        r#type: "submit",
                        if loading.read().clone() {
                            "Signing in..."
                        } else {
                            "Sign In"
                        }
                    }
                }

                p { class: "mt-4 text-center text-sm text-gray-600",
                    "Don't have an account? "
                    Link { to: Route::Register {}, class: "text-blue-600 hover:text-blue-500 font-medium",
                        "Register"
                    }
                }
            }
        }
    }
}