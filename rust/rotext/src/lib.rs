mod blend;
mod block;
mod common;
mod events;
mod global;
mod inline;

pub use events::{BlendEvent, Event};

pub fn parse(input: &[u8]) -> blend::BlockEventStreamInlineSegmentMapper {
    let global_parser = global::Parser::new(input, 0);
    let block_parser = block::Parser::new(input, global_parser);

    blend::BlockEventStreamInlineSegmentMapper::new(block_parser, Box::new(inline::Parser::new))
}

pub struct RenderToHTMLOptions {
    pub initial_output_string_capacity: usize,
}
pub fn render_to_html<I: Iterator<Item = BlendEvent>>(
    input: &[u8],
    mut input_stream: I,
    opts: RenderToHTMLOptions,
) -> String {
    let mut result = String::with_capacity(opts.initial_output_string_capacity);
    let mut stack: Vec<&'static str> = vec![];
    loop {
        let Some(ev) = input_stream.next() else {
            break;
        };

        match ev {
            BlendEvent::LineBreak => result.push_str("<br>"),
            BlendEvent::Text(content) => {
                write_escaped_html_text(&mut result, content.content(input));
            }
            BlendEvent::Exit => result.push_str(stack.pop().unwrap()),
            BlendEvent::Separator => unreachable!(),
            BlendEvent::EnterParagraph => {
                stack.push("</p>");
                result.push_str("<p>")
            }
            BlendEvent::ThematicBreak => result.push_str("<hr>"),
            BlendEvent::EnterHeading1 => {
                stack.push("</h1>");
                result.push_str("<h1>")
            }
            BlendEvent::EnterHeading2 => {
                stack.push("</h2>");
                result.push_str("<h2>")
            }
            BlendEvent::EnterHeading3 => {
                stack.push("</h3>");
                result.push_str("<h3>")
            }
            BlendEvent::EnterHeading4 => {
                stack.push("</h4>");
                result.push_str("<h4>")
            }
            BlendEvent::EnterHeading5 => {
                stack.push("</h5>");
                result.push_str("<h5>")
            }
            BlendEvent::EnterHeading6 => {
                stack.push("</h6>");
                result.push_str("<h6>")
            }
            BlendEvent::EnterBlockQuote => {
                stack.push("</blockquote>");
                result.push_str("<blockquote>")
            }
            BlendEvent::EnterOrderedList => {
                stack.push("</ol>");
                result.push_str("<ol>")
            }
            BlendEvent::EnterUnorderedList => {
                stack.push("</ul>");
                result.push_str("<ul>")
            }
            BlendEvent::EnterListItem => {
                stack.push("</li>");
                result.push_str("<li>")
            }
            BlendEvent::EnterCodeBlock => {
                stack.push("</x-code-block>");
                result.push_str("<x-code-block info-string=\"");
                loop {
                    match input_stream.next().unwrap() {
                        BlendEvent::Text(content) => write_escaped_double_quoted_attribute_value(
                            &mut result,
                            content.content(input),
                        ),
                        BlendEvent::Separator => break,
                        _ => unreachable!(),
                    }
                }
                result.push_str("\">")
            }
        };
    }

    result
}

fn write_escaped_html_text(dest: &mut String, input: &str) {
    for char in input.chars() {
        match char {
            '<' => dest.push_str("&lt;"),
            '&' => dest.push_str("&amp;"),
            _ => dest.push(char),
        }
    }
}

fn write_escaped_double_quoted_attribute_value(dest: &mut String, input: &str) {
    for char in input.chars() {
        match char {
            '"' => dest.push_str("&quot;"),
            '&' => dest.push_str("&amp;"),
            _ => dest.push(char),
        }
    }
}
