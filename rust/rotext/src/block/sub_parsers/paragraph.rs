use std::ops::Range;

use crate::{
    block::{
        context::Context,
        sub_parsers::{self, content::TableRelatedCondition},
    },
    events::{BlockEvent, BlockWithId, ExitBlock, NewLine},
    types::{BlockId, LineNumber},
};

use super::{HaveMet, InTable};

enum State {
    Initial(StateInitial),
    /// XXX: 其中的 Option 只是为了能在转移到其他状态时能够夺走 [State::StateContent]
    /// 的所有权，实际上在有效时总是为 [Some]。
    Content(Option<StateContent>),
    Exiting(HaveMet),
    Exited,

    Paused(StateContent),
    /// 此状态仅在实现
    /// [sub_parsers::SubParser::resume_from_pause_for_new_line_and_exit] 时设置。
    /// 其他情况下会直接进入 [State::Exiting]。
    ToExit {
        id: BlockId,
    },

    Invalid,
}
struct StateInitial {
    content_before: Option<Range<usize>>,
}
struct StateContent {
    id: BlockId,
    content_parser: Box<sub_parsers::content::Parser>,
}

pub struct Parser {
    inner: ParserInner,

    state: State,
}
struct ParserInner {
    start_line: LineNumber,

    in_table: Option<InTable>,

    have_ever_yielded: bool,
    deferred: Option<BlockEvent>,
}

pub struct NewParserOptions {
    pub start_line: LineNumber,

    pub content_before: Option<Range<usize>>,

    pub in_table: Option<InTable>,
}

impl Parser {
    /// XXX: 不会尝试解析 `content_before` 中的内容，而是直接把这些内容当成文本。
    pub fn new(opts: NewParserOptions) -> Self {
        Self {
            inner: ParserInner {
                start_line: opts.start_line,
                in_table: opts.in_table,
                have_ever_yielded: false,
                deferred: None,
            },
            state: State::Initial(StateInitial {
                content_before: opts.content_before,
            }),
        }
    }

    #[inline(always)]
    fn next(&mut self, ctx: &mut Context) -> sub_parsers::Output {
        if let Some(ev) = self.inner.deferred.take() {
            return sub_parsers::Output::ToYield(ev);
        }

        loop {
            debug_assert!(self.inner.deferred.is_none());

            match &mut self.state {
                State::Initial(state) => {
                    self.state = Self::process_in_initial_state(ctx, &self.inner, state);
                }
                State::Content(state) => {
                    let (ret, state) = Self::process_in_content_state(ctx, &mut self.inner, state);
                    if let Some(state) = state {
                        self.state = state
                    }
                    break ret;
                }
                State::Exiting(have_met) => {
                    let have_met = *have_met;
                    self.state = State::Exited;
                    break sub_parsers::Output::Done(have_met);
                }
                // 当解析器作为迭代器被耗尽而返回 `None` 时，解析器进入状态
                // [State::Exited]。此后，不应该再调用 `next` 方法，否则就会执行到
                // 这里。正确的做法是 `take_context` 取回 [Context]，并将解析器
                // drop 掉。
                State::Exited | State::Paused { .. } | State::Invalid => unreachable!(),
                State::ToExit { id } => {
                    let exit_block = ExitBlock {
                        id: *id,
                        start_line: self.inner.start_line,
                        end_line: ctx.current_line,
                    };
                    self.state = State::Exiting(HaveMet::None);
                    break sub_parsers::Output::ToYield(BlockEvent::ExitBlock(exit_block));
                }
            };
        }
    }

    #[inline(always)]
    fn process_in_initial_state(
        #[allow(unused_variables)] ctx: &mut Context,
        consts: &ParserInner,
        state: &StateInitial,
    ) -> State {
        let opts = sub_parsers::content::Options {
            initial_state: match state.content_before {
                Some(ref content_before) => sub_parsers::content::State::ExpectingContentNextChar {
                    content: content_before.clone(),
                    spaces_after: 0,
                },
                None => sub_parsers::content::State::default(),
            },
            end_conditions: sub_parsers::content::EndConditions {
                before_blank_line: true,
                on_table_related: consts.in_table.as_ref().map(|x| TableRelatedCondition {
                    is_caption_applicable: !x.has_yielded_since_entered,
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        let parser = sub_parsers::content::Parser::new(opts);
        let id = ctx.pop_block_id();
        State::Content(Some(StateContent {
            id,
            content_parser: Box::new(parser),
        }))
    }

    #[inline(always)]
    fn process_in_content_state(
        ctx: &mut Context,
        #[allow(unused_variables)] inner: &mut ParserInner,
        state: &mut Option<StateContent>,
    ) -> (sub_parsers::Output, Option<State>) {
        let state_unchecked = unsafe { state.as_mut().unwrap_unchecked() };
        let next = state_unchecked.content_parser.next(ctx);
        match next {
            sub_parsers::Output::ToYield(ev) => {
                if !inner.have_ever_yielded {
                    inner.have_ever_yielded = true;
                    let paragraph = BlockEvent::EnterParagraph(BlockWithId {
                        id: state_unchecked.id,
                    });
                    debug_assert!(inner.deferred.is_none());
                    inner.deferred = Some(ev);
                    return (sub_parsers::Output::ToYield(paragraph), None);
                }

                (sub_parsers::Output::ToYield(ev), None)
            }
            sub_parsers::Output::ToPauseForNewLine => {
                let state = State::Paused(unsafe { state.take().unwrap_unchecked() });
                (sub_parsers::Output::ToPauseForNewLine, Some(state))
            }
            sub_parsers::Output::Done(have_met) => {
                if inner.have_ever_yielded {
                    let exit_block = BlockEvent::ExitBlock(ExitBlock {
                        id: state_unchecked.id,
                        start_line: inner.start_line,
                        end_line: ctx.current_line,
                    });
                    (
                        sub_parsers::Output::ToYield(exit_block),
                        Some(State::Exiting(have_met)),
                    )
                } else {
                    (sub_parsers::Output::Done(have_met), Some(State::Exited))
                }
            }
        }
    }
}

impl<'a> sub_parsers::SubParser<'a> for Parser {
    fn next(&mut self, ctx: &mut Context<'a>) -> sub_parsers::Output {
        self.next(ctx)
    }

    fn resume_from_pause_for_new_line_and_continue(&mut self, new_line: NewLine) {
        let state = std::mem::replace(&mut self.state, State::Invalid);
        if let State::Paused(mut state_content) = state {
            state_content
                .content_parser
                .resume_from_pause_for_new_line_and_continue(new_line);
            self.state = State::Content(Some(state_content));
        } else {
            unreachable!()
        };
    }

    fn resume_from_pause_for_new_line_and_exit(&mut self) {
        #[allow(unused_variables)]
        if let State::Paused(state_content) = &self.state {
            self.state = State::ToExit {
                id: state_content.id,
            };
        } else {
            unreachable!()
        }
    }
}
