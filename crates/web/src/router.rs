use dioxus::prelude::*;
// use dioxus_router::prelude::*;
use crate::context::auth::use_auth;
use crate::Route;

#[component]
pub fn RouteGuards() -> Element {
    let auth = use_auth();
    let route = use_route::<Route>();

    // Check auth state and current route
    let is_authenticated = auth.read().is_authenticated;
    let current_route = route.clone();

    // Use use_effect to handle redirects reactively
    use_effect(move || {
        let is_auth = auth.read().is_authenticated;

        match current_route {
            // Auth pages - redirect to dashboard if already logged in
            Route::Login {} | Route::Register {} => {
                if is_auth {
                    // Use navigator for programmatic redirect
                    let nav = use_navigator();
                    nav.replace(Route::Dashboard {});
                }
            }
            // Protected pages - redirect to login if not logged in
            Route::Dashboard {} | Route::Profile {} | Route::Properties {} => {
                if !is_auth {
                    let nav = use_navigator();
                    nav.replace(Route::Login {});
                }
            }
            // Public pages - no guard
            _ => {}
        }
    });

    // Render the actual page content via Outlet
    rsx! {
        Outlet::<Route> {}
    }
}

/// Helper component that wraps protected routes
#[component]
pub fn ProtectedRoute(children: Element) -> Element {
    let auth = use_auth();
    let nav = use_navigator();

    use_effect(move || {
        if !auth.read().is_authenticated {
            nav.replace(Route::Login {});
        }
    });

    if auth.read().is_authenticated {
        rsx! { {children} }
    } else {
        rsx! {
            div { class: "flex items-center justify-center min-h-screen",
                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
            }
        }
    }
}

/// Helper component that redirects authenticated users away from auth pages
#[component]
pub fn GuestRoute(children: Element) -> Element {
    let auth = use_auth();
    let nav = use_navigator();

    use_effect(move || {
        if auth.read().is_authenticated {
            nav.replace(Route::Dashboard {});
        }
    });

    if !auth.read().is_authenticated {
        rsx! { {children} }
    } else {
        rsx! {
            div { class: "flex items-center justify-center min-h-screen",
                div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
            }
        }
    }
}