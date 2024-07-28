use crate::events::BlendEvent;
use crate::events::VerbatimEscaping;

pub struct RenderToHTMLOptions {
    pub initial_output_string_capacity: usize,

    #[cfg(feature = "block-id")]
    pub with_block_id: bool,
}

pub use using_vec_u8::render_to_html;

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
                #[allow(unused_variables)] opts: &RenderToHTMLOptions,
                result: &mut Vec<u8>,
                stack: &mut Vec<&'static [u8]>,
                tag_name: &'static [u8],
                #[cfg(feature = "block-id")] id: usize,
            ) {
                result.push(b'<');
                result.extend(tag_name);

                {
                    #[cfg(feature = "block-id")]
                    {
                        if opts.with_block_id {
                            result.extend(br#" data-block-id=""#);
                            result.extend(id.to_string().as_bytes());
                            result.extend(br#"">"#);
                        } else {
                            result.push(b'>');
                        }
                    }
                    #[cfg(not(feature = "block-id"))]
                    {
                        result.push(b'>');
                    }
                }

                stack.push(tag_name);
            }

            match ev {
                BlendEvent::NewLine(_) => result.extend(b"<br>"),
                BlendEvent::Text(content)
                | BlendEvent::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                    write_escaped_html_text(&mut result, content.content_in_u8_array(input));
                }
                BlendEvent::ExitBlock(_) => {
                    result.extend(b"</");
                    result.extend(stack.pop().unwrap());
                    result.push(b'>');
                }
                BlendEvent::Separator => unreachable!(),
                #[allow(unused_variables)]
                BlendEvent::EnterParagraph(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"p",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::ThematicBreak(data) => {
                    #[cfg(feature = "block-id")]
                    {
                        if opts.with_block_id {
                            result.extend(br#"<hr data-block-id=""#);
                            result.extend(data.id.to_string().as_bytes());
                            result.extend(br#"">"#);
                        } else {
                            result.extend(b"<hr>")
                        }
                    }
                    #[cfg(not(feature = "block-id"))]
                    {
                        result.extend(b"<hr>")
                    }
                }
                #[allow(unused_variables)]
                BlendEvent::EnterHeading1(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"h1",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterHeading2(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"h2",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterHeading3(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"h3",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterHeading4(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"h4",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterHeading5(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"h5",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterHeading6(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"h6",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterBlockQuote(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"blockquote",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterOrderedList(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"ol",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterUnorderedList(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"ul",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterListItem(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"li",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterDescriptionList(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"dl",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterDescriptionTerm(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"dt",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterDescriptionDetails(data) => push_simple(
                    &opts,
                    &mut result,
                    &mut stack,
                    b"dd",
                    #[cfg(feature = "block-id")]
                    data.id,
                ),
                #[allow(unused_variables)]
                BlendEvent::EnterCodeBlock(data) => {
                    result.extend(br#"<x-code-block info-string=""#);
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

                    #[cfg(feature = "block-id")]
                    {
                        if opts.with_block_id {
                            result.extend(br#"" data-block-id=""#);
                            result.extend(data.id.to_string().as_bytes());
                        }
                    }

                    result.extend(br#"">"#);
                    stack.push(b"x-code-block");
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
