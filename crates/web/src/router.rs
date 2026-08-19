use dioxus::prelude::*;
use crate::context::auth::use_auth;
use crate::Route;

#[component]
pub fn RouteGuards() -> Element {
    let auth = use_auth();
    let route = use_route::<Route>();
    let current_route = route.clone();

    use_effect(move || {
        let is_auth = auth.read().is_authenticated;
        let is_loading = auth.read().is_loading;  // ✅ KEY FIX

        // ✅ Don't make routing decisions while auth is still loading
        if is_loading {
            return;
        }

        match current_route {
            // ─── PUBLIC ROUTES (no auth required) ───
            Route::Home {}
            | Route::Properties {}
            | Route::PropertyDetailPage { .. }
            | Route::TourViewPage { .. }
            | Route::Login {}
            | Route::Register {} => {
                // Auth pages: redirect to dashboard if already logged in
                if matches!(current_route, Route::Login {} | Route::Register {}) && is_auth {
                    let nav = use_navigator();
                    nav.replace(Route::Dashboard {});
                }
            }

            // ─── PROTECTED ROUTES (require auth) ───
            Route::Dashboard {}
            | Route::Profile {}
            | Route::MyToursPage {} => {
                if !is_auth {
                    let nav = use_navigator();
                    nav.replace(Route::Login {});
                }
            }

            _ => {}
        }
    });

    rsx! {
        Outlet::<Route> {}
    }
}