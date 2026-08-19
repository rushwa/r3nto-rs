use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::api::auth::get_access_token;
use crate::Route;

#[component]
pub fn MyToursPage() -> Element {
    let auth = use_auth();

    // ✅ FIX: Get token from localStorage, not AuthState
    let token: String = get_access_token().unwrap_or_default();

    let user_email: String = auth.read().user.as_ref()
        .map(|u| u.email.clone())
        .unwrap_or_default();

    let mut tours: Signal<Vec<serde_json::Value>> = use_signal(|| Vec::new());
    let mut loading: Signal<bool> = use_signal(|| true);

    let token_clone = token.clone();
    let email_clone = user_email.clone();

    use_effect(move || {
        let _t = token_clone.clone();
        let _email = email_clone.clone();
        let mut tours_sig = tours;
        let mut loading_sig = loading;

        spawn(async move {
            tours_sig.set(vec![]);
            loading_sig.set(false);
        });
    });

    rsx! {
        div { class: "min-h-screen bg-gray-50",
            div { class: "max-w-6xl mx-auto px-4 py-8",
                h1 { class: "text-3xl font-bold text-gray-900 mb-2", "🎬 My Virtual Tours" }
                p { class: "text-gray-600 mb-8", "Track your requested virtual tours" }

                if *loading.read() {
                    div { class: "flex items-center justify-center py-12",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
                    }
                } else if tours.read().is_empty() {
                    div { class: "bg-white rounded-xl shadow-sm p-12 text-center",
                        div { class: "text-6xl mb-4", "🎬" }
                        h2 { class: "text-2xl font-bold text-gray-900 mb-2", "No Tours Yet" }
                        p { class: "text-gray-600 mb-6 max-w-md mx-auto",
                            "You haven't requested any virtual tours yet. Browse properties and click \"Request Virtual Tour\" to get started."
                        }
                        Link {
                            to: Route::Properties {},
                            class: "inline-block bg-blue-600 hover:bg-blue-700 text-white font-bold py-3 px-6 rounded-lg",
                            "Browse Properties"
                        }
                    }
                } else {
                    div { class: "space-y-4",
                        for tour in tours.read().iter() {
                            div { class: "bg-white rounded-lg shadow-sm p-4",
                                p { "Tour item" }
                            }
                        }
                    }
                }
            }
        }
    }
}