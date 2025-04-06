macro_rules! case {
    (@with_id, $input:literal, [$($ev_tts:tt)*], $expected:expr,) => {
        case!(@__inner, $input, [$($ev_tts)*], $expected, {
            should_include_block_id: true,
        })
    };
    ($input:literal, [$($ev_tts:tt)*], $expected:expr,) => {
        case!(@__inner, $input, [$($ev_tts)*], $expected, {
            should_include_block_id: false,
        })
    };
    (@__inner, $input:literal, [$($ev_tts:tt)*], $expected:expr, {
        should_include_block_id: $should_include_block_id:literal,
    }) => {
        $crate::executing::tests::support::Case {
            input_events: $crate::executing::tests::support::events!($($ev_tts)*),
            input: $input,
            expected: $expected,

            options: $crate::executing::tests::support::CaseOptions {
                tag_name_map: crate::executing::TagNameMap::new_demo_instance_for_test(),
                #[cfg(feature = "block-id")]
                should_include_block_id: $should_include_block_id,
            },
        }
    };
}

macro_rules! __event {
    (VerbatimEscaping ($start:literal..$end:literal)) => {
        $crate::Event::VerbatimEscaping(rotext_core::events::VerbatimEscaping {
            content: $start..$end,
            is_closed_forcedly: false,
            line_after: rotext_core::LineNumber::new_invalid(),
        })
    };

    (NewLine (..)) => {
        $crate::Event::NewLine(rotext_core::events::NewLine {
            line_after: rotext_core::LineNumber::new_invalid(),
        })
    };

    (Text ($start:literal..$end:literal)) => {
        $crate::Event::Text($start..$end)
    };

    (IndicateCodeBlockCode ()) => {
        $crate::Event::IndicateCodeBlockCode
    };
    (IndicateTableCaption ()) => {
        $crate::Event::IndicateTableCaption
    };
    (IndicateTableRow ()) => {
        $crate::Event::IndicateTableRow
    };
    (IndicateTableHeaderCell ()) => {
        $crate::Event::IndicateTableHeaderCell
    };
    (IndicateTableDataCell ()) => {
        $crate::Event::IndicateTableDataCell
    };

    (ExitBlock (..)) => {
        $crate::Event::ExitBlock(rotext_core::events::ExitBlock {
            id: rotext_core::BlockId::new_invalid(),
            start_line: rotext_core::LineNumber::new_invalid(),
            end_line: rotext_core::LineNumber::new_invalid(),
        })
    };
    (ExitBlock (.., id = $id:literal)) => {
        $crate::Event::ExitBlock(rotext_core::events::ExitBlock {
            id: rotext_core::BlockId::new($id),
            start_line: rotext_core::LineNumber::new_invalid(),
            end_line: rotext_core::LineNumber::new_invalid(),
        })
    };
    (ExitBlock (.., id = $id:literal, lns = $ln_s:literal..=$ln_e:literal)) => {
        $crate::Event::ExitBlock(rotext_core::events::ExitBlock {
            id: rotext_core::BlockId::new($id),
            start_line: rotext_core::LineNumber::new($ln_s),
            end_line: rotext_core::LineNumber::new($ln_e),
        })
    };

    (ThematicBreak (..)) => {
        $crate::Event::ThematicBreak(rotext_core::events::ThematicBreak {
            id: rotext_core::BlockId::new_invalid(),
            line: rotext_core::LineNumber::new_invalid(),
        })
    };
    (ThematicBreak (.., id = $id:literal)) => {
        $crate::Event::ThematicBreak(rotext_core::events::ThematicBreak {
            id: rotext_core::BlockId::new($id),
            line: rotext_core::LineNumber::new_invalid(),
        })
    };
    (ThematicBreak (.., id = $id:literal, ln = $ln:literal)) => {
        $crate::Event::ThematicBreak(rotext_core::events::ThematicBreak {
            id: rotext_core::BlockId::new($id),
            line: rotext_core::LineNumber::new($ln),
        })
    };

    (RefLink ($start:literal..$end:literal)) => {
        $crate::Event::RefLink($start..$end)
    };
    (Dicexp ($start:literal..$end:literal)) => {
        $crate::Event::Dicexp($start..$end)
    };

    (EnterWikiLink ($start:literal..$end:literal)) => {
        $crate::Event::EnterWikiLink($start..$end)
    };

    (EnterCallOnExtension ($start:literal..$end:literal)) => {
        $crate::Event::EnterCallOnExtension(rotext_core::events::Call {
            id: rotext_core::BlockId::new_invalid(),
            name: $start..$end,
        })
    };
    (EnterCallOnExtension ($start:literal..$end:literal, id = $id:literal)) => {
        $crate::Event::EnterCallOnExtension(rotext_core::events::Call {
            id: rotext_core::BlockId::new($id),
            name: $start..$end,
        })
    };

    (IndicateCallNormalArgument ()) => {
        $crate::Event::IndicateCallNormalArgument(None)
    };
    (IndicateCallNormalArgument ($start:literal..$end:literal)) => {
        $crate::Event::IndicateCallNormalArgument(Some($start..$end))
    };
    (IndicateCallVerbatimArgument ()) => {
        $crate::Event::IndicateCallVerbatimArgument(None)
    };
    (IndicateCallVerbatimArgument ($start:literal..$end:literal)) => {
        $crate::Event::IndicateCallVerbatimArgument(Some($start..$end))
    };

    (@inline $v:tt (..)) => {
        $crate::Event::$v
    };

    ($v:tt (..)) => {
        $crate::Event::$v(rotext_core::events::BlockWithId {
            id: rotext_core::BlockId::new_invalid(),
        })
    };
    ($v:tt (.., id = $id:literal)) => {
        $crate::Event::$v(rotext_core::events::BlockWithId {
            id: rotext_core::BlockId::new($id),
        })
    };
}

/// 用于在编写测试用例时快速列举一系列属于 `Blend` 分组的事件。
macro_rules! events {
    ($(($($ev:tt)*)),*,) => {
        vec![$($crate::executing::tests::support::__event!($($ev)*)),*]
    };
}

pub(super) use __event;
pub(super) use case;
pub(super) use events;

use super::*;

#[allow(dead_code)]
pub(super) struct CaseOptions<'a> {
    pub tag_name_map: TagNameMap<'a>,
    #[cfg(feature = "block-id")]
    pub should_include_block_id: bool,
}

#[allow(dead_code)]
pub(super) struct Case<'a> {
    pub input: &'static str,
    /// 属于 `Blend` 分组的事件。
    pub input_events: Vec<Event>,
    pub expected: &'static str,

    #[allow(dead_code)]
    pub options: CaseOptions<'a>,
}
impl rotext_internal_test::support::Case for Case<'_> {
    fn assert_ok(&self) {
        let input = self.input.as_bytes();

        let tag_name_map = crate::TagNameMap::new_demo_instance_for_test();
        let compile_opts = crate::CompileOption {
            restrictions: crate::CompileRestrictions {
                max_call_depth_in_document: 100,
            },
        };
        let compiled = crate::compile(input, &self.input_events, &compile_opts).unwrap();

        let exec_opts = crate::ExecuteOptions {
            tag_name_map: &tag_name_map,
            block_extension_map: &fixtures::new_block_extension_map(),
            #[cfg(feature = "block-id")]
            should_include_block_ids: self.options.should_include_block_id,
        };
        let actual = crate::execute(input, &self.input_events, &compiled, &exec_opts);

        assert_eq!(self.expected, String::from_utf8(actual).unwrap());
    }

    fn input(&self) -> String {
        format!("{:?}", self.input_events)
    }
}

pub(super) fn run_cases(cases: Vec<GroupedCases<Case>>) {
    let failed_cases: Vec<_> = cases
        .iter()
        .flat_map(|row| -> Vec<FailedCase> { row.collect_failed() })
        .collect();

    if failed_cases.is_empty() {
        return;
    }
    let faild_case_count = failed_cases.len();

    report_panicked_cases(failed_cases);

    panic!("{} cases failed!", faild_case_count);
}
