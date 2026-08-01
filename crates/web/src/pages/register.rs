use dioxus::prelude::*;
use crate::api::auth::{register as register_api, store_tokens, request_email_otp};
use crate::context::auth::{use_auth, set_authenticated};
use crate::Route;

#[derive(Debug, Clone)]
struct RegisterForm {
    first_name: String,
    last_name: String,
    email: String,
    phone: String,
    password: String,
    confirm_password: String,
    otp_code: String,
}

fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

fn is_valid_phone(phone: &str) -> bool {
    // Allow empty or at least 10 digits
    phone.is_empty() || phone.chars().filter(|c| c.is_digit(10)).count() >= 10
}

#[component]
pub fn Register() -> Element {
    let mut form = use_signal(|| RegisterForm {
        first_name: String::new(),
        last_name: String::new(),
        email: String::new(),
        phone: String::new(),
        password: String::new(),
        confirm_password: String::new(),
        otp_code: String::new(),
    });
    let mut error = use_signal(|| None::<String>);
    let mut success = use_signal(|| None::<String>);
    let mut loading = use_signal(|| false);
    let mut step = use_signal(|| 1);
    let mut sent_otp = use_signal(|| None::<String>); // Store the OTP that was sent
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

    let handle_request_otp = move |_| {
        let email = form.read().email.trim().to_string();

        // Validate email before sending OTP
        if email.is_empty() {
            error.set(Some("Email is required".to_string()));
            return;
        }
        if !is_valid_email(&email) {
            error.set(Some("Please enter a valid email address".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);
        success.set(None);

        spawn(async move {
            match request_email_otp(&email).await {
                Ok(_) => {
                    // Note: In production, you wouldn't store the OTP client-side.
                    // For this demo, we log it. The user gets it via email/MailHog.
                    success.set(Some(
                        "Verification code sent! Check your email or MailHog (http://localhost:8025)".to_string()
                    ));
                    step.set(2);
                }
                Err(e) => {
                    error.set(Some(e.to_string()));
                }
            }
            loading.set(false);
        });
    };

    let handle_submit = move |event: FormEvent| {
        event.prevent_default();

        let f = form.read().clone();

        // ── Step 1: Validate all required fields ──
        let email = f.email.trim();
        if email.is_empty() {
            error.set(Some("Email is required".to_string()));
            return;
        }
        if !is_valid_email(email) {
            error.set(Some("Please enter a valid email address".to_string()));
            return;
        }

        if f.first_name.trim().is_empty() {
            error.set(Some("First name is required".to_string()));
            return;
        }
        if f.last_name.trim().is_empty() {
            error.set(Some("Last name is required".to_string()));
            return;
        }

        if !f.phone.is_empty() && !is_valid_phone(&f.phone) {
            error.set(Some("Please enter a valid phone number (at least 10 digits)".to_string()));
            return;
        }

        if f.otp_code.trim().is_empty() {
            error.set(Some("Verification code is required".to_string()));
            return;
        }
        if f.otp_code.trim().len() != 6 {
            error.set(Some("Verification code must be 6 digits".to_string()));
            return;
        }

        // ── Step 2: Validate password ──
        if f.password.is_empty() {
            error.set(Some("Password is required".to_string()));
            return;
        }
        if f.password.len() < 8 {
            error.set(Some("Password must be at least 8 characters".to_string()));
            return;
        }
        if f.password != f.confirm_password {
            error.set(Some("Passwords do not match".to_string()));
            return;
        }

        loading.set(true);
        error.set(None);

        spawn(async move {
            match register_api(&f.first_name, &f.last_name, &f.email, &f.phone, &f.password, &f.otp_code).await {
                Ok(response) => {
                    store_tokens(&response.access_token, &response.refresh_token);
                    set_authenticated(response.user);
                    success.set(Some("Registration successful! Redirecting to dashboard...".to_string()));

                    gloo_timers::future::TimeoutFuture::new(1500).await;
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
                h2 { class: "text-3xl font-bold text-center text-gray-900 mb-8", "Create Account" }

                if let Some(err) = error.read().as_ref() {
                    div { class: "bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4",
                        "{err}"
                    }
                }

                if let Some(msg) = success.read().as_ref() {
                    div { class: "bg-green-50 border border-green-200 text-green-700 px-4 py-3 rounded-lg mb-4",
                        "{msg}"
                    }
                }

                if step.read().clone() == 1 {
                    // ── Step 1: Email + OTP Request ──
                    p { class: "text-gray-600 mb-4 text-sm",
                        "Step 1 of 2: Enter your email to receive a verification code."
                    }

                    div { class: "mb-4",
                        label { class: "block text-sm font-medium text-gray-700 mb-1",
                            "Email *"
                        }
                        input {
                            class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                            r#type: "email",
                            placeholder: "you@example.com",
                            value: "{form.read().email}",
                            oninput: move |e| form.write().email = e.value(),
                        }
                    }

                    button {
                        class: "w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition disabled:opacity-50",
                        disabled: loading.read().clone(),
                        onclick: handle_request_otp,
                        if loading.read().clone() { "Sending..." } else { "Send Verification Code" }
                    }
                } else {
                    // ── Step 2: Full Registration Form ──
                    p { class: "text-gray-600 mb-4 text-sm",
                        "Step 2 of 2: Complete your registration. Code sent to {form.read().email}"
                    }

                    form { onsubmit: handle_submit,
                        // Verification Code
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "Verification Code *"
                            }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500",
                                placeholder: "Enter 6-digit code",
                                maxlength: "6",
                                value: "{form.read().otp_code}",
                                oninput: move |e| {
                                    // Only allow digits
                                    let val: String = e.value().chars().filter(|c| c.is_digit(10)).collect();
                                    form.write().otp_code = val;
                                },
                            }
                            p { class: "text-xs text-gray-500 mt-1",
                                "Check your email or MailHog at localhost:8025"
                            }
                        }

                        // Name fields
                        div { class: "grid grid-cols-2 gap-4 mb-4",
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1",
                                    "First Name *"
                                }
                                input {
                                    class: "w-full px-4 py-2 border border-gray-300 rounded-lg",
                                    placeholder: "John",
                                    value: "{form.read().first_name}",
                                    oninput: move |e| form.write().first_name = e.value(),
                                }
                            }
                            div {
                                label { class: "block text-sm font-medium text-gray-700 mb-1",
                                    "Last Name *"
                                }
                                input {
                                    class: "w-full px-4 py-2 border border-gray-300 rounded-lg",
                                    placeholder: "Doe",
                                    value: "{form.read().last_name}",
                                    oninput: move |e| form.write().last_name = e.value(),
                                }
                            }
                        }

                        // Email (readonly)
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-1", "Email" }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg bg-gray-100",
                                r#type: "email",
                                value: "{form.read().email}",
                                readonly: true,
                            }
                        }

                        // Phone
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "Phone (optional)"
                            }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg",
                                r#type: "tel",
                                placeholder: "+254704900545",
                                value: "{form.read().phone}",
                                oninput: move |e| form.write().phone = e.value(),
                            }
                        }

                        // Password
                        div { class: "mb-4",
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "Password * (min 8 chars)"
                            }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg",
                                r#type: "password",
                                placeholder: "Create a strong password",
                                value: "{form.read().password}",
                                oninput: move |e| form.write().password = e.value(),
                            }
                        }

                        // Confirm Password
                        div { class: "mb-6",
                            label { class: "block text-sm font-medium text-gray-700 mb-1",
                                "Confirm Password *"
                            }
                            input {
                                class: "w-full px-4 py-2 border border-gray-300 rounded-lg",
                                r#type: "password",
                                placeholder: "Repeat your password",
                                value: "{form.read().confirm_password}",
                                oninput: move |e| form.write().confirm_password = e.value(),
                            }
                        }

                        // Submit button
                        button {
                            class: "w-full bg-blue-600 hover:bg-blue-700 text-white font-medium py-2 px-4 rounded-lg transition disabled:opacity-50",
                            disabled: loading.read().clone(),
                            r#type: "submit",
                            if loading.read().clone() { "Creating Account..." } else { "Create Account" }
                        }

                        // Back to step 1
                        button {
                            class: "w-full mt-2 text-gray-600 hover:text-blue-600 text-sm py-2",
                            r#type: "button",
                            onclick: move |_| {
                                step.set(1);
                                error.set(None);
                            },
                            "← Use a different email"
                        }
                    }
                }

                p { class: "mt-4 text-center text-sm text-gray-600",
                    "Already have an account? "
                    Link { to: Route::Login {}, class: "text-blue-600 hover:text-blue-500 font-medium",
                        "Sign In"
                    }
                }
            }
        }
    }
}