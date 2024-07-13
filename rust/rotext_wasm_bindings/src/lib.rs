use wasm_bindgen::prelude::*;

use std::sync::Once;

#[wasm_bindgen]
pub fn parse(input: &[u8]) -> usize {
    rotext::parse(input)
}

static INIT: Once = Once::new();

#[wasm_bindgen]
pub fn dev(input: &[u8]) -> String {
    console_error_panic_hook::set_once();
    INIT.call_once(|| {
        console_log::init_with_level(log::Level::Debug).unwrap();
    });

    rotext::dev(input)
}
