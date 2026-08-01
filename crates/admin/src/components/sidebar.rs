use dioxus::prelude::*;

use crate::AdminRoute;
use crate::context::admin_auth::{use_admin_auth, clear_token};

#[derive(Clone, PartialEq, Eq, Hash)]
enum SidebarSection {
    Overview,
    Properties,
    Finance,
    Accounts,
    System,
}

#[component]
pub fn AdminSidebar() -> Element {
    let nav = use_navigator();
    let current = use_route::<AdminRoute>();
    let mut expanded = use_signal(|| {
        let mut m = std::collections::HashSet::new();
        m.insert(SidebarSection::Overview);
        m.insert(SidebarSection::Properties);
        m
    });

    let is_active = |route: &AdminRoute| -> bool {
        std::mem::discriminant(&current) == std::mem::discriminant(route)
    };

    let is_expanded = |section: &SidebarSection| -> bool {
        expanded.read().contains(section)
    };

    let toggle_section = move |section: SidebarSection| {
        let mut e = expanded;
        move |_| {
            let mut set = e.read().clone();
            if set.contains(&section) {
                set.remove(&section);
            } else {
                set.insert(section.clone());
            }
            e.set(set);
        }
    };

    rsx! {
        aside { class: "fixed left-0 top-0 h-full w-64 bg-gray-900 border-r border-gray-800 flex flex-col z-40",
            div { class: "p-4 border-b border-gray-800",
                div { class: "flex items-center gap-3",
                    div { class: "w-8 h-8 bg-blue-600 rounded-lg flex items-center justify-center text-white font-bold text-sm",
                        "R"
                    }
                    div {
                        h2 { class: "text-white font-semibold text-sm", "Rento Admin" }
                        p { class: "text-gray-500 text-xs", "Real Estate Management" }
                    }
                }
            }

            nav { class: "flex-1 p-3 space-y-1 overflow-y-auto",
                div {
                    button {
                        class: "w-full flex items-center justify-between px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider hover:text-gray-400",
                        onclick: toggle_section(SidebarSection::Overview),
                        span { "Overview" }
                        span { if is_expanded(&SidebarSection::Overview) { "-" } else { "+" } }
                    }
                    if is_expanded(&SidebarSection::Overview) {
                        div { class: "space-y-1 mt-1",
                            NavLink { route: AdminRoute::DashboardPage, label: "Dashboard", icon: "📊", is_active: is_active(&AdminRoute::DashboardPage) }
                        }
                    }
                }

                div {
                    button {
                        class: "w-full flex items-center justify-between px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider hover:text-gray-400",
                        onclick: toggle_section(SidebarSection::Properties),
                        span { "Properties" }
                        span { if is_expanded(&SidebarSection::Properties) { "-" } else { "+" } }
                    }
                    if is_expanded(&SidebarSection::Properties) {
                        div { class: "space-y-1 mt-1",
                            NavLink { route: AdminRoute::PropertiesPage, label: "All Properties", icon: "🏠", is_active: is_active(&AdminRoute::PropertiesPage) }
                            NavLink { route: AdminRoute::InquiriesPage, label: "Inquiries", icon: "📨", is_active: is_active(&AdminRoute::InquiriesPage) }
                        }
                    }
                }

                div {
                    button {
                        class: "w-full flex items-center justify-between px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider hover:text-gray-400",
                        onclick: toggle_section(SidebarSection::Finance),
                        span { "Finance" }
                        span { if is_expanded(&SidebarSection::Finance) { "-" } else { "+" } }
                    }
                    if is_expanded(&SidebarSection::Finance) {
                        div { class: "space-y-1 mt-1",
                            NavLink { route: AdminRoute::CommissionsPage, label: "Commissions", icon: "💰", is_active: is_active(&AdminRoute::CommissionsPage) }
                            NavLink { route: AdminRoute::SubscriptionsPage, label: "Subscriptions", icon: "💳", is_active: is_active(&AdminRoute::SubscriptionsPage) }
                        }
                    }
                }

                div {
                    button {
                        class: "w-full flex items-center justify-between px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider hover:text-gray-400",
                        onclick: toggle_section(SidebarSection::Accounts),
                        span { "Accounts" }
                        span { if is_expanded(&SidebarSection::Accounts) { "-" } else { "+" } }
                    }
                    if is_expanded(&SidebarSection::Accounts) {
                        div { class: "space-y-1 mt-1",
                            NavLink { route: AdminRoute::UsersPage, label: "Users", icon: "👥", is_active: is_active(&AdminRoute::UsersPage) }
                            NavLink { route: AdminRoute::AgentsPage, label: "Agents", icon: "🏢", is_active: is_active(&AdminRoute::AgentsPage) }
                            NavLink { route: AdminRoute::PropertyOwnersPage, label: "Property Owners", icon: "🏘️", is_active: is_active(&AdminRoute::PropertyOwnersPage) }
                        }
                    }
                }

                div {
                    button {
                        class: "w-full flex items-center justify-between px-3 py-2 text-xs font-semibold text-gray-500 uppercase tracking-wider hover:text-gray-400",
                        onclick: toggle_section(SidebarSection::System),
                        span { "System" }
                        span { if is_expanded(&SidebarSection::System) { "-" } else { "+" } }
                    }
                    if is_expanded(&SidebarSection::System) {
                        div { class: "space-y-1 mt-1",
                            NavLink { route: AdminRoute::AnalyticsPage, label: "Analytics", icon: "📈", is_active: is_active(&AdminRoute::AnalyticsPage) }
                            NavLink { route: AdminRoute::SettingsPage, label: "Settings", icon: "⚙️", is_active: is_active(&AdminRoute::SettingsPage) }
                        }
                    }
                }
            }

            div { class: "p-3 border-t border-gray-800",
                {
                    let auth = use_admin_auth();
                    let user = auth.read().user.clone();
                    let name = user.as_ref().map(|u| u.name.clone()).unwrap_or_else(|| "Admin".to_string());
                    let email = user.as_ref().map(|u| u.email.clone()).unwrap_or_else(|| "".to_string());
                    let initial = user.as_ref().map(|u| u.name.chars().next().unwrap_or('A')).unwrap_or('A');
                    rsx! {
                        div { class: "flex items-center gap-3",
                            div { class: "w-8 h-8 rounded-full bg-gray-700 flex items-center justify-center text-white text-xs font-bold",
                                "{initial}"
                            }
                            div { class: "flex-1 min-w-0",
                                p { class: "text-white text-sm font-medium truncate", "{name}" }
                                p { class: "text-gray-500 text-xs truncate", "{email}" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NavLink(route: AdminRoute, label: String, icon: String, is_active: bool) -> Element {
    let nav = use_navigator();
    let cls = if is_active {
        "w-full flex items-center gap-3 px-4 py-2 rounded-lg bg-gray-700 text-white text-sm"
    } else {
        "w-full flex items-center gap-3 px-4 py-2 rounded-lg text-gray-400 hover:bg-gray-700/50 hover:text-white text-sm transition-colors"
    };
    rsx! {
        button {
            class: "{cls}",
            onclick: move |_| { let _ = nav.push(route.clone()); },
            span { class: "w-5 text-center", "{icon}" }
            span { "{label}" }
        }
    }
}

#[component]
pub fn AdminHeader() -> Element {
    let mut auth = use_admin_auth();
    let nav = use_navigator();
    let current = use_route::<AdminRoute>();

    let title = match current {
        AdminRoute::DashboardPage => "Dashboard",
        AdminRoute::UsersPage => "Users",
        AdminRoute::UserProfilePage { .. } => "User Profile",
        AdminRoute::AgentsPage => "Agents",
        AdminRoute::PropertyOwnersPage => "Property Owners",
        AdminRoute::PropertiesPage => "Properties",
        AdminRoute::PropertyDetailPage { .. } => "Property Detail",
        AdminRoute::SubscriptionsPage => "Subscriptions",
        AdminRoute::CommissionsPage => "Commissions",
        AdminRoute::InquiriesPage => "Inquiries",
        AdminRoute::AnalyticsPage => "Analytics",
        AdminRoute::SettingsPage => "Settings",
        AdminRoute::LoginPage => "Login",
        AdminRoute::NotFoundPage { .. } => "Not Found",
    };

    let logout = move |_| {
        clear_token();
        auth.set(crate::context::admin_auth::AdminAuthState::default());
        let _ = nav.push(AdminRoute::LoginPage);
    };

    rsx! {
        header { class: "fixed top-0 right-0 left-64 h-14 bg-gray-900 border-b border-gray-800 flex items-center justify-between px-6 z-30",
            h1 { class: "text-white font-semibold text-sm", "{title}" }
            div { class: "flex items-center gap-4",
                button {
                    class: "text-gray-500 hover:text-red-400 text-sm transition-colors",
                    onclick: logout,
                    "Logout"
                }
            }
        }
    }
}

#[component]
pub fn StatCard(title: String, value: String, icon: String, change: String, change_positive: bool) -> Element {
    let change_color = if change_positive { "text-emerald-400" } else { "text-red-400" };
    let icon_bg = match icon.as_str() {
        "🏠" => "bg-blue-500/10 text-blue-400",
        "💰" => "bg-emerald-500/10 text-emerald-400",
        "📈" => "bg-purple-500/10 text-purple-400",
        "👥" => "bg-orange-500/10 text-orange-400",
        "🏢" => "bg-cyan-500/10 text-cyan-400",
        "📨" => "bg-pink-500/10 text-pink-400",
        _ => "bg-gray-500/10 text-gray-400",
    };
    rsx! {
        div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700",
            div { class: "flex items-start justify-between",
                div {
                    p { class: "text-gray-400 text-xs font-medium uppercase tracking-wider", "{title}" }
                    p { class: "text-2xl font-bold text-white mt-1", "{value}" }
                    p { class: "text-xs mt-2 {change_color}", "{change}" }
                }
                div { class: "w-10 h-10 rounded-lg {icon_bg} flex items-center justify-center text-lg",
                    "{icon}"
                }
            }
        }
    }
}

#[component]
pub fn StatusBadge(status: String) -> Element {
    let (color, dot) = match status.as_str() {
        "active" | "verified" | "paid" | "completed" | "approved" | "sold" | "closed" => {
            ("bg-emerald-500/10 text-emerald-400 border-emerald-500/20", "bg-emerald-400")
        }
        "pending" | "unverified" | "processing" | "review" | "new" | "contacted" => {
            ("bg-amber-500/10 text-amber-400 border-amber-500/20", "bg-amber-400")
        }
        "inactive" | "suspended" | "failed" | "rejected" | "banned" | "expired" => {
            ("bg-red-500/10 text-red-400 border-red-500/20", "bg-red-400")
        }
        "viewing" | "negotiating" => {
            ("bg-blue-500/10 text-blue-400 border-blue-500/20", "bg-blue-400")
        }
        _ => ("bg-gray-500/10 text-gray-400 border-gray-500/20", "bg-gray-400"),
    };
    rsx! {
        span { class: "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium border {color}",
            span { class: "w-1.5 h-1.5 rounded-full {dot}" }
            "{status}"
        }
    }
}

#[component]
pub fn PageHeader(title: String, subtitle: String) -> Element {
    rsx! {
        div { class: "mb-6",
            h1 { class: "text-xl font-bold text-white", "{title}" }
            p { class: "text-gray-400 text-sm mt-0.5", "{subtitle}" }
        }
    }
}

#[component]
pub fn FilterBar(children: Element) -> Element {
    rsx! {
        div { class: "flex items-center gap-3 mb-4 p-3 bg-gray-800 rounded-lg border border-gray-700",
            {children}
        }
    }
}

#[component]
pub fn DataTable(headers: Vec<String>, children: Element) -> Element {
    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
            table { class: "w-full text-left text-sm",
                thead {
                    tr { class: "border-b border-gray-700 bg-gray-800/50",
                        for header in headers {
                            th { class: "px-4 py-3 font-medium text-gray-400 text-xs uppercase tracking-wider",
                                "{header}"
                            }
                        }
                    }
                }
                tbody { class: "divide-y divide-gray-700",
                    {children}
                }
            }
        }
    }
}

#[component]
pub fn EmptyState(icon: String, title: String, message: String) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center py-12 text-center",
            div { class: "text-4xl mb-3", "{icon}" }
            p { class: "text-white font-medium", "{title}" }
            p { class: "text-gray-400 text-sm mt-1", "{message}" }
        }
    }
}