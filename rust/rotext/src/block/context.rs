use std::ops::Range;

use crate::types::BlockId;

use super::{
    global_mapper::GlobalEventStreamMapper,
    utils::{InputCursor, Peekable},
};

pub struct Context<'a> {
    pub input: &'a [u8],
    pub mapper: Peekable<3, GlobalEventStreamMapper<'a>>,
    pub cursor: InputCursor,

    #[cfg(feature = "line-number")]
    pub current_line_number: usize,

    #[cfg(feature = "block-id")]
    pub next_block_id: usize,
}

impl<'a> Context<'a> {
    #[cfg(feature = "block-id")]
    pub fn pop_block_id(&mut self) -> BlockId {
        let id = self.next_block_id;
        self.next_block_id += 1;
        BlockId::new(id)
    }
    #[cfg(not(feature = "block-id"))]
    #[inline(always)]
    pub fn pop_block_id(&mut self) -> BlockId {
        BlockId::new()
    }

    pub fn take_from_mapper_and_apply_to_cursor_if_applied_cursor_satisfies<
        F: FnOnce(&InputCursor) -> bool,
    >(
        &mut self,
        condition: F,
    ) -> bool {
        let Some(peeked) = self.mapper.peek(0) else {
            return false;
        };
        let applied = self.cursor.applying(peeked);
        let result = condition(&applied);
        if result {
            self.cursor = applied;
            self.mapper.next();
        }
        result
    }

    pub fn must_take_from_mapper_and_apply_to_cursor(&mut self, n: usize) {
        for _ in 0..n {
            self.cursor.apply(&self.mapper.next().unwrap());
        }
    }

    /// 从 `mapper` 窥视接下来至多 1 个 `u8` 字符。
    pub fn peek_next_char(&mut self) -> Option<u8> {
        let mut cursor = self.cursor;

        cursor.apply(self.mapper.peek(0)?);
        cursor.at(self.input)
    }

    /// 从 `mapper` 窥视接下来至多 3 个 `u8` 字符。
    pub fn peek_next_three_chars(&mut self) -> [Option<u8>; 3] {
        let mut cursor = self.cursor;
        let mut result: [Option<u8>; 3] = [None; 3];

        (|| -> Option<()> {
            cursor.apply(self.mapper.peek(0)?);
            result[0] = Some(cursor.at(self.input)?);

            cursor.apply(self.mapper.peek(1)?);
            result[1] = Some(cursor.at(self.input)?);

            cursor.apply(self.mapper.peek(2)?);
            result[2] = Some(cursor.at(self.input)?);

            None
        })();

        result
    }

    pub fn drop_from_mapper_while_char(&mut self, char: u8) -> usize {
        let mut dropped = 0;

        loop {
            if self.take_from_mapper_and_apply_to_cursor_if_applied_cursor_satisfies(
                |applied_cursor| {
                    applied_cursor
                        .at(self.input)
                        .is_some_and(|actual_char| char == actual_char)
                },
            ) {
                dropped += 1;
            } else {
                break;
            }
        }

        dropped
    }
    pub fn drop_from_mapper_while_char_with_maximum(&mut self, char: u8, maximum: usize) -> usize {
        if maximum == 0 {
            return 0;
        }
        let mut dropped = 0;

        loop {
            if self.take_from_mapper_and_apply_to_cursor_if_applied_cursor_satisfies(
                |applied_cursor| {
                    applied_cursor
                        .at(self.input)
                        .is_some_and(|actual_char| char == actual_char)
                },
            ) {
                dropped += 1;
                if dropped == maximum {
                    break;
                }
            } else {
                break;
            }
        }

        dropped
    }

    pub fn scan_blank_text(&mut self) -> Option<Range<usize>> {
        let start = self.cursor.applying(self.mapper.peek(0)?).value()?;
        let mut length = 0;
        while self.peek_next_char() == Some(b' ') {
            length += 1;
            self.must_take_from_mapper_and_apply_to_cursor(1);
        }
        if length > 0 {
            Some(start..(start + length))
        } else {
            None
        }
    }
}
