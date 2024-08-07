use std::ops::Range;

use crate::{
    block::{
        context::Context,
        global_mapper,
        sub_parsers::{self, utils::consume_peeked},
    },
    common::m,
    events::{BlockEvent, NewLine},
};

use super::HaveMet;

#[derive(Debug)]
pub enum State {
    ExpectingNewContent(StateExpectingNewContent),
    ExpectingContentNextChar {
        content: Range<usize>,
        spaces_after: usize,
    },

    ToOutputDone(HaveMet),

    Invalid,
}
impl Default for State {
    fn default() -> Self {
        StateExpectingNewContent::default().into()
    }
}
#[derive(Debug, Default)]
pub struct StateExpectingNewContent {
    new_line: Option<NewLine>,
    skipped_spaces: usize,
}
impl From<StateExpectingNewContent> for State {
    fn from(value: StateExpectingNewContent) -> Self {
        Self::ExpectingNewContent(value)
    }
}

#[allow(clippy::enum_variant_names)]
enum InternalOutput {
    /// 继续 next 中的循环。
    ToContinue,
    /// 改变 next 内部的状态，并继续循环。
    ToContinueIn(State),
    /// 打破 next 中的循环，输出 [BlockEvent]。
    ToYield(BlockEvent),
    /// 由于遇到了 LF，打破 next 中的循环，通知外部暂停本解析器的解析。遇到的 LF
    /// 应由外部负责产出。当外部认为可以恢复本解析器的解析时，那之后第一次调用
    /// [Parser::next] 时对应于 `is_resumed_from_line_feed` 位置的参数应该填
    /// `true`。
    ToPauseForNewLine,

    /// 打破 next 中的循环，并向外部表明本解析器的解析到此为止。不产出任何事件。
    ToBeDone(HaveMet),
    ToYieldAndBeDone(BlockEvent, HaveMet),
}

pub struct Parser {
    inner: ParserInner,

    state: State,
}
struct ParserInner {
    mode: Mode,
    end_conditions: EndConditions,
    indentation: usize,

    is_at_first_line: bool,
    has_yielded_at_current_line: bool,
}

#[derive(Default)]
pub struct Options {
    pub initial_state: State,
    pub mode: Mode,
    pub end_conditions: EndConditions,
    /// 逐字模式下，每行开头至多忽略此数量的空格。
    ///
    /// 不应在行内模式下设置为非 0 值。
    pub indentation: usize,
}

pub enum Mode {
    /// 行内模式，无视行首空白。
    Inline,
    /// 逐字模式，保留行首空白。
    Verbatim,
}
impl Default for Mode {
    fn default() -> Self {
        Self::Inline
    }
}

#[derive(Default)]
pub struct EndConditions {
    pub before_new_line: bool,
    pub before_blank_line: bool,
    pub after_repetitive_characters: Option<RepetitiveCharactersCondition>,

    pub on_table_related: Option<TableRelatedCondition>,
}

pub struct RepetitiveCharactersCondition {
    /// 如果是在行首，则满足条件。
    pub at_line_beginning: bool,
    /// 如果是在行尾，且之前有一个空白，则满足条件。
    ///
    /// XXX: 只应在模式为行内时启用，并非为逐字模式准备。
    pub at_line_end_and_with_space_before: bool,

    pub character: u8,
    /// 如果是对应于 `at_line_beginning` 的判断，则是最少需要的数量；如果对应于
    /// `at_line_end_and_with_space_before` 的判断，则是准确需要的数量。
    pub minimal_count: usize,
}

pub struct TableRelatedCondition {
    pub is_caption_applicable: bool,
}

macro_rules! done {
    () => {
        InternalOutput::ToBeDone(HaveMet::None)
    };
    ($have_met:expr) => {
        InternalOutput::ToBeDone($have_met)
    };
}

impl Parser {
    pub fn new(options: Options) -> Self {
        Self {
            inner: ParserInner {
                mode: options.mode,
                end_conditions: options.end_conditions,
                indentation: options.indentation,
                is_at_first_line: true,
                has_yielded_at_current_line: false,
            },
            state: options.initial_state,
        }
    }

    pub fn next(&mut self, ctx: &mut Context) -> sub_parsers::Output {
        let output = loop {
            let result = match &mut self.state {
                State::ExpectingNewContent(state) => {
                    Self::process_in_expecting_new_content_state(ctx, &mut self.inner, state)
                }

                State::ExpectingContentNextChar {
                    content,
                    spaces_after,
                } => Self::process_in_expecting_content_next_char_state(
                    ctx,
                    &self.inner,
                    content,
                    spaces_after,
                ),
                State::ToOutputDone(have_met) => InternalOutput::ToBeDone(*have_met),
                State::Invalid => unreachable!(),
            };

            match result {
                InternalOutput::ToContinue => {}
                InternalOutput::ToContinueIn(state) => self.state = state,
                InternalOutput::ToYield(to_yield) => {
                    self.inner.has_yielded_at_current_line = true;
                    break sub_parsers::Output::ToYield(to_yield);
                }
                InternalOutput::ToPauseForNewLine => {
                    self.inner.is_at_first_line = false;
                    self.inner.has_yielded_at_current_line = false;
                    break sub_parsers::Output::ToPauseForNewLine;
                }
                InternalOutput::ToBeDone(have_met) => {
                    self.state = State::Invalid;
                    return sub_parsers::Output::Done(have_met);
                }
                InternalOutput::ToYieldAndBeDone(ev, have_met) => {
                    self.state = State::ToOutputDone(have_met);
                    return sub_parsers::Output::ToYield(ev);
                }
            }
        };

        if let Some(on_table_related) = &mut self.inner.end_conditions.on_table_related {
            on_table_related.is_caption_applicable = false;
        }

        self.state = State::default();

        output
    }

    #[inline(always)]
    fn process_in_expecting_new_content_state(
        ctx: &mut Context,
        inner: &mut ParserInner,
        state: &mut StateExpectingNewContent,
    ) -> InternalOutput {
        let Some(peeked) = ctx.mapper.peek(0) else {
            return done!();
        };
        let peeked = peeked.clone();

        match &peeked {
            global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::NextChar => {
                // NOTE: 初始状态也可能遇到 `NextChar`，比如在一个并非结束与换行的块
                // 级元素（最简单的，如分割线）后面存在文本时。

                if state.skipped_spaces > 0 {
                    debug_assert!(matches!(peeked, global_mapper::Mapped::NextChar))
                }

                if ctx.peek_next_char() == Some(b' ') {
                    let should_skip_space = match inner.mode {
                        Mode::Inline => true,
                        Mode::Verbatim => state.skipped_spaces < inner.indentation,
                    };

                    if should_skip_space {
                        state.skipped_spaces += 1;
                        consume_peeked!(ctx, &peeked);
                        return InternalOutput::ToContinue;
                    }
                }

                if let Some(condition) = &inner.end_conditions.on_table_related {
                    if let Some(have_met) = try_parse_potential_table_related(ctx, condition) {
                        return done!(have_met);
                    }
                }

                if !inner.has_yielded_at_current_line {
                    if let Some(condition) = inner
                        .end_conditions
                        .after_repetitive_characters
                        .as_ref()
                        .filter(|c| c.at_line_beginning)
                    {
                        let consumed = ctx.drop_from_mapper_while_char(condition.character);
                        if consumed >= condition.minimal_count {
                            return done!();
                        } else if consumed > 0 {
                            let content = {
                                let after_current_cursor = ctx.cursor.value().unwrap() + 1;
                                (after_current_cursor - consumed)..after_current_cursor
                            };
                            return InternalOutput::ToContinueIn(State::ExpectingContentNextChar {
                                content,
                                spaces_after: 0,
                            });
                        }
                    }
                }

                if state.skipped_spaces > 0 {
                    if let Some(condition) = inner
                        .end_conditions
                        .after_repetitive_characters
                        .as_ref()
                        .filter(|c| c.at_line_end_and_with_space_before)
                    {
                        match try_process_potential_repetitive_characters(ctx, condition) {
                            TryProcessPotentialRepetitiveCharactersResult::Matched => {
                                return InternalOutput::ToContinue;
                            }
                            TryProcessPotentialRepetitiveCharactersResult::Unmatched(consumed) => {
                                if consumed > 0 {
                                    let content = {
                                        let after_current_cursor = ctx.cursor.value().unwrap() + 1;
                                        (after_current_cursor - consumed)..after_current_cursor
                                    };
                                    return InternalOutput::ToContinueIn(
                                        State::ExpectingContentNextChar {
                                            content,
                                            spaces_after: 0,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }

                if let Some(new_line) = state.new_line.take() {
                    if !inner.is_at_first_line {
                        return InternalOutput::ToYield(BlockEvent::NewLine(new_line));
                    }
                }

                consume_peeked!(ctx, &peeked);
                let content = {
                    let current_cursor = ctx.cursor.value().unwrap();
                    if inner.has_yielded_at_current_line {
                        (current_cursor - state.skipped_spaces)..(1 + current_cursor)
                    } else {
                        inner.has_yielded_at_current_line = true;
                        current_cursor..(current_cursor + 1)
                    }
                };
                InternalOutput::ToContinueIn(State::ExpectingContentNextChar {
                    content,
                    spaces_after: 0,
                })
            }
            global_mapper::Mapped::NewLine(_) => {
                if inner.end_conditions.before_new_line
                    || (inner.end_conditions.before_blank_line
                        && !inner.has_yielded_at_current_line)
                {
                    done!()
                } else {
                    if let Some(new_line) = state.new_line.take() {
                        return InternalOutput::ToYield(BlockEvent::NewLine(new_line));
                    }
                    InternalOutput::ToPauseForNewLine
                }
            }
            global_mapper::Mapped::VerbatimEscaping(verbatim_escaping) => {
                if let Some(new_line) = state.new_line.take() {
                    if !inner.is_at_first_line {
                        return InternalOutput::ToYield(BlockEvent::NewLine(new_line));
                    }
                }

                consume_peeked!(ctx, &peeked);
                InternalOutput::ToYield(BlockEvent::VerbatimEscaping(verbatim_escaping.clone()))
            }
        }
    }

    #[inline(always)]
    fn process_in_expecting_content_next_char_state(
        ctx: &mut Context,
        inner: &ParserInner,
        state_content: &mut Range<usize>,
        spaces_after: &mut usize,
    ) -> InternalOutput {
        let Some(peeked) = ctx.mapper.peek(0) else {
            return InternalOutput::ToYield(make_content_event(&inner.mode, state_content.clone()));
        };
        let peeked = peeked.clone();

        match peeked {
            global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::VerbatimEscaping(_) => {
                state_content.end += *spaces_after;
                *spaces_after = 0;
                InternalOutput::ToYield(make_content_event(&inner.mode, state_content.clone()))
            }
            global_mapper::Mapped::NewLine(_) => {
                InternalOutput::ToYield(make_content_event(&inner.mode, state_content.clone()))
            }
            global_mapper::Mapped::NextChar => {
                if matches!(inner.mode, Mode::Inline) && ctx.peek_next_char() == Some(b' ') {
                    *spaces_after += 1;
                    consume_peeked!(ctx, &peeked);
                    return InternalOutput::ToContinue;
                }

                if let Some(condition) = &inner.end_conditions.on_table_related {
                    if let Some(have_met) = try_parse_potential_table_related(ctx, condition) {
                        if !Range::is_empty(state_content) {
                            return InternalOutput::ToYieldAndBeDone(
                                BlockEvent::Unparsed(state_content.clone()),
                                have_met,
                            );
                        } else {
                            return done!(have_met);
                        }
                    }
                }

                let mut has_already_consumed = false;
                if *spaces_after > 0 {
                    if let Some(condition) = inner
                        .end_conditions
                        .after_repetitive_characters
                        .as_ref()
                        .filter(|c| c.at_line_end_and_with_space_before)
                    {
                        match try_process_potential_repetitive_characters(ctx, condition) {
                            TryProcessPotentialRepetitiveCharactersResult::Matched => {
                                return InternalOutput::ToYield(BlockEvent::Unparsed(
                                    state_content.clone(),
                                ));
                            }
                            TryProcessPotentialRepetitiveCharactersResult::Unmatched(consumed) => {
                                if consumed > 0 {
                                    state_content.end += consumed;
                                    has_already_consumed = true;
                                }
                            }
                        }
                    }
                }

                state_content.end += *spaces_after;
                *spaces_after = 0;

                if !has_already_consumed {
                    consume_peeked!(ctx, &peeked);
                    state_content.end += 1;
                }
                InternalOutput::ToContinue
            }
        }
    }

    pub fn resume_from_pause_for_new_line_and_continue(&mut self, new_line: NewLine) {
        self.state = StateExpectingNewContent {
            new_line: Some(new_line),
            skipped_spaces: 0,
        }
        .into()
    }
}

#[inline(always)]
fn make_content_event(mode: &Mode, content: Range<usize>) -> BlockEvent {
    match mode {
        Mode::Inline => BlockEvent::Unparsed(content),
        Mode::Verbatim => BlockEvent::Text(content),
    }
}

enum TryProcessPotentialRepetitiveCharactersResult {
    Matched,
    Unmatched(usize),
}

fn try_process_potential_repetitive_characters(
    ctx: &mut Context,
    condition: &RepetitiveCharactersCondition,
) -> TryProcessPotentialRepetitiveCharactersResult {
    let consumed =
        ctx.drop_from_mapper_while_char_with_maximum(condition.character, condition.minimal_count);
    if consumed == condition.minimal_count && !ctx.mapper.peek(0).is_some_and(|p| !p.is_new_line())
    {
        TryProcessPotentialRepetitiveCharactersResult::Matched
    } else {
        TryProcessPotentialRepetitiveCharactersResult::Unmatched(consumed)
    }
}

fn try_parse_potential_table_related(
    ctx: &mut Context,
    condition: &TableRelatedCondition,
) -> Option<HaveMet> {
    match ctx.peek_next_three_chars() {
        [Some(m!('|')), Some(m!('}')), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(2);
            Some(HaveMet::TableClosing)
        }
        [Some(m!('|')), Some(m!('+')), ..] if condition.is_caption_applicable => {
            ctx.must_take_from_mapper_and_apply_to_cursor(2);
            Some(HaveMet::TableCaptionIndicator)
        }
        [Some(m!('|')), Some(m!('-')), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(2);
            Some(HaveMet::TableRowIndicator)
        }
        [Some(m!('|')), Some(m!('|')), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(2);
            Some(HaveMet::DoublePipes)
        }
        [Some(m!('!')), Some(m!('!')), ..] => {
            ctx.must_take_from_mapper_and_apply_to_cursor(2);
            Some(HaveMet::TableHeaderCellIndicator)
        }
        _ => None,
    }
}
