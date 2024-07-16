use crate::{
    block::{context::Context, global_mapper, sub_parsers::utils::consume_peeked, Event},
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
    ToSkip,
    ToChangeState(StepState),
    ToYield(Event),
    Done,
}

pub struct Parser;

impl Parser {
    pub fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn next<'a, I: 'a + Iterator<Item = global::Event>>(
        &mut self,
        ctx: &mut Context<'a, I>,
    ) -> Option<Event> {
        let mut state = StepState::Initial;

        loop {
            let internal_result = match state {
                StepState::Initial => self.process_in_initial_state(ctx),
                StepState::Normal(ref mut content) => self.process_in_normal_state(ctx, content),
                StepState::IsAfterLineFeed => self.process_in_is_after_line_feed_state(ctx),
            };

            match internal_result {
                InternalResult::ToSkip => {}
                InternalResult::ToChangeState(new_state) => {
                    state = new_state;
                }
                InternalResult::ToYield(ev) => break Some(ev),
                InternalResult::Done => break None,
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
                InternalResult::ToChangeState(StepState::Normal(content))
            }
            global_mapper::Mapped::LineFeed => {
                consume_peeked!(ctx, peeked);
                InternalResult::ToChangeState(StepState::IsAfterLineFeed)
            }
            global_mapper::Mapped::BlankAtLineBeginning(_) => {
                consume_peeked!(ctx, peeked);
                InternalResult::ToSkip
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
            return InternalResult::ToYield(Event::Undetermined(*state_content));
        };

        match peeked {
            global_mapper::Mapped::CharAt(_)
            | global_mapper::Mapped::LineFeed
            | global_mapper::Mapped::Text(_) => {
                InternalResult::ToYield(Event::Undetermined(*state_content))
            }
            global_mapper::Mapped::NextChar => {
                consume_peeked!(ctx, peeked);
                state_content.set_length(state_content.length() + 1);
                InternalResult::ToSkip
            }
            global_mapper::Mapped::BlankAtLineBeginning(_) => {
                consume_peeked!(ctx, peeked);
                InternalResult::ToSkip
            }
        }
    }

    #[inline(always)]
    fn process_in_is_after_line_feed_state<'a, I: 'a + Iterator<Item = global::Event>>(
        &mut self,
        ctx: &mut Context<'a, I>,
    ) -> InternalResult {
        let Some(peeked) = ctx.mapper.peek_1() else {
            return InternalResult::ToYield(Event::LineFeed);
        };

        match peeked {
            global_mapper::Mapped::CharAt(_) => InternalResult::ToYield(Event::LineFeed),
            global_mapper::Mapped::NextChar => unreachable!(),
            global_mapper::Mapped::LineFeed => {
                consume_peeked!(ctx, peeked);
                InternalResult::Done
            }
            global_mapper::Mapped::BlankAtLineBeginning(_) => {
                consume_peeked!(ctx, peeked);
                InternalResult::ToSkip
            }
            global_mapper::Mapped::Text(_) => InternalResult::ToYield(Event::LineFeed),
        }
    }
}
