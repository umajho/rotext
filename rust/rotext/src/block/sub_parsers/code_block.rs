#[cfg(feature = "block-id")]
use crate::types::BlockID;
use crate::{
    block::{context::Context, sub_parsers, utils::match_pop_block_id},
    events::{BlockEvent, BlockWithID, ExitBlock, NewLine},
};

enum State {
    /// 构造解析器后，解析器所处的初始状态。此时，其所解析语法的开启部分应已经被
    /// 消耗。
    Initial,
    InfoStringContent {
        #[cfg(feature = "block-id")]
        code_block_id: BlockID,

        info_string_content_parser: Box<sub_parsers::content::Parser>,
    },
    BeforeCodeContent {
        #[cfg(feature = "block-id")]
        code_block_id: BlockID,

        code_content_parser: Box<sub_parsers::content::Parser>,
    },
    CodeContent {
        #[cfg(feature = "block-id")]
        code_block_id: BlockID,

        code_content_parser: Box<sub_parsers::content::Parser>,
    },
    Exiting,
    Exited,

    Paused {
        #[cfg(feature = "block-id")]
        code_block_id: BlockID,

        code_content_parser: Box<sub_parsers::content::Parser>,
    },
    /// 此状态仅在实现
    /// [sub_parsers::SubParser::resume_from_pause_for_new_line_and_exit] 时设置。
    /// 其他情况下会直接进入 [State::Exiting]。
    ToExit {
        #[cfg(feature = "block-id")]
        code_block_id: BlockID,
    },

    Invalid,
}

pub struct Parser {
    #[cfg(feature = "line-number")]
    start_line_number: usize,
    leading_backticks: usize,

    state: State,
}

pub struct NewParserOptions {
    #[cfg(feature = "line-number")]
    pub start_line_number: usize,
    pub leading_backticks: usize,
}

impl Parser {
    pub fn new(opts: NewParserOptions) -> Self {
        Self {
            #[cfg(feature = "line-number")]
            start_line_number: opts.start_line_number,
            leading_backticks: opts.leading_backticks,

            state: State::Initial,
        }
    }

    #[inline(always)]
    fn next(&mut self, ctx: &mut Context) -> sub_parsers::Result {
        let ret: sub_parsers::Result;

        let state = std::mem::replace(&mut self.state, State::Invalid);
        (ret, self.state) = match state {
            State::Initial => {
                let opts = sub_parsers::content::Options {
                    mode: sub_parsers::content::Mode::Verbatim,
                    end_conditions: sub_parsers::content::EndConditions {
                        before_new_line: true,
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let info_string_content_parser = sub_parsers::content::Parser::new(opts);
                match_pop_block_id! {
                    ctx,
                    Some(id) => {
                        let code_block = BlockEvent::EnterCodeBlock(BlockWithID { id });
                        (
                            sub_parsers::Result::ToYield(code_block),
                            State::InfoStringContent {
                                code_block_id: id,
                                info_string_content_parser: Box::new(info_string_content_parser),
                            },
                        )
                    },
                    None => {
                         let code_block = BlockEvent::EnterCodeBlock(BlockWithID {});
                        (
                            sub_parsers::Result::ToYield(code_block),
                            State::InfoStringContent {
                                info_string_content_parser: Box::new(info_string_content_parser),
                            },
                        )
                    },
                }
            }
            State::InfoStringContent {
                #[cfg(feature = "block-id")]
                code_block_id,
                mut info_string_content_parser,
            } => {
                let next = info_string_content_parser.next(ctx);
                match next {
                    sub_parsers::Result::ToYield(ev) => (
                        sub_parsers::Result::ToYield(ev),
                        State::InfoStringContent {
                            #[cfg(feature = "block-id")]
                            code_block_id,
                            info_string_content_parser,
                        },
                    ),
                    sub_parsers::Result::ToPauseForNewLine => unreachable!(),
                    sub_parsers::Result::Done => {
                        let opts = sub_parsers::content::Options {
                            initial_step_state: sub_parsers::content::StepState::Invalid,
                            mode: sub_parsers::content::Mode::Verbatim,
                            end_conditions: sub_parsers::content::EndConditions {
                                after_repetitive_characters: Some(
                                    sub_parsers::content::RepetitiveCharactersCondition {
                                        at_line_beginning: true,
                                        at_line_end_and_with_space_before: false,
                                        character: b'`',
                                        minimal_count: self.leading_backticks,
                                    },
                                ),
                                ..Default::default()
                            },
                        };
                        let code_content_parser = sub_parsers::content::Parser::new(opts);
                        (
                            sub_parsers::Result::ToYield(BlockEvent::Separator),
                            State::BeforeCodeContent {
                                #[cfg(feature = "block-id")]
                                code_block_id,
                                code_content_parser: Box::new(code_content_parser),
                            },
                        )
                    }
                }
            }
            State::BeforeCodeContent {
                #[cfg(feature = "block-id")]
                code_block_id,
                code_content_parser,
            } => (
                sub_parsers::Result::ToPauseForNewLine,
                State::Paused {
                    #[cfg(feature = "block-id")]
                    code_block_id,
                    code_content_parser,
                },
            ),
            State::CodeContent {
                #[cfg(feature = "block-id")]
                code_block_id,
                mut code_content_parser,
            } => {
                let next = code_content_parser.next(ctx);
                match next {
                    sub_parsers::Result::ToYield(ev) => (
                        sub_parsers::Result::ToYield(ev),
                        State::CodeContent {
                            #[cfg(feature = "block-id")]
                            code_block_id,
                            code_content_parser,
                        },
                    ),
                    sub_parsers::Result::ToPauseForNewLine => (
                        sub_parsers::Result::ToPauseForNewLine,
                        State::Paused {
                            #[cfg(feature = "block-id")]
                            code_block_id,
                            code_content_parser,
                        },
                    ),
                    sub_parsers::Result::Done => (
                        sub_parsers::Result::ToYield(BlockEvent::ExitBlock(ExitBlock {
                            #[cfg(feature = "block-id")]
                            id: code_block_id,
                            #[cfg(feature = "line-number")]
                            start_line_number: self.start_line_number,
                            #[cfg(feature = "line-number")]
                            end_line_number: ctx.current_line_number,
                        })),
                        State::Exiting,
                    ),
                }
            }
            State::Exiting => (sub_parsers::Result::Done, State::Exited),
            // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入状态
            // [State::Exited]。此后，不应该再调用 `next` 方法，否则就会执行到
            // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
            // drop 掉。
            State::Exited | State::Paused { .. } | State::Invalid => unreachable!(),
            State::ToExit {
                #[cfg(feature = "block-id")]
                code_block_id,
            } => (
                sub_parsers::Result::ToYield(BlockEvent::ExitBlock(ExitBlock {
                    #[cfg(feature = "block-id")]
                    id: code_block_id,
                    #[cfg(feature = "line-number")]
                    start_line_number: self.start_line_number,
                    #[cfg(feature = "line-number")]
                    end_line_number: ctx.current_line_number,
                })),
                State::Exiting,
            ),
        };

        ret
    }
}

impl<'a> sub_parsers::SubParser<'a> for Parser {
    fn next(&mut self, ctx: &mut Context<'a>) -> sub_parsers::Result {
        self.next(ctx)
    }

    fn resume_from_pause_for_new_line_and_continue(&mut self, new_line: NewLine) {
        let state = std::mem::replace(&mut self.state, State::Invalid);
        if let State::Paused {
            #[cfg(feature = "block-id")]
            code_block_id,
            mut code_content_parser,
        } = state
        {
            code_content_parser.resume_from_pause_for_new_line_and_continue(new_line);
            self.state = State::CodeContent {
                #[cfg(feature = "block-id")]
                code_block_id,
                code_content_parser,
            };
        } else {
            unreachable!()
        };
    }

    fn resume_from_pause_for_new_line_and_exit(&mut self) {
        if let State::Paused {
            #[cfg(feature = "block-id")]
            code_block_id,
            ..
        } = self.state
        {
            self.state = State::ToExit {
                #[cfg(feature = "block-id")]
                code_block_id,
            };
        } else {
            unreachable!()
        }
    }
}
