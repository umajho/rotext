use crate::events::BlendEvent;

pub struct RenderToHTMLOptions {
    pub initial_output_string_capacity: usize,
}

pub use using_vec_u8::render_to_html;

pub mod using_string {
    use super::*;

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

            fn push_simple(
                result: &mut String,
                stack: &mut Vec<&'static str>,
                tag_name: &'static str,
            ) {
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
                BlendEvent::EnterParagraph => push_simple(&mut result, &mut stack, "p"),
                BlendEvent::ThematicBreak => result.push_str("<hr>"),
                BlendEvent::EnterHeading1 => push_simple(&mut result, &mut stack, "h1"),
                BlendEvent::EnterHeading2 => push_simple(&mut result, &mut stack, "h2"),
                BlendEvent::EnterHeading3 => push_simple(&mut result, &mut stack, "h3"),
                BlendEvent::EnterHeading4 => push_simple(&mut result, &mut stack, "h4"),
                BlendEvent::EnterHeading5 => push_simple(&mut result, &mut stack, "h5"),
                BlendEvent::EnterHeading6 => push_simple(&mut result, &mut stack, "h6"),
                BlendEvent::EnterBlockQuote => push_simple(&mut result, &mut stack, "blockquote"),
                BlendEvent::EnterOrderedList => push_simple(&mut result, &mut stack, "ol"),
                BlendEvent::EnterUnorderedList => push_simple(&mut result, &mut stack, "ul"),
                BlendEvent::EnterListItem => push_simple(&mut result, &mut stack, "li"),
                BlendEvent::EnterDescriptionList => push_simple(&mut result, &mut stack, "dl"),
                BlendEvent::EnterDescriptionTerm => push_simple(&mut result, &mut stack, "dt"),
                BlendEvent::EnterDescriptionDetails => push_simple(&mut result, &mut stack, "dd"),
                BlendEvent::EnterCodeBlock => {
                    stack.push("</x-code-block>");
                    result.push_str("<x-code-block info-string=\"");
                    loop {
                        match input_stream.next().unwrap() {
                            BlendEvent::Text(content) => {
                                write_escaped_double_quoted_attribute_value(
                                    &mut result,
                                    content.content(input),
                                )
                            }
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
}

pub mod using_vec_u8 {
    use super::*;

    pub fn render_to_html<I: Iterator<Item = BlendEvent>>(
        input: &[u8],
        mut input_stream: I,
        opts: RenderToHTMLOptions,
    ) -> String {
        let mut result: Vec<u8> = Vec::with_capacity(opts.initial_output_string_capacity);
        let mut stack: Vec<&'static [u8]> = vec![];
        loop {
            let Some(ev) = input_stream.next() else {
                break;
            };

            fn push_simple(
                result: &mut Vec<u8>,
                stack: &mut Vec<&'static [u8]>,
                tag_name: &'static [u8],
            ) {
                result.push(b'<');
                result.extend(tag_name);
                result.push(b'>');
                stack.push(tag_name);
            }

            match ev {
                BlendEvent::LineBreak => result.extend(b"<br>"),
                BlendEvent::Text(content) => {
                    write_escaped_html_text(&mut result, content.content_in_u8_array(input));
                }
                BlendEvent::Exit => {
                    result.extend(b"</");
                    result.extend(stack.pop().unwrap());
                    result.push(b'>');
                }
                BlendEvent::Separator => unreachable!(),
                BlendEvent::EnterParagraph => push_simple(&mut result, &mut stack, b"p"),
                BlendEvent::ThematicBreak => result.extend(b"<hr>"),
                BlendEvent::EnterHeading1 => push_simple(&mut result, &mut stack, b"h1"),
                BlendEvent::EnterHeading2 => push_simple(&mut result, &mut stack, b"h2"),
                BlendEvent::EnterHeading3 => push_simple(&mut result, &mut stack, b"h3"),
                BlendEvent::EnterHeading4 => push_simple(&mut result, &mut stack, b"h4"),
                BlendEvent::EnterHeading5 => push_simple(&mut result, &mut stack, b"h5"),
                BlendEvent::EnterHeading6 => push_simple(&mut result, &mut stack, b"h6"),
                BlendEvent::EnterBlockQuote => push_simple(&mut result, &mut stack, b"blockquote"),
                BlendEvent::EnterOrderedList => push_simple(&mut result, &mut stack, b"ol"),
                BlendEvent::EnterUnorderedList => push_simple(&mut result, &mut stack, b"ul"),
                BlendEvent::EnterListItem => push_simple(&mut result, &mut stack, b"li"),
                BlendEvent::EnterDescriptionList => push_simple(&mut result, &mut stack, b"dl"),
                BlendEvent::EnterDescriptionTerm => push_simple(&mut result, &mut stack, b"dt"),
                BlendEvent::EnterDescriptionDetails => push_simple(&mut result, &mut stack, b"dd"),
                BlendEvent::EnterCodeBlock => {
                    stack.push(b"</x-code-block>");
                    result.extend(b"<x-code-block info-string=\"");
                    loop {
                        match input_stream.next().unwrap() {
                            BlendEvent::Text(content) => {
                                write_escaped_double_quoted_attribute_value(
                                    &mut result,
                                    content.content_in_u8_array(input),
                                )
                            }
                            BlendEvent::Separator => break,
                            _ => unreachable!(),
                        }
                    }
                    result.extend(b"\">")
                }
            };
        }

        unsafe { String::from_utf8_unchecked(result) }
    }

    fn write_escaped_html_text(dest: &mut Vec<u8>, input: &[u8]) {
        for char in input {
            match *char {
                b'<' => dest.extend(b"&lt;"),
                b'&' => dest.extend(b"&amp;"),
                char => dest.push(char),
            }
        }
    }

    fn write_escaped_double_quoted_attribute_value(dest: &mut Vec<u8>, input: &[u8]) {
        for char in input {
            match *char {
                b'"' => dest.extend(b"&quot;"),
                b'&' => dest.extend(b"&amp;"),
                char => dest.push(char),
            }
        }
    }
}
