use events::Event;

mod blend;
mod block;
mod common;
mod events;
mod global;
mod inline;

pub fn parse_and_render_to_html(input: &[u8]) -> String {
    let mut output = "".to_string();

    for event in parse(input) {
        // output.push_str(&format!("{:?}\n", event));
        output.push_str(&format!(
            "{:?} {:?}\n",
            event,
            Event::from(event.clone()).content(input)
        ));
    }

    output
}

fn parse(input: &[u8]) -> blend::BlockEventStreamInlineSegmentMapper {
    let global_parser = global::Parser::new(input, 0);
    let block_parser = block::Parser::new(input, global_parser);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser, Box::new(inline::Parser::new))
}
