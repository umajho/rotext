mod tests;

use crate::events::BlendEvent;
use crate::events::BlockWithID;
use crate::events::VerbatimEscaping;

const TABLE_TR_TH: &[u8] = b"table tr th";
const TABLE_TR_TD: &[u8] = b"table tr td";

macro_rules! write_data_block_id_attribute_if_applicable {
    ($self:ident, $data:ident) => {
        #[cfg(feature = "block-id")]
        {
            if $self.with_block_id {
                $self.write_data_block_id_attribute($data.id.value());
            }
        }
    };
}

pub struct NewHtmlRendererOptoins<'a> {
    pub tag_name_map: TagNameMap<'a>,

    pub initial_output_string_capacity: usize,

    #[cfg(feature = "block-id")]
    pub should_include_block_ids: bool,
}

#[derive(Clone)]
pub struct TagNameMap<'a> {
    pub code_block: &'a [u8],
}
impl<'a> Default for TagNameMap<'a> {
    fn default() -> Self {
        Self {
            code_block: b"x-code-block",
        }
    }
}

pub struct HtmlRenderer<'a> {
    tag_name_map: TagNameMap<'a>,

    input: &'a [u8],

    #[cfg(feature = "block-id")]
    with_block_id: bool,

    result: Vec<u8>,

    should_enter_table_row: bool,
    should_enter_table_cell: bool,
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(input: &'a [u8], opts: NewHtmlRendererOptoins<'a>) -> Self {
        Self {
            tag_name_map: opts.tag_name_map,
            input,
            #[cfg(feature = "block-id")]
            with_block_id: opts.should_include_block_ids,
            result: Vec::with_capacity(opts.initial_output_string_capacity),
            should_enter_table_row: false,
            should_enter_table_cell: false,
        }
    }

    pub fn render(mut self, mut input_stream: impl Iterator<Item = BlendEvent>) -> String {
        let mut stack: Vec<&[u8]> = vec![];

        loop {
            let Some(ev) = input_stream.next() else {
                break;
            };

            if self.should_enter_table_row {
                self.should_enter_table_row = false;
                self.result.extend(b"<tr>");
                self.should_enter_table_cell = true;
                if matches!(ev, BlendEvent::IndicateTableRow) {
                    continue;
                }
            }

            if self.should_enter_table_cell {
                self.should_enter_table_cell = false;
                let (is_th, should_continue) = match &ev {
                    BlendEvent::IndicateTableHeaderCell => (true, true),
                    BlendEvent::IndicateTableDataCell => (false, true),
                    _ => (false, false),
                };
                if is_th {
                    self.result.extend(b"<th>");
                    stack.push(TABLE_TR_TH)
                } else {
                    self.result.extend(b"<td>");
                    stack.push(TABLE_TR_TD)
                }
                if should_continue {
                    continue;
                }
            }

            match ev {
                BlendEvent::NewLine(_) => self.result.extend(b"<br>"),
                BlendEvent::Text(content)
                | BlendEvent::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                    self.write_escaped_html_text(content.content_in_u8_array(self.input));
                }
                #[allow(unused_variables)]
                BlendEvent::ThematicBreak(data) => {
                    self.result.extend(b"<hr");
                    write_data_block_id_attribute_if_applicable!(self, data);
                    self.result.push(b'>');
                }

                BlendEvent::ExitBlock(_) => {
                    let top = stack.pop().unwrap();
                    match top {
                        TABLE_TR_TH => self.result.extend(b"</th></tr></table>"),
                        TABLE_TR_TD => self.result.extend(b"</td></tr></table>"),
                        _ => {
                            self.result.extend(b"</");
                            self.result.extend(top);
                            self.result.push(b'>');
                        }
                    }
                }

                BlendEvent::EnterParagraph(data) => self.push_simple(&mut stack, b"p", &data),
                BlendEvent::EnterHeading1(data) => self.push_simple(&mut stack, b"h1", &data),
                BlendEvent::EnterHeading2(data) => self.push_simple(&mut stack, b"h2", &data),
                BlendEvent::EnterHeading3(data) => self.push_simple(&mut stack, b"h3", &data),
                BlendEvent::EnterHeading4(data) => self.push_simple(&mut stack, b"h4", &data),
                BlendEvent::EnterHeading5(data) => self.push_simple(&mut stack, b"h5", &data),
                BlendEvent::EnterHeading6(data) => self.push_simple(&mut stack, b"h6", &data),
                BlendEvent::EnterBlockQuote(data) => {
                    self.push_simple(&mut stack, b"blockquote", &data)
                }
                BlendEvent::EnterOrderedList(data) => self.push_simple(&mut stack, b"ol", &data),
                BlendEvent::EnterUnorderedList(data) => self.push_simple(&mut stack, b"ul", &data),
                BlendEvent::EnterListItem(data) => self.push_simple(&mut stack, b"li", &data),
                BlendEvent::EnterDescriptionList(data) => {
                    self.push_simple(&mut stack, b"dl", &data)
                }
                BlendEvent::EnterDescriptionTerm(data) => {
                    self.push_simple(&mut stack, b"dt", &data)
                }
                BlendEvent::EnterDescriptionDetails(data) => {
                    self.push_simple(&mut stack, b"dd", &data)
                }
                #[allow(unused_variables)]
                BlendEvent::EnterCodeBlock(data) => {
                    self.result.push(b'<');
                    self.result.extend(self.tag_name_map.code_block);
                    self.result.extend(br#" info-string=""#);
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
                    self.result.push(b'"');

                    write_data_block_id_attribute_if_applicable!(self, data);

                    self.result.push(b'>');
                    stack.push(self.tag_name_map.code_block);
                }
                #[allow(unused_variables)]
                BlendEvent::EnterTable(data) => {
                    self.result.push(b'<');
                    self.result.extend(b"table");
                    write_data_block_id_attribute_if_applicable!(self, data);
                    self.result.push(b'>');
                    self.should_enter_table_row = true;
                }

                BlendEvent::IndicateCodeBlockCode => unreachable!(),
                BlendEvent::IndicateTableRow => {
                    self.pop_stack_and_write_table_cell_closing(&mut stack);
                    self.result.extend(b"</tr><tr>");
                    self.should_enter_table_cell = true;
                }
                BlendEvent::IndicateTableHeaderCell => {
                    self.pop_stack_and_write_table_cell_closing(&mut stack);
                    self.result.extend(b"<th>");
                    stack.push(TABLE_TR_TH);
                }
                BlendEvent::IndicateTableDataCell => {
                    self.pop_stack_and_write_table_cell_closing(&mut stack);
                    self.result.extend(b"<td>");
                    stack.push(TABLE_TR_TD);
                }
            };
        }

        unsafe { String::from_utf8_unchecked(self.result) }
    }

    fn push_simple(
        &mut self,
        stack: &mut Vec<&[u8]>,
        tag_name: &'static [u8],
        #[allow(unused_variables)] data: &BlockWithID,
    ) {
        self.result.push(b'<');
        self.result.extend(tag_name);
        write_data_block_id_attribute_if_applicable!(self, data);
        self.result.push(b'>');

        stack.push(tag_name);
    }

    fn pop_stack_and_write_table_cell_closing(&mut self, stack: &mut Vec<&[u8]>) {
        match stack.pop().unwrap() {
            TABLE_TR_TH => self.result.extend(b"</th>"),
            TABLE_TR_TD => self.result.extend(b"</td>"),
            _ => unreachable!(),
        }
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
    fn write_data_block_id_attribute(&mut self, id: usize) {
        self.result.extend(br#" data-block-id=""#);
        self.write_usize(id);
        self.result.push(b'"');
    }

    #[cfg(feature = "block-id")]
    fn write_usize(&mut self, n: usize) {
        let mut buffer = itoa::Buffer::new();
        self.result.extend(buffer.format(n).as_bytes());
    }
}
