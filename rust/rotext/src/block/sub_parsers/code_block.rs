use crate::{
    block::{
        context::Context,
        sub_parsers::{self, HaveMet},
    },
    events::{BlockEvent, BlockWithId, ExitBlock, NewLine},
    types::{BlockId, LineNumber},
};

enum State {
    /// 构造解析器后，解析器所处的初始状态。此时，其所解析语法的开启部分应已经被
    /// 消耗。
    Initial,
    InfoStringContent {
        code_block_id: BlockId,

        info_string_content_parser: Box<sub_parsers::content::Parser>,
    },
    BeforeCodeContent {
        code_block_id: BlockId,

        code_content_parser: Box<sub_parsers::content::Parser>,
    },
    CodeContent {
        code_block_id: BlockId,

        code_content_parser: Box<sub_parsers::content::Parser>,
    },
    Exiting,
    Exited,

    Paused {
        code_block_id: BlockId,

        code_content_parser: Box<sub_parsers::content::Parser>,
    },
    /// 此状态仅在实现
    /// [sub_parsers::SubParser::resume_from_pause_for_new_line_and_exit] 时设置。
    /// 其他情况下会直接进入 [State::Exiting]。
    ToExit {
        code_block_id: BlockId,
    },

    Invalid,
}

pub struct Parser {
    start_line: LineNumber,
    leading_backticks: usize,
    indentation: usize,

    state: State,
}

pub struct NewParserOptions {
    pub start_line: LineNumber,
    pub leading_backticks: usize,
    /// 每行开头至多忽略此数量的空格。
    pub indentation: usize,
}

impl Parser {
    pub fn new(opts: NewParserOptions) -> Self {
        Self {
            start_line: opts.start_line,
            leading_backticks: opts.leading_backticks,
            indentation: opts.indentation,

            state: State::Initial,
        }
    }

    #[inline(always)]
    fn next(&mut self, ctx: &mut Context) -> sub_parsers::Output {
        let ret: sub_parsers::Output;

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
                let id = ctx.pop_block_id();
                let code_block = BlockEvent::EnterCodeBlock(BlockWithId { id });
                (
                    sub_parsers::Output::ToYield(code_block),
                    State::InfoStringContent {
                        code_block_id: id,
                        info_string_content_parser: Box::new(info_string_content_parser),
                    },
                )
            }
            State::InfoStringContent {
                code_block_id,
                mut info_string_content_parser,
            } => {
                let next = info_string_content_parser.next(ctx);
                match next {
                    sub_parsers::Output::ToYield(ev) => (
                        sub_parsers::Output::ToYield(ev),
                        State::InfoStringContent {
                            code_block_id,
                            info_string_content_parser,
                        },
                    ),
                    sub_parsers::Output::ToPauseForNewLine => unreachable!(),
                    sub_parsers::Output::Done(have_met) => {
                        debug_assert!(matches!(have_met, HaveMet::None));
                        let opts = sub_parsers::content::Options {
                            initial_state: sub_parsers::content::State::Invalid,
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
                            indentation: self.indentation,
                        };
                        let code_content_parser = sub_parsers::content::Parser::new(opts);
                        (
                            sub_parsers::Output::ToYield(BlockEvent::IndicateCodeBlockCode),
                            State::BeforeCodeContent {
                                code_block_id,
                                code_content_parser: Box::new(code_content_parser),
                            },
                        )
                    }
                }
            }
            State::BeforeCodeContent {
                code_block_id,
                code_content_parser,
            } => (
                sub_parsers::Output::ToPauseForNewLine,
                State::Paused {
                    code_block_id,
                    code_content_parser,
                },
            ),
            State::CodeContent {
                code_block_id,
                mut code_content_parser,
            } => {
                let next = code_content_parser.next(ctx);
                match next {
                    sub_parsers::Output::ToYield(ev) => (
                        sub_parsers::Output::ToYield(ev),
                        State::CodeContent {
                            code_block_id,
                            code_content_parser,
                        },
                    ),
                    sub_parsers::Output::ToPauseForNewLine => (
                        sub_parsers::Output::ToPauseForNewLine,
                        State::Paused {
                            code_block_id,
                            code_content_parser,
                        },
                    ),
                    sub_parsers::Output::Done(have_met) => {
                        debug_assert!(matches!(have_met, HaveMet::None));
                        let output =
                            sub_parsers::Output::ToYield(BlockEvent::ExitBlock(ExitBlock {
                                id: code_block_id,
                                start_line: self.start_line,
                                end_line: ctx.current_line,
                            }));
                        (output, State::Exiting)
                    }
                }
            }
            State::Exiting => (sub_parsers::Output::Done(HaveMet::None), State::Exited),
            // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入状态
            // [State::Exited]。此后，不应该再调用 `next` 方法，否则就会执行到
            // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
            // drop 掉。
            State::Exited | State::Paused { .. } | State::Invalid => unreachable!(),
            State::ToExit { code_block_id } => (
                sub_parsers::Output::ToYield(BlockEvent::ExitBlock(ExitBlock {
                    id: code_block_id,
                    start_line: self.start_line,
                    end_line: ctx.current_line,
                })),
                State::Exiting,
            ),
        };

        ret
    }
}

impl<'a> sub_parsers::SubParser<'a> for Parser {
    fn next(&mut self, ctx: &mut Context<'a>) -> sub_parsers::Output {
        self.next(ctx)
    }

    fn resume_from_pause_for_new_line_and_continue(&mut self, new_line: NewLine) {
        let state = std::mem::replace(&mut self.state, State::Invalid);
        if let State::Paused {
            code_block_id,
            mut code_content_parser,
        } = state
        {
            code_content_parser.resume_from_pause_for_new_line_and_continue(new_line);
            self.state = State::CodeContent {
                code_block_id,
                code_content_parser,
            };
        } else {
            unreachable!()
        };
    }

    fn resume_from_pause_for_new_line_and_exit(&mut self) {
        if let State::Paused { code_block_id, .. } = self.state {
            self.state = State::ToExit { code_block_id };
        } else {
            unreachable!()
        }
    }
}
