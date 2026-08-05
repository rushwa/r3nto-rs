#![allow(non_snake_case)]
pub mod api;
pub mod components;
pub mod context;
pub mod pages;
use dioxus::prelude::*;
use crate::components::sidebar::{AdminHeader, AdminSidebar};
use crate::context::admin_auth::use_admin_auth;
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

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum AdminRoute {
    #[route("/login")]
    LoginPage,

    #[layout(AdminLayout)]
    #[route("/")]
    DashboardPage,

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

    #[route("/:..segments")]
    NotFoundPage { segments: Vec<String> },
}

#[component]
fn AdminLayout() -> Element {
    let auth = use_admin_auth();

    if auth.read().token.is_none() {
        return rsx! {
            LoginPage {}
        };
    }

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
                        // FIX: Map all required fields, including is_superuser and is_staff
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