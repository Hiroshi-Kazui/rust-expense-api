mod api;
mod components;
mod error;
mod pages;
mod router;
mod store;
mod types;

use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");

    mount_to_body(router::App);
}
