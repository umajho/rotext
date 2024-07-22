mod context;
mod global_mapper;
mod root_parser;
mod sub_parsers;
mod utils;

mod tests;

use context::Context;

use crate::{events::BlockEvent, global};
use global_mapper::GlobalEventStreamMapper;
use utils::Peekable3;

pub struct Parser<'a> {
    context: Context<'a>,

    /// 如果为 true，代表没有后续输入了，要清理栈中余留的内容。
    is_cleaning_up: bool,
    state: State<'a>,
    stack: Vec<StackEntry>,
    nesting: Nesting,
}

enum State<'a> {
    InRootParser(root_parser::Parser<'a>),
    InSubParser(Box<dyn sub_parsers::SubParser<'a> + 'a>),

    Invalid,
}

pub struct Nesting {
    /// 目前栈中有多少 item-likes。
    item_likes_in_stack: usize,
    /// 目前已处理了多少 item-likes。（每次换行后重置。）
    processed_item_likes: usize,

    is_exiting_discountinued_item_likes: bool,
}

enum StackEntry {
    ItemLike(ItemLikeType),
}
impl From<ItemLikeType> for StackEntry {
    fn from(value: ItemLikeType) -> Self {
        Self::ItemLike(value)
    }
}
impl StackEntry {
    pub fn is_item_like(&self) -> bool {
        matches!(self, StackEntry::ItemLike(_))
    }
}

enum ItemLikeType {
    BlockQuoteLine,
}
impl From<u8> for ItemLikeType {
    fn from(value: u8) -> Self {
        match value {
            b'>' => ItemLikeType::BlockQuoteLine,
            _ => unreachable!(),
        }
    }
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8], global_stream: global::Parser<'a>) -> Parser<'a> {
        let context = Context {
            input,
            mapper: Peekable3::new(GlobalEventStreamMapper::new(input, global_stream)),
            cursor: utils::InputCursor::new(),
        };

        Parser {
            context,

            is_cleaning_up: false,
            state: State::InRootParser(root_parser::Parser::new(None)),
            stack: vec![],
            nesting: Nesting {
                item_likes_in_stack: 0,
                processed_item_likes: 0,
                is_exiting_discountinued_item_likes: false,
            },
        }
    }

    pub fn next(&mut self) -> Option<BlockEvent> {
        loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返
                // 回 `None`。
                return self.stack.pop().map(|_| BlockEvent::Exit);
            }

            let to_break: Option<BlockEvent>;
            (to_break, self.state) = match std::mem::replace(&mut self.state, State::Invalid) {
                State::InRootParser(parser) => self.process_in_root_parser_state(parser),
                State::InSubParser(sub_parser) => self.process_in_sub_parser_state(sub_parser),
                State::Invalid => unreachable!(),
            };

            if to_break.is_some() {
                break to_break;
            }
        }
    }

    #[inline(always)]
    fn process_in_root_parser_state(
        &mut self,
        mut parser: root_parser::Parser<'a>,
    ) -> (Option<BlockEvent>, State<'a>) {
        match parser.parse(&mut self.context, &mut self.stack, &mut self.nesting) {
            root_parser::Result::ToYield(ev) => (Some(ev), State::InRootParser(parser)),
            root_parser::Result::ToSwitchToSubParser(sub_parser) => {
                (None, State::InSubParser(sub_parser))
            }
            root_parser::Result::Done => {
                self.is_cleaning_up = true;
                (None, State::Invalid)
            }
        }
    }

    #[inline(always)]
    fn process_in_sub_parser_state(
        &mut self,
        mut sub_parser: Box<dyn sub_parsers::SubParser<'a> + 'a>,
    ) -> (Option<BlockEvent>, State<'a>) {
        match sub_parser.next(&mut self.context) {
            sub_parsers::Result::ToYield(ev) => (Some(ev), State::InSubParser(sub_parser)),
            sub_parsers::Result::ToPauseForNewLine => {
                let new_state = State::InRootParser(root_parser::Parser::new(Some(sub_parser)));
                (None, new_state)
            }
            sub_parsers::Result::Done => {
                let new_state = State::InRootParser(root_parser::Parser::new(None));
                (None, new_state)
            }
        }
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = BlockEvent;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
