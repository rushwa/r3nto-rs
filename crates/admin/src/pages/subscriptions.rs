use dioxus::prelude::*;
use crate::components::sidebar::PageHeader;
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_subscription_plans, get_my_subscriptions, subscribe_property, SubscriptionPlan};

#[component]
pub fn SubscriptionsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let user_role = auth.read().user.as_ref().map(|u| u.role.clone()).unwrap_or_default();

    let mut plans = use_signal(|| Vec::<SubscriptionPlan>::new());
    let mut my_subscriptions = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    // ✅ FIX: Clone token before use_effect to avoid move error
    let token_for_effect = token.clone();
    let user_role_for_effect = user_role.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        let role = user_role_for_effect.clone();
        spawn(async move {
            match get_subscription_plans(&t).await {
                Ok(data) => plans.set(data),
                Err(e) => {
                    message.set(Some(format!("Failed to load plans: {}", e)));
                    is_error.set(true);
                }
            }
            if role.to_uppercase() == "PROPERTY_OWNER" {
                if let Ok(subs) = get_my_subscriptions(&t).await {
                    my_subscriptions.set(subs);
                }
            }
            loading.set(false);
        });
    });

    let is_property_owner = user_role.to_uppercase() == "PROPERTY_OWNER";

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Subscription Plans".to_string(),
                subtitle: "Choose a plan to boost your property visibility".to_string(),
            }

            if *loading.read() {
                div { class: "flex items-center justify-center h-64",
                    div { class: "text-white", "Loading plans..." }
                }
            } else {
                if let Some(msg) = message.read().as_ref() {
                    div {
                        class: if *is_error.read() {
                            "bg-red-900/20 border border-red-500/30 rounded-lg p-3"
                        } else {
                            "bg-green-900/20 border border-green-500/30 rounded-lg p-3"
                        },
                        p {
                            class: if *is_error.read() { "text-red-400" } else { "text-green-400" },
                            "{msg}"
                        }
                    }
                }

                if is_property_owner && !my_subscriptions.read().is_empty() {
                    div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6",
                        h2 { class: "text-xl font-bold text-white mb-4", "My Active Subscriptions" }
                        div { class: "space-y-3",
                            for sub in my_subscriptions.read().iter() {
                                SubscriptionCard { subscription: sub.clone() }
                            }
                        }
                    }
                }

                div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                    for plan in plans.read().iter() {
                        PlanCard {
                            plan: plan.clone(),
                            token: token.clone(),
                            is_property_owner,
                            on_subscribed: {
                                let mut message = message.clone();
                                let mut is_error = is_error.clone();
                                move |msg: String| {
                                    message.set(Some(msg));
                                    is_error.set(false);
                                }
                            },
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PlanCard(
    plan: SubscriptionPlan,
    token: String,
    is_property_owner: bool,
    on_subscribed: EventHandler<String>,
) -> Element {
    let is_popular = plan.name.to_lowercase().contains("professional") || plan.name.to_lowercase().contains("premium");

    // ✅ FIX: Extract duration text before using in format string
    let duration_text = plan.features.first().cloned().unwrap_or_else(|| "month".to_string());

    rsx! {
        div {
            class: if is_popular {
                "bg-gradient-to-b from-blue-900/40 to-gray-800 rounded-lg border-2 border-blue-500 p-6 relative"
            } else {
                "bg-gray-800 rounded-lg border border-gray-700 p-6"
            },
            if is_popular {
                span { class: "absolute -top-3 left-1/2 -translate-x-1/2 bg-blue-600 text-white text-xs px-3 py-1 rounded-full",
                    "MOST POPULAR"
                }
            }
            h3 { class: "text-xl font-bold text-white mb-2", "{plan.name}" }
            p { class: "text-3xl font-bold text-blue-400 mb-4",
                "KES {plan.price as i32}"
                span { class: "text-sm text-gray-400 font-normal", "/{duration_text}" }
            }
            ul { class: "space-y-2 mb-6",
                for feature in plan.features.iter() {
                    li { class: "flex items-center gap-2 text-gray-300 text-sm",
                        span { class: "text-green-400", "✓" }
                        span { "{feature}" }
                    }
                }
            }
            button {
                class: "w-full py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium transition-colors disabled:opacity-50",
                disabled: !is_property_owner,
                onclick: move |_| {
                    if !is_property_owner {
                        return;
                    }
                    on_subscribed.call(format!("Subscribe flow for {} coming soon", plan.name));
                },
                if is_property_owner { "Subscribe Now" } else { "Convert to Property Owner first" }
            }
        }
    }
}

#[component]
fn SubscriptionCard(subscription: serde_json::Value) -> Element {
    let property_title = subscription.get("property_title").and_then(|v| v.as_str()).unwrap_or("—");
    let plan_name = subscription.get("plan_name").and_then(|v| v.as_str()).unwrap_or("—");
    let status = subscription.get("status").and_then(|v| v.as_str()).unwrap_or("—");
    let end_date = subscription.get("end_date").and_then(|v| v.as_str()).unwrap_or("—");
    let amount_paid = subscription.get("amount_paid").and_then(|v| v.as_f64()).unwrap_or(0.0);

    let status_color = match status {
        "active" => "text-green-400",
        "expired" => "text-red-400",
        _ => "text-gray-400",
    };

    rsx! {
        div { class: "bg-gray-900 rounded-lg border border-gray-700 p-4 flex items-center justify-between",
            div {
                h4 { class: "text-white font-semibold", "{property_title}" }
                p { class: "text-gray-400 text-sm", "{plan_name} • KES {amount_paid as i32}" }
            }
            div { class: "text-right",
                p { class: "{status_color} font-medium capitalize", "{status}" }
                p { class: "text-gray-500 text-xs", "Expires: {end_date}" }
            }
        }
    }
}