mod tests;

use crate::events::BlockWithId;
use crate::events::VerbatimEscaping;
use crate::Event;

macro_rules! write_data_block_id_attribute_if_applicable {
    ($self:ident, $data:ident) => {
        #[cfg(feature = "block-id")]
        {
            if $self.with_block_id {
                $self.write_data_block_id_attribute($data.id.value())?;
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
    pub internal_link: &'a [u8],
}
impl<'a> Default for TagNameMap<'a> {
    fn default() -> Self {
        Self {
            code_block: b"x-code-block",

            ref_link: b"x-ref-link",
            dicexp: b"x-dicexp",
            internal_link: b"x-internal-link",
        }
    }
}

pub struct HtmlRenderer<'a, W: std::io::Write> {
    tag_name_map: TagNameMap<'a>,

    input: &'a [u8],

    #[cfg(feature = "block-id")]
    with_block_id: bool,

    writer: W,
}

enum StackEntry<'a> {
    Normal(&'a [u8]),
    Table(TableState),
    InternalLink,
}
enum TableState {
    AtBeginning,
    InCaption,
    InRow,
    InHeaderCell,
    InDataCell,
}
impl<'a> From<TableState> for StackEntry<'a> {
    fn from(val: TableState) -> Self {
        StackEntry::Table(val)
    }
}

impl<'a, W: std::io::Write> HtmlRenderer<'a, W> {
    pub fn new(w: W, input: &'a [u8], opts: NewHtmlRendererOptions<'a>) -> Self {
        Self {
            tag_name_map: opts.tag_name_map,
            input,
            #[cfg(feature = "block-id")]
            with_block_id: opts.should_include_block_ids,
            writer: w,
        }
    }

    /// `input_stream` 的迭代对象是属于 `Blend` 分组的事件。
    fn render(mut self, mut input_stream: impl Iterator<Item = Event>) -> std::io::Result<()> {
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
                            TableState::AtBeginning => self.writer.write_all(b"<tr>")?,
                            TableState::InCaption => self.writer.write_all(b"</caption><tr>")?,
                            TableState::InRow => self.writer.write_all(b"</tr><tr>")?,
                            TableState::InHeaderCell => self.writer.write_all(b"</th></tr><tr>")?,
                            TableState::InDataCell => self.writer.write_all(b"</td></tr><tr>")?,
                        }
                        *table_state = TableState::InRow;
                        continue;
                    }
                    Event::IndicateTableCaption => {
                        match table_state {
                            TableState::AtBeginning => self.writer.write_all(b"<caption>")?,
                            _ => unreachable!(),
                        }
                        *table_state = TableState::InCaption;
                        continue;
                    }
                    Event::IndicateTableHeaderCell => {
                        match table_state {
                            TableState::AtBeginning => self.writer.write_all(b"<tr><th>")?,
                            TableState::InCaption => {
                                self.writer.write_all(b"</caption><tr><th>")?
                            }
                            TableState::InRow => self.writer.write_all(b"<th>")?,
                            TableState::InHeaderCell => self.writer.write_all(b"</th><th>")?,
                            TableState::InDataCell => self.writer.write_all(b"</td><th>")?,
                        }
                        *table_state = TableState::InHeaderCell;
                        continue;
                    }
                    Event::IndicateTableDataCell => {
                        match table_state {
                            TableState::AtBeginning => self.writer.write_all(b"<tr><td>")?,
                            TableState::InCaption => {
                                self.writer.write_all(b"</caption><tr><td>")?
                            }
                            TableState::InRow => self.writer.write_all(b"<td>")?,
                            TableState::InHeaderCell => self.writer.write_all(b"</th><td>")?,
                            TableState::InDataCell => self.writer.write_all(b"</td><td>")?,
                        };
                        *table_state = TableState::InDataCell;
                        continue;
                    }
                    Event::ExitBlock(_) => {
                        let top = stack.pop().unwrap();
                        match top {
                            StackEntry::Normal(top) => {
                                self.writer.write_all(b"</")?;
                                self.writer.write_all(top)?;
                                self.writer.write_all(b">")?;
                            }
                            StackEntry::Table(TableState::AtBeginning) => {
                                self.writer.write_all(b"</table>")?
                            }
                            StackEntry::Table(TableState::InCaption) => {
                                self.writer.write_all(b"</caption></table>")?
                            }
                            StackEntry::Table(TableState::InRow) => {
                                self.writer.write_all(b"</tr></table>")?
                            }
                            StackEntry::Table(TableState::InHeaderCell) => {
                                self.writer.write_all(b"</th></tr></table>")?
                            }
                            StackEntry::Table(TableState::InDataCell) => {
                                self.writer.write_all(b"</td></tr></table>")?
                            }
                            _ => unreachable!(),
                        }
                        continue;
                    }
                    _ => match table_state {
                        TableState::AtBeginning => {
                            self.writer.write_all(b"<tr><td>")?;
                            *table_state = TableState::InDataCell;
                        }
                        TableState::InRow => {
                            self.writer.write_all(b"<td>")?;
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
                Event::Raw(content) => self.write_raw_html(&self.input[content])?,
                Event::NewLine(_) => self.writer.write_all(b"<br>")?,
                Event::Text(content)
                | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                    self.write_escaped_html_text(&self.input[content])?;
                }

                Event::ExitBlock(_) | Event::ExitInline => {
                    let top = stack.pop().unwrap();
                    match top {
                        StackEntry::Normal(top) => {
                            self.writer.write_all(b"</")?;
                            self.writer.write_all(top)?;
                            self.writer.write_all(b">")?;
                        }
                        StackEntry::InternalLink => {
                            self.writer.write_all(b"</span></")?;
                            self.writer.write_all(self.tag_name_map.internal_link)?;
                            self.writer.write_all(b">")?;
                        }
                        _ => unreachable!(),
                    }
                }

                #[allow(unused_variables)]
                Event::ThematicBreak(data) => {
                    self.writer.write_all(b"<hr")?;
                    write_data_block_id_attribute_if_applicable!(self, data);
                    self.writer.write_all(b">")?;
                }

                Event::EnterParagraph(data) => self.push_simple_block(&mut stack, b"p", &data)?,
                Event::EnterHeading1(data) => self.push_simple_block(&mut stack, b"h1", &data)?,
                Event::EnterHeading2(data) => self.push_simple_block(&mut stack, b"h2", &data)?,
                Event::EnterHeading3(data) => self.push_simple_block(&mut stack, b"h3", &data)?,
                Event::EnterHeading4(data) => self.push_simple_block(&mut stack, b"h4", &data)?,
                Event::EnterHeading5(data) => self.push_simple_block(&mut stack, b"h5", &data)?,
                Event::EnterHeading6(data) => self.push_simple_block(&mut stack, b"h6", &data)?,
                Event::EnterBlockQuote(data) => {
                    self.push_simple_block(&mut stack, b"blockquote", &data)?
                }
                Event::EnterOrderedList(data) => {
                    self.push_simple_block(&mut stack, b"ol", &data)?
                }
                Event::EnterUnorderedList(data) => {
                    self.push_simple_block(&mut stack, b"ul", &data)?
                }
                Event::EnterListItem(data) => self.push_simple_block(&mut stack, b"li", &data)?,
                Event::EnterDescriptionList(data) => {
                    self.push_simple_block(&mut stack, b"dl", &data)?
                }
                Event::EnterDescriptionTerm(data) => {
                    self.push_simple_block(&mut stack, b"dt", &data)?
                }
                Event::EnterDescriptionDetails(data) => {
                    self.push_simple_block(&mut stack, b"dd", &data)?
                }
                #[allow(unused_variables)]
                Event::EnterCodeBlock(data) => {
                    self.writer.write_all(b"<")?;
                    self.writer.write_all(self.tag_name_map.code_block)?;

                    self.writer.write_all(br#" info-string=""#)?;
                    loop {
                        match input_stream.next().unwrap() {
                            Event::Text(content)
                            | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => self
                                .write_escaped_double_quoted_attribute_value(
                                    &self.input[content],
                                )?,
                            Event::IndicateCodeBlockCode => break,
                            _ => unreachable!(),
                        }
                    }

                    self.writer.write_all(br#"" content=""#)?;
                    loop {
                        match input_stream.next().unwrap() {
                            Event::Text(content)
                            | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => self
                                .write_escaped_double_quoted_attribute_value(
                                    &self.input[content],
                                )?,
                            Event::NewLine(_) => {
                                self.writer.write_all(b"&#10;")?;
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

                    self.writer.write_all(b"\"")?;

                    write_data_block_id_attribute_if_applicable!(self, data);

                    self.writer.write_all(b"></")?;
                    self.writer.write_all(self.tag_name_map.code_block)?;
                    self.writer.write_all(b">")?;
                }
                #[allow(unused_variables)]
                Event::EnterTable(data) => {
                    self.writer.write_all(b"<table")?;
                    write_data_block_id_attribute_if_applicable!(self, data);
                    self.writer.write_all(b">")?;
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
                    )?;
                }
                Event::Dicexp(content) => {
                    self.write_empty_element_with_single_attribute(
                        self.tag_name_map.dicexp,
                        b"code",
                        &self.input[content],
                    )?;
                }

                Event::EnterCodeSpan => self.push_simple_inline(&mut stack, b"code")?,
                Event::EnterStrong => self.push_simple_inline(&mut stack, b"strong")?,
                Event::EnterStrikethrough => self.push_simple_inline(&mut stack, b"s")?,

                Event::EnterInternalLink(address) => {
                    self.write_opening_tag_with_single_attribute(
                        self.tag_name_map.internal_link,
                        b"address",
                        &self.input[address],
                    )?;
                    self.write_opening_tag_with_single_attribute(b"span", b"slot", b"content")?;
                    stack.push(StackEntry::InternalLink);
                }
            }
        }

        debug_assert!(stack.is_empty());

        Ok(())
    }

    fn push_simple_block(
        &mut self,
        stack: &mut Vec<StackEntry>,
        tag_name: &'static [u8],
        #[allow(unused_variables)] data: &BlockWithId,
    ) -> std::io::Result<()> {
        self.writer.write_all(b"<")?;
        self.writer.write_all(tag_name)?;
        write_data_block_id_attribute_if_applicable!(self, data);
        self.writer.write_all(b">")?;

        stack.push(StackEntry::Normal(tag_name));

        Ok(())
    }

    fn push_simple_inline(
        &mut self,
        stack: &mut Vec<StackEntry>,
        tag_name: &'static [u8],
    ) -> std::io::Result<()> {
        self.writer.write_all(b"<")?;
        self.writer.write_all(tag_name)?;
        self.writer.write_all(b">")?;

        stack.push(StackEntry::Normal(tag_name));

        Ok(())
    }

    fn write_raw_html(&mut self, input: &[u8]) -> std::io::Result<()> {
        self.writer.write_all(input)?;

        Ok(())
    }

    fn write_escaped_html_text(&mut self, input: &[u8]) -> std::io::Result<()> {
        for char in input {
            match *char {
                b'<' => self.writer.write_all(b"&lt;")?,
                b'&' => self.writer.write_all(b"&amp;")?,
                char => self.writer.write_all(&[char])?,
            }
        }

        Ok(())
    }

    fn write_escaped_double_quoted_attribute_value(&mut self, input: &[u8]) -> std::io::Result<()> {
        for char in input {
            match *char {
                b'"' => self.writer.write_all(b"&quot;")?,
                b'&' => self.writer.write_all(b"&amp;")?,
                char => self.writer.write_all(&[char])?,
            }
        }

        Ok(())
    }

    #[cfg(feature = "block-id")]
    fn write_data_block_id_attribute(&mut self, id: usize) -> std::io::Result<()> {
        self.writer.write_all(br#" data-block-id=""#)?;
        self.write_usize(id)?;
        self.writer.write_all(b"\"")?;

        Ok(())
    }

    #[cfg(feature = "block-id")]
    fn write_usize(&mut self, n: usize) -> std::io::Result<()> {
        let mut buffer = itoa::Buffer::new();
        self.writer.write_all(buffer.format(n).as_bytes())?;

        Ok(())
    }

    fn write_opening_tag_with_single_attribute(
        &mut self,
        tag_name: &[u8],
        attr_name: &[u8],
        attr_value: &[u8],
    ) -> std::io::Result<()> {
        self.writer.write_all(b"<")?;
        self.writer.write_all(tag_name)?;
        self.writer.write_all(b" ")?;
        self.writer.write_all(attr_name)?;
        self.writer.write_all(br#"=""#)?;
        self.write_escaped_double_quoted_attribute_value(attr_value)?;
        self.writer.write_all(br#"">"#)?;

        Ok(())
    }

    fn write_empty_element_with_single_attribute(
        &mut self,
        tag_name: &[u8],
        attr_name: &[u8],
        attr_value: &[u8],
    ) -> std::io::Result<()> {
        self.write_opening_tag_with_single_attribute(tag_name, attr_name, attr_value)?;
        self.writer.write_all(br#"</"#)?;
        self.writer.write_all(tag_name)?;
        self.writer.write_all(b">")?;

        Ok(())
    }
}

pub struct SimpleHtmlRenderer {}

impl SimpleHtmlRenderer {
    pub fn render_as_string<'a>(
        input: &'a [u8],
        input_stream: impl Iterator<Item = Event>,
        opts: NewHtmlRendererOptions<'a>,
    ) -> std::io::Result<String> {
        let mut buf: Vec<u8> = vec![];
        let renderer = HtmlRenderer::new(&mut buf, input, opts);
        renderer.render(input_stream)?;
        Ok(unsafe { String::from_utf8_unchecked(buf) })
    }
}
