mod events;
mod global_level_parser;

use global_level_parser::GlobalLevelParser;

pub fn parse(input: &[u8]) -> usize {
    input.len()
}

pub fn dev(input: &[u8]) -> String {
    let parser = GlobalLevelParser::new(input, 0);

    let mut output = "".to_string();

    for event in parser {
        output.push_str(&format!("{:?} {:?}\n", event, event.content(input)))
    }

    output
}
