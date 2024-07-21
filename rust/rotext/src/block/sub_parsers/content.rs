use crate::{
    block::{
        context::Context,
        global_mapper::{self},
        sub_parsers::{self, utils::consume_peeked},
    },
    common::Range,
    events::BlockEvent,
};

#[derive(Debug)]
pub enum StepState {
    Initial,
    Normal(Range),
    IsAfterLineFeed,
}
impl Default for StepState {
    fn default() -> Self {
        Self::Initial
    }
}

#[derive(Debug)]
enum InternalResult {
    /// 继续 next 中的循环。
    ToContinue,
    /// 改变 next 内部的状态，并继续循环。
    ToContinueIn(StepState),

    /// 打破 next 中的循环，产出 [BlockEvent]。
    ToYield(BlockEvent),
    /// 由于遇到了 LF，打破 next 中的循环，通知外部暂停本解析器的解析。遇到的 LF
    /// 应由外部负责产出。当外部认为可以恢复本解析器的解析时，那之后第一次调用
    /// [Parser::next] 时对应于 `is_resumed_from_line_feed` 位置的参数应该填
    /// `true`。
    ToPauseForNewLine,
    /// 打破 next 中的循环，并向外部表明本解析器的解析到此为止。不产出任何事件。
    Done,
}

pub struct Parser {
    mode: Mode,
    end_conditions: EndConditions,

    next_initial_step_state: StepState,
    is_at_first_line: bool,
}

#[derive(Default)]
pub struct Options {
    pub initial_step_state: StepState,
    pub mode: Mode,
    pub end_conditions: EndConditions,
}

pub enum Mode {
    /// 无视空白。
    Inline,
    /// 保留空白。
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
}

pub struct RepetitiveCharactersCondition {
    /// 如果是在行首，则满足条件。
    pub at_line_beginning: bool,
    /// 如果是在行尾，且之前有一个空白，则满足条件。
    pub at_line_end_and_with_space_before: bool,

    pub character: u8,
    /// 如果是对应于 `at_line_beginning` 的判断，则是最少需要的数量；如果对应于
    /// `at_line_end_and_with_space_before` 的判断，则是准确需要的数量。
    pub minimal_count: usize,
}

impl Parser {
    pub fn new(options: Options) -> Self {
        Self {
            mode: options.mode,
            end_conditions: options.end_conditions,
            next_initial_step_state: options.initial_step_state,
            is_at_first_line: true,
        }
    }

    pub fn next(&mut self, ctx: &mut Context) -> sub_parsers::Result {
        let mut state = std::mem::replace(&mut self.next_initial_step_state, StepState::Initial);

        loop {
            let internal_result = match state {
                StepState::Initial => self.process_in_initial_state(ctx),
                StepState::Normal(ref mut content) => self.process_in_normal_state(ctx, content),
                StepState::IsAfterLineFeed => self.process_in_is_after_line_feed_state(ctx),
            };

            match internal_result {
                InternalResult::ToContinue => {}
                InternalResult::ToContinueIn(new_state) => {
                    state = new_state;
                }
                InternalResult::ToYield(ev) => break sub_parsers::Result::ToYield(ev),
                InternalResult::ToPauseForNewLine => {
                    self.is_at_first_line = false;
                    break sub_parsers::Result::ToPauseForNewLine;
                }
                InternalResult::Done => break sub_parsers::Result::Done,
            }
        }
    }

    #[inline(always)]
    fn process_in_initial_state(&mut self, ctx: &mut Context) -> InternalResult {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::Done;
        };
        let peeked = peeked.clone();

        match peeked {
            global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::NextChar => {
                // NOTE: 初始状态也可能遇到 `NextChar`，比如在一个并非结束与换行的块
                // 级元素（最简单的，如分割线）后面存在文本时。

                let Some(condition) = self
                    .end_conditions
                    .after_repetitive_characters
                    .as_ref()
                    .filter(|c| {
                        c.at_line_end_and_with_space_before && ctx.peek_next_char() == Some(b' ')
                    })
                else {
                    consume_peeked!(ctx, &peeked);
                    let content = Range::new(ctx.cursor.value().unwrap(), 1);
                    return InternalResult::ToContinueIn(StepState::Normal(content));
                };

                process_potential_closing_part_at_line_end_and_with_space_before(
                    ctx,
                    condition,
                    Range::new(ctx.cursor.applying(&peeked).value().unwrap(), 0),
                    peeked,
                )
            }
            global_mapper::Mapped::LineFeed => {
                // consume_peeked!(ctx, &peeked);
                if self.end_conditions.before_new_line {
                    InternalResult::Done
                } else {
                    InternalResult::ToPauseForNewLine
                }
            }
            global_mapper::Mapped::BlankAtLineBeginning(blank) => {
                consume_peeked!(ctx, &peeked);
                match self.mode {
                    Mode::Inline => InternalResult::ToContinue,
                    Mode::Verbatim => InternalResult::ToYield(BlockEvent::Text(blank)),
                }
            }
            global_mapper::Mapped::Text(content) => {
                consume_peeked!(ctx, &peeked);
                InternalResult::ToYield(BlockEvent::Text(content))
            }
        }
    }

    #[inline(always)]
    fn process_in_normal_state(
        &mut self,
        ctx: &mut Context,
        state_content: &mut Range,
    ) -> InternalResult {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::ToYield(self.make_content_event(*state_content));
        };
        let peeked = peeked.clone();

        match peeked {
            global_mapper::Mapped::CharAt(_)
            | global_mapper::Mapped::LineFeed
            | global_mapper::Mapped::Text(_) => {
                InternalResult::ToYield(self.make_content_event(*state_content))
            }
            global_mapper::Mapped::NextChar => {
                let Some(condition) = self
                    .end_conditions
                    .after_repetitive_characters
                    .as_ref()
                    .filter(|c| {
                        c.at_line_end_and_with_space_before && ctx.peek_next_char() == Some(b' ')
                    })
                else {
                    consume_peeked!(ctx, &peeked);
                    state_content.increase_length(1);
                    return InternalResult::ToContinue;
                };

                process_potential_closing_part_at_line_end_and_with_space_before(
                    ctx,
                    condition,
                    *state_content,
                    peeked,
                )
            }
            global_mapper::Mapped::BlankAtLineBeginning(blank) => {
                consume_peeked!(ctx, &peeked);
                match self.mode {
                    Mode::Inline => InternalResult::ToContinue,
                    Mode::Verbatim => InternalResult::ToYield(BlockEvent::Text(blank)),
                }
            }
        }
    }

    #[inline(always)]
    fn process_in_is_after_line_feed_state(&mut self, ctx: &mut Context) -> InternalResult {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::Done;
        };

        match peeked {
            &global_mapper::Mapped::CharAt(index) => {
                let Some(condition) = self
                    .end_conditions
                    .after_repetitive_characters
                    .as_ref()
                    .filter(|c| c.at_line_beginning && c.character == ctx.input[index])
                else {
                    if self.is_at_first_line {
                        return InternalResult::ToContinueIn(StepState::Initial);
                    } else {
                        return InternalResult::ToYield(BlockEvent::LineBreak);
                    }
                };

                consume_peeked!(ctx, peeked);
                let mut potential_closing_part = Range::new(index, 1);

                let dropped = ctx.drop_from_mapper_while_char(condition.character);
                if 1 + dropped >= condition.minimal_count {
                    InternalResult::Done
                } else {
                    // XXX: 被 drop 的那些不会重新尝试解析，而是直接当成文本。
                    potential_closing_part.set_length(1 + dropped);
                    InternalResult::ToContinueIn(StepState::Normal(potential_closing_part))
                }
            }
            global_mapper::Mapped::NextChar => unreachable!(),
            global_mapper::Mapped::LineFeed => {
                if self.end_conditions.before_blank_line {
                    consume_peeked!(ctx, peeked);
                    InternalResult::Done
                } else {
                    InternalResult::ToYield(BlockEvent::LineBreak)
                }
            }
            global_mapper::Mapped::BlankAtLineBeginning(_) | global_mapper::Mapped::Text(_) => {
                if self.is_at_first_line {
                    InternalResult::ToContinueIn(StepState::Initial)
                } else {
                    InternalResult::ToYield(BlockEvent::LineBreak)
                }
            }
        }
    }

    pub fn resume_from_pause_for_new_line_and_continue(&mut self) {
        self.next_initial_step_state = StepState::IsAfterLineFeed;
    }

    #[inline(always)]
    fn make_content_event(&self, content: Range) -> BlockEvent {
        match self.mode {
            Mode::Inline => BlockEvent::Unparsed(content),
            Mode::Verbatim => BlockEvent::Text(content),
        }
    }
}

fn process_potential_closing_part_at_line_end_and_with_space_before(
    ctx: &mut Context,
    condition: &RepetitiveCharactersCondition,
    mut confirmed_content: Range,
    peeked: global_mapper::Mapped,
) -> InternalResult {
    let mut potential_closing_part_length = 0;

    if condition.at_line_end_and_with_space_before {
        consume_peeked!(ctx, &peeked);
        potential_closing_part_length += 1;
    }

    if ctx.peek_next_char() != Some(condition.character) {
        confirmed_content.increase_length(potential_closing_part_length);
        return InternalResult::ToContinueIn(StepState::Normal(confirmed_content));
    }
    ctx.must_take_from_mapper_and_apply_to_cursor(1);
    potential_closing_part_length += 1;

    let dropped =
        ctx.drop_from_mapper_while_char_with_maximum(condition.character, condition.minimal_count);
    // XXX: 被 drop 的那些不会重新尝试解析，而是直接当成文本。
    potential_closing_part_length += dropped;
    if 1 + dropped == condition.minimal_count {
        let peeked = ctx.mapper.peek_1();
        if !matches!(peeked, Some(global_mapper::Mapped::LineFeed) | None) {
            confirmed_content.increase_length(potential_closing_part_length);
            return InternalResult::ToContinueIn(StepState::Normal(confirmed_content));
        };

        if confirmed_content.length() == 0 {
            // XXX: 只有在 [StepState::Initial] 下（逐字文本转义后直接是闭合部
            // 分）有可能走到这里。
            InternalResult::ToContinue
        } else {
            InternalResult::ToYield(BlockEvent::Unparsed(confirmed_content))
        }
    } else {
        confirmed_content.increase_length(potential_closing_part_length);
        InternalResult::ToContinueIn(StepState::Normal(confirmed_content))
    }
}
