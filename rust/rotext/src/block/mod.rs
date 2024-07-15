mod events;
mod global_mapper;
mod utils;

pub use events::Event;

use crate::{common::Range, global};
use global_mapper::GlobalEventStreamMapper;
use utils::{InputCursor, Peekable3};

pub struct Parser<'a, I: 'a + Iterator<Item = global::Event>> {
    input: &'a [u8],
    mapper: Peekable3<GlobalEventStreamMapper<'a, I>>,

    cursor: utils::InputCursor,
    state: State,
    stack: Vec<StackEntry>,
}

enum State {
    /// 没有后续输入了，要清理栈中余留的内容。
    CleaningUp,
    /// 在块与块之间，期待新的块的开始或其所在父级块（或文档）的结束。
    BetweenBlocks,
    /// 在行内内容之中，期待其所在父级块（或文档）的结束。
    InInline,
}

enum StackEntry {
    InParagraph,
}

impl<'a, I: 'a + Iterator<Item = global::Event>> Parser<'a, I> {
    pub fn new(input: &'a [u8], global_stream: I) -> Parser<'a, I> {
        Parser {
            input,
            mapper: Peekable3::new(GlobalEventStreamMapper::new(input, global_stream)),

            cursor: utils::InputCursor::new(),
            state: State::BetweenBlocks,
            stack: vec![],
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        loop {
            let event = match self.state {
                State::CleaningUp => self.stack.pop().map(|_| Event::Exit),
                State::BetweenBlocks => self.scan_enter_or_exit_or_leaf(),
                State::InInline => self.scan_inline_stuff_or_exit(),
            };
            if event.is_some() {
                return event;
            } else if self.stack.is_empty() {
                return None;
            } else {
                self.state = State::CleaningUp;
            }
        }
    }

    fn scan_enter_or_exit_or_leaf(&mut self) -> Option<Event> {
        loop {
            let peeked = self.mapper.peek_1()?;

            match peeked {
                global_mapper::Mapped::LineFeed
                | global_mapper::Mapped::BlankLine { .. }
                | global_mapper::Mapped::SpacesAtLineBeginning(_) => {
                    self.mapper.next();
                    continue;
                }
                global_mapper::Mapped::Text(_) => {
                    self.mapper.next();
                    self.state = State::InInline;
                    self.stack.push(StackEntry::InParagraph);
                    return Some(Event::EnterParagraph);
                }
                global_mapper::Mapped::CharAt(_) | global_mapper::Mapped::NextChar => {
                    if !self.take_from_mapper_and_apply_to_cursor_if_applied_cursor_satisfies(
                        |applied_cursor| applied_cursor.at(self.input).is_some_and(is_space_char),
                    ) {
                        // peeked 所对应的字符不是空白字符。
                        break;
                    }
                }
            }
        }

        let peeked = peek_next_three_chars(self.input, self.cursor, &mut self.mapper);
        match peeked[..] {
            [b'-', b'-', b'-'] => {
                self.must_take_from_mapper_and_apply_to_cursor(3);
                Some(self.scan_rest_thematic_break())
            }
            [b'`', b'`', b'`'] => {
                self.must_take_from_mapper_and_apply_to_cursor(3);
                Some(self.scan_rest_code_block())
            }
            _ => {
                self.state = State::InInline;
                self.stack.push(StackEntry::InParagraph);
                Some(Event::EnterParagraph)
            }
        }
    }

    fn scan_rest_thematic_break(&mut self) -> Event {
        loop {
            if !self.take_from_mapper_and_apply_to_cursor_if_applied_cursor_satisfies(
                |applied_cursor| {
                    applied_cursor
                        .at(self.input)
                        .is_some_and(|char| char == b'-')
                },
            ) {
                break;
            }
        }

        Event::ThematicBreak
    }

    fn scan_rest_code_block(&mut self) -> Event {
        todo!()
    }

    fn scan_code_block_meta() {
        todo!()
    }

    fn scan_code_block_content() {
        todo!()
    }

    fn scan_inline_stuff_or_exit(&mut self) -> Option<Event> {
        enum LocalState {
            Initial,
            Normal { start: usize, length: usize },
            IsAfterLineFeed,
        }

        let mut local_state = LocalState::Initial;

        loop {
            let Some(peeked) = self.mapper.peek_1() else {
                break;
            };

            // log::debug!("{:?} {:?}", self.cursor.value(), mapped);

            match peeked {
                &global_mapper::Mapped::CharAt(cursor_new) => {
                    if matches!(local_state, LocalState::Initial) {
                        self.cursor.apply(peeked);
                        self.mapper.next();
                        local_state = LocalState::Normal {
                            start: cursor_new,
                            length: 1,
                        }
                    } else {
                        break;
                    }
                }
                global_mapper::Mapped::NextChar => match local_state {
                    LocalState::Initial => {
                        self.cursor.apply(peeked);
                        self.mapper.next();
                        local_state = LocalState::Normal {
                            start: self.cursor.value().unwrap(),
                            length: 1,
                        }
                    }
                    LocalState::Normal { ref mut length, .. } => {
                        self.cursor.apply(peeked);
                        self.mapper.next();
                        *length += 1;
                    }
                    _ => unreachable!(),
                },
                global_mapper::Mapped::LineFeed => match local_state {
                    LocalState::Initial => {
                        self.cursor.apply(peeked);
                        self.mapper.next();
                        local_state = LocalState::IsAfterLineFeed;
                    }
                    LocalState::Normal { .. } => {
                        break;
                    }
                    _ => unreachable!(),
                },
                global_mapper::Mapped::BlankLine { .. } => match local_state {
                    LocalState::Initial => {
                        self.cursor.apply(peeked);
                        self.mapper.next();
                        return Some(Event::LineFeed);
                    }
                    LocalState::Normal { .. } => {
                        break;
                    }
                    LocalState::IsAfterLineFeed => {
                        self.cursor.apply(peeked);
                        self.mapper.next();
                        self.stack.pop();
                        self.state = State::BetweenBlocks;
                        return Some(Event::Exit);
                    }
                },
                global_mapper::Mapped::SpacesAtLineBeginning(_) => {
                    self.cursor.apply(peeked);
                    self.mapper.next();
                    continue;
                }
                global_mapper::Mapped::Text(content) => match local_state {
                    LocalState::Initial => {
                        self.cursor.apply(peeked);
                        let ret = Some(Event::Text(*content));
                        self.mapper.next();
                        return ret;
                    }
                    LocalState::Normal { .. } => {
                        break;
                    }
                    _ => unreachable!(),
                },
            }
        }

        match local_state {
            LocalState::Initial => None,
            LocalState::Normal { start, length } => {
                Some(Event::Undetermined(Range::new(start, length)))
            }
            LocalState::IsAfterLineFeed => Some(Event::LineFeed),
        }
    }

    fn take_from_mapper_and_apply_to_cursor_if_applied_cursor_satisfies<
        F: FnOnce(&InputCursor) -> bool,
    >(
        &mut self,
        condition: F,
    ) -> bool {
        let Some(peeked) = self.mapper.peek_1() else {
            return false;
        };
        let applied = self.cursor.applying(peeked);
        if condition(&applied) {
            self.cursor = applied;
            self.mapper.next();
            true
        } else {
            false
        }
    }

    fn must_take_from_mapper_and_apply_to_cursor(&mut self, n: usize) {
        for _ in 0..n {
            self.cursor.apply(&self.mapper.next().unwrap());
        }
    }
}

impl<'a, I: 'a + Iterator<Item = global::Event>> Iterator for Parser<'a, I> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}

/// 窥视 `iter` 接下来至多 3 个 `u8` 字符。
fn peek_next_three_chars<I: Iterator<Item = global_mapper::Mapped>>(
    input: &[u8],
    mut cursor: InputCursor,
    iter: &mut Peekable3<I>,
) -> Vec<u8> {
    let mut result: Vec<u8> = Vec::with_capacity(3);

    let Some(first) = iter.peek_1() else {
        return result;
    };
    cursor.apply(first);
    if cursor.value().is_none() {
        return result;
    }
    result.push(cursor.at(input).unwrap());

    let Some(second) = iter.peek_2() else {
        return result;
    };
    cursor.apply(second);
    if cursor.value().is_none() {
        return result;
    }
    result.push(cursor.at(input).unwrap());

    let Some(third) = iter.peek_3() else {
        return result;
    };
    cursor.apply(third);
    if cursor.value().is_none() {
        return result;
    }
    result.push(cursor.at(input).unwrap());

    result
}

fn is_space_char(char: u8) -> bool {
    char == b' ' || char == b'\t'
}
