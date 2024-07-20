extern crate alloc;

use wasm_bindgen::prelude::*;

use std::sync::Once;

#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[wasm_bindgen]
pub fn parse(input: &[u8]) -> usize {
    todo!()
}

#[cfg(debug_assertions)]
static INIT: Once = Once::new();

#[wasm_bindgen]
pub fn dev(input: &[u8]) -> String {
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        INIT.call_once(|| {
            console_log::init_with_level(log::Level::Debug).unwrap();
        });
    }

    parse_and_render_to_html(input)
}

fn parse_and_render_to_html(input: &[u8]) -> String {
    // let mut output = "".to_string();

    // for event in parse(input) {
    //     // output.push_str(&format!("{:?}\n", event));
    //     output.push_str(&format!(
    //         "{:?} {:?}\n",
    //         event,
    //         Event::from(event.clone()).content(input)
    //     ));
    // }

    let input_stream = rotext::parse(input);

    rotext::render_to_html(
        input,
        input_stream,
        rotext::RenderToHTMLOptions {
            initial_output_string_capacity: 20_000,
        },
    )
}
