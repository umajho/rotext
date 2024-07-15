mod events;
mod global_mapper;
mod sub_parsers;
mod utils;

pub use events::Event;
use sub_parsers::inline::scan_inline_or_exit;

use crate::global;
use global_mapper::GlobalEventStreamMapper;
use utils::{InputCursor, Peekable3};

pub struct Parser<'a, I: 'a + Iterator<Item = global::Event>> {
    input: &'a [u8],
    mapper: Peekable3<GlobalEventStreamMapper<'a, I>>,

    cursor: utils::InputCursor,
    /// 如果位 true，代表没有后续输入了，要清理栈中余留的内容。
    is_cleaning_up: bool,
    stack: Vec<StackEntry>,
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
            is_cleaning_up: false,
            stack: vec![],
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返回
                // `None`。
                return self.stack.pop().map(|_| Event::Exit);
            }

            let event = match self.stack.last() {
                Some(StackEntry::InParagraph) => self.scan_inline_or_exit(),
                None => self.scan_enter_or_exit_or_leaf(),
            };
            if event.is_some() {
                return event;
            } else if self.stack.is_empty() {
                return None;
            } else {
                self.is_cleaning_up = true
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

    fn scan_inline_or_exit(&mut self) -> Option<Event> {
        match scan_inline_or_exit(self.input, &mut self.cursor, &mut self.mapper) {
            sub_parsers::inline::Result::ToYield(ev) => Some(ev),
            sub_parsers::inline::Result::Done => {
                self.stack.pop();
                Some(Event::Exit)
            }
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
