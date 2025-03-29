use rotext_core::{
    events::{BlockWithId, VerbatimEscaping},
    Event,
};

macro_rules! write_data_block_id_attribute_if_applicable {
    ($self:ident, $buf:ident, $data:ident) => {
        #[cfg(feature = "block-id")]
        {
            if $self.with_block_id {
                render_data_block_id_attribute($buf, $data.id.value());
            }
        }
    };
}

pub struct NewRendererOptions<'a> {
    pub tag_name_map: &'a TagNameMap<'a>,

    #[cfg(feature = "block-id")]
    pub should_include_block_ids: bool,
}

/// XXX: 调用者需自行确保各标签的名称不会导致 XSS。
#[derive(Clone)]
pub struct TagNameMap<'a> {
    pub block_call_error: &'a [u8],

    pub code_block: &'a [u8],

    pub ref_link: &'a [u8],
    pub dicexp: &'a [u8],
    pub wiki_link: &'a [u8],
}

pub struct Renderer<'a> {
    tag_name_map: &'a TagNameMap<'a>,

    #[cfg(feature = "block-id")]
    with_block_id: bool,
}

pub struct StackEntryBox<'a>(StackEntry<'a>);

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

impl<'a> Renderer<'a> {
    pub fn new(opts: NewRendererOptions<'a>) -> Self {
        Self {
            tag_name_map: opts.tag_name_map,
            #[cfg(feature = "block-id")]
            with_block_id: opts.should_include_block_ids,
        }
    }

    /// `input_stream` 的迭代对象是属于 `Blend` 分组的事件。
    pub fn render_events(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        evs: &[Event],
        stack: &mut Vec<StackEntryBox>,
    ) {
        let mut i = 0;

        while i < evs.len() {
            i = self.render_event(buf, input, evs, i, stack);
        }
    }

    /// `input_stream` 的迭代对象是属于 `Blend` 分组的事件。
    fn render_event(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        evs: &[Event],
        mut i: usize,
        stack: &mut Vec<StackEntryBox>,
    ) -> usize {
        let ev = &evs[i];

        if let Some(()) = self.render_table_related_event(buf, ev, stack) {
            return i + 1;
        }

        #[rotext_internal_macros::ensure_cases_for_event(
                prefix = Event,
                group = Blend,
            )]
        // NOTE: rust-analyzer 会错误地认为这里的 `match` 没有覆盖到全部分支，
        // 实际上并不存在问题。
        match ev {
            Event::Raw(content) => render_raw_html(buf, &input[content.clone()]),
            Event::NewLine(_) => buf.extend(b"<br>"),
            Event::Text(content) | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                crate::utils::render_escaped_html_text(buf, &input[content.clone()]);
            }
            Event::ExitBlock(_) | Event::ExitInline => {
                let top = stack.pop().unwrap();
                match top.0 {
                    StackEntry::Normal(top) => {
                        buf.extend(b"</");
                        buf.extend(top);
                        buf.push(b'>');
                    }
                    StackEntry::WikiLink => {
                        buf.extend(b"</span></");
                        buf.extend(self.tag_name_map.wiki_link);
                        buf.push(b'>');
                    }
                    _ => unreachable!(),
                }
            }
            #[allow(unused_variables)]
            Event::ThematicBreak(data) => {
                buf.extend(b"<hr");
                write_data_block_id_attribute_if_applicable!(self, buf, data);
                buf.push(b'>');
            }
            Event::EnterParagraph(data) => self.push_simple_block(buf, stack, b"p", data),
            Event::EnterHeading1(data) => self.push_simple_block(buf, stack, b"h1", data),
            Event::EnterHeading2(data) => self.push_simple_block(buf, stack, b"h2", data),
            Event::EnterHeading3(data) => self.push_simple_block(buf, stack, b"h3", data),
            Event::EnterHeading4(data) => self.push_simple_block(buf, stack, b"h4", data),
            Event::EnterHeading5(data) => self.push_simple_block(buf, stack, b"h5", data),
            Event::EnterHeading6(data) => self.push_simple_block(buf, stack, b"h6", data),
            Event::EnterBlockQuote(data) => self.push_simple_block(buf, stack, b"blockquote", data),
            Event::EnterOrderedList(data) => self.push_simple_block(buf, stack, b"ol", data),
            Event::EnterUnorderedList(data) => self.push_simple_block(buf, stack, b"ul", data),
            Event::EnterListItem(data) => self.push_simple_block(buf, stack, b"li", data),
            Event::EnterDescriptionList(data) => self.push_simple_block(buf, stack, b"dl", data),
            Event::EnterDescriptionTerm(data) => self.push_simple_block(buf, stack, b"dt", data),
            Event::EnterDescriptionDetails(data) => self.push_simple_block(buf, stack, b"dd", data),
            #[allow(unused_variables)]
            Event::EnterCodeBlock(data) => {
                buf.push(b'<');
                buf.extend(self.tag_name_map.code_block);

                buf.extend(br#" info-string=""#);
                loop {
                    i += 1;
                    match &evs[i] {
                        Event::Text(content)
                        | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                            crate::utils::render_escaped_double_quoted_attribute_value(
                                buf,
                                &input[content.clone()],
                            )
                        }
                        Event::IndicateCodeBlockCode => break,
                        _ => unreachable!(),
                    }
                }

                buf.extend(br#"" content=""#);
                loop {
                    i += 1;
                    match &evs[i] {
                        Event::Text(content)
                        | Event::VerbatimEscaping(VerbatimEscaping { content, .. }) => {
                            crate::utils::render_escaped_double_quoted_attribute_value(
                                buf,
                                &input[content.clone()],
                            )
                        }
                        Event::NewLine(_) => {
                            buf.extend(b"&#10;");
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

                buf.push(b'"');

                write_data_block_id_attribute_if_applicable!(self, buf, data);

                buf.extend(b"></");
                buf.extend(self.tag_name_map.code_block);
                buf.push(b'>');
            }
            #[allow(unused_variables)]
            Event::EnterTable(data) => {
                buf.extend(b"<table");
                write_data_block_id_attribute_if_applicable!(self, buf, data);
                buf.push(b'>');
                stack.push(StackEntryBox(TableState::AtBeginning.into()))
            }
            Event::EnterCallOnTemplate(_)
            | Event::EnterCallOnExtension(_)
            | Event::IndicateCodeBlockCode
            | Event::IndicateTableCaption
            | Event::IndicateTableRow
            | Event::IndicateTableHeaderCell
            | Event::IndicateTableDataCell
            | Event::IndicateCallNormalArgument(_)
            | Event::IndicateCallVerbatimArgument(_) => unreachable!(),
            Event::RefLink(content) => {
                crate::utils::render_empty_element(
                    buf,
                    self.tag_name_map.ref_link,
                    &[(b"address", &input[content.clone()])],
                );
            }
            Event::Dicexp(content) => {
                crate::utils::render_empty_element(
                    buf,
                    self.tag_name_map.dicexp,
                    &[(b"code", &input[content.clone()])],
                );
            }
            Event::EnterCodeSpan => self.push_simple_inline(buf, stack, b"code"),
            Event::EnterEmphasis => self.push_simple_inline(buf, stack, b"em"),
            Event::EnterStrong => self.push_simple_inline(buf, stack, b"strong"),
            Event::EnterStrikethrough => self.push_simple_inline(buf, stack, b"s"),
            Event::EnterRuby => self.push_simple_inline(buf, stack, b"ruby"),
            Event::EnterRubyText => self.push_simple_inline(buf, stack, b"rt"),
            Event::EnterWikiLink(address) => {
                crate::utils::render_eopening_tag(
                    buf,
                    self.tag_name_map.wiki_link,
                    &[(b"address", &input[address.clone()])],
                );
                crate::utils::render_eopening_tag(buf, b"span", &[(b"slot", b"content")]);
                stack.push(StackEntryBox(StackEntry::WikiLink));
            }
        }

        i + 1
    }

    fn render_table_related_event(
        &self,
        buf: &mut Vec<u8>,
        ev: &Event,
        stack: &mut Vec<StackEntryBox>,
    ) -> Option<()> {
        if let Some(StackEntryBox(StackEntry::Table(table_state))) = stack.last_mut() {
            #[rotext_internal_macros::ensure_cases_for_event(
                    prefix = Event,
                    group = Blend,
                )]
            match ev {
                Event::IndicateTableRow => {
                    match table_state {
                        TableState::AtBeginning => buf.extend(b"<tr>"),
                        TableState::InCaption => buf.extend(b"</caption><tr>"),
                        TableState::InRow => buf.extend(b"</tr><tr>"),
                        TableState::InHeaderCell => buf.extend(b"</th></tr><tr>"),
                        TableState::InDataCell => buf.extend(b"</td></tr><tr>"),
                    }
                    *table_state = TableState::InRow;
                }
                Event::IndicateTableCaption => {
                    match table_state {
                        TableState::AtBeginning => buf.extend(b"<caption>"),
                        _ => unreachable!(),
                    }
                    *table_state = TableState::InCaption;
                }
                Event::IndicateTableHeaderCell => {
                    match table_state {
                        TableState::AtBeginning => buf.extend(b"<tr><th>"),
                        TableState::InCaption => buf.extend(b"</caption><tr><th>"),
                        TableState::InRow => buf.extend(b"<th>"),
                        TableState::InHeaderCell => buf.extend(b"</th><th>"),
                        TableState::InDataCell => buf.extend(b"</td><th>"),
                    }
                    *table_state = TableState::InHeaderCell;
                }
                Event::IndicateTableDataCell => {
                    match table_state {
                        TableState::AtBeginning => buf.extend(b"<tr><td>"),
                        TableState::InCaption => buf.extend(b"</caption><tr><td>"),
                        TableState::InRow => buf.extend(b"<td>"),
                        TableState::InHeaderCell => buf.extend(b"</th><td>"),
                        TableState::InDataCell => buf.extend(b"</td><td>"),
                    };
                    *table_state = TableState::InDataCell;
                }
                Event::ExitBlock(_) => {
                    let top = stack.pop().unwrap().0;
                    match top {
                        StackEntry::Normal(top) => {
                            buf.extend(b"</");
                            buf.extend(top);
                            buf.push(b'>');
                        }
                        StackEntry::Table(TableState::AtBeginning) => buf.extend(b"</table>"),
                        StackEntry::Table(TableState::InCaption) => {
                            buf.extend(b"</caption></table>")
                        }
                        StackEntry::Table(TableState::InRow) => buf.extend(b"</tr></table>"),
                        StackEntry::Table(TableState::InHeaderCell) => {
                            buf.extend(b"</th></tr></table>")
                        }
                        StackEntry::Table(TableState::InDataCell) => {
                            buf.extend(b"</td></tr></table>")
                        }
                        _ => unreachable!(),
                    }
                }
                _ => {
                    match table_state {
                        TableState::AtBeginning => {
                            buf.extend(b"<tr><td>");
                            *table_state = TableState::InDataCell;
                        }
                        TableState::InRow => {
                            buf.extend(b"<td>");
                            *table_state = TableState::InDataCell;
                        }
                        _ => {}
                    }
                    return None;
                }
            }
            Some(())
        } else {
            None
        }
    }

    fn push_simple_block(
        &self,
        buf: &mut Vec<u8>,
        stack: &mut Vec<StackEntryBox>,
        tag_name: &'static [u8],
        #[allow(unused_variables)] data: &BlockWithId,
    ) {
        buf.push(b'<');
        buf.extend(tag_name);
        write_data_block_id_attribute_if_applicable!(self, buf, data);
        buf.push(b'>');

        stack.push(StackEntryBox(StackEntry::Normal(tag_name)));
    }

    fn push_simple_inline(
        &self,
        buf: &mut Vec<u8>,
        stack: &mut Vec<StackEntryBox>,
        tag_name: &'static [u8],
    ) {
        buf.push(b'<');
        buf.extend(tag_name);
        buf.push(b'>');

        stack.push(StackEntryBox(StackEntry::Normal(tag_name)));
    }
}

fn render_raw_html(buf: &mut Vec<u8>, input: &[u8]) {
    buf.extend(input);
}

#[cfg(feature = "block-id")]
fn render_data_block_id_attribute(buf: &mut Vec<u8>, id: usize) {
    buf.extend(br#" data-block-id=""#);
    crate::utils::write_usize(buf, id);
    buf.push(b'"');
}
