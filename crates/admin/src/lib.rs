#![allow(non_snake_case)]
pub mod api;
pub mod components;
pub mod context;
pub mod pages;

use dioxus::prelude::*;
use crate::components::sidebar::{AdminHeader, AdminSidebar};
use crate::context::admin_auth::use_admin_auth;

// Pages
use crate::pages::login::LoginPage;
use crate::pages::dashboard::DashboardPage;
use crate::pages::users::UsersPage;
use crate::pages::agents::AgentsPage;
use crate::pages::properties::PropertiesPage;
use crate::pages::property_detail::PropertyDetailPage;
use crate::pages::subscriptions::SubscriptionsPage;
use crate::pages::commissions::CommissionsPage;
use crate::pages::inquiries::InquiriesPage;
use crate::pages::analytics::AnalyticsPage;
use crate::pages::settings::SettingsPage;
use crate::pages::not_found::NotFoundPage;
use crate::pages::user_profile::UserProfilePage;
use crate::pages::property_owners::PropertyOwnersPage;
use crate::pages::agent_leads::LeadsPage;
use crate::pages::agent_conversion::ConversionPage;
use crate::pages::property_owner_dashboard::PropertyOwnerDashboard;

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum AdminRoute {
    #[route("/login")]
    LoginPage,

    #[layout(AdminLayout)]
        #[route("/")]
        DashboardPage,
        #[route("/owner-dashboard")]
        PropertyOwnerDashboard,
        #[route("/users")]
        UsersPage,
        #[route("/users/:id")]
        UserProfilePage { id: String },
        #[route("/agents")]
        AgentsPage,
        #[route("/property-owners")]
        PropertyOwnersPage,
        #[route("/properties")]
        PropertiesPage,
        #[route("/properties/:id")]
        PropertyDetailPage { id: String },
        #[route("/subscriptions")]
        SubscriptionsPage,
        #[route("/commissions")]
        CommissionsPage,
        #[route("/inquiries")]
        InquiriesPage,
        #[route("/analytics")]
        AnalyticsPage,
        #[route("/settings")]
        SettingsPage,
        #[route("/leads")]
        LeadsPage,
        #[route("/conversion")]
        ConversionPage,
    #[end_layout]

    #[route("/:..segments")]
    NotFoundPage { segments: Vec<String> },
}

// ───────────────────────────────────────────
// Role-Aware Layout
// Routes users to the right view based on their role
// ───────────────────────────────────────────
#[component]
fn AdminLayout() -> Element {
    let auth = use_admin_auth();
    let nav = use_navigator();

    // Not logged in → show login
    if auth.read().token.is_none() {
        return rsx! { LoginPage {} };
    }

    let user_role = auth.read().user.as_ref()
        .map(|u| u.role.to_uppercase())
        .unwrap_or_default();

    // ───────────────────────────────────────────
    // PROPERTY_OWNER → Personal Dashboard (no admin sidebar)
    // ───────────────────────────────────────────
    if user_role == "PROPERTY_OWNER" {
        // If they try to access an admin-only route, redirect them
        let current_route: AdminRoute = use_route();
        if !matches!(current_route, AdminRoute::PropertyOwnerDashboard { .. }) {
            // Use an effect to redirect without blocking render
            use_effect(move || {
                nav.push(AdminRoute::PropertyOwnerDashboard {});
            });
        }

        return rsx! {
            div { class: "flex min-h-screen bg-gray-900",
                // Simplified sidebar for property owners
                OwnerSidebar {}
                div { class: "flex-1 ml-64",
                    AdminHeader {}
                    main { class: "p-8 pt-20",
                        Outlet::<AdminRoute> {}
                    }
                }
            }
        };
    }

    // ───────────────────────────────────────────
    // AGENT / ADMIN / SUPERUSER → Full Admin Panel
    // ───────────────────────────────────────────
    rsx! {
        div { class: "flex min-h-screen bg-gray-900",
            AdminSidebar {}
            div { class: "flex-1 ml-64",
                AdminHeader {}
                main { class: "p-8 pt-20",
                    Outlet::<AdminRoute> {}
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Simplified Sidebar for Property Owners
// ───────────────────────────────────────────
#[component]
fn OwnerSidebar() -> Element {
    let auth = use_admin_auth();
    let auth_state = auth.read();
    let nav = use_navigator();

    let user_name = auth_state.user.as_ref()
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "Owner".to_string());

    let handle_logout = move |_| {
        crate::context::admin_auth::clear_token();
        let _ = nav.push(AdminRoute::LoginPage);
    };

    rsx! {
        aside { class: "fixed left-0 top-0 h-full w-64 bg-gray-800 border-r border-gray-700 overflow-y-auto flex flex-col z-20",
            div { class: "flex-1",
                div { class: "p-6 border-b border-gray-700",
                    h2 { class: "text-xl font-bold text-white", "Rento Owner Portal" }
                    p { class: "text-sm text-gray-400 mt-1", "{user_name}" }
                    p { class: "text-[10px] text-gray-500 mt-1 font-mono", "PROPERTY_OWNER" }
                }

                nav { class: "p-4 space-y-2",
                    Link {
                        to: AdminRoute::PropertyOwnerDashboard,
                        class: "flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-gray-700 text-white transition-colors",
                        active_class: "bg-gray-700",
                        span { "🏠" }
                        span { "My Dashboard" }
                    }
                }
            }

            div { class: "p-4 border-t border-gray-700 mt-auto",
                button {
                    class: "w-full flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-red-600/20 text-red-400 hover:text-red-300 transition-colors font-medium",
                    onclick: handle_logout,
                    span { "🚪" }
                    span { "Log Out" }
                }
            }
        }
    }
}

#[component]
pub fn AdminApp() -> Element {
    let mut auth = use_signal(|| context::admin_auth::AdminAuthState::default());
    use_context_provider(|| auth);

    use_hook(move || {
        let token = context::admin_auth::get_token();
        if let Some(t) = token {
            spawn(async move {
                match api::admin::get_current_admin(&t).await {
                    Ok(user) => {
                        let admin_user = context::admin_auth::AdminUser {
                            id: user.id.clone(),
                            email: user.email.clone(),
                            name: user.name.clone(),
                            role: user.role.clone(),
                            is_superuser: user.is_superuser,
                            is_staff: user.is_staff,
                        };
                        auth.set(context::admin_auth::AdminAuthState {
                            token: Some(t),
                            user: Some(admin_user),
                        });
                    }
                    Err(_) => {
                        context::admin_auth::clear_token();
                    }
                }
            });
        }
    });

    rsx! {
        Router::<AdminRoute> {}
    }
}