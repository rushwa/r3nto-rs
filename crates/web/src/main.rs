#![allow(non_snake_case)]

use dioxus::prelude::*;

mod api;
mod components;
mod context;
mod pages;

use components::navbar::Navbar;
use context::auth::provide_auth_context;
use pages::{
    dashboard::Dashboard, home::Home, login::Login, profile::Profile,
    properties::Properties, register::Register,tour_view::TourViewPage,
};

#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub enum Route {
    #[layout(Navbar)]
        #[route("/")]
        Home {},

        #[route("/login")]
        Login {},

        #[route("/register")]
        Register {},

        #[route("/dashboard")]
        Dashboard {},

        #[route("/profile")]
        Profile {},

        #[route("/properties")]
        Properties {},
         // ✅ ADD: Tour viewing route (public, no auth)
        #[route("/tour/view/:token")]
        TourViewPage { token: String },
    #[end_layout]

    #[route("/:..segments")]
    PageNotFound { segments: Vec<String> },
}

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    launch(App);
}

#[component]
fn App() -> Element {
    // Provide auth context BEFORE Router renders any children.
    // This ensures Navbar (and all other components) can call use_auth().
    provide_auth_context();

    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn PageNotFound(segments: Vec<String>) -> Element {
    let path = segments.join("/");
    rsx! {
        div { class: "min-h-screen flex items-center justify-center bg-gray-50",
            div { class: "text-center",
                h1 { class: "text-6xl font-bold text-gray-900 mb-4", "404" }
                p { class: "text-xl text-gray-600 mb-8", "Page not found: /{path}" }
                Link {
                    to: Route::Home {},
                    class: "bg-blue-600 hover:bg-blue-700 text-white px-6 py-3 rounded-lg transition",
                    "Go Home"
                }
            }
        }
    }
}