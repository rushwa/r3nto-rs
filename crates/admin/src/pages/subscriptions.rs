use dioxus::prelude::*;
use crate::api::admin::{get_subscription_plans, SubscriptionPlan};
use crate::components::sidebar::PageHeader;
use crate::context::admin_auth::use_admin_auth;

#[component]
pub fn SubscriptionsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let token_for_resource = token.clone();

    let plans = use_resource(move || {
        let t = token_for_resource.clone();
        async move {                              // <-- removed -> Result<...>
            if t.is_empty() {
                return Ok(vec![
                    SubscriptionPlan {
                        id: "basic".to_string(),
                        name: "Basic".to_string(),
                        price: 9.99,
                        features: vec!["5 listings".to_string(), "Basic analytics".to_string(), "Email support".to_string()],
                        subscribers: 120,
                    },
                    SubscriptionPlan {
                        id: "pro".to_string(),
                        name: "Pro".to_string(),
                        price: 29.99,
                        features: vec!["Unlimited listings".to_string(), "Advanced analytics".to_string(), "Priority support".to_string(), "Featured listings".to_string()],
                        subscribers: 85,
                    },
                    SubscriptionPlan {
                        id: "enterprise".to_string(),
                        name: "Enterprise".to_string(),
                        price: 99.99,
                        features: vec!["Everything in Pro".to_string(), "API access".to_string(), "Dedicated manager".to_string(), "Custom branding".to_string(), "White-label option".to_string()],
                        subscribers: 12,
                    },
                ]);
            }
            get_subscription_plans(&t).await
        }
    });

    let plans_ref = plans.read();
    let plans_data: Option<Vec<SubscriptionPlan>> = match plans_ref.as_ref() {
        Some(Ok(d)) => Some(d.clone()),
        _ => None,
    };

    rsx! {
        div { class: "space-y-6",
            PageHeader { title: "Subscriptions".to_string(), subtitle: "Manage subscription plans and pricing".to_string() }

            if let Some(data) = plans_data.as_ref() {
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    for plan in data.iter() {
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                            div { class: "flex items-center justify-between mb-4",
                                h3 { class: "text-lg font-bold text-white", "{plan.name}" }
                                span { class: "bg-blue-500/10 text-blue-400 px-2.5 py-1 rounded text-xs font-medium border border-blue-500/20",
                                    "{plan.subscribers} subscribers"
                                }
                            }
                            p { class: "text-3xl font-bold text-white mb-1", "${plan.price as i64}" }
                            p { class: "text-gray-500 text-sm mb-6", "per month" }
                            ul { class: "space-y-3 mb-6",
                                for feature in plan.features.iter() {
                                    li { class: "text-gray-400 text-sm flex items-center gap-2",
                                        span { class: "text-emerald-400 text-xs", "✓" }
                                        "{feature}"
                                    }
                                }
                            }
                            div { class: "flex gap-2",
                                button { class: "flex-1 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded text-sm font-medium transition-colors", "Edit" }
                                button { class: "px-3 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded text-sm transition-colors", "Delete" }
                            }
                        }
                    }
                }
            } else if plans_ref.as_ref().is_none() {
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    for _ in 0..3 {
                        div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 animate-pulse",
                            div { class: "h-6 bg-gray-700 rounded w-1/2 mb-4" }
                            div { class: "h-8 bg-gray-700 rounded w-1/3 mb-6" }
                            for _ in 0..4 {
                                div { class: "h-4 bg-gray-700 rounded w-3/4 mb-3" }
                            }
                        }
                    }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-8 text-center",
                    p { class: "text-red-400", "Failed to load subscription plans" }
                }
            }
        }
    }
}