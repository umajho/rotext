mod block;
mod common;
mod events;
mod global;

pub fn parse(input: &[u8]) -> usize {
    input.len()
}

pub fn dev(input: &[u8]) -> String {
    let global_parser = global::Parser::new(input, 0);
    let block_parser = block::Parser::new(input, global_parser);

    let mut output = "".to_string();

    for event in block_parser {
        // output.push_str(&format!("{:?}\n", event));
        output.push_str(&format!("{:?} {:?}\n", event, event.content(input)));
    }

    output
}
