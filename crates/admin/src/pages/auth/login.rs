use dioxus::prelude::*;

#[component]
pub fn LoginPage() -> Element {
    let mut email = use_signal(String::new);
    let mut password = use_signal(String::new);
    
    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gray-900",
            div { class: "bg-gray-800 rounded-lg p-8 w-full max-w-md",
                h1 { class: "text-2xl font-bold mb-6 text-center", "Admin Login" }
                form { class: "space-y-4",
                    div {
                        label { class: "block text-sm text-gray-300 mb-1", "Email" }
                        input { class: "w-full bg-gray-700 px-3 py-2 rounded-lg", r#type: "email",
                            oninput: move |e| email.set(e.value()) }
                    }
                    div {
                        label { class: "block text-sm text-gray-300 mb-1", "Password" }
                        input { class: "w-full bg-gray-700 px-3 py-2 rounded-lg", r#type: "password",
                            oninput: move |e| password.set(e.value()) }
                    }
                    button { class: "w-full bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded-lg", r#type: "submit", "Login" }
                }
            }
        }
    }
}
