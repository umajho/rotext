use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(input: &[u8]) -> usize {
    rotext::parse(input)
}
