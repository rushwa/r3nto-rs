// use dioxus::prelude::*;
// use crate::components::sidebar::PageHeader;
// use crate::context::admin_auth::use_admin_auth;
// use crate::api::admin::get_leaderboard;
//
// #[component]
// pub fn LeaderboardPage() -> Element {
//     let auth = use_admin_auth();
//     let token = auth.read().token.clone().unwrap_or_default();
//     let user_role = auth.read().user.as_ref().map(|u| u.role.to_uppercase()).unwrap_or_default();
//
//     let mut leaderboard_data = use_signal(|| Option::<serde_json::Value>::None);
//     let mut loading = use_signal(|| true);
//
//     let token_for_effect = token.clone();
//
//     use_effect(move || {
//         let t = token_for_effect.clone();
//         spawn(async move {
//             if let Ok(data) = get_leaderboard(&t).await {
//                 leaderboard_data.set(Some(data));
//             }
//             loading.set(false);
//         });
//     });
//
//     if *loading.read() {
//         return rsx! {
//             div { class: "flex items-center justify-center h-96",
//                 div { class: "text-white text-lg", "Loading leaderboard..." }
//             }
//         };
//     }
//
//     let data = leaderboard_data.read().clone().unwrap_or(serde_json::json!({}));
//     let agents = data.get("leaderboard").and_then(|v| v.as_array()).cloned().unwrap_or_default();
//     let my_rank = data.get("my_rank").cloned();
//     let total_agents = data.get("total_agents").and_then(|v| v.as_i64()).unwrap_or(0);
//     let is_admin = user_role == "ADMIN" || user_role == "SUPERUSER";
//
//     rsx! {
//         div { class: "space-y-6",
//             PageHeader {
//                 title: "🏆 Agent Leaderboard".to_string(),
//                 subtitle: format!("Top performing agents out of {} total", total_agents),
//             }
//
//             // My Rank Banner (for agents not in top N)
//             if let Some(my) = my_rank.as_ref() {
//                 div { class: "bg-gradient-to-r from-blue-900/40 to-purple-900/40 rounded-lg border border-blue-500/30 p-5",
//                     div { class: "flex items-center justify-between",
//                         div { class: "flex items-center gap-4",
//                             div { class: "w-12 h-12 bg-blue-600 rounded-full flex items-center justify-center text-white font-bold text-lg",
//                                 "#{my.get("rank").and_then(|v| v.as_i64()).unwrap_or(0)}"
//                             }
//                             div {
//                                 p { class: "text-white font-semibold", "Your Ranking" }
//                                 p { class: "text-gray-400 text-sm",
//                                     "KES {my.get("total_commissions").and_then(|v| v.as_f64()).unwrap_or(0.0) as i32} earned • {my.get("total_conversions").and_then(|v| v.as_i64()).unwrap_or(0)} conversions"
//                                 }
//                             }
//                         }
//                         span { class: "text-blue-400 text-sm", "Keep climbing! 🚀" }
//                     }
//                 }
//             }
//
//             // Leaderboard Table
//             div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
//                 if agents.is_empty() {
//                     div { class: "p-12 text-center",
//                         span { class: "text-4xl", "🏆" }
//                         p { class: "text-gray-400 mt-4", "No agents on the leaderboard yet." }
//                     }
//                 } else {
//                     table { class: "w-full",
//                         thead { class: "bg-gray-900",
//                             tr {
//                                 th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Rank" }
//                                 th { class: "px-4 py-3 text-left text-xs font-medium text-gray-400 uppercase", "Agent" }
//                                 th { class: "px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase", "Commissions" }
//                                 th { class: "px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase", "Conversions" }
//                                 th { class: "px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase", "Referrals" }
//                                 th { class: "px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase", "Properties" }
//                                 th { class: "px-4 py-3 text-right text-xs font-medium text-gray-400 uppercase", "Score" }
//                             }
//                         }
//                         tbody { class: "divide-y divide-gray-700",
//                             for agent in agents.iter() {
//                                 LeaderboardRow { agent: agent.clone(), is_admin }
//                             }
//                         }
//                     }
//                 }
//             }
//
//             // Scoring Explanation
//             div { class: "bg-gray-800 rounded-lg border border-gray-700 p-5",
//                 h3 { class: "text-white font-semibold mb-3", "📐 How Score is Calculated" }
//                 div { class: "grid grid-cols-2 md:grid-cols-4 gap-3 text-sm",
//                     div { class: "bg-gray-900 rounded p-3",
//                         p { class: "text-green-400 font-semibold", "40%" }
//                         p { class: "text-gray-400", "Commissions earned" }
//                     }
//                     div { class: "bg-gray-900 rounded p-3",
//                         p { class: "text-blue-400 font-semibold", "30%" }
//                         p { class: "text-gray-400", "Conversions made" }
//                     }
//                     div { class: "bg-gray-900 rounded p-3",
//                         p { class: "text-purple-400 font-semibold", "20%" }
//                         p { class: "text-gray-400", "Referrals brought" }
//                     }
//                     div { class: "bg-gray-900 rounded p-3",
//                         p { class: "text-orange-400 font-semibold", "10%" }
//                         p { class: "text-gray-400", "Properties managed" }
//                     }
//                 }
//             }
//         }
//     }
// }
//
// #[component]
// fn LeaderboardRow(agent: serde_json::Value, is_admin: bool) -> Element {
//     let rank = agent.get("rank").and_then(|v| v.as_i64()).unwrap_or(0);
//     let name = agent.get("agent_name").and_then(|v| v.as_str()).unwrap_or("Unknown");
//     let commissions = agent.get("total_commissions").and_then(|v| v.as_f64()).unwrap_or(0.0);
//     let conversions = agent.get("total_conversions").and_then(|v| v.as_i64()).unwrap_or(0);
//     let referrals = agent.get("total_referrals").and_then(|v| v.as_i64()).unwrap_or(0);
//     let properties = agent.get("properties_managed").and_then(|v| v.as_i64()).unwrap_or(0);
//     let score = agent.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0);
//     let is_current = agent.get("is_current_user").and_then(|v| v.as_bool()).unwrap_or(false);
//
//     let rank_display = match rank {
//         1 => "🥇",
//         2 => "🥈",
//         3 => "🥉",
//         _ => "",
//     };
//
//     let row_class = if is_current {
//         "bg-blue-900/20 hover:bg-blue-900/30"
//     } else {
//         "hover:bg-gray-700/30"
//     };
//
//     // Anonymize names for non-admin agents viewing others
//     let display_name = if is_admin || is_current {
//         name.to_string()
//     } else {
//         // Show first name + last initial for privacy
//         let parts: Vec<&str> = name.split_whitespace().collect();
//         if parts.len() >= 2 {
//             format!("{} {}.", parts[0], parts[1].chars().next().unwrap_or('?'))
//         } else {
//             name.to_string()
//         }
//     };
//
//     rsx! {
//         tr { class: "{row_class} transition-colors",
//             td { class: "px-4 py-3",
//                 div { class: "flex items-center gap-2",
//                     if !rank_display.is_empty() {
//                         span { class: "text-xl", "{rank_display}" }
//                     } else {
//                         span { class: "text-gray-400 font-mono", "#{rank}" }
//                     }
//                 }
//             }
//             td { class: "px-4 py-3",
//                 span { class: "text-white font-medium", "{display_name}" }
//                 if is_current {
//                     span { class: "ml-2 px-2 py-0.5 rounded text-xs bg-blue-600/20 text-blue-400", "You" }
//                 }
//             }
//             td { class: "px-4 py-3 text-right text-green-400 font-semibold", "KES {commissions as i32}" }
//             td { class: "px-4 py-3 text-right text-white", "{conversions}" }
//             td { class: "px-4 py-3 text-right text-white", "{referrals}" }
//             td { class: "px-4 py-3 text-right text-white", "{properties}" }
//             td { class: "px-4 py-3 text-right text-yellow-400 font-bold", "{score as i32}" }
//         }
//     }
// }