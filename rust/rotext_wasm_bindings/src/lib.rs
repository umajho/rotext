mod data_exchange;

extern crate alloc;

use data_exchange::block_id_to_lines_map::create_block_id_to_lines_map;

#[cfg(debug_assertions)]
use data_exchange::events_in_debug_format::render_events_in_debug_format;

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

#[derive(serde::Deserialize)]
pub struct ParseAndRenderOptions {
    pub tag_name_map: data_exchange::tag_name_map::TagNameMapInput,
    pub should_include_block_ids: bool,
}

#[derive(Default, Clone, serde::Serialize)]
pub struct ParseAndRenderOutput {
    pub html: String,
    pub block_id_to_lines_map: data_exchange::block_id_to_lines_map::BlockIdToLInesMap,

    #[cfg(debug_assertions)]
    pub dev_events_in_debug_format: String,
}

#[wasm_bindgen]
pub fn parse_and_render(input: &[u8], opts: &[u8]) -> Result<Vec<u8>, String> {
    let opts: ParseAndRenderOptions = match serde_json_wasm::from_slice(opts) {
        Ok(opts) => opts,
        Err(error) => return Err(format!("InputError/DeserializationError|{}", error)),
    };

    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        INIT.call_once(|| {
            console_log::init_with_level(log::Level::Debug).unwrap();
        });
    }

    let tag_name_map = opts.tag_name_map.to_tag_name_map();

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
        should_include_block_ids: opts.should_include_block_ids,
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

    serde_json_wasm::to_vec(&output)
        .map_err(|err| format!("OutputError/SerializationError|{}", err))
}
