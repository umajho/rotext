mod data_exchange;

extern crate alloc;

use data_exchange::{
    block_id_to_lines_map::create_block_id_to_lines_map, tag_name_map::new_tag_name_map_from_str,
};

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

#[wasm_bindgen]
pub struct ParseAndRenderResult {
    ok: Option<ParseAndRenderOutput>,
    error: Option<String>,
}
#[wasm_bindgen]
impl ParseAndRenderResult {
    pub fn clone_ok(&self) -> Option<ParseAndRenderOutput> {
        self.ok.clone()
    }
    pub fn clone_error(&self) -> Option<String> {
        self.error.clone()
    }
}

#[wasm_bindgen]
#[derive(Default, Clone)]
pub struct ParseAndRenderOutput {
    html: String,
    block_id_to_lines_map: String,

    #[cfg(debug_assertions)]
    dev_events_in_debug_format: String,
}
#[wasm_bindgen]
impl ParseAndRenderOutput {
    pub fn clone_html(&self) -> String {
        self.html.clone()
    }
    pub fn clone_block_id_to_lines_map(&self) -> String {
        self.block_id_to_lines_map.clone()
    }
    #[cfg(debug_assertions)]
    pub fn clone_dev_events_in_debug_format(&self) -> String {
        self.dev_events_in_debug_format.clone()
    }
}

#[wasm_bindgen]
pub fn parse_and_render(
    input: &[u8],
    tag_name_map: String,
    should_include_block_ids: bool,
) -> ParseAndRenderResult {
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
        Err(error) => {
            return ParseAndRenderResult {
                ok: None,
                error: Some(format!("ParseError/{}", error.name())),
            }
        }
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
        Err(error) => {
            return ParseAndRenderResult {
                ok: None,
                error: Some(format!("CompilationError/{}", error.name())),
            }
        }
    };

    let html = rotext::render(&compiled);
    let html = match String::from_utf8(html) {
        Ok(html) => html,
        Err(error) => {
            return ParseAndRenderResult {
                ok: None,
                error: Some(error.to_string()),
            }
        }
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

    ParseAndRenderResult {
        ok: Some(output),
        error: None,
    }
}
