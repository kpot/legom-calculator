mod app;
mod calculator;
mod store;
mod ui_components;
mod yew_utils;

use std::panic;
use web_sys::window;

const APP_ROOT_ID: &str = "calculator";

fn main() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let app_root_element = window()
        .and_then(|window| window.document())
        .and_then(|doc| doc.get_element_by_id(APP_ROOT_ID))
        .unwrap_or_else(|| {
            panic!(
                "DOM must be available and contain element with ID '{}'",
                APP_ROOT_ID
            )
        });
    let renderer = yew::Renderer::<crate::app::App>::with_root(app_root_element);
    renderer.render();
}
