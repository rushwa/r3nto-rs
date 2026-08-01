#![allow(non_snake_case)]

use dioxus::prelude::*;
use rento_admin::AdminApp;

fn main() {
    dioxus_logger::init(dioxus_logger::tracing::Level::INFO).expect("failed to init logger");
    launch(AdminApp);
}