mod context;
mod global_mapper;
mod root_parser;
mod sub_parsers;
mod utils;

mod tests;

use context::Context;
use root_parser::ExitingDiscontinuedItemLikesState;

#[cfg(feature = "block-id")]
use crate::types::BlockID;
use crate::{
    events::{BlockEvent, BlockWithID, ExitBlock, NewLine},
    global,
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

    is_exiting_discontinued_item_likes: Option<ExitingDiscontinuedItemLikesState>,
}

pub struct StackEntry {
    block: BlockInStack,

    #[cfg(feature = "block-id")]
    block_id: BlockID,

    #[cfg(feature = "line-number")]
    start_line_number: usize,
}
impl StackEntry {
    pub fn into_exit_block(
        self,
        #[cfg(feature = "line-number")] end_line_number: usize,
    ) -> ExitBlock {
        ExitBlock {
            #[cfg(feature = "block-id")]
            id: self.block_id,
            #[cfg(feature = "line-number")]
            start_line_number: self.start_line_number,
            #[cfg(feature = "line-number")]
            end_line_number,
        }
    }
}

enum BlockInStack {
    BlockQuote,
    ItemLike { typ: ItemLikeType },
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
    pub fn into_enter_container_block_event(
        self,
        #[cfg(feature = "block-id")] id: BlockID,
    ) -> BlockEvent {
        match self {
            ItemLikeType::OrderedListItem => BlockEvent::EnterOrderedList(BlockWithID {
                #[cfg(feature = "block-id")]
                id,
            }),
            ItemLikeType::UnorderedListItem => BlockEvent::EnterUnorderedList(BlockWithID {
                #[cfg(feature = "block-id")]
                id,
            }),
            ItemLikeType::DescriptionTerm | ItemLikeType::DescriptionDetails => {
                BlockEvent::EnterDescriptionList(BlockWithID {
                    #[cfg(feature = "block-id")]
                    id,
                })
            }
        }
    }

    pub fn into_enter_block_event(self, #[cfg(feature = "block-id")] id: BlockID) -> BlockEvent {
        match self {
            ItemLikeType::OrderedListItem | ItemLikeType::UnorderedListItem => {
                BlockEvent::EnterListItem(BlockWithID {
                    #[cfg(feature = "block-id")]
                    id,
                })
            }
            ItemLikeType::DescriptionTerm => BlockEvent::EnterDescriptionTerm(BlockWithID {
                #[cfg(feature = "block-id")]
                id,
            }),
            ItemLikeType::DescriptionDetails => BlockEvent::EnterDescriptionDetails(BlockWithID {
                #[cfg(feature = "block-id")]
                id,
            }),
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
            state: State::InRootParser(root_parser::Parser::new(root_parser::NewParserOptions {
                new_line: Some(new_line),
                paused_sub_parser: None,
            })),
            stack: TStack::new(),
            nesting: Nesting {
                item_likes_in_stack: 0,
                processed_item_likes: 0,
                is_exiting_discontinued_item_likes: None,
            },
        }
    }

    pub fn next(&mut self) -> Option<crate::Result<BlockEvent>> {
        let next = loop {
            if self.is_cleaning_up {
                // 若栈中还有内容，出栈并返回 `Some(Event::Exit)`；若栈已空，返回 `None`。
                break self.stack.pop().map(|#[allow(unused_variables)] entry| {
                    Ok(BlockEvent::ExitBlock(entry.into_exit_block(
                        #[cfg(feature = "line-number")]
                        self.context.current_line_number,
                    )))
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
                                    new_line: None,
                                    paused_sub_parser: Some(unsafe {
                                        sub_parser.take().unwrap_unchecked()
                                    }),
                                },
                            ));
                        }
                        sub_parsers::Output::Done => {
                            self.state = State::InRootParser(root_parser::Parser::new(
                                root_parser::NewParserOptions {
                                    new_line: None,
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

        next
    }
}

impl<'a, TStack: Stack<StackEntry>> Iterator for Parser<'a, TStack> {
    type Item = crate::Result<BlockEvent>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
