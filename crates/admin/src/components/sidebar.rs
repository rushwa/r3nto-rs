use dioxus::prelude::*;
use crate::auth::use_auth_context;

#[component]
pub fn AdminSidebar() -> Element {
    let auth = use_auth_context();
    let is_admin = auth.role() == Some("admin".to_string());
    let role_label = if is_admin { "Admin" } else { "Agent" };
    
    rsx! {
        aside { class: "fixed left-0 top-0 h-full w-64 bg-gray-800 border-r border-gray-700",
            div { class: "p-6 border-b border-gray-700",
                h2 { class: "text-xl font-bold", "Rento {role_label}" }
                p { class: "text-sm text-gray-400 mt-1", "{auth.user_name()}" }
            }
            nav { class: "p-4 space-y-2",
                SidebarLink { to: crate::Route::Dashboard {}, icon: "📊", label: "Dashboard" }
                SidebarLink { to: crate::Route::Leads {}, icon: "👥", label: "Leads" }
                SidebarLink { to: crate::Route::Properties {}, icon: "🏠", label: "Properties" }
                SidebarLink { to: crate::Route::Commissions {}, icon: "💰", label: "Commissions" }
                SidebarLink { to: crate::Route::Conversion {}, icon: "🤝", label: "Conversion" }
                if is_admin {
                    div { class: "border-t border-gray-700 my-4" }
                    p { class: "px-4 text-xs text-gray-500 uppercase", "Admin Only" }
                    SidebarLink { to: crate::Route::Users {}, icon: "👤", label: "All Users" }
                    SidebarLink { to: crate::Route::Analytics {}, icon: "📈", label: "Analytics" }
                    SidebarLink { to: crate::Route::Settings {}, icon: "⚙️", label: "Settings" }
                }
            }
        }
    }
}

#[component]
fn SidebarLink(to: crate::Route, icon: String, label: String) -> Element {
    rsx! {
        Link { to: to, class: "flex items-center gap-3 px-4 py-3 rounded-lg hover:bg-gray-700",
            span { "{icon}" }
            span { "{label}" }
        }
    }
}
