pub use crate::events::BlendEvent;

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

        fn push_simple(result: &mut String, stack: &mut Vec<&'static str>, tag_name: &'static str) {
            result.push('<');
            result.push_str(tag_name);
            result.push('>');
            stack.push(tag_name);
        }

        match ev {
            BlendEvent::LineBreak => result.push_str("<br>"),
            BlendEvent::Text(content) => {
                write_escaped_html_text(&mut result, content.content(input));
            }
            BlendEvent::Exit => {
                result.push_str("</");
                result.push_str(stack.pop().unwrap());
                result.push('>');
            }
            BlendEvent::Separator => unreachable!(),
            BlendEvent::EnterParagraph => {
                push_simple(&mut result, &mut stack, "p");
            }
            BlendEvent::ThematicBreak => result.push_str("<hr>"),
            BlendEvent::EnterHeading1 => {
                push_simple(&mut result, &mut stack, "h1");
            }
            BlendEvent::EnterHeading2 => {
                push_simple(&mut result, &mut stack, "h2");
            }
            BlendEvent::EnterHeading3 => {
                push_simple(&mut result, &mut stack, "h3");
            }
            BlendEvent::EnterHeading4 => {
                push_simple(&mut result, &mut stack, "h4");
            }
            BlendEvent::EnterHeading5 => {
                push_simple(&mut result, &mut stack, "h5");
            }
            BlendEvent::EnterHeading6 => {
                push_simple(&mut result, &mut stack, "h6");
            }
            BlendEvent::EnterBlockQuote => {
                push_simple(&mut result, &mut stack, "blockquote");
            }
            BlendEvent::EnterOrderedList => {
                push_simple(&mut result, &mut stack, "ol");
            }
            BlendEvent::EnterUnorderedList => {
                push_simple(&mut result, &mut stack, "ul");
            }
            BlendEvent::EnterListItem => {
                push_simple(&mut result, &mut stack, "li");
            }
            BlendEvent::EnterDescriptionList => {
                push_simple(&mut result, &mut stack, "dl");
            }
            BlendEvent::EnterDescriptionTerm => {
                push_simple(&mut result, &mut stack, "dt");
            }
            BlendEvent::EnterDescriptionDetails => {
                push_simple(&mut result, &mut stack, "dd");
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
