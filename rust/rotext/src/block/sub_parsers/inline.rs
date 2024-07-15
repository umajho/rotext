use crate::{
    block::{
        global_mapper,
        utils::{InputCursor, Peekable3},
        Event,
    },
    common::Range,
    global,
};

#[derive(Debug)]
enum State {
    Initial,
    Normal(Range),
    IsAfterLineFeed,
}

pub enum Result {
    ToYield(Event),
    Done,
}

#[derive(Debug)]
enum InternalResult {
    ToSkip,
    ToChangeState(State),
    ToYield(Event),
    Done,
}

#[inline(always)]
pub fn scan_inline_or_exit<'a, I: 'a + Iterator<Item = global::Event>>(
    input: &[u8],
    cursor: &mut InputCursor,
    mapper: &mut Peekable3<global_mapper::GlobalEventStreamMapper<'a, I>>,
) -> Result {
    let mut state = State::Initial;

    loop {
        let internal_result = match state {
            State::Initial => process_in_initial_state(input, cursor, mapper),
            State::Normal(ref mut content) => {
                process_in_normal_state(input, cursor, mapper, content)
            }
            State::IsAfterLineFeed => process_in_is_after_line_feed_state(input, cursor, mapper),
        };

        match internal_result {
            InternalResult::ToSkip => {}
            InternalResult::ToChangeState(new_state) => {
                state = new_state;
            }
            InternalResult::ToYield(ev) => break Result::ToYield(ev),
            InternalResult::Done => break Result::Done,
        }
    }
}

macro_rules! consume_peeked {
    ($cursor:ident, $mapper:ident, $peeked:ident) => {
        $cursor.apply($peeked);
        $mapper.next();
    };
}

#[inline(always)]
fn process_in_initial_state<'a, I: 'a + Iterator<Item = global::Event>>(
    _input: &[u8],
    cursor: &mut InputCursor,
    mapper: &mut Peekable3<global_mapper::GlobalEventStreamMapper<'a, I>>,
) -> InternalResult {
    let Some(peeked) = mapper.peek_1() else {
        return InternalResult::Done;
    };

    match peeked {
        global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::NextChar => {
            // NOTE: 初始状态也可能遇到 `NextChar`，比如在一个并非结束与换行的块
            // 级元素（最简单的，如分割线）后面存在文本时。
            consume_peeked!(cursor, mapper, peeked);
            let content = Range::new(cursor.value().unwrap(), 1);
            InternalResult::ToChangeState(State::Normal(content))
        }
        global_mapper::Mapped::LineFeed => {
            consume_peeked!(cursor, mapper, peeked);
            InternalResult::ToChangeState(State::IsAfterLineFeed)
        }
        global_mapper::Mapped::BlankLine { .. } => {
            consume_peeked!(cursor, mapper, peeked);
            InternalResult::ToYield(Event::LineFeed)
        }
        global_mapper::Mapped::SpacesAtLineBeginning(_) => {
            consume_peeked!(cursor, mapper, peeked);
            InternalResult::ToSkip
        }
        global_mapper::Mapped::Text(content) => {
            let content = *content;
            consume_peeked!(cursor, mapper, peeked);
            InternalResult::ToYield(Event::Text(content))
        }
    }
}

#[inline(always)]
fn process_in_normal_state<'a, I: 'a + Iterator<Item = global::Event>>(
    _input: &[u8],
    cursor: &mut InputCursor,
    mapper: &mut Peekable3<global_mapper::GlobalEventStreamMapper<'a, I>>,
    state_content: &mut Range,
) -> InternalResult {
    let Some(peeked) = mapper.peek_1() else {
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
            consume_peeked!(cursor, mapper, peeked);
            state_content.set_length(state_content.length() + 1);
            InternalResult::ToSkip
        }
        global_mapper::Mapped::SpacesAtLineBeginning(_) => {
            consume_peeked!(cursor, mapper, peeked);
            InternalResult::ToSkip
        }
    }
}

#[inline(always)]
fn process_in_is_after_line_feed_state<'a, I: 'a + Iterator<Item = global::Event>>(
    _input: &[u8],
    cursor: &mut InputCursor,
    mapper: &mut Peekable3<global_mapper::GlobalEventStreamMapper<'a, I>>,
) -> InternalResult {
    let Some(peeked) = mapper.peek_1() else {
        return InternalResult::ToYield(Event::LineFeed);
    };

    match peeked {
        global_mapper::Mapped::CharAt(_) => InternalResult::ToYield(Event::LineFeed),
        global_mapper::Mapped::NextChar | global_mapper::Mapped::LineFeed => unreachable!(),
        global_mapper::Mapped::BlankLine { .. } => {
            consume_peeked!(cursor, mapper, peeked);
            InternalResult::Done
        }
        global_mapper::Mapped::SpacesAtLineBeginning(_) => {
            consume_peeked!(cursor, mapper, peeked);
            InternalResult::ToSkip
        }
        global_mapper::Mapped::Text(_) => InternalResult::ToYield(Event::LineFeed),
    }
}
