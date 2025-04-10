use std::ops::Range;

use rotext_core::{BlockId, Event, events::Call};

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    RecursionDepthExceeded,
}

impl Error {
    pub fn name(&self) -> &'static str {
        match self {
            Error::RecursionDepthExceeded => "RecursionDepthExceeded",
        }
    }
}

pub enum CompiledItem<'a> {
    SimpleEvents(Range<usize>),
    BlockTransclusion(BlockCall<'a>),
    BlockExtension(BlockCall<'a>),
}

pub struct BlockCall<'a> {
    pub name: &'a [u8],
    pub arguments: Vec<(ArgumentKey<'a>, Vec<CompiledItem<'a>>)>,
    pub verbatim_arguments: Vec<(ArgumentKey<'a>, Vec<u8>)>,

    pub block_id: BlockId,
}

#[derive(Eq, Hash, PartialEq)]
pub enum ArgumentKey<'a> {
    Named(&'a [u8]),
    Unnamed(usize),
}
impl ArgumentKey<'_> {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            ArgumentKey::Named(name) => name.to_vec(),
            ArgumentKey::Unnamed(index) => {
                let mut buffer = itoa::Buffer::new();
                buffer.format(*index).as_bytes().to_vec()
            }
        }
    }
}

pub struct NewCompileOptions {
    pub restrictions: Restrictions,
}

pub struct Restrictions {
    /// 单份文档中最多允许的调用（包括最外层的）的嵌套数量。
    pub max_call_depth_in_document: usize,
}

pub struct Compiler<'a> {
    restrictions: &'a Restrictions,
}

impl<'a> Compiler<'a> {
    pub fn new(opts: &'a NewCompileOptions) -> Self {
        Self {
            restrictions: &opts.restrictions,
        }
    }

    pub fn compile(&self, input: &'a [u8], evs: &[Event]) -> Result<Vec<CompiledItem<'a>>> {
        let (_, result) = self.compile_internal(1, input, evs, 0)?;
        Ok(result)
    }

    fn compile_internal(
        &self,
        depth: usize,
        input: &'a [u8],
        evs: &[Event],
        mut i: usize,
    ) -> Result<(usize, Vec<CompiledItem<'a>>)> {
        if depth > self.restrictions.max_call_depth_in_document {
            return Err(Error::RecursionDepthExceeded);
        }

        let mut result: Vec<CompiledItem> = vec![];
        let mut last_simple_evs: Option<Range<usize>> = None;

        let mut stack_depth: usize = 0;

        fn push_simple_events(result: &mut Vec<CompiledItem>, range: &mut Option<Range<usize>>) {
            if let Some(range) = range.take() {
                if !range.is_empty() {
                    result.push(CompiledItem::SimpleEvents(range));
                }
            }
        }
        fn advance_simple_events(range: &mut Option<Range<usize>>, i: &mut usize) {
            let range = range.get_or_insert_with(|| *i..*i);
            *i += 1;
            range.end = *i;
        }

        loop {
            if i >= evs.len() {
                push_simple_events(&mut result, &mut last_simple_evs);
                return Ok((i, result));
            }

            #[rotext_internal_macros::ensure_cases_for_event(
                prefix = Event,
                group = Blend,
            )]
            // NOTE: rust-analyzer 会错误地认为这里的 `match` 没有覆盖到全部分支，
            // 实际上并不存在问题。
            match &evs[i] {
                Event::ExitBlock(_)
                | Event::IndicateCallNormalArgument(_)
                | Event::IndicateCallVerbatimArgument(_)
                    if stack_depth == 0 =>
                {
                    push_simple_events(&mut result, &mut last_simple_evs);
                    return Ok((i, result));
                }
                Event::IndicateCallNormalArgument(_) | Event::IndicateCallVerbatimArgument(_) => {
                    unreachable!()
                }
                Event::EnterCallOnTemplate(call) | Event::EnterCallOnExtension(call) => {
                    let is_transclusion = matches!(evs[i], Event::EnterCallOnTemplate(_));

                    push_simple_events(&mut result, &mut last_simple_evs);

                    let mut arguments = Vec::new();
                    let mut verbatim_arguments = Vec::new();

                    let mut unnamed_arg_name_gen = crate::utils::SequenceGenerator::new(1);
                    let mut unnamed_verbatim_arg_name_gen = crate::utils::SequenceGenerator::new(1);

                    i += 1;
                    loop {
                        match &evs[i] {
                            Event::ExitBlock(_) => match call {
                                Call::Block { id, name } => {
                                    let call_compiled: BlockCall = BlockCall {
                                        name: &input[name.clone()],
                                        arguments,
                                        verbatim_arguments,
                                        block_id: *id,
                                    };

                                    result.push(if is_transclusion {
                                        CompiledItem::BlockTransclusion(call_compiled)
                                    } else {
                                        CompiledItem::BlockExtension(call_compiled)
                                    });
                                    i += 1;
                                    break;
                                }
                            },
                            Event::IndicateCallNormalArgument(arg_name) => {
                                let arg_name = if let Some(arg_name) = arg_name {
                                    ArgumentKey::Named(&input[arg_name.clone()])
                                } else {
                                    ArgumentKey::Unnamed(unnamed_arg_name_gen.next())
                                };

                                let value: Vec<CompiledItem>;
                                (i, value) = self.compile_internal(depth + 1, input, evs, i + 1)?;

                                arguments.push((arg_name, value));
                            }
                            Event::IndicateCallVerbatimArgument(arg_name) => {
                                let arg_name = if let Some(arg_name) = arg_name {
                                    ArgumentKey::Named(&input[arg_name.clone()])
                                } else {
                                    ArgumentKey::Unnamed(unnamed_verbatim_arg_name_gen.next())
                                };

                                let mut value: Vec<u8> = vec![];
                                loop {
                                    i += 1;
                                    match &evs[i] {
                                        Event::Text(content)
                                        | Event::VerbatimEscaping(
                                            rotext_core::events::VerbatimEscaping {
                                                content, ..
                                            },
                                        ) => value.extend(&input[content.clone()]),
                                        Event::NewLine(_) => value.push(b'\n'),
                                        _ => break,
                                    }
                                }

                                verbatim_arguments.push((arg_name, value));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                Event::ExitBlock(_) | Event::ExitInline => {
                    stack_depth -= 1;
                    advance_simple_events(&mut last_simple_evs, &mut i);
                }
                Event::EnterParagraph(_)
                | Event::EnterHeading1(_)
                | Event::EnterHeading2(_)
                | Event::EnterHeading3(_)
                | Event::EnterHeading4(_)
                | Event::EnterHeading5(_)
                | Event::EnterHeading6(_)
                | Event::EnterBlockQuote(_)
                | Event::EnterOrderedList(_)
                | Event::EnterUnorderedList(_)
                | Event::EnterListItem(_)
                | Event::EnterDescriptionList(_)
                | Event::EnterDescriptionTerm(_)
                | Event::EnterDescriptionDetails(_)
                | Event::EnterCodeBlock(_)
                | Event::EnterTable(_)
                | Event::EnterCodeSpan
                | Event::EnterEmphasis
                | Event::EnterStrong
                | Event::EnterStrikethrough
                | Event::EnterRuby
                | Event::EnterRubyText
                | Event::EnterWikiLink(_) => {
                    stack_depth += 1;
                    advance_simple_events(&mut last_simple_evs, &mut i);
                }
                Event::Raw(_)
                | Event::NewLine(_)
                | Event::Text(_)
                | Event::VerbatimEscaping(_)
                | Event::ThematicBreak(_)
                | Event::RefLink(_)
                | Event::Dicexp(_)
                | Event::IndicateTableRow
                | Event::IndicateTableCaption
                | Event::IndicateTableHeaderCell
                | Event::IndicateTableDataCell
                | Event::IndicateCodeBlockCode => {
                    advance_simple_events(&mut last_simple_evs, &mut i);
                }
            }
        }
    }
}
