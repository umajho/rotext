mod tests;

use crate::events::BlockWithId;
use crate::events::VerbatimEscaping;
use crate::Event;

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

pub struct NewHtmlRendererOptions<'a> {
    pub tag_name_map: TagNameMap<'a>,

    pub initial_output_string_capacity: usize,

    #[cfg(feature = "block-id")]
    pub should_include_block_ids: bool,
}

#[derive(Clone)]
pub struct TagNameMap<'a> {
    pub code_block: &'a [u8],

    pub ref_link: &'a [u8],
    pub dicexp: &'a [u8],
    pub wiki_link: &'a [u8],
}
impl Default for TagNameMap<'_> {
    fn default() -> Self {
        Self {
            code_block: b"x-code-block",

            ref_link: b"x-ref-link",
            dicexp: b"x-dicexp",
            wiki_link: b"x-wiki-link",
        }
    }
}

pub struct HtmlRenderer<'a> {
    tag_name_map: TagNameMap<'a>,

    input: &'a [u8],

    #[cfg(feature = "block-id")]
    with_block_id: bool,

    result: Vec<u8>,
}

enum StackEntry<'a> {
    Normal(&'a [u8]),
    Table(TableState),
    WikiLink,
}
enum TableState {
    AtBeginning,
    InCaption,
    InRow,
    InHeaderCell,
    InDataCell,
}
impl From<TableState> for StackEntry<'_> {
    fn from(val: TableState) -> Self {
        StackEntry::Table(val)
    }
}

impl<'a> HtmlRenderer<'a> {
    pub fn new(input: &'a [u8], opts: NewHtmlRendererOptions<'a>) -> Self {
        Self {
            tag_name_map: opts.tag_name_map,
            input,
            #[cfg(feature = "block-id")]
            with_block_id: opts.should_include_block_ids,
            result: Vec::with_capacity(opts.initial_output_string_capacity),
        }
    }

    /// `input_stream` 的迭代对象是属于 `Blend` 分组的事件。
    pub fn render(mut self, mut input_stream: impl Iterator<Item = Event>) -> String {
        let mut stack: Vec<StackEntry> = vec![];

        loop {
            let Some(ev) = input_stream.next() else {
                break;
            };

            if let Some(StackEntry::Table(table_state)) = stack.last_mut() {
                #[rotext_internal_macros::ensure_cases_for_event(
                    prefix = Event,
                    group = Blend,
                )]
                match ev {
                    Event::IndicateTableRow => {
                        match table_state {
                            TableState::AtBeginning => self.result.extend(b"<tr>"),
                            TableState::InCaption => self.result.extend(b"</caption><tr>"),
                            TableState::InRow => self.result.extend(b"</tr><tr>"),
                            TableState::InHeaderCell => self.result.extend(b"</th></tr><tr>"),
                            TableState::InDataCell => self.result.extend(b"</td></tr><tr>"),
                        }
                        *table_state = TableState::InRow;
                        continue;
                    }
                    Event::IndicateTableCaption => {
                        match table_state {
                            TableState::AtBeginning => self.result.extend(b"<caption>"),
                            _ => unreachable!(),
                        }
                        *table_state = TableState::InCaption;
                        continue;
                    }
                    Event::IndicateTableHeaderCell => {
                        match table_state {
                            TableState::AtBeginning => self.result.extend(b"<tr><th>"),
                            TableState::InCaption => self.result.extend(b"</caption><tr><th>"),
                            TableState::InRow => self.result.extend(b"<th>"),
                            TableState::InHeaderCell => self.result.extend(b"</th><th>"),
                            TableState::InDataCell => self.result.extend(b"</td><th>"),
                        }
                        *table_state = TableState::InHeaderCell;
                        continue;
                    }
                    Event::IndicateTableDataCell => {
                        match table_state {
                            TableState::AtBeginning => self.result.extend(b"<tr><td>"),
                            TableState::InCaption => self.result.extend(b"</caption><tr><td>"),
                            TableState::InRow => self.result.extend(b"<td>"),
                            TableState::InHeaderCell => self.result.extend(b"</th><td>"),
                            TableState::InDataCell => self.result.extend(b"</td><td>"),
                        };
                        *table_state = TableState::InDataCell;
                        continue;
                    }
                    Event::ExitBlock(_) => {
                        let top = stack.pop().unwrap();
                        match top {
                            StackEntry::Normal(top) => {
                                self.result.extend(b"</");
                                self.result.extend(top);
                                self.result.push(b'>');
                            }
                            StackEntry::Table(TableState::AtBeginning) => {
                                self.result.extend(b"</table>")
                            }
                            StackEntry::Table(TableState::InCaption) => {
                                self.result.extend(b"</caption></table>")
                            }
                            StackEntry::Table(TableState::InRow) => {
                                self.result.extend(b"</tr></table>")
                            }
                            StackEntry::Table(TableState::InHeaderCell) => {
                                self.result.extend(b"</th></tr></table>")
                            }
                            StackEntry::Table(TableState::InDataCell) => {
                                self.result.extend(b"</td></tr></table>")
                            }
                            _ => unreachable!(),
                        }
                        continue;
                    }
                    _ => match table_state {
                        TableState::AtBeginning => {
                            self.result.extend(b"<tr><td>");
                            *table_state = TableState::InDataCell;
                        }
                        TableState::InRow => {
                            self.result.extend(b"<td>");
                            *table_state = TableState::InDataCell;
                        }
                        _ => {}
                    },
                }
            }

            #[rotext_internal_macros::ensure_cases_for_event(
                prefix = Event,
                group = Blend,
            )]
            // NOTE: rust-analyzer 会错误地认为这里的 `match` 没有覆盖到全部分支，
            // 实际上并不存在问题。
            match ev {
                Event::Raw(content) => self.write_raw_html(&self.input[content]),
                Event::NewLine(_) => self.result.extend(b"<br>"),
                Event::Text(content)
                | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                    self.write_escaped_html_text(&self.input[content]);
                }

                Event::ExitBlock(_) | Event::ExitInline => {
                    let top = stack.pop().unwrap();
                    match top {
                        StackEntry::Normal(top) => {
                            self.result.extend(b"</");
                            self.result.extend(top);
                            self.result.push(b'>');
                        }
                        StackEntry::WikiLink => {
                            self.result.extend(b"</span></");
                            self.result.extend(self.tag_name_map.wiki_link);
                            self.result.push(b'>');
                        }
                        _ => unreachable!(),
                    }
                }

                #[allow(unused_variables)]
                Event::ThematicBreak(data) => {
                    self.result.extend(b"<hr");
                    write_data_block_id_attribute_if_applicable!(self, data);
                    self.result.push(b'>');
                }

                Event::EnterParagraph(data) => self.push_simple_block(&mut stack, b"p", &data),
                Event::EnterHeading1(data) => self.push_simple_block(&mut stack, b"h1", &data),
                Event::EnterHeading2(data) => self.push_simple_block(&mut stack, b"h2", &data),
                Event::EnterHeading3(data) => self.push_simple_block(&mut stack, b"h3", &data),
                Event::EnterHeading4(data) => self.push_simple_block(&mut stack, b"h4", &data),
                Event::EnterHeading5(data) => self.push_simple_block(&mut stack, b"h5", &data),
                Event::EnterHeading6(data) => self.push_simple_block(&mut stack, b"h6", &data),
                Event::EnterBlockQuote(data) => {
                    self.push_simple_block(&mut stack, b"blockquote", &data)
                }
                Event::EnterOrderedList(data) => self.push_simple_block(&mut stack, b"ol", &data),
                Event::EnterUnorderedList(data) => self.push_simple_block(&mut stack, b"ul", &data),
                Event::EnterListItem(data) => self.push_simple_block(&mut stack, b"li", &data),
                Event::EnterDescriptionList(data) => {
                    self.push_simple_block(&mut stack, b"dl", &data)
                }
                Event::EnterDescriptionTerm(data) => {
                    self.push_simple_block(&mut stack, b"dt", &data)
                }
                Event::EnterDescriptionDetails(data) => {
                    self.push_simple_block(&mut stack, b"dd", &data)
                }
                #[allow(unused_variables)]
                Event::EnterCodeBlock(data) => {
                    self.result.push(b'<');
                    self.result.extend(self.tag_name_map.code_block);

                    self.result.extend(br#" info-string=""#);
                    loop {
                        match input_stream.next().unwrap() {
                            Event::Text(content)
                            | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => self
                                .write_escaped_double_quoted_attribute_value(&self.input[content]),
                            Event::IndicateCodeBlockCode => break,
                            _ => unreachable!(),
                        }
                    }

                    self.result.extend(br#"" content=""#);
                    loop {
                        match input_stream.next().unwrap() {
                            Event::Text(content)
                            | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => self
                                .write_escaped_double_quoted_attribute_value(&self.input[content]),
                            Event::NewLine(_) => {
                                self.result.extend(b"&#10;");
                            }
                            Event::ExitBlock(exit_block) => {
                                #[cfg(feature = "block-id")]
                                {
                                    debug_assert_eq!(data.id, exit_block.id);
                                }

                                break;
                            }
                            _ => unreachable!(),
                        }
                    }

                    self.result.push(b'"');

                    write_data_block_id_attribute_if_applicable!(self, data);

                    self.result.extend(b"></");
                    self.result.extend(self.tag_name_map.code_block);
                    self.result.push(b'>');
                }
                #[allow(unused_variables)]
                Event::EnterTable(data) => {
                    self.result.extend(b"<table");
                    write_data_block_id_attribute_if_applicable!(self, data);
                    self.result.push(b'>');
                    stack.push(TableState::AtBeginning.into())
                }

                Event::IndicateCodeBlockCode
                | Event::IndicateTableCaption
                | Event::IndicateTableRow
                | Event::IndicateTableHeaderCell
                | Event::IndicateTableDataCell => unreachable!(),

                Event::RefLink(content) => {
                    self.write_empty_element_with_single_attribute(
                        self.tag_name_map.ref_link,
                        b"address",
                        &self.input[content],
                    );
                }
                Event::Dicexp(content) => {
                    self.write_empty_element_with_single_attribute(
                        self.tag_name_map.dicexp,
                        b"code",
                        &self.input[content],
                    );
                }

                Event::EnterCodeSpan => self.push_simple_inline(&mut stack, b"code"),
                Event::EnterStrong => self.push_simple_inline(&mut stack, b"strong"),
                Event::EnterStrikethrough => self.push_simple_inline(&mut stack, b"s"),
                Event::EnterRuby => self.push_simple_inline(&mut stack, b"ruby"),
                Event::EnterRubyText => self.push_simple_inline(&mut stack, b"rt"),

                Event::EnterWikiLink(address) => {
                    self.write_opening_tag_with_single_attribute(
                        self.tag_name_map.wiki_link,
                        b"address",
                        &self.input[address],
                    );
                    self.write_opening_tag_with_single_attribute(b"span", b"slot", b"content");
                    stack.push(StackEntry::WikiLink);
                }
            }
        }

        debug_assert!(stack.is_empty());

        unsafe { String::from_utf8_unchecked(self.result) }
    }

    fn push_simple_block(
        &mut self,
        stack: &mut Vec<StackEntry>,
        tag_name: &'static [u8],
        #[allow(unused_variables)] data: &BlockWithId,
    ) {
        self.result.push(b'<');
        self.result.extend(tag_name);
        write_data_block_id_attribute_if_applicable!(self, data);
        self.result.push(b'>');

        stack.push(StackEntry::Normal(tag_name));
    }

    fn push_simple_inline(&mut self, stack: &mut Vec<StackEntry>, tag_name: &'static [u8]) {
        self.result.push(b'<');
        self.result.extend(tag_name);
        self.result.push(b'>');

        stack.push(StackEntry::Normal(tag_name));
    }

    fn write_raw_html(&mut self, input: &[u8]) {
        self.result.extend(input);
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

    fn write_opening_tag_with_single_attribute(
        &mut self,
        tag_name: &[u8],
        attr_name: &[u8],
        attr_value: &[u8],
    ) {
        self.result.push(b'<');
        self.result.extend(tag_name);
        self.result.push(b' ');
        self.result.extend(attr_name);
        self.result.extend(br#"=""#);
        self.write_escaped_double_quoted_attribute_value(attr_value);
        self.result.extend(br#"">"#);
    }

    fn write_empty_element_with_single_attribute(
        &mut self,
        tag_name: &[u8],
        attr_name: &[u8],
        attr_value: &[u8],
    ) {
        self.write_opening_tag_with_single_attribute(tag_name, attr_name, attr_value);
        self.result.extend(br#"</"#);
        self.result.extend(tag_name);
        self.result.push(b'>');
    }
}
