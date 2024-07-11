mod events;
mod global_level_parser;

use std::string::FromUtf8Error;

use events::Event;
use global_level_parser::GlobalLevelParser;

pub fn parse(input: &[u8]) -> usize {
    input.len()
}

pub fn dev(input: &[u8]) -> String {
    let parser = GlobalLevelParser::new(input, 0);

    let mut output = "".to_string();

    for event in parser {
        output.push_str(&format!("{:?} {:?}\n", event, content(&event, input)))
    }

    output
}

fn content(event: &Event, input: &[u8]) -> Result<String, FromUtf8Error> {
    let slice = match *event {
        Event::Undetermined { start, length } => &input[start..start + length],
        Event::Comment {
            content_start,
            content_length,
            is_closed_forcedly: _,
        } => &input[content_start..content_start + content_length],
        Event::VerbatimEscaping {
            content_start,
            content_length,
            is_closed_forcedly: _,
        } => &input[content_start..content_start + content_length],
    };

    String::from_utf8(slice.to_vec())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
// }
