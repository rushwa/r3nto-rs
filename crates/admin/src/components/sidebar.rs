use dioxus::prelude::*;
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn AdminSidebar() -> Element {
    let auth = use_admin_auth();
    let auth_state = auth.read();

    // PURE ROLE-BASED ADMIN CHECK (case-insensitive):
    // 'admin' or 'superuser' -> Admin dashboard
    // 'agent' -> Agent dashboard  
    // 'property_owner' -> Property owner dashboard
    let user_role = auth_state.user.as_ref()
        .map(|u| u.role.to_lowercase())
        .unwrap_or_default();
    
    let is_admin = user_role == "admin" || user_role == "superuser";
    let is_agent = user_role == "agent";
    let is_property_owner = user_role == "property_owner";

    let role_label = if is_admin { 
        "Admin" 
    } else if is_agent { 
        "Agent" 
    } else if is_property_owner { 
        "Property Owner" 
    } else { 
        "User" 
    };

    let user_name = auth_state.user.as_ref()
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "User".to_string());

    rsx! {
        aside { class: "fixed left-0 top-0 h-full w-64 bg-gray-800 border-r border-gray-700 overflow-y-auto",
            div { class: "p-6 border-b border-gray-700",
                h2 { class: "text-xl font-bold text-white", "Rento {role_label}" }
                p { class: "text-sm text-gray-400 mt-1", "{user_name}" }
                p { class: "text-xs text-gray-500 mt-1", "Role: {user_role}" }
            }
            nav { class: "p-4 space-y-2",
                // Common links for everyone
                SidebarLink { to: crate::AdminRoute::DashboardPage, icon: "📊", label: "Dashboard" }
                SidebarLink { to: crate::AdminRoute::PropertiesPage, icon: "🏠", label: "Properties" }
                SidebarLink { to: crate::AdminRoute::CommissionsPage, icon: "💰", label: "Commissions" }

                // Agent-specific links
                if is_agent || is_admin {
                    SidebarLink { to: crate::AdminRoute::LeadsPage, icon: "👥", label: "Leads" }
                    SidebarLink { to: crate::AdminRoute::ConversionPage, icon: "🤝", label: "Conversion" }
                }

                // Admin-only links
                if is_admin {
                    div { class: "border-t border-gray-700 my-4" }
                    p { class: "px-4 text-xs text-gray-500 uppercase font-semibold", "Admin Only" }
                    SidebarLink { to: crate::AdminRoute::UsersPage, icon: "👤", label: "Users" }
                    SidebarLink { to: crate::AdminRoute::AgentsPage, icon: "🏢", label: "Agents" }
                    SidebarLink { to: crate::AdminRoute::PropertyOwnersPage, icon: "🏘️", label: "Property Owners" }
                    SidebarLink { to: crate::AdminRoute::SubscriptionsPage, icon: "💳", label: "Subscriptions" }
                    SidebarLink { to: crate::AdminRoute::InquiriesPage, icon: "📨", label: "Inquiries" }
                    SidebarLink { to: crate::AdminRoute::AnalyticsPage, icon: "📈", label: "Analytics" }
                    SidebarLink { to: crate::AdminRoute::SettingsPage, icon: "⚙️", label: "Settings" }
                }
            }
        }
    }
}

#[component]
fn SidebarLink(to: crate::AdminRoute, icon: String, label: String) -> Element {
    rsx! {
        Link {
            to: to,
            class: "flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-gray-700 text-white transition-colors",
            active_class: "bg-gray-700",
            span { "{icon}" }
            span { "{label}" }
        }
    }
}

#[component]
pub fn AdminHeader() -> Element {
    rsx! {
        header { class: "fixed top-0 right-0 left-64 h-16 bg-gray-900/80 backdrop-blur-md border-b border-gray-800 flex items-center justify-between px-8 z-10",
            div { class: "text-white font-semibold", "Rento Portal" }
        }
    }
}

#[component]
pub fn PageHeader(title: String, subtitle: String) -> Element {
    rsx! {
        div { class: "mb-6",
            h1 { class: "text-2xl font-bold text-white", "{title}" }
            p { class: "text-gray-400 mt-1", "{subtitle}" }
        }
    }
}

#[component]
pub fn StatCard(title: String, value: String, icon: String, change: String, change_positive: bool) -> Element {
    let change_color = if change_positive { "text-emerald-400" } else { "text-red-400" };
    rsx! {
        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
            div { class: "flex items-center justify-between mb-3",
                span { class: "text-2xl", "{icon}" }
                span { class: "{change_color} text-sm font-medium", "{change}" }
            }
            p { class: "text-2xl font-bold text-white mb-1", "{value}" }
            p { class: "text-gray-500 text-sm", "{title}" }
        }
    }
}

#[component]
pub fn StatusBadge(status: String) -> Element {
    let color = match status.as_str() {
        "pending" | "new" => "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
        "active" | "verified" | "approved" | "converted" | "closed" => "bg-green-500/10 text-green-400 border-green-500/20",
        "inactive" | "rejected" | "unverified" => "bg-red-500/10 text-red-400 border-red-500/20",
        _ => "bg-gray-500/10 text-gray-400 border-gray-500/20",
    };
    rsx! {
        span { class: "px-2 py-1 rounded-full text-xs border {color}", "{status}" }
    }
}

#[component]
pub fn EmptyState(icon: String, title: String, message: String) -> Element {
    rsx! {
        div { class: "flex flex-col items-center justify-center p-12 text-center",
            div { class: "text-4xl mb-4", "{icon}" }
            h3 { class: "text-lg font-medium text-white mb-2", "{title}" }
            p { class: "text-gray-400", "{message}" }
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
        div { class: "overflow-x-auto rounded-lg border border-gray-700",
            table { class: "min-w-full divide-y divide-gray-700",
                thead { class: "bg-gray-800",
                    tr {
                        for header in headers.iter() {
                            th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase tracking-wider", "{header}" }
                        }
                    }
                }
                tbody { class: "divide-y divide-gray-700 bg-gray-800/50",
                    {children}
                }
            }
        }
    }
}
