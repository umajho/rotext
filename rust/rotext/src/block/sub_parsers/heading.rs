use crate::{
    block::{
        context::Context,
        sub_parsers::{self, content::TableRelatedCondition},
    },
    events::{BlockEvent, BlockWithID, ExitBlock, NewLine},
    types::BlockId,
};

use super::{HaveMet, InTable};

enum State {
    /// 构造解析器后，解析器所处的初始状态。此时，其所解析语法的开启部分应已经被
    /// 消耗。
    Initial,
    Content {
        id: BlockId,

        content_parser: Box<sub_parsers::content::Parser>,
    },
    Exiting(HaveMet),
    Exited,

    Invalid,
}

pub struct Parser {
    #[cfg(feature = "line-number")]
    start_line_number: usize,
    leading_signs: usize,

    in_table: Option<InTable>,

    state: State,
}

pub struct NewParserOptions {
    #[cfg(feature = "line-number")]
    pub start_line_number: usize,
    pub leading_signs: usize,

    pub in_table: Option<InTable>,
}

impl Parser {
    pub fn new(opts: NewParserOptions) -> Self {
        Self {
            #[cfg(feature = "line-number")]
            start_line_number: opts.start_line_number,
            leading_signs: opts.leading_signs,
            in_table: opts.in_table,
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
                    mode: sub_parsers::content::Mode::Inline,
                    end_conditions: sub_parsers::content::EndConditions {
                        before_new_line: true,
                        after_repetitive_characters: Some(
                            sub_parsers::content::RepetitiveCharactersCondition {
                                at_line_beginning: false,
                                at_line_end_and_with_space_before: true,
                                character: b'=',
                                minimal_count: self.leading_signs,
                            },
                        ),
                        on_table_related: self.in_table.as_ref().map(|x| TableRelatedCondition {
                            is_caption_applicable: !x.has_yielded_since_entered,
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let content_parser = sub_parsers::content::Parser::new(opts);
                let id = ctx.pop_block_id();
                (
                    sub_parsers::Output::ToYield(self.make_enter_heading_event(id)),
                    State::Content {
                        id,
                        content_parser: Box::new(content_parser),
                    },
                )
            }
            State::Content {
                id,
                mut content_parser,
            } => {
                let next = content_parser.next(ctx);
                match next {
                    sub_parsers::Output::ToYield(ev) => (
                        sub_parsers::Output::ToYield(ev),
                        State::Content { id, content_parser },
                    ),
                    sub_parsers::Output::ToPauseForNewLine => unreachable!(),
                    sub_parsers::Output::Done(have_met) => (
                        sub_parsers::Output::ToYield(BlockEvent::ExitBlock(ExitBlock {
                            id,
                            #[cfg(feature = "line-number")]
                            start_line_number: self.start_line_number,
                            #[cfg(feature = "line-number")]
                            end_line_number: ctx.current_line_number,
                        })),
                        State::Exiting(have_met),
                    ),
                }
            }
            State::Exiting(have_met) => (sub_parsers::Output::Done(have_met), State::Exited),
            // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入状态
            // [State::Exited]。此后，不应该再调用 `next` 方法，否则就会执行到
            // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
            // drop 掉。
            State::Exited | State::Invalid => unreachable!(),
        };

        ret
    }

    fn make_enter_heading_event(&self, id: BlockId) -> BlockEvent {
        let data = BlockWithID { id };
        match self.leading_signs {
            1 => BlockEvent::EnterHeading1(data),
            2 => BlockEvent::EnterHeading2(data),
            3 => BlockEvent::EnterHeading3(data),
            4 => BlockEvent::EnterHeading4(data),
            5 => BlockEvent::EnterHeading5(data),
            6 => BlockEvent::EnterHeading6(data),
            _ => unreachable!(),
        }
    }
}

impl<'a> sub_parsers::SubParser<'a> for Parser {
    fn next(&mut self, ctx: &mut Context<'a>) -> sub_parsers::Output {
        self.next(ctx)
    }

    fn resume_from_pause_for_new_line_and_continue(&mut self, _new_line: NewLine) {
        unreachable!()
    }

    fn resume_from_pause_for_new_line_and_exit(&mut self) {
        unreachable!()
    }
}
