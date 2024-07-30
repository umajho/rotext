mod tests;

use crate::events::BlendEvent;
use crate::events::BlockWithID;
use crate::events::VerbatimEscaping;

pub struct NewHtmlRendererOptoins {
    pub initial_output_string_capacity: usize,

    #[cfg(feature = "block-id")]
    pub with_block_id: bool,
}

pub struct HtmlRenderer<'a> {
    input: &'a [u8],

    #[cfg(feature = "block-id")]
    with_block_id: bool,

    result: Vec<u8>,
    stack: Vec<&'static [u8]>,
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(input: &'a [u8], opts: NewHtmlRendererOptoins) -> Self {
        Self {
            input,
            #[cfg(feature = "block-id")]
            with_block_id: opts.with_block_id,
            result: Vec::with_capacity(opts.initial_output_string_capacity),
            stack: vec![],
        }
    }

    pub fn render(mut self, mut input_stream: impl Iterator<Item = BlendEvent>) -> String {
        loop {
            let Some(ev) = input_stream.next() else {
                break;
            };

            match ev {
                BlendEvent::NewLine(_) => self.result.extend(b"<br>"),
                BlendEvent::Text(content)
                | BlendEvent::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                    self.write_escaped_html_text(content.content_in_u8_array(self.input));
                }
                #[allow(unused_variables)]
                BlendEvent::ThematicBreak(data) => {
                    #[cfg(feature = "block-id")]
                    {
                        if self.with_block_id {
                            self.result.extend(br#"<hr data-block-id=""#);
                            self.write_usize(data.id.value());
                            self.result.extend(br#"">"#);
                        } else {
                            self.result.extend(b"<hr>")
                        }
                    }
                    #[cfg(not(feature = "block-id"))]
                    {
                        self.result.extend(b"<hr>")
                    }
                }

                BlendEvent::ExitBlock(_) => {
                    self.result.extend(b"</");
                    self.result.extend(self.stack.pop().unwrap());
                    self.result.push(b'>');
                }

                BlendEvent::EnterParagraph(data) => self.push_simple(b"p", &data),
                BlendEvent::EnterHeading1(data) => self.push_simple(b"h1", &data),
                BlendEvent::EnterHeading2(data) => self.push_simple(b"h2", &data),
                BlendEvent::EnterHeading3(data) => self.push_simple(b"h3", &data),
                BlendEvent::EnterHeading4(data) => self.push_simple(b"h4", &data),
                BlendEvent::EnterHeading5(data) => self.push_simple(b"h5", &data),
                BlendEvent::EnterHeading6(data) => self.push_simple(b"h6", &data),
                BlendEvent::EnterBlockQuote(data) => self.push_simple(b"blockquote", &data),
                BlendEvent::EnterOrderedList(data) => self.push_simple(b"ol", &data),
                BlendEvent::EnterUnorderedList(data) => self.push_simple(b"ul", &data),
                BlendEvent::EnterListItem(data) => self.push_simple(b"li", &data),
                BlendEvent::EnterDescriptionList(data) => self.push_simple(b"dl", &data),
                BlendEvent::EnterDescriptionTerm(data) => self.push_simple(b"dt", &data),
                BlendEvent::EnterDescriptionDetails(data) => self.push_simple(b"dd", &data),
                #[allow(unused_variables)]
                BlendEvent::EnterCodeBlock(data) => {
                    self.result.extend(br#"<x-code-block info-string=""#);
                    loop {
                        match input_stream.next().unwrap() {
                            BlendEvent::Text(content) => self
                                .write_escaped_double_quoted_attribute_value(
                                    content.content_in_u8_array(self.input),
                                ),
                            BlendEvent::IndicateCodeBlockCode => break,
                            _ => unreachable!(),
                        }
                    }

                    #[cfg(feature = "block-id")]
                    {
                        if self.with_block_id {
                            self.result.extend(br#"" data-block-id=""#);
                            self.write_usize(data.id.value());
                        }
                    }

                    self.result.extend(br#"">"#);
                    self.stack.push(b"x-code-block");
                }

                BlendEvent::IndicateCodeBlockCode => unreachable!(),
            };
        }

        unsafe { String::from_utf8_unchecked(self.result) }
    }

    fn push_simple(
        &mut self,
        tag_name: &'static [u8],
        #[allow(unused_variables)] data: &BlockWithID,
    ) {
        self.result.push(b'<');
        self.result.extend(tag_name);

        {
            #[cfg(feature = "block-id")]
            {
                if self.with_block_id {
                    self.result.extend(br#" data-block-id=""#);
                    self.write_usize(data.id.value());
                    self.result.extend(br#"">"#);
                } else {
                    self.result.push(b'>');
                }
            }
            #[cfg(not(feature = "block-id"))]
            {
                self.result.push(b'>');
            }
        }

        self.stack.push(tag_name);
    }

    fn write_escaped_html_text(&mut self, input: &[u8]) {
        for char in input {
            match *char {
                b'<' => self.result.extend(b"&lt;"),
                b'&' => self.result.extend(b"&amp;"),
                char => self.result.push(char),
            }
        }
    }

    fn write_escaped_double_quoted_attribute_value(&mut self, input: &[u8]) {
        for char in input {
            match *char {
                b'"' => self.result.extend(b"&quot;"),
                b'&' => self.result.extend(b"&amp;"),
                char => self.result.push(char),
            }
        }
    }

    #[cfg(feature = "block-id")]
    fn write_usize(&mut self, n: usize) {
        let mut buffer = itoa::Buffer::new();
        self.result.extend(buffer.format(n).as_bytes());
    }
}
