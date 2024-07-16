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

pub struct Parser<'a, I: 'a + Iterator<Item = global::Event>> {
    context: Box<Context<'a, I>>,
}

impl<'a, I: 'a + Iterator<Item = global::Event>> Parser<'a, I> {
    pub fn new(context: Box<Context<'a, I>>) -> Self {
        Self { context }
    }

    pub fn drop(self) -> Box<Context<'a, I>> {
        self.context
    }

    #[inline(always)]
    fn next(&mut self) -> Option<Event> {
        let mut state = StepState::Initial;

        loop {
            let internal_result = match state {
                StepState::Initial => self.process_in_initial_state(),
                StepState::Normal(ref mut content) => self.process_in_normal_state(content),
                StepState::IsAfterLineFeed => self.process_in_is_after_line_feed_state(),
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
    fn process_in_initial_state(&mut self) -> InternalResult {
        let Some(peeked) = self.context.mapper.peek_1() else {
            return InternalResult::Done;
        };

        match peeked {
            global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::NextChar => {
                // NOTE: 初始状态也可能遇到 `NextChar`，比如在一个并非结束与换行的块
                // 级元素（最简单的，如分割线）后面存在文本时。
                consume_peeked!(self.context, peeked);
                let content = Range::new(self.context.cursor.value().unwrap(), 1);
                InternalResult::ToChangeState(StepState::Normal(content))
            }
            global_mapper::Mapped::LineFeed => {
                consume_peeked!(self.context, peeked);
                InternalResult::ToChangeState(StepState::IsAfterLineFeed)
            }
            global_mapper::Mapped::BlankLine { .. } => {
                consume_peeked!(self.context, peeked);
                InternalResult::ToYield(Event::LineFeed)
            }
            global_mapper::Mapped::SpacesAtLineBeginning(_) => {
                consume_peeked!(self.context, peeked);
                InternalResult::ToSkip
            }
            global_mapper::Mapped::Text(content) => {
                let content = *content;
                consume_peeked!(self.context, peeked);
                InternalResult::ToYield(Event::Text(content))
            }
        }
    }

    #[inline(always)]
    fn process_in_normal_state(&mut self, state_content: &mut Range) -> InternalResult {
        let Some(peeked) = self.context.mapper.peek_1() else {
            return InternalResult::ToYield(Event::Undetermined(*state_content));
        };

        match peeked {
            global_mapper::Mapped::CharAt(_)
            | global_mapper::Mapped::LineFeed
            | global_mapper::Mapped::BlankLine { .. }
            | global_mapper::Mapped::Text(_) => {
                InternalResult::ToYield(Event::Undetermined(*state_content))
            }
            global_mapper::Mapped::NextChar => {
                consume_peeked!(self.context, peeked);
                state_content.set_length(state_content.length() + 1);
                InternalResult::ToSkip
            }
            global_mapper::Mapped::SpacesAtLineBeginning(_) => {
                consume_peeked!(self.context, peeked);
                InternalResult::ToSkip
            }
        }
    }

    #[inline(always)]
    fn process_in_is_after_line_feed_state(&mut self) -> InternalResult {
        let Some(peeked) = self.context.mapper.peek_1() else {
            return InternalResult::ToYield(Event::LineFeed);
        };

        match peeked {
            global_mapper::Mapped::CharAt(_) => InternalResult::ToYield(Event::LineFeed),
            global_mapper::Mapped::NextChar | global_mapper::Mapped::LineFeed => unreachable!(),
            global_mapper::Mapped::BlankLine { .. } => {
                consume_peeked!(self.context, peeked);
                InternalResult::Done
            }
            global_mapper::Mapped::SpacesAtLineBeginning(_) => {
                consume_peeked!(self.context, peeked);
                InternalResult::ToSkip
            }
            global_mapper::Mapped::Text(_) => InternalResult::ToYield(Event::LineFeed),
        }
    }
}

impl<'a, I: 'a + Iterator<Item = global::Event>> Iterator for Parser<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
