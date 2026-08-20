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
use crate::pages::payouts::PayoutsPage;
use crate::pages::owner_profile::OwnerProfilePage;
use crate::pages::payment_history::PaymentHistoryPage;
use crate::pages::agent_payouts::AgentPayoutsPage;
use crate::pages::owner_inquiries::OwnerInquiriesPage;
use crate::pages::agent_performance::AgentPerformancePage;
use crate::pages::agent_referrals::AgentReferralsPage;
use crate::pages::agent_tour_studio::AgentTourStudioPage;
use crate::pages::tour_oversight::TourOversightPage;

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
        #[route("/agent-payouts")]
        AgentPayoutsPage,
        #[route("/tour-studio")]
        AgentTourStudioPage,
        // In AdminRoute enum:
        #[route("/performance")]
        AgentPerformancePage,
        #[route("/referrals")]
        AgentReferralsPage,
        
        #[route("/property-owners")]
        PropertyOwnersPage,
        #[route("/owner-inquiries")]
        OwnerInquiriesPage,
        #[route("/properties")]
        PropertiesPage,
        #[route("/properties/:id")]
        PropertyDetailPage { id: String },
        #[route("/payment-history")]
        PaymentHistoryPage,
        #[route("/subscriptions")]
        SubscriptionsPage,
        #[route("/commissions")]
        CommissionsPage,
        #[route("/owner-profile")]  // ✅ NEW: Dedicated route for Property Owner profile
        OwnerProfilePage,
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
        #[route("/payouts")]
        PayoutsPage,
        #[route("/admin/tours")]
        TourOversightPage {},
    #[end_layout]

    #[route("/:..segments")]
    NotFoundPage { segments: Vec<String> },
}

// ───────────────────────────────────────────
// Role-Aware Layout
// ───────────────────────────────────────────
#[component]
fn AdminLayout() -> Element {
    let auth = use_admin_auth();
    let nav = use_navigator();

    if auth.read().token.is_none() {
        return rsx! { LoginPage {} };
    }

    let user_role = auth.read().user.as_ref()
        .map(|u| u.role.to_uppercase())
        .unwrap_or_default();

    // PROPERTY_OWNER → Show specialized sidebar
    if user_role == "PROPERTY_OWNER" {
        return rsx! {
            div { class: "flex min-h-screen bg-gray-900",
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

    // AGENT / ADMIN / SUPERUSER → Full admin sidebar
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
// Property Owner Sidebar
// ───────────────────────────────────────────
#[component]
fn OwnerSidebar() -> Element {
    let auth = use_admin_auth();
    let auth_read = auth.read();
    let nav = use_navigator();

    let user_name = auth_read.user.as_ref()
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "Owner".to_string());

    let handle_logout = move |_| {
        crate::context::admin_auth::clear_token();
        let _ = nav.push(AdminRoute::LoginPage);
    };

    rsx! {
        aside { class: "fixed left-0 top-0 h-full w-64 bg-gray-800 border-r border-gray-700 overflow-y-auto flex flex-col z-20",
            div { class: "flex-1",
                // Logo / Brand
                div { class: "p-6 border-b border-gray-700",
                    h2 { class: "text-xl font-bold text-white", "🏠 Rento" }
                    p { class: "text-sm text-gray-400 mt-1", "Owner Portal" }
                }

                // User info
                div { class: "p-4 border-b border-gray-700",
                    div { class: "flex items-center gap-3",
                        div { class: "w-10 h-10 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold",
                            {user_name.chars().next().unwrap_or('O').to_string()}
                        }
                        div { class: "flex-1 min-w-0",
                            p { class: "text-white text-sm font-medium truncate", "{user_name}" }
                            p { class: "text-gray-500 text-xs", "Property Owner" }
                        }
                    }
                }

                // ✅ UPDATED NAVIGATION
                nav { class: "p-4 space-y-1",
                    OwnerSidebarLink {
                        to: AdminRoute::PropertyOwnerDashboard,
                        icon: "📊",
                        label: "Dashboard",
                    }
                    OwnerSidebarLink {
                        to: AdminRoute::PropertiesPage,
                        icon: "🏘️",
                        label: "My Properties",
                    }
                    OwnerSidebarLink {
                        to: AdminRoute::SubscriptionsPage,
                        icon: "⭐",
                        label: "Subscriptions",
                    }
                    // ✅ THIS IS THE MISSING LINK FOR PROPERTY OWNERS
                    OwnerSidebarLink {
                        to: AdminRoute::OwnerInquiriesPage,
                        icon: "✉️",
                        label: "Inquiries",
                    }
                    OwnerSidebarLink {
                        to: AdminRoute::PaymentHistoryPage,
                        icon: "💳",
                        label: "Payment History",
                    }
                    OwnerSidebarLink {
                        to: AdminRoute::OwnerProfilePage,
                        icon: "👤",
                        label: "My Profile",
                    }
                }
            }

            // Logout
            div { class: "p-4 border-t border-gray-700",
                button {
                    class: "w-full flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-red-600/20 text-red-400 hover:text-red-300 transition-colors",
                    onclick: handle_logout,
                    span { "🚪" }
                    span { class: "font-medium", "Log Out" }
                }
            }
        }
    }
}
#[component]
fn OwnerSidebarLink(to: AdminRoute, icon: String, label: String) -> Element {
    rsx! {
        Link {
            to: to.clone(),
            class: "flex items-center gap-3 px-4 py-2.5 rounded-lg text-gray-300 hover:bg-gray-700 hover:text-white transition-colors",
            active_class: "bg-blue-600/20 text-blue-400 border-l-2 border-blue-400",
            span { class: "text-lg", "{icon}" }
            span { class: "font-medium", "{label}" }
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