use dioxus::prelude::*;
use crate::api::admin::{get_sales_data, get_top_agents, get_market_trends, SalesData, TopAgent, MarketTrend};
use crate::components::sidebar::{PageHeader, StatCard};
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn AnalyticsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_sales = token.clone();
    let token_agents = token.clone();
    let token_trends = token.clone();
    
    let sales = use_resource(move || {
        let t = token_sales.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![
                    SalesData { month: "Jan".to_string(), sales: 18, revenue: 210000.0 },
                    SalesData { month: "Feb".to_string(), sales: 22, revenue: 265000.0 },
                    SalesData { month: "Mar".to_string(), sales: 15, revenue: 180000.0 },
                    SalesData { month: "Apr".to_string(), sales: 24, revenue: 284750.0 },
                    SalesData { month: "May".to_string(), sales: 20, revenue: 245000.0 },
                    SalesData { month: "Jun".to_string(), sales: 28, revenue: 320000.0 },
                ]);
            }
            get_sales_data(&t).await
        }
    });
    
    let top_agents = use_resource(move || {
        let t = token_agents.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![
                    TopAgent { id: "1".to_string(), name: "Sarah Johnson".to_string(), sales: 12, revenue: 1450000.0, commission: 43500.0 },
                    TopAgent { id: "2".to_string(), name: "Mike Chen".to_string(), sales: 10, revenue: 1200000.0, commission: 36000.0 },
                    TopAgent { id: "3".to_string(), name: "David Park".to_string(), sales: 8, revenue: 980000.0, commission: 29400.0 },
                ]);
            }
            get_top_agents(&t).await
        }
    });
    
    let trends = use_resource(move || {
        let t = token_trends.clone();
        async move {
            if t.is_empty() {
                return Ok(vec![
                    MarketTrend { area: "Downtown".to_string(), avg_price: 650000.0, price_change: 5.2, volume: 45 },
                    MarketTrend { area: "Suburbs".to_string(), avg_price: 420000.0, price_change: 3.1, volume: 78 },
                    MarketTrend { area: "Waterfront".to_string(), avg_price: 1200000.0, price_change: 8.7, volume: 12 },
                ]);
            }
            get_market_trends(&t).await
        }
    });
    
    let sales_ref = sales.read();
    let sales_data = match sales_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };
    
    let agents_ref = top_agents.read();
    let agents_data = match agents_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };
    
    let trends_ref = trends.read();
    let trends_data = match trends_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };
    
    rsx! {
        div { class: "space-y-6",
            PageHeader { title: "Analytics".to_string(), subtitle: "Sales performance and market insights".to_string() }
            
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                StatCard { title: "Total Sales".to_string(), value: "142".to_string(), icon: "📈".to_string(), change: "+18%".to_string(), change_positive: true }
                StatCard { title: "Total Revenue".to_string(), value: "$1.5M".to_string(), icon: "💰".to_string(), change: "+12%".to_string(), change_positive: true }
                StatCard { title: "Avg Sale Price".to_string(), value: "$485K".to_string(), icon: "🏠".to_string(), change: "+4%".to_string(), change_positive: true }
            }
            
            div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                    h3 { class: "text-white font-semibold mb-4", "Monthly Sales" }
                    if let Some(data) = &sales_data {
                        div { class: "flex items-end gap-2 h-48",
                            for item in data.iter() {
                                div { class: "flex-1 flex flex-col items-center gap-2",
                                    div { class: "w-full bg-blue-500/20 rounded-t relative",
                                        style: format!("height: {}px", item.sales * 5),
                                        div { class: "absolute inset-0 bg-blue-500/40 rounded-t" }
                                    }
                                    span { class: "text-gray-400 text-xs", "{item.month}" }
                                }
                            }
                        }
                    } else {
                        div { class: "h-48 bg-gray-700/30 rounded animate-pulse" }
                    }
                }
                
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                    h3 { class: "text-white font-semibold mb-4", "Top Agents" }
                    if let Some(data) = &agents_data {
                        div { class: "space-y-3",
                            for (i, agent) in data.iter().enumerate() {
                                div { class: "flex items-center gap-3 p-3 bg-gray-900 rounded-lg",
                                    div { class: "w-8 h-8 rounded-full bg-purple-600 flex items-center justify-center text-white text-xs font-bold",
                                        "{i + 1}"
                                    }
                                    div { class: "flex-1",
                                        p { class: "text-white text-sm font-medium", "{agent.name}" }
                                        p { class: "text-gray-500 text-xs", "{agent.sales} sales • ${agent.revenue as i64} revenue" }
                                    }
                                    span { class: "text-emerald-400 text-sm font-medium", "${agent.commission as i64}" }
                                }
                            }
                        }
                    } else {
                        div { class: "space-y-3",
                            for _ in 0..3 {
                                div { class: "h-16 bg-gray-700/30 rounded animate-pulse" }
                            }
                        }
                    }
                }
            }
            
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
                h3 { class: "text-white font-semibold mb-4", "Market Trends" }
                if let Some(data) = &trends_data {
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        for trend in data.iter() {
                            div { class: "p-4 bg-gray-900 rounded-lg",
                                p { class: "text-white font-medium", "{trend.area}" }
                                p { class: "text-2xl font-bold text-white mt-1", "${trend.avg_price as i64}" }
                                div { class: "flex items-center gap-2 mt-2",
                                    span { class: "text-emerald-400 text-sm", "+{trend.price_change}%" }
                                    span { class: "text-gray-500 text-xs", "{trend.volume} sales" }
                                }
                            }
                        }
                    }
                } else {
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                        for _ in 0..3 {
                            div { class: "h-24 bg-gray-700/30 rounded animate-pulse" }
                        }
                    }
                }
            }
        }
    }
}
