use dioxus::prelude::*;
use crate::components::sidebar::PageHeader;
use crate::context::admin_auth::use_admin_auth;
use crate::api::admin::{get_my_bonus_progress, claim_bonus};

#[component]
pub fn AgentBonusesPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();

    let mut progress = use_signal(|| Option::<serde_json::Value>::None);
    let mut loading = use_signal(|| true);
    let mut message = use_signal(|| Option::<String>::None);
    let mut is_error = use_signal(|| false);
    let mut claiming = use_signal(|| false);

    let token_for_effect = token.clone();

    use_effect(move || {
        let t = token_for_effect.clone();
        spawn(async move {
            if let Ok(data) = get_my_bonus_progress(&t).await {
                progress.set(Some(data));
            }
            loading.set(false);
        });
    });

    let handle_claim = {
        let token = token.clone();
        let mut progress_signal = progress.clone();
        let mut message_signal = message.clone();
        let mut is_error_signal = is_error.clone();
        let mut claiming_signal = claiming.clone();
        move |_| {
            claiming_signal.set(true);
            let t = token.clone();
            spawn(async move {
                match claim_bonus(&t).await {
                    Ok(result) => {
                        let msg = result.get("message").and_then(|v| v.as_str()).unwrap_or("Checked");
                        message_signal.set(Some(format!("✅ {}", msg)));
                        is_error_signal.set(false);
                        // Refresh progress
                        if let Ok(data) = get_my_bonus_progress(&t).await {
                            progress_signal.set(Some(data));
                        }
                    }
                    Err(e) => {
                        message_signal.set(Some(format!("Failed: {}", e)));
                        is_error_signal.set(true);
                    }
                }
                claiming_signal.set(false);
            });
        }
    };

    if *loading.read() {
        return rsx! {
            div { class: "flex items-center justify-center h-96",
                div { class: "text-white text-lg", "Loading bonus progress..." }
            }
        };
    }

    let prog = progress.read().clone().unwrap_or(serde_json::json!({}));
    let current_referrals = prog.get("current_referrals").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_earned = prog.get("total_bonuses_earned").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let tiers_claimed = prog.get("tiers_claimed").and_then(|v| v.as_i64()).unwrap_or(0);
    let total_tiers = prog.get("total_tiers").and_then(|v| v.as_i64()).unwrap_or(0);
    let tier_progress = prog.get("tier_progress").and_then(|v| v.as_array()).cloned().unwrap_or_default();
    let next_tier = prog.get("next_tier").cloned();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Referral Bonus Tiers".to_string(),
                subtitle: "Earn bonuses as you bring more property owners to Rento".to_string(),
            }

            // Stats Row
            div { class: "grid grid-cols-1 md:grid-cols-4 gap-4",
                div { class: "bg-gradient-to-br from-yellow-900/40 to-gray-800 rounded-lg border border-yellow-500/30 p-5",
                    p { class: "text-yellow-400 text-sm", "🔗 Current Referrals" }
                    p { class: "text-3xl font-bold text-white mt-2", "{current_referrals}" }
                }
                div { class: "bg-gradient-to-br from-green-900/40 to-gray-800 rounded-lg border border-green-500/30 p-5",
                    p { class: "text-green-400 text-sm", "💰 Bonuses Earned" }
                    p { class: "text-3xl font-bold text-white mt-2", "KES {total_earned as i32}" }
                }
                div { class: "bg-gradient-to-br from-blue-900/40 to-gray-800 rounded-lg border border-blue-500/30 p-5",
                    p { class: "text-blue-400 text-sm", "🏆 Tiers Unlocked" }
                    p { class: "text-3xl font-bold text-white mt-2", "{tiers_claimed}/{total_tiers}" }
                }
                div { class: "bg-gradient-to-br from-purple-900/40 to-gray-800 rounded-lg border border-purple-500/30 p-5",
                    p { class: "text-purple-400 text-sm", "🎯 Next Milestone" }
                    p { class: "text-3xl font-bold text-white mt-2",
                        {next_tier.as_ref().and_then(|t| t.get("min_referrals")).and_then(|v| v.as_i64()).unwrap_or(0).to_string()}
                    }
                    p { class: "text-gray-400 text-xs mt-1",
                        {next_tier.as_ref().and_then(|t| t.get("tier_name")).and_then(|v| v.as_str()).unwrap_or("All claimed!").to_string()}
                    }
                }
            }

            if let Some(msg) = message.read().as_ref() {
                div {
                    class: if *is_error.read() { "bg-red-900/20 border border-red-500/30 rounded-lg p-3" } else { "bg-green-900/20 border border-green-500/30 rounded-lg p-3" },
                    p { class: if *is_error.read() { "text-red-400" } else { "text-green-400" }, "{msg}" }
                }
            }

            // Claim Button
            div { class: "flex justify-end",
                button {
                    class: "px-6 py-2.5 bg-yellow-600 hover:bg-yellow-500 text-white rounded-lg font-medium disabled:opacity-50",
                    disabled: *claiming.read(),
                    onclick: handle_claim,
                    if *claiming.read() { "Checking..." } else { "🏆 Check & Claim Bonuses" }
                }
            }

            // Tier Progress Cards
            div { class: "space-y-4",
                for tier_data in tier_progress.iter() {
                    TierCard { tier_data: tier_data.clone() }
                }
            }
        }
    }
}

#[component]
fn TierCard(tier_data: serde_json::Value) -> Element {
    let tier = tier_data.get("tier").cloned().unwrap_or(serde_json::json!({}));
    let tier_name = tier.get("tier_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
    let min_referrals = tier.get("min_referrals").and_then(|v| v.as_i64()).unwrap_or(0);
    let bonus_amount = tier.get("bonus_amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let is_claimed = tier_data.get("is_claimed").and_then(|v| v.as_bool()).unwrap_or(false);
    let progress_pct = tier_data.get("progress_percent").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let referrals_needed = tier_data.get("referrals_needed").and_then(|v| v.as_i64()).unwrap_or(0);

    let (icon, border_color, bg_color) = match tier_name {
        "Bronze" => ("🥉", "border-orange-500/30", "from-orange-900/20"),
        "Silver" => ("🥈", "border-gray-400/30", "from-gray-700/20"),
        "Gold" => ("🥇", "border-yellow-500/30", "from-yellow-900/20"),
        "Platinum" => ("💎", "border-cyan-500/30", "from-cyan-900/20"),
        "Diamond" => ("👑", "border-purple-500/30", "from-purple-900/20"),
        _ => ("🏆", "border-gray-500/30", "from-gray-800/20"),
    };

    let bar_width = format!("{}%", progress_pct as i32);
    let bar_color = if is_claimed { "bg-green-500" } else { "bg-blue-500" };

    rsx! {
        div { class: "bg-gradient-to-r {bg_color} to-gray-800 rounded-lg border {border_color} p-5",
            div { class: "flex items-center justify-between mb-3",
                div { class: "flex items-center gap-3",
                    span { class: "text-3xl", "{icon}" }
                    div {
                        h3 { class: "text-white font-bold text-lg", "{tier_name} Tier" }
                        p { class: "text-gray-400 text-sm", "{min_referrals} referrals required" }
                    }
                }
                div { class: "text-right",
                    if is_claimed {
                        span { class: "px-3 py-1 rounded-full text-xs bg-green-500/20 text-green-400 border border-green-500/20", "✅ Claimed" }
                    } else {
                        span { class: "text-2xl font-bold text-yellow-400", "KES {bonus_amount as i32}" }
                        p { class: "text-gray-500 text-xs", "Bonus reward" }
                    }
                }
            }

            // Progress bar
            div { class: "w-full bg-gray-700 rounded-full h-3 mb-2",
                div { class: "{bar_color} h-3 rounded-full transition-all duration-500", style: "width: {bar_width}" }
            }
            div { class: "flex justify-between text-xs text-gray-400",
                span { "{progress_pct as i32}% complete" }
                if !is_claimed && referrals_needed > 0 {
                    span { "{referrals_needed} more referral(s) needed" }
                }
            }
        }
    }
}