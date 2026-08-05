use dioxus::prelude::*;
use crate::api::admin::{
    get_users, grant_admin_privileges, create_user, toggle_user_active,
    CreateUserRequest, GrantPrivilegesRequest, ToggleUserActiveRequest,
    User as AdminUser,
};
use crate::context::admin_auth::use_admin_auth;
use crate::AdminRoute;

#[component]
fn UserRow(
    user: AdminUser,
    token: String,
    current_user_is_superuser: bool,
    on_updated: EventHandler<()>,
) -> Element {
    let id = user.id.clone();
    let name = user.name.clone();
    let email = user.email.clone();
    let role = user.role.clone();
    let status = user.status.clone();
    let is_superuser = user.is_superuser;
    let is_staff = user.is_staff;
    let is_active = user.is_active;

    let mut show_edit_modal = use_signal(|| false);
    let mut edit_role = use_signal(|| role.clone());
    let mut edit_is_superuser = use_signal(|| is_superuser);
    let mut edit_is_staff = use_signal(|| is_staff);

    let save_role = {
        let t = token.clone();
        let uid = id.clone();
        let on_updated = on_updated.clone();
        move |_| {
            let t = t.clone();
            let uid = uid.clone();
            let on_updated = on_updated.clone();
            let req = GrantPrivilegesRequest {
                user_id: uid,
                role: edit_role.read().clone(),
                is_superuser: edit_is_superuser.read().clone(),
                is_staff: edit_is_staff.read().clone(),
            };
            spawn(async move {
                let _ = grant_admin_privileges(&t, &req).await;
                show_edit_modal.set(false);
                on_updated.call(());
            });
        }
    };

    let toggle_active = {
        let t = token.clone();
        let uid = id.clone();
        let on_updated = on_updated.clone();
        move |_| {
            let t = t.clone();
            let uid = uid.clone();
            let on_updated = on_updated.clone();
            let new_state = !is_active;
            spawn(async move {
                let req = ToggleUserActiveRequest {
                    user_id: uid,
                    is_active: new_state,
                };
                let _ = toggle_user_active(&t, &req).await;
                on_updated.call(());
            });
        }
    };

    let nav = use_navigator();
    let view_profile = {
        let uid = id.clone();
        move |_| {
            let _ = nav.push(AdminRoute::UserProfilePage { id: uid.clone() });
        }
    };

    rsx! {
        tr { class: "hover:bg-gray-700/50",
            td { class: "px-6 py-4", "{name}" }
            td { class: "px-6 py-4", "{email}" }
            td { class: "px-6 py-4",
                span {
                    class: match role.as_str() {
                        "ADMIN" => "px-2 py-1 bg-purple-900 text-purple-200 rounded text-xs font-medium",
                        "AGENT" => "px-2 py-1 bg-blue-900 text-blue-200 rounded text-xs font-medium",
                        "PROPERTY_OWNER" => "px-2 py-1 bg-green-900 text-green-200 rounded text-xs font-medium",
                        _ => "px-2 py-1 bg-gray-700 text-gray-300 rounded text-xs font-medium",
                    },
                    "{role}"
                }
            }
            td { class: "px-6 py-4",
                if is_superuser {
                    span { class: "text-yellow-400 text-xs", "★ Superuser" }
                } else if is_staff {
                    span { class: "text-blue-400 text-xs", "Staff" }
                }
            }
            td { class: "px-6 py-4",
                span {
                    class: if is_active { "text-green-400" } else { "text-red-400" },
                    {if is_active { "Active" } else { "Disabled" }}  // <-- braces
                }
            }
            td { class: "px-6 py-4",
                div { class: "flex flex-wrap gap-2",
                    button {
                        class: "px-3 py-1 text-xs bg-gray-600 hover:bg-gray-500 text-white rounded transition-colors",
                        onclick: view_profile,
                        "View"
                    }
                    button {
                        class: "px-3 py-1 text-xs bg-blue-600 hover:bg-blue-500 text-white rounded transition-colors",
                        onclick: move |_| show_edit_modal.set(true),
                        "Edit Role"
                    }
                    if !is_superuser {
                        button {
                            class: if is_active {
                                "px-3 py-1 text-xs bg-red-600 hover:bg-red-500 text-white rounded transition-colors"
                            } else {
                                "px-3 py-1 text-xs bg-green-600 hover:bg-green-500 text-white rounded transition-colors"
                            },
                            onclick: toggle_active,
                            {if is_active { "Disable" } else { "Enable" }}  // <-- braces
                        }
                    }
                }
            }
        }

        if show_edit_modal.read().clone() {
            div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                div { class: "bg-gray-800 rounded-xl border border-gray-700 p-6 w-full max-w-sm",
                    h3 { class: "text-lg font-bold text-white mb-4", "Edit User: {name}" }

                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm text-gray-400 mb-1", "Role" }
                            select {
                                class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white",
                                value: "{edit_role}",
                                onchange: move |e| edit_role.set(e.value()),
                                option { value: "CLIENT", "Client" }
                                option { value: "AGENT", "Agent" }
                                option { value: "PROPERTY_OWNER", "Property Owner" }
                                option { value: "ADMIN", "Admin" }
                            }
                        }

                        div { class: "flex items-center gap-2",
                            input {
                                r#type: "checkbox",
                                checked: "{edit_is_staff}",
                                onchange: move |e| edit_is_staff.set(e.value().parse().unwrap_or(false)),
                            }
                            label { class: "text-sm text-gray-300", "Is Staff" }
                        }

                        if current_user_is_superuser {
                            div { class: "flex items-center gap-2",
                                input {
                                    r#type: "checkbox",
                                    checked: "{edit_is_superuser}",
                                    onchange: move |e| edit_is_superuser.set(e.value().parse().unwrap_or(false)),
                                }
                                label { class: "text-sm text-gray-300", "Is Superuser (only one allowed)" }
                            }
                        }
                    }

                    div { class: "flex gap-3 mt-6",
                        button {
                            class: "flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg",
                            onclick: move |_| show_edit_modal.set(false),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg",
                            onclick: save_role,
                            "Save"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn UsersPage() -> Element {
    let auth = use_admin_auth();
    let token = auth.read().token.clone().unwrap_or_default();
    let current_user_is_superuser = auth.read().user.as_ref().map(|u| u.role == "superuser").unwrap_or(false);

    let mut users = use_signal(Vec::<AdminUser>::new);
    let mut loading = use_signal(|| true);
    let mut error = use_signal(|| None::<String>);  // <-- fixed: added closure
    let mut show_create_modal = use_signal(|| false);

    let mut new_email = use_signal(|| String::new());
    let mut new_username = use_signal(|| String::new());
    let mut new_password = use_signal(|| String::new());
    let mut new_first_name = use_signal(|| String::new());
    let mut new_last_name = use_signal(|| String::new());
    let mut new_role = use_signal(|| "CLIENT".to_string());

    let fetch_users = {
        let t = token.clone();
        move || {
            let t = t.clone();
            spawn(async move {
                loading.set(true);
                match get_users(&t).await {
                    Ok(data) => {
                        users.set(data);
                        error.set(None);
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        }
    };

    use_hook({
        let f = fetch_users.clone();
        move || { f(); }
    });

    let create_user_submit = {
        let t = token.clone();
        let fetch = fetch_users.clone();
        move |_| {
            let t = t.clone();
            let fetch = fetch.clone();
            let req = CreateUserRequest {
                email: new_email.read().clone(),
                username: if new_username.read().is_empty() {
                    new_email.read().clone()
                } else {
                    new_username.read().clone()
                },
                password: new_password.read().clone(),
                first_name: new_first_name.read().clone(),
                last_name: new_last_name.read().clone(),
                role: new_role.read().clone(),
                phone_number: None,
            };
            spawn(async move {
                match create_user(&t, &req).await {
                    Ok(_) => {
                        show_create_modal.set(false);
                        new_email.set(String::new());
                        new_username.set(String::new());
                        new_password.set(String::new());
                        new_first_name.set(String::new());
                        new_last_name.set(String::new());
                        new_role.set("CLIENT".to_string());
                        fetch();
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        }
    };

    rsx! {
        div { class: "p-6",
            div { class: "flex justify-between items-center mb-6",
                h1 { class: "text-2xl font-bold text-white", "Users" }
                button {
                    class: "px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors",
                    onclick: move |_| show_create_modal.set(true),
                    "Create User"
                }
            }

            if let Some(msg) = error.read().as_ref() {
                div { class: "mb-4 p-4 bg-red-900/50 border border-red-700 rounded-lg text-red-200",
                    "{msg}"
                }
            }

            if loading.read().clone() {
                div { class: "flex justify-center py-12",
                    div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-500" }
                }
            } else {
                div { class: "bg-gray-800 rounded-lg border border-gray-700 overflow-hidden",
                    table { class: "w-full text-left",
                        thead { class: "bg-gray-900 text-gray-400 text-sm uppercase",
                            tr {
                                th { class: "px-6 py-3", "Name" }
                                th { class: "px-6 py-3", "Email" }
                                th { class: "px-6 py-3", "Role" }
                                th { class: "px-6 py-3", "Privileges" }
                                th { class: "px-6 py-3", "Status" }
                                th { class: "px-6 py-3", "Actions" }
                            }
                        }
                        tbody { class: "divide-y divide-gray-700 text-gray-300",
                            for user in users.read().iter().cloned() {
                                {
                                    let t = token.clone();
                                    let fetch = fetch_users.clone();
                                    let is_su = current_user_is_superuser;
                                    rsx! {
                                        UserRow {
                                            user: user,
                                            token: t,
                                            current_user_is_superuser: is_su,
                                            on_updated: move |_| fetch(),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if show_create_modal.read().clone() {
            div { class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
                div { class: "bg-gray-800 rounded-xl border border-gray-700 p-6 w-full max-w-md",
                    h2 { class: "text-xl font-bold text-white mb-4", "Create New User" }

                    div { class: "space-y-4",
                        div {
                            label { class: "block text-sm text-gray-400 mb-1", "Email" }
                            input {
                                class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                r#type: "email",
                                value: "{new_email}",
                                oninput: move |e| new_email.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-400 mb-1", "Username (optional)" }
                            input {
                                class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                value: "{new_username}",
                                oninput: move |e| new_username.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-400 mb-1", "Password" }
                            input {
                                class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                r#type: "password",
                                value: "{new_password}",
                                oninput: move |e| new_password.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-400 mb-1", "First Name" }
                            input {
                                class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                value: "{new_first_name}",
                                oninput: move |e| new_first_name.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-400 mb-1", "Last Name" }
                            input {
                                class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                value: "{new_last_name}",
                                oninput: move |e| new_last_name.set(e.value()),
                            }
                        }
                        div {
                            label { class: "block text-sm text-gray-400 mb-1", "Role" }
                            select {
                                class: "w-full px-3 py-2 bg-gray-900 border border-gray-700 rounded-lg text-white focus:outline-none focus:border-blue-500",
                                value: "{new_role}",
                                onchange: move |e| new_role.set(e.value()),
                                option { value: "CLIENT", "Client" }
                                option { value: "AGENT", "Agent" }
                                option { value: "PROPERTY_OWNER", "Property Owner" }
                                option { value: "ADMIN", "Admin" }
                            }
                        }
                    }

                    div { class: "flex gap-3 mt-6",
                        button {
                            class: "flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg transition-colors",
                            onclick: move |_| show_create_modal.set(false),
                            "Cancel"
                        }
                        button {
                            class: "flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-500 text-white rounded-lg transition-colors",
                            onclick: create_user_submit,
                            "Create User"
                        }
                    }
                }
            }
        }
    }
}