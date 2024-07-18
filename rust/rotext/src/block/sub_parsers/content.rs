use crate::{
    block::{
        context::Context,
        global_mapper::{self},
        sub_parsers::{self, utils::consume_peeked},
        Event,
    },
    common::Range,
    global,
};

#[derive(Debug)]
enum StepState {
    Initial,
    Normal(Range),
    IsAfterLineFeed,
}

#[derive(Debug)]
enum InternalResult {
    /// 继续 next 中的循环。
    ToContinue,
    /// 改变 next 内部的状态，并继续循环。
    ToChangeStepStateAndContinue(StepState),

    /// 打破 next 中的循环，产出 [Event]。
    ToYield(Event),
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

    is_at_line_beginning: bool,
    is_at_first_line: bool,
}

#[derive(Default)]
pub struct Options {
    pub mode: Mode,
    pub end_conditions: EndConditions,

    pub is_at_line_beginning: bool,
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
    pub at_end_of_leading_repetitive_characters_at_new_line: Option<RepetitiveCharactersCondition>,
}

pub struct RepetitiveCharactersCondition {
    pub character: u8,
    pub minimal_count: usize,
}

impl Parser {
    pub fn new(options: Options) -> Self {
        Self {
            mode: options.mode,
            end_conditions: options.end_conditions,
            is_at_line_beginning: options.is_at_line_beginning,
            is_at_first_line: true,
        }
    }

    pub fn next<'a, I: 'a + Iterator<Item = global::Event>>(
        &mut self,
        ctx: &mut Context<'a, I>,
    ) -> sub_parsers::Result {
        let mut state = if self.is_at_line_beginning {
            self.is_at_line_beginning = false;
            StepState::IsAfterLineFeed
        } else {
            StepState::Initial
        };

        loop {
            let internal_result = match state {
                StepState::Initial => self.process_in_initial_state(ctx),
                StepState::Normal(ref mut content) => self.process_in_normal_state(ctx, content),
                StepState::IsAfterLineFeed => self.process_in_is_after_line_feed_state(ctx),
            };

            match internal_result {
                InternalResult::ToContinue => {}
                InternalResult::ToChangeStepStateAndContinue(new_state) => {
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
    fn process_in_initial_state<'a, I: 'a + Iterator<Item = global::Event>>(
        &mut self,
        ctx: &mut Context<'a, I>,
    ) -> InternalResult {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::Done;
        };

        match peeked {
            global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::NextChar => {
                // NOTE: 初始状态也可能遇到 `NextChar`，比如在一个并非结束与换行的块
                // 级元素（最简单的，如分割线）后面存在文本时。
                consume_peeked!(ctx, peeked);
                let content = Range::new(ctx.cursor.value().unwrap(), 1);
                InternalResult::ToChangeStepStateAndContinue(StepState::Normal(content))
            }
            global_mapper::Mapped::LineFeed => {
                consume_peeked!(ctx, peeked);
                if self.end_conditions.before_new_line {
                    InternalResult::Done
                } else {
                    InternalResult::ToPauseForNewLine
                }
            }
            &global_mapper::Mapped::BlankAtLineBeginning(blank) => {
                consume_peeked!(ctx, peeked);
                match self.mode {
                    Mode::Inline => InternalResult::ToContinue,
                    Mode::Verbatim => InternalResult::ToYield(Event::Text(blank)),
                }
            }
            global_mapper::Mapped::Text(content) => {
                let content = *content;
                consume_peeked!(ctx, peeked);
                InternalResult::ToYield(Event::Text(content))
            }
        }
    }

    #[inline(always)]
    fn process_in_normal_state<'a, I: 'a + Iterator<Item = global::Event>>(
        &mut self,
        ctx: &mut Context<'a, I>,
        state_content: &mut Range,
    ) -> InternalResult {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::ToYield(Event::Unparsed(*state_content));
        };

        match peeked {
            global_mapper::Mapped::CharAt(_)
            | global_mapper::Mapped::LineFeed
            | global_mapper::Mapped::Text(_) => {
                InternalResult::ToYield(Event::Unparsed(*state_content))
            }
            global_mapper::Mapped::NextChar => {
                consume_peeked!(ctx, peeked);
                state_content.set_length(state_content.length() + 1);
                InternalResult::ToContinue
            }
            &global_mapper::Mapped::BlankAtLineBeginning(blank) => {
                consume_peeked!(ctx, peeked);
                match self.mode {
                    Mode::Inline => InternalResult::ToContinue,
                    Mode::Verbatim => InternalResult::ToYield(Event::Text(blank)),
                }
            }
        }
    }

    #[inline(always)]
    fn process_in_is_after_line_feed_state<'a, I: 'a + Iterator<Item = global::Event>>(
        &mut self,
        ctx: &mut Context<'a, I>,
    ) -> InternalResult {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::Done;
        };

        match peeked {
            global_mapper::Mapped::CharAt(index) => {
                let Some((expected_char, minimal_count)) = (|| {
                    if let Some(RepetitiveCharactersCondition {
                        character,
                        minimal_count,
                    }) = self
                        .end_conditions
                        .at_end_of_leading_repetitive_characters_at_new_line
                    {
                        if ctx.input[*index] == character {
                            return Some((character, minimal_count));
                        }
                    }
                    None
                })() else {
                    if self.is_at_first_line {
                        return InternalResult::ToChangeStepStateAndContinue(StepState::Initial);
                    } else {
                        return InternalResult::ToYield(Event::LineFeed);
                    }
                };

                let index = *index;
                consume_peeked!(ctx, peeked);
                let mut potential_closing_part = Range::new(index, 1);

                let dropped = ctx.drop_from_mapper_while_char(expected_char);
                if 1 + dropped >= minimal_count {
                    InternalResult::Done
                } else {
                    potential_closing_part.set_length(1 + dropped);
                    InternalResult::ToChangeStepStateAndContinue(StepState::Normal(
                        potential_closing_part,
                    ))
                }
            }
            global_mapper::Mapped::NextChar => unreachable!(),
            global_mapper::Mapped::LineFeed => {
                if self.end_conditions.before_blank_line {
                    consume_peeked!(ctx, peeked);
                    InternalResult::Done
                } else {
                    InternalResult::ToYield(Event::LineFeed)
                }
            }
            &global_mapper::Mapped::BlankAtLineBeginning(blank) => {
                consume_peeked!(ctx, peeked);
                match self.mode {
                    Mode::Inline => InternalResult::ToContinue,
                    Mode::Verbatim => InternalResult::ToYield(Event::Text(blank)),
                }
            }
            global_mapper::Mapped::Text(_) => InternalResult::ToYield(Event::LineFeed),
        }
    }

    pub fn resume_from_pause_for_new_line_and_continue(&mut self) {
        self.is_at_line_beginning = true;
    }
}
