mod context;
mod global_mapper;
mod root_parser;
mod sub_parsers;
mod utils;

mod tests;

use context::Context;
use root_parser::ExitingDiscontinuedItemLikesState;

use crate::{
    events::{BlockEvent, ExitBlock, NewLine},
    global,
};
use global_mapper::GlobalEventStreamMapper;
use utils::Peekable;

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

/// 与嵌套的块级语法有关的状态。
pub struct Nesting {
    /// 目前栈中有多少 item-likes。
    item_likes_in_stack: usize,
    /// 当前行目前已处理了多少 item-likes。（每次换行后重置。）
    processed_item_likes: usize,

    is_exiting_discontinued_item_likes: Option<ExitingDiscontinuedItemLikesState>,
}

struct StackEntry {
    block: BlockInStack,
    #[cfg(feature = "line-number")]
    start_line_number: usize,
}

enum BlockInStack {
    BlockQuote,
    ItemLike(ItemLikeType),
    Container,
}
impl From<ItemLikeType> for BlockInStack {
    fn from(value: ItemLikeType) -> Self {
        Self::ItemLike(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemLikeType {
    OrderedListItem,
    UnorderedListItem,
    DescriptionTerm,
    DescriptionDetails,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a [u8], global_stream: global::Parser<'a>) -> Parser<'a> {
        let context = Context {
            input,
            mapper: Peekable::new(GlobalEventStreamMapper::new(input, global_stream)),
            cursor: utils::InputCursor::new(),

            // 这里只是随便初始化一下，实际在 [State::Start] 中决定。
            #[cfg(feature = "line-number")]
            current_line_number: 0,

            #[cfg(feature = "block-id")]
            next_block_id: 1,
        };

        let new_line = NewLine {
            #[cfg(feature = "line-number")]
            line_number_after: 1,
        };

        Parser {
            context,

            is_cleaning_up: false,
            state: State::InRootParser(root_parser::Parser::new(Some(new_line), None)),
            stack: vec![],
            nesting: Nesting {
                item_likes_in_stack: 0,
                processed_item_likes: 0,
                is_exiting_discontinued_item_likes: None,
            },
        }
    }

    pub fn next(&mut self) -> Option<BlockEvent> {
        let next = loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返回 `None`。
                break self.stack.pop().map(|#[allow(unused_variables)] entry| {
                    BlockEvent::ExitBlock(ExitBlock {
                        #[cfg(feature = "line-number")]
                        start_line_number: entry.start_line_number,
                        #[cfg(feature = "line-number")]
                        end_line_number: self.context.current_line_number,
                    })
                });
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
        };

        #[cfg(debug_assertions)]
        log::debug!("NEXT {:?}", next);

        next
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
                // TODO: 搞明白为什么如果在这里 take 走 LF，然后在构造 Parser 时以
                // `Some(true)` 作为 `is_certain_is_new_line` 的值，解析的结果就会
                // 变得不正确。明明这么做和现在的做法应该没有区别。
                // self.context.must_take_from_mapper_and_apply_to_cursor(1);
                let new_state =
                    State::InRootParser(root_parser::Parser::new(None, Some(sub_parser)));
                (None, new_state)
            }
            sub_parsers::Result::Done => {
                let new_state = State::InRootParser(root_parser::Parser::new(None, None));
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
