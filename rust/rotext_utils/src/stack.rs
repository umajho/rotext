use std::mem::MaybeUninit;

use rotext_core::{Error, Stack};

pub struct VecStack<T> {
    items: Vec<T>,
}
impl<T> Stack<T> for VecStack<T> {
    fn new() -> Self {
        Self { items: vec![] }
    }

    fn try_push(&mut self, item: T) -> Result<(), Error> {
        self.items.push(item);
        Ok(())
    }

    fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    fn as_slice(&self) -> &[T] {
        &self.items
    }
}

pub struct ArrayStack<T, const N: usize> {
    items: [T; N],
    len: usize,
}
impl<T, const N: usize> Stack<T> for ArrayStack<T, N> {
    fn new() -> Self {
        Self {
            items: unsafe {
                #[allow(clippy::uninit_assumed_init)]
                MaybeUninit::uninit().assume_init()
            },
            len: 0,
        }
    }

    fn try_push(&mut self, item: T) -> Result<(), Error> {
        if self.len == N {
            Err(Error::OutOfStackSpace)
        } else {
            self.items[self.len] = item;
            self.len += 1;
            Ok(())
        }
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            Some(std::mem::replace(&mut self.items[self.len], unsafe {
                #[allow(clippy::uninit_assumed_init)]
                MaybeUninit::uninit().assume_init()
            }))
        }
    }

    fn as_slice(&self) -> &[T] {
        &self.items[0..self.len]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use rotext_core::EventType;
    use rotext_internal_test::{
        BlockContext,
        suites::block::support::{
            assert_parse_error_with_stack, assert_parse_ok_and_output_matches_with_stack,
        },
    };

    #[test]
    fn array_stack_works() {
        let mut s = ArrayStack::<usize, 2>::new();
        assert!(s.pop().is_none());
        assert!(s.try_push(1).is_ok());
        assert!(s.try_push(2).is_ok());
        assert!(s.try_push(3).is_err());
        assert_eq!(s.pop(), Some(2));
        assert_eq!(s.pop(), Some(1));
        assert!(s.try_push(4).is_ok());
    }

    #[test]
    fn array_stack_works_with_block_parser() {
        let ctx: BlockContext<ArrayStack<_, 2>> = BlockContext::new();

        assert_parse_ok_and_output_matches_with_stack(&ctx, "", &vec![]);
        assert_parse_ok_and_output_matches_with_stack(&ctx, ">", &vec![
            (EventType::EnterBlockQuote, None),
            (EventType::ExitBlock, None),
        ]);
        assert_parse_ok_and_output_matches_with_stack(&ctx, "> >", &vec![
            (EventType::EnterBlockQuote, None),
            (EventType::EnterBlockQuote, None),
            (EventType::ExitBlock, None),
            (EventType::ExitBlock, None),
        ]);
        assert_parse_ok_and_output_matches_with_stack(&ctx, "> > foo", &vec![
            (EventType::EnterBlockQuote, None),
            (EventType::EnterBlockQuote, None),
            (EventType::EnterParagraph, None),
            (EventType::__Unparsed, Some("foo")),
            (EventType::ExitBlock, None),
            (EventType::ExitBlock, None),
            (EventType::ExitBlock, None),
        ]);
        assert_parse_error_with_stack(&ctx, "> > >", Error::OutOfStackSpace)
    }
}
