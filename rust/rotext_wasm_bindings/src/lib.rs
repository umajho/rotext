mod data_exchange;

extern crate alloc;

use data_exchange::{
    block_id_to_lines_map::create_block_id_to_lines_map, tag_name_map::new_tag_name_map_from_str,
};

#[cfg(debug_assertions)]
use data_exchange::events_in_debug_format::render_events_in_debug_format;

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use lol_alloc::{AssumeSingleThreaded, FreeListAllocator};
#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: AssumeSingleThreaded<FreeListAllocator> =
    unsafe { AssumeSingleThreaded::new(FreeListAllocator::new()) };

#[cfg(debug_assertions)]
use std::sync::Once;
#[cfg(debug_assertions)]
static INIT: Once = Once::new();

#[derive(Default, Clone, serde::Serialize)]
pub struct ParseAndRenderOutput {
    pub html: String,
    pub block_id_to_lines_map: data_exchange::block_id_to_lines_map::BlockIdToLInesMap,

    #[cfg(debug_assertions)]
    pub dev_events_in_debug_format: String,
}

#[wasm_bindgen]
pub fn parse_and_render(
    input: &[u8],
    tag_name_map: String,
    should_include_block_ids: bool,
) -> Result<JsValue, String> {
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        INIT.call_once(|| {
            console_log::init_with_level(log::Level::Debug).unwrap();
        });
    }

    let tag_name_map = new_tag_name_map_from_str(&tag_name_map);

    let all_events: Result<Vec<_>, _> = rotext::parse(input).collect();
    let all_events = match all_events {
        Ok(all_events) => all_events,
        Err(error) => return Err(format!("ParseError/{}", error.name())),
    };

    let compile_opts = rotext::CompileOption {
        restrictions: rotext::CompileRestrictions {
            document_max_call_depth: 100,
        },
        tag_name_map: &tag_name_map,
        should_include_block_ids,
    };
    let compiled = rotext::compile(input, &all_events, &compile_opts);
    let compiled = match compiled {
        Ok(compiled) => compiled,
        Err(error) => return Err(format!("CompilationError/{}", error.name())),
    };

    let render_opts = rotext::RenderOptions {
        tag_name_map: &tag_name_map,
    };
    let html = rotext::render(&compiled, render_opts);
    let html = match String::from_utf8(html) {
        Ok(html) => html,
        Err(error) => return Err(error.to_string()),
    };

    let block_id_to_lines_map = create_block_id_to_lines_map(&all_events);

    #[allow(unused_mut)]
    let mut output = ParseAndRenderOutput {
        html,
        block_id_to_lines_map,
        ..Default::default()
    };

    #[cfg(debug_assertions)]
    {
        output.dev_events_in_debug_format = render_events_in_debug_format(input, &all_events);
    }

    output
        .serialize(&serde_wasm_bindgen::Serializer::new())
        .map_err(|err| format!("SerializationError|{}", err))
}
