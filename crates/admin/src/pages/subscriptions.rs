use dioxus::prelude::*;
use crate::components::sidebar::PageHeader;
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{
    get_subscription_plans, get_subscriptions_overview, subscribe_property_with_payment,
    SubscriptionPlan,
};

#[component]
pub fn SubscriptionsPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut plans = use_signal(|| Vec::<SubscriptionPlan>::new());
    let mut properties = use_signal(|| Vec::<serde_json::Value>::new());
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);

    // Modal state
    let mut show_subscribe_modal = use_signal(|| false);
    let mut selected_property = use_signal(|| Option::<serde_json::Value>::None);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            let _ = get_subscription_plans(&t).await.map(|p| plans.set(p));
            match get_subscriptions_overview(&t).await {
                Ok(data) => properties.set(data),
                Err(e) => {
                    message.set(Some(format!("Failed to load: {}", e)));
                    is_error.set(true);
                }
            }
            loading.set(false);
        });
    });

    // Categorize properties
    let active_props: Vec<_> = properties.read().iter()
        .filter(|p| p.get("sub_status").and_then(|s| s.as_str()) == Some("active"))
        .cloned().collect();
    let expiring_props: Vec<_> = properties.read().iter()
        .filter(|p| p.get("sub_status").and_then(|s| s.as_str()) == Some("expiring"))
        .cloned().collect();
    let unsubscribed_props: Vec<_> = properties.read().iter()
        .filter(|p| {
            let s = p.get("sub_status").and_then(|v| v.as_str()).unwrap_or("none");
            s == "none" || s == "expired"
        })
        .cloned().collect();

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading subscriptions..." }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Property Subscriptions".to_string(),
                subtitle: "Manage subscriptions for each of your properties".to_string(),
            }

            // Summary Stats
            div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
                div { class: "bg-green-900/20 border border-green-500/30 rounded-lg p-4",
                    p { class: "text-green-400 text-sm", "✅ Active Subscriptions" }
                    p { class: "text-3xl font-bold text-white mt-1", "{active_props.len()}" }
                }
                div { class: "bg-yellow-900/20 border border-yellow-500/30 rounded-lg p-4",
                    p { class: "text-yellow-400 text-sm", "⚠️ Expiring Soon" }
                    p { class: "text-3xl font-bold text-white mt-1", "{expiring_props.len()}" }
                }
                div { class: "bg-gray-800 border border-gray-700 rounded-lg p-4",
                    p { class: "text-gray-400 text-sm", "📋 Not Subscribed" }
                    p { class: "text-3xl font-bold text-white mt-1", "{unsubscribed_props.len()}" }
                }
            }

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

            if properties.read().is_empty() {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 p-12 text-center",
                    span { class: "text-6xl", "🏘️" }
                    h3 { class: "text-xl font-bold text-white mt-4 mb-2", "No Properties Yet" }
                    p { class: "text-gray-400", "Create a property first, then subscribe it to a plan." }
                }
            } else {
                if !active_props.is_empty() {
                    SubscriptionSection {
                        title: "Active Subscriptions".to_string(),
                        icon: "✅".to_string(),
                        color_class: "border-green-500/30".to_string(),
                        properties: active_props,
                        on_action_click: {
                            let mut show = show_subscribe_modal.clone();
                            let mut selected = selected_property.clone();
                            move |prop: serde_json::Value| {
                                selected.set(Some(prop));
                                show.set(true);
                            }
                        },
                    }
                }
                if !expiring_props.is_empty() {
                    SubscriptionSection {
                        title: "Expiring Soon (within 7 days)".to_string(),
                        icon: "⚠️".to_string(),
                        color_class: "border-yellow-500/30".to_string(),
                        properties: expiring_props,
                        on_action_click: {
                            let mut show = show_subscribe_modal.clone();
                            let mut selected = selected_property.clone();
                            move |prop: serde_json::Value| {
                                selected.set(Some(prop));
                                show.set(true);
                            }
                        },
                    }
                }
                if !unsubscribed_props.is_empty() {
                    SubscriptionSection {
                        title: "Not Subscribed".to_string(),
                        icon: "📋".to_string(),
                        color_class: "border-gray-700".to_string(),
                        properties: unsubscribed_props,
                        on_action_click: {
                            let mut show = show_subscribe_modal.clone();
                            let mut selected = selected_property.clone();
                            move |prop: serde_json::Value| {
                                selected.set(Some(prop));
                                show.set(true);
                            }
                        },
                    }
                }
            }

            // Subscribe Modal
            if *show_subscribe_modal.read() {
                SubscribeModal {
                    property: selected_property.read().clone(),
                    plans: plans.read().clone(),
                    token: token.clone(),
                    on_close: {
                        let mut show = show_subscribe_modal.clone();
                        let mut selected = selected_property.clone();
                        move |_| {
                            show.set(false);
                            selected.set(None);
                        }
                    },
                    on_success: {
                        let mut show = show_subscribe_modal.clone();
                        let mut selected = selected_property.clone();
                        let mut msg_signal = message.clone();
                        let mut err_signal = is_error.clone();
                        move |msg: String| {
                            show.set(false);
                            selected.set(None);
                            msg_signal.set(Some(msg));
                            err_signal.set(false);
                        }
                    },
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Subscription Section
// ───────────────────────────────────────────
#[component]
fn SubscriptionSection(
    title: String,
    icon: String,
    color_class: String,
    properties: Vec<serde_json::Value>,
    on_action_click: EventHandler<serde_json::Value>,
) -> Element {
    rsx! {
        div { class: "bg-gray-800 rounded-lg border {color_class} p-6",
            div { class: "flex items-center justify-between mb-4",
                h2 { class: "text-xl font-bold text-white",
                    span { class: "mr-2", "{icon}" }
                    "{title}"
                    span { class: "ml-2 text-sm font-normal text-gray-400", "({properties.len()})" }
                }
            }
            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                for prop in properties.iter() {
                    PropertySubCard {
                        property: prop.clone(),
                        on_action: move |p: serde_json::Value| on_action_click.call(p),
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────────
// Property Subscription Card
// ───────────────────────────────────────────
#[component]
fn PropertySubCard(
    property: serde_json::Value,
    on_action: EventHandler<serde_json::Value>,
) -> Element {
    let title = property.get("title").and_then(|v| v.as_str()).unwrap_or("—");
    let location = property.get("location").and_then(|v| v.as_str()).unwrap_or("—");
    let price = property.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let plan_name = property.get("plan_name").and_then(|v| v.as_str()).unwrap_or("No Plan");
    let plan_price = property.get("plan_price").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let sub_status = property.get("sub_status").and_then(|v| v.as_str()).unwrap_or("none");
    let days_remaining = property.get("days_remaining").and_then(|v| v.as_i64()).unwrap_or(0);
    let end_date = property.get("end_date").and_then(|v| v.as_str());

    let (status_badge, status_text, button_text, button_color) = match sub_status {
        "active" => (
            "bg-green-500/10 text-green-400 border-green-500/20",
            format!("✅ Active • {} days left", days_remaining),
            "Change Plan",
            "bg-gray-700 hover:bg-gray-600",
        ),
        "expiring" => (
            "bg-yellow-500/10 text-yellow-400 border-yellow-500/20",
            format!("⚠️ Expires in {} days", days_remaining),
            "Renew Now",
            "bg-yellow-600 hover:bg-yellow-500",
        ),
        "expired" => (
            "bg-red-500/10 text-red-400 border-red-500/20",
            "❌ Expired".to_string(),
            "Resubscribe",
            "bg-blue-600 hover:bg-blue-500",
        ),
        _ => (
            "bg-gray-500/10 text-gray-400 border-gray-500/20",
            "📋 Not subscribed".to_string(),
            "Subscribe",
            "bg-blue-600 hover:bg-blue-500",
        ),
    };

    let prop_for_click = property.clone();

    rsx! {
        div { class: "bg-gray-900 rounded-lg border border-gray-700 p-4 flex flex-col",
            div { class: "flex-1",
                h4 { class: "text-white font-semibold mb-1", "{title}" }
                p { class: "text-gray-400 text-sm mb-3", "{location}" }
                p { class: "text-blue-400 font-bold mb-3", "KES {price as i32}" }

                div { class: "bg-gray-800 rounded p-3 mb-3",
                    p { class: "text-gray-400 text-xs mb-1", "Current Plan" }
                    p { class: "text-white font-medium", "{plan_name}" }
                    if plan_price > 0.0 {
                        p { class: "text-gray-500 text-xs", "KES {plan_price as i32}/period" }
                    }
                }

                span { class: "inline-block px-2 py-1 rounded-full text-xs border {status_badge}",
                    "{status_text}"
                }
                if let Some(date) = end_date {
                    if date.len() > 10 {
                        p { class: "text-gray-500 text-xs mt-2", "Ends: {&date[..10]}" }
                    }
                }
            }

            button {
                class: "w-full mt-3 py-2 {button_color} text-white rounded-lg text-sm font-medium transition-colors",
                onclick: move |_| on_action.call(prop_for_click.clone()),
                "{button_text}"
            }
        }
    }
}

// ───────────────────────────────────────────
// Subscribe Modal (Plan Selection + M-Pesa Phone)
// ───────────────────────────────────────────
// ───────────────────────────────────────────
// Subscribe Modal (Plan Selection + M-Pesa Phone)
// ───────────────────────────────────────────
#[component]
fn SubscribeModal(
    property: Option<serde_json::Value>,
    plans: Vec<SubscriptionPlan>,
    token: String,
    on_close: EventHandler<()>,
    on_success: EventHandler<String>,
) -> Element {
    let mut step = use_signal(|| 1u8);
    let mut selected_plan_id = use_signal(|| Option::<String>::None);
    let mut phone_number = use_signal(|| String::new());
    let mut loading = use_signal(|| false);
    let mut error_message = use_signal(|| Option::<String>::None);

    let prop_title = property.as_ref()
        .and_then(|p| p.get("title")).and_then(|v| v.as_str()).unwrap_or("Property");
    let prop_id = property.as_ref()
        .and_then(|p| p.get("id")).and_then(|v| v.as_str()).unwrap_or("").to_string();

    let is_valid_phone = {
        let phone = phone_number.read().clone();
        let digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
        (digits.starts_with("254") && digits.len() == 12)
            || (digits.starts_with("0") && digits.len() == 10)
            || (digits.starts_with("7") && digits.len() == 9)
    };

    let selected_plan = plans.iter().find(|p| selected_plan_id.read().as_ref() == Some(&p.id)).cloned();
    let plan_price = selected_plan.as_ref().map(|p| p.price).unwrap_or(0.0);
    let plan_name = selected_plan.as_ref().map(|p| p.name.clone()).unwrap_or_default();

    // ✅ FIX: Clone values before moving into closures
    let token_for_submit = token.clone();
    let prop_id_for_submit = prop_id.clone();
    let prop_title_for_msg = prop_title.to_string();
    let plan_name_for_success = plan_name.clone(); // ✅ Clone for use in spawn

    let handle_subscribe = move |_| {
        let plan_id = match selected_plan_id.read().clone() {
            Some(id) => id,
            None => {
                error_message.set(Some("Please select a plan".to_string()));
                return;
            }
        };
        let phone = phone_number.read().clone();
        if !is_valid_phone {
            error_message.set(Some("Invalid phone number".to_string()));
            return;
        }

        loading.set(true);
        error_message.set(None);

        let t = token_for_submit.clone();
        let pid = prop_id_for_submit.clone();
        let title_clone = prop_title_for_msg.clone();
        let success_handler = on_success.clone();
        let pname = plan_name_for_success.clone(); // ✅ Use the cloned value

        spawn(async move {
            match subscribe_property_with_payment(&t, &plan_id, &pid, &phone).await {
                Ok(result) => {
                    let receipt = result.get("receipt_number")
                        .and_then(|v| v.as_str()).unwrap_or("N/A");
                    let commission = result.get("agent_commission")
                        .and_then(|v| v.as_f64()).unwrap_or(0.0);
                    success_handler.call(format!(
                        "✅ '{}' subscribed to {}!\nReceipt: {}\nAgent commission: KES {:.2}",
                        title_clone, pname, receipt, commission
                    ));
                }
                Err(e) => {
                    error_message.set(Some(format!("Failed: {}", e)));
                    loading.set(false);
                }
            }
        });
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4",
            div { class: "bg-gray-800 rounded-lg border border-gray-700 p-6 max-w-3xl w-full max-h-[90vh] overflow-y-auto",
                div { class: "flex items-center justify-between mb-4",
                    div {
                        h3 { class: "text-xl font-bold text-white",
                            if *step.read() == 1 { "Choose a Plan" } else { "Complete Payment" }
                        }
                        p { class: "text-gray-400 text-sm", "For: {prop_title}" }
                    }
                    button {
                        class: "text-gray-400 hover:text-white text-2xl leading-none",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                if *step.read() == 1 {
                    if plans.is_empty() {
                        div { class: "text-center py-8",
                            p { class: "text-gray-400", "No subscription plans available." }
                        }
                    } else {
                        div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mb-4",
                            for plan in plans.iter() {
                                PlanOption {
                                    plan: plan.clone(),
                                    is_selected: selected_plan_id.read().as_ref() == Some(&plan.id),
                                    on_select: {
                                        let plan_id = plan.id.clone();
                                        move |_| selected_plan_id.set(Some(plan_id.clone()))
                                    },
                                }
                            }
                        }
                    }

                    if let Some(err) = error_message.read().as_ref() {
                        div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3 mb-4",
                            p { class: "text-red-400 text-sm", "❌ {err}" }
                        }
                    }

                    div { class: "flex gap-2 pt-4 border-t border-gray-700",
                        button {
                            class: "flex-1 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium",
                            onclick: move |_| on_close.call(()),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 py-2.5 bg-blue-600 hover:bg-blue-500 text-white rounded-lg font-medium disabled:opacity-50",
                            disabled: selected_plan_id.read().is_none() || plans.is_empty(),
                            onclick: move |_| {
                                if selected_plan_id.read().is_some() {
                                    step.set(2);
                                    error_message.set(None);
                                }
                            },
                            "Continue →"
                        }
                    }
                } else {
                    div { class: "bg-blue-900/20 border border-blue-500/30 rounded-lg p-4 mb-4",
                        div { class: "flex items-center justify-between",
                            div {
                                p { class: "text-blue-400 font-semibold", "Plan: {plan_name}" }
                                p { class: "text-gray-300 text-sm", "Property: {prop_title}" }
                            }
                            p { class: "text-2xl font-bold text-white", "KES {plan_price as i32}" }
                        }
                    }

                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm font-medium text-gray-400 mb-1", "M-Pesa Phone Number" }
                            input {
                                class: "w-full px-4 py-2.5 bg-gray-900 border border-gray-700 rounded-lg text-white",
                                r#type: "tel",
                                placeholder: "254712345678",
                                value: "{phone_number}",
                                oninput: move |evt| {
                                    phone_number.set(evt.value());
                                    error_message.set(None);
                                },
                            }
                            p { class: "text-gray-500 text-xs mt-1", "An STK push will be sent to this number for KES {plan_price as i32}" }
                        }

                        if let Some(err) = error_message.read().as_ref() {
                            div { class: "bg-red-900/20 border border-red-500/30 rounded-lg p-3",
                                p { class: "text-red-400 text-sm", "❌ {err}" }
                            }
                        }

                        div { class: "flex gap-2 pt-4 border-t border-gray-700",
                            button {
                                class: "flex-1 py-2.5 bg-gray-700 hover:bg-gray-600 text-white rounded-lg font-medium",
                                onclick: move |_| {
                                    step.set(1);
                                    error_message.set(None);
                                },
                                "← Back"
                            }
                            button {
                                class: "flex-1 py-2.5 bg-green-600 hover:bg-green-500 text-white rounded-lg font-medium disabled:opacity-50",
                                disabled: *loading.read() || !is_valid_phone,
                                onclick: handle_subscribe,
                                if *loading.read() { "Processing..." } else { "Pay & Subscribe" }
                            }
                        }
                    }
                }
            }
        }
    }
}
// ───────────────────────────────────────────
// Plan Option (selectable card)
// ───────────────────────────────────────────
#[component]
fn PlanOption(
    plan: SubscriptionPlan,
    is_selected: bool,
    on_select: EventHandler<()>,
) -> Element {
    let border_class = if is_selected {
        "p-4 bg-blue-600/20 border-2 border-blue-500 rounded-lg text-left cursor-pointer"
    } else {
        "p-4 bg-gray-900 border border-gray-700 rounded-lg text-left cursor-pointer hover:border-gray-600"
    };

    // ✅ FIX: Extract duration text BEFORE rsx! to avoid format string issues
    let duration_text = plan.features.first().cloned().unwrap_or_else(|| "month".to_string());
    let plan_name = plan.name.clone();
    let plan_price = plan.price;
    let features = plan.features.clone();

    rsx! {
        button {
            class: "{border_class}",
            onclick: move |_| on_select.call(()),
            h4 { class: "text-white font-bold mb-1", "{plan_name}" }
            p { class: "text-2xl font-bold text-blue-400 mb-2",
                "KES {plan_price as i32}"
                span { class: "text-sm text-gray-400 font-normal", "/{duration_text}" }
            }
            ul { class: "space-y-1",
                for feature in features.iter() {
                    li { class: "text-gray-300 text-xs flex items-center gap-1",
                        span { class: "text-green-400", "✓" }
                        span { "{feature}" }
                    }
                }
            }
        }
    }
}