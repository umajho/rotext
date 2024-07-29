extern crate alloc;

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
#[derive(Default)]
pub struct ParseAndRenderResult {
    html: String,
    block_id_to_lines_map: String,

    #[cfg(debug_assertions)]
    dev_events_in_debug_format: String,
}
#[wasm_bindgen]
impl ParseAndRenderResult {
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
pub fn parse_and_render(input: &[u8]) -> ParseAndRenderResult {
    #[cfg(debug_assertions)]
    {
        console_error_panic_hook::set_once();
        INIT.call_once(|| {
            console_log::init_with_level(log::Level::Debug).unwrap();
        });
    }

    let all_events: Vec<_> = rotext::parse(input).collect();

    let renderer = rotext::HtmlRenderer::new(
        input,
        rotext::NewHtmlRendererOptoins {
            initial_output_string_capacity: input.len() * 3,
            with_block_id: true,
        },
    );
    let html: String = renderer.render(all_events.clone().into_iter());

    let block_id_to_lines_map = create_block_id_to_lines_map(&all_events);

    #[allow(unused_mut)]
    let mut result = ParseAndRenderResult {
        html,
        block_id_to_lines_map,
        ..Default::default()
    };

    #[cfg(debug_assertions)]
    {
        result.dev_events_in_debug_format = render_events_in_debug_format(input, &all_events);
    }

    result
}

fn create_block_id_to_lines_map(all_events: &[rotext::BlendEvent]) -> String {
    let mut result = String::new();

    for ev in all_events.iter() {
        match ev {
            rotext::BlendEvent::ThematicBreak(data) => {
                write_id_and_line_range(
                    &mut result,
                    data.id.value(),
                    data.line_number,
                    data.line_number,
                );
            }
            rotext::BlendEvent::ExitBlock(data) => {
                write_id_and_line_range(
                    &mut result,
                    data.id.value(),
                    data.start_line_number,
                    data.end_line_number,
                );
            }
            _ => continue,
        }

        result.push(';');
    }

    if !result.is_empty() {
        result.pop().unwrap();
    }

    result
}

fn write_id_and_line_range(target: &mut String, id: usize, range_start: usize, range_end: usize) {
    write_usize(target, id);
    target.push(':');
    write_usize(target, range_start);
    target.push('-');
    write_usize(target, range_end);
}

fn write_usize(target: &mut String, n: usize) {
    let mut buffer = itoa::Buffer::new();
    target.push_str(buffer.format(n));
}

#[cfg(debug_assertions)]
fn render_events_in_debug_format(input: &[u8], all_events: &Vec<rotext::BlendEvent>) -> String {
    let mut output = "".to_string();

    for event in all_events {
        // output.push_str(&format!("{:?}\n", event));
        output.push_str(&format!(
            "{:?} {:?}\n",
            event,
            rotext::Event::from(event.clone()).content(input)
        ));
    }

    output
}
