mod context;
mod global_mapper;
mod root_parser;
mod sub_parsers;
mod utils;

mod tests;

use context::Context;
use root_parser::ExitingUntil;

use crate::{
    events::{BlockEvent, BlockWithId, ExitBlock, NewLine},
    global,
    types::{BlockId, LineNumber},
    utils::stack::Stack,
};
use global_mapper::GlobalEventStreamMapper;
use utils::Peekable;

pub struct Parser<'a, TStack: Stack<StackEntry>> {
    context: Context<'a>,

    /// 如果为 true，代表没有后续输入了，要清理栈中余留的内容。
    is_cleaning_up: bool,
    state: State<'a>,
    stack: TStack,
    nesting: Nesting,
}

/// NOTE: 虽然各 variants 大小差别很大，但由于一个解析器只会有一个 [State]，因此不是问题。
#[allow(clippy::large_enum_variant)]
enum State<'a> {
    InRootParser(root_parser::Parser<'a>),
    InSubParser(Option<Box<dyn sub_parsers::SubParser<'a> + 'a>>),

    Invalid,
}

/// 与嵌套的块级语法有关的状态。
pub struct Nesting {
    /// 目前栈中有多少 item-likes。
    item_likes_in_stack: usize,
    /// 当前行目前已处理了多少 item-likes。（每次换行后重置。）
    processed_item_likes: usize,

    exiting: Option<ExitingUntil>,

    tables_in_stack: usize,
    /// 只在 `tables_in_stack` 不为 0 时有效，记录是否在进入最新的表格后曾产出过事件。
    has_yielded_since_entered_last_table: bool,
}

pub struct StackEntry {
    block: BlockInStack,

    block_id: BlockId,

    start_line_number: LineNumber,
}
impl StackEntry {
    pub fn into_exit_block(self, end_line_number: LineNumber) -> ExitBlock {
        ExitBlock {
            id: self.block_id,
            start_line_number: self.start_line_number,
            end_line_number,
        }
    }
}

enum BlockInStack {
    BlockQuote,
    ItemLike { typ: ItemLikeType },
    Table,
    Container,
}
impl BlockInStack {
    pub fn is_item_like(&self) -> bool {
        matches!(self, BlockInStack::ItemLike { .. })
    }
}
impl From<ItemLikeType> for BlockInStack {
    fn from(value: ItemLikeType) -> Self {
        Self::ItemLike { typ: value }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItemLikeType {
    OrderedListItem,
    UnorderedListItem,
    DescriptionTerm,
    DescriptionDetails,
}
impl ItemLikeType {
    pub fn into_enter_container_block_event(self, id: BlockId) -> BlockEvent {
        match self {
            ItemLikeType::OrderedListItem => BlockEvent::EnterOrderedList(BlockWithId { id }),
            ItemLikeType::UnorderedListItem => BlockEvent::EnterUnorderedList(BlockWithId { id }),
            ItemLikeType::DescriptionTerm | ItemLikeType::DescriptionDetails => {
                BlockEvent::EnterDescriptionList(BlockWithId { id })
            }
        }
    }

    pub fn into_enter_block_event(self, id: BlockId) -> BlockEvent {
        match self {
            ItemLikeType::OrderedListItem | ItemLikeType::UnorderedListItem => {
                BlockEvent::EnterListItem(BlockWithId { id })
            }
            ItemLikeType::DescriptionTerm => BlockEvent::EnterDescriptionTerm(BlockWithId { id }),
            ItemLikeType::DescriptionDetails => {
                BlockEvent::EnterDescriptionDetails(BlockWithId { id })
            }
        }
    }
}

impl<'a, TStack: Stack<StackEntry>> Parser<'a, TStack> {
    pub fn new(input: &'a [u8], global_stream: global::Parser<'a>) -> Self {
        let context = Context {
            input,
            mapper: Peekable::new(GlobalEventStreamMapper::new(input, global_stream)),
            cursor: utils::InputCursor::new(),

            // 这里只是随便初始化一下，实际在 [State::Start] 中决定。
            current_line_number: LineNumber::new_universal(0),

            #[cfg(feature = "block-id")]
            next_block_id: 1,
        };

        let new_line = NewLine {
            line_number_after: LineNumber::new_universal(1),
        };

        Parser {
            context,

            is_cleaning_up: false,
            state: State::InRootParser(root_parser::Parser::new(root_parser::NewParserOptions {
                initial_state: root_parser::InitialState::Start {
                    new_line: Some(new_line),
                },
                paused_sub_parser: None,
            })),
            stack: TStack::new(),
            nesting: Nesting {
                item_likes_in_stack: 0,
                processed_item_likes: 0,
                exiting: None,
                tables_in_stack: 0,
                has_yielded_since_entered_last_table: false,
            },
        }
    }

    pub fn next(&mut self) -> Option<crate::Result<BlockEvent>> {
        let next = loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返回 `None`。
                break self.stack.pop().map(|#[allow(unused_variables)] entry| {
                    Ok(BlockEvent::ExitBlock(
                        entry.into_exit_block(self.context.current_line_number),
                    ))
                });
            }

            match self.state {
                State::InRootParser(ref mut parser) => {
                    // 原先是：`fn process_in_root_parser_state`。

                    let result =
                        parser.parse(&mut self.context, &mut self.stack, &mut self.nesting);
                    match result {
                        Ok(output) => match output {
                            root_parser::ParseStepOutput::ToYield(ev) => break Some(Ok(ev)),
                            root_parser::ParseStepOutput::ToSwitchToSubParser(sub_parser) => {
                                self.state = State::InSubParser(Some(sub_parser));
                            }
                            root_parser::ParseStepOutput::Done => {
                                self.is_cleaning_up = true;
                                self.state = State::Invalid;
                            }
                        },
                        Err(err) => return Some(Err(err)),
                    }
                }
                State::InSubParser(ref mut sub_parser) => {
                    // 原先是：`fn process_in_sub_parser_state`。

                    let sub_parser_unchecked = unsafe { sub_parser.as_mut().unwrap_unchecked() };
                    match sub_parser_unchecked.next(&mut self.context) {
                        sub_parsers::Output::ToYield(ev) => break Some(Ok(ev)),
                        sub_parsers::Output::ToPauseForNewLine => {
                            self.state = State::InRootParser(root_parser::Parser::new(
                                root_parser::NewParserOptions {
                                    initial_state: root_parser::InitialState::Start {
                                        new_line: None,
                                    },
                                    paused_sub_parser: Some(unsafe {
                                        sub_parser.take().unwrap_unchecked()
                                    }),
                                },
                            ));
                        }
                        sub_parsers::Output::Done(have_met) => {
                            self.state = State::InRootParser(root_parser::Parser::new(
                                root_parser::NewParserOptions {
                                    initial_state: have_met.into(),
                                    paused_sub_parser: None,
                                },
                            ));
                        }
                    }
                }
                State::Invalid => unreachable!(),
            };
        };

        #[cfg(debug_assertions)]
        log::debug!("NEXT {:?}", next);

        self.nesting.has_yielded_since_entered_last_table =
            !matches!(next, Some(Ok(BlockEvent::EnterTable(_))));

        next
    }
}

impl<'a, TStack: Stack<StackEntry>> Iterator for Parser<'a, TStack> {
    type Item = crate::Result<BlockEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
