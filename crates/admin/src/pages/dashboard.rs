use dioxus::prelude::*;

use crate::api::admin::{get_stats, StatsData, ActivityItem, SalesData};
use crate::components::sidebar::{StatCard, PageHeader};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn DashboardPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let stats = use_resource(move || {
        let t = token.clone();
        async move {
            if t.is_empty() {
                return Ok(StatsData {
                    total_users: 1247,
                    total_agents: 86,
                    total_properties: 342,
                    total_revenue: 284750.0,
                    active_listings: 198,
                    sold_this_month: 24,
                    avg_price: 425000.0,
                    pending_commissions: 12,
                    user_growth: "+12%".to_string(),
                    revenue_growth: "+8%".to_string(),
                });
            }
            get_stats(&t).await
        }
    });

    let sales = use_signal(|| vec![
        SalesData { month: "Jan".to_string(), sales: 18, revenue: 210000.0 },
        SalesData { month: "Feb".to_string(), sales: 22, revenue: 265000.0 },
        SalesData { month: "Mar".to_string(), sales: 15, revenue: 180000.0 },
        SalesData { month: "Apr".to_string(), sales: 24, revenue: 284750.0 },
        SalesData { month: "May".to_string(), sales: 20, revenue: 245000.0 },
        SalesData { month: "Jun".to_string(), sales: 28, revenue: 320000.0 },
    ]);

    let activities = use_signal(|| vec![
        ActivityItem { id: "1".to_string(), action: "New property listed".to_string(), user: "Sarah Johnson".to_string(), time: "2 min ago".parse().unwrap() },
        ActivityItem { id: "2".to_string(), action: "Inquiry received for 123 Main St".to_string(), user: "Mike Chen".to_string(), time: "15 min ago".parse().unwrap() },
        ActivityItem { id: "3".to_string(), action: "Property marked as sold".to_string(), user: "Agent David".to_string(), time: "1 hour ago".parse().unwrap() },
        ActivityItem { id: "4".to_string(), action: "Commission payment processed".to_string(), user: "System".to_string(), time: "3 hours ago".parse().unwrap() },
        ActivityItem { id: "5".to_string(), action: "New agent registration".to_string(), user: "Lisa Park".to_string(), time: "5 hours ago".parse().unwrap() },
    ]);

    let stats_ref = stats.read();
    let stats_data = match stats_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader { title: "Dashboard".to_string(), subtitle: "Real estate platform overview".to_string() }

            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4",
                if let Some(d) = &stats_data {
                    StatCard { title: "Active Listings".to_string(), value: d.active_listings.to_string(), icon: "🏠".to_string(), change: "+5%".to_string(), change_positive: true }
                    StatCard { title: "Sold This Month".to_string(), value: d.sold_this_month.to_string(), icon: "📈".to_string(), change: "+12%".to_string(), change_positive: true }
                    StatCard { title: "Avg Price".to_string(), value: format!("${:.0}", d.avg_price as i64), icon: "💰".to_string(), change: "+3%".to_string(), change_positive: true }
                    StatCard { title: "Revenue".to_string(), value: format!("${:.0}", d.total_revenue as i64), icon: "💰".to_string(), change: d.revenue_growth.clone(), change_positive: true }
                } else {
                    for _ in 0..4 {
                        div { class: "bg-gray-800 rounded-lg p-5 border border-gray-700 animate-pulse",
                            div { class: "h-3 bg-gray-700 rounded w-1/3 mb-3" }
                            div { class: "h-8 bg-gray-700 rounded w-2/3 mb-2" }
                            div { class: "h-3 bg-gray-700 rounded w-1/4" }
                        }
                    }
                }
            }

            div { class: "grid grid-cols-1 lg:grid-cols-3 gap-6",
                div { class: "lg:col-span-2 bg-gray-800 rounded-lg border border-gray-700 p-5",
                    h3 { class: "text-white font-semibold mb-4", "Sales Performance" }
                    div { class: "flex items-end gap-2 h-48",
                        for item in sales.read().iter() {
                            div { class: "flex-1 flex flex-col items-center gap-2",
                                div { class: "w-full bg-blue-500/20 rounded-t relative",
                                    style: "height: {item.sales * 4}px",
                                    div { class: "absolute inset-0 bg-blue-500/40 rounded-t" }
                                }
                                span { class: "text-gray-400 text-xs", "{item.month}" }
                            }
                        }
                    }
                }

                div { class: "bg-gray-800 rounded-lg border border-gray-700",
                    div { class: "px-5 py-4 border-b border-gray-700",
                        h3 { class: "text-white font-semibold", "Recent Activity" }
                    }
                    div { class: "divide-y divide-gray-700",
                        for activity in activities.read().iter() {
                            div { class: "px-5 py-3",
                                p { class: "text-white text-sm", "{activity.action}" }
                                p { class: "text-gray-500 text-xs mt-0.5", "{activity.user} • {activity.time}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
