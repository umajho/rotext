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
        $crate::rendering::tests::support::Case {
            input_events: $crate::rendering::tests::support::events!($($ev_tts)*),
            input: $input,
            expected: $expected,

            options: $crate::rendering::tests::support::CaseOptions {
                tag_name_map: Default::default(),
                #[cfg(feature = "block-id")]
                should_include_block_id: $should_include_block_id,
            },
        }
    };
}

macro_rules! __event {
    (VerbatimEscaping ($start:literal..$end:literal)) => {
        $crate::events::Event::VerbatimEscaping($crate::events::VerbatimEscaping {
            content: $start..$end,
            is_closed_forcedly: false,
            line_after: $crate::types::LineNumber::new_invalid(),
        })
    };

    (NewLine (..)) => {
        $crate::events::Event::NewLine($crate::events::NewLine {
            line_after: $crate::types::LineNumber::new_invalid(),
        })
    };

    (Text ($start:literal..$end:literal)) => {
        $crate::events::Event::Text($start..$end)
    };

    (IndicateCodeBlockCode ()) => {
        $crate::events::Event::IndicateCodeBlockCode
    };
    (IndicateTableCaption ()) => {
        $crate::events::Event::IndicateTableCaption
    };
    (IndicateTableRow ()) => {
        $crate::events::Event::IndicateTableRow
    };
    (IndicateTableHeaderCell ()) => {
        $crate::events::Event::IndicateTableHeaderCell
    };
    (IndicateTableDataCell ()) => {
        $crate::events::Event::IndicateTableDataCell
    };

    (ExitBlock (..)) => {
        $crate::events::Event::ExitBlock($crate::events::ExitBlock {
            id: $crate::types::BlockId::new_invalid(),
            start_line: $crate::types::LineNumber::new_invalid(),
            end_line: $crate::types::LineNumber::new_invalid(),
        })
    };
    (ExitBlock (.., id = $id:literal)) => {
        $crate::events::Event::ExitBlock($crate::events::ExitBlock {
            id: $crate::types::BlockId::new($id),
            start_line: $crate::types::LineNumber::new_invalid(),
            end_line: $crate::types::LineNumber::new_invalid(),
        })
    };
    (ExitBlock (.., id = $id:literal, lns = $ln_s:literal..=$ln_e:literal)) => {
        $crate::events::Event::ExitBlock($crate::events::ExitBlock {
            id: $crate::types::BlockId::new($id),
            start_line: $crate::types::LineNumber::new($ln_s),
            end_line: $crate::types::LineNumber::new($ln_e),
        })
    };

    (ThematicBreak (..)) => {
        $crate::events::Event::ThematicBreak($crate::events::ThematicBreak {
            id: $crate::types::BlockId::new_invalid(),
            line: $crate::types::LineNumber::new_invalid(),
        })
    };
    (ThematicBreak (.., id = $id:literal)) => {
        $crate::events::Event::ThematicBreak($crate::events::ThematicBreak {
            id: $crate::types::BlockId::new($id),
            line: $crate::types::LineNumber::new_invalid(),
        })
    };
    (ThematicBreak (.., id = $id:literal, ln = $ln:literal)) => {
        $crate::events::Event::ThematicBreak($crate::events::ThematicBreak {
            id: $crate::types::BlockId::new($id),
            line: $crate::types::LineNumber::new($ln),
        })
    };

    (RefLink ($start:literal..$end:literal)) => {
        $crate::events::Event::RefLink($start..$end)
    };
    (Dicexp ($start:literal..$end:literal)) => {
        $crate::events::Event::Dicexp($start..$end)
    };

    (EnterWikiLink ($start:literal..$end:literal)) => {
        $crate::events::Event::EnterWikiLink($start..$end)
    };

    (@inline $v:tt (..)) => {
        $crate::events::Event::$v
    };

    ($v:tt (..)) => {
        $crate::events::Event::$v($crate::events::BlockWithId {
            id: $crate::types::BlockId::new_invalid(),
        })
    };
    ($v:tt (.., id = $id:literal)) => {
        $crate::events::Event::$v($crate::events::BlockWithId {
            id: $crate::types::BlockId::new($id),
        })
    };
}

/// 用于在编写测试用例时快速列举一系列属于 `Blend` 分组的事件。
macro_rules! events {
    ($(($($ev:tt)*)),*,) => {
        vec![$($crate::rendering::tests::support::__event!($($ev)*)),*]
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
impl test_support::Case for Case<'_> {
    fn assert_ok(&self) {
        let opts = NewHtmlRendererOptions {
            tag_name_map: self.options.tag_name_map.clone(),
            initial_output_string_capacity: 0,
            #[cfg(feature = "block-id")]
            should_include_block_ids: self.options.should_include_block_id,
        };
        let renderer = HtmlRenderer::new(self.input.as_bytes(), opts);
        let actual = renderer.render(self.input_events.clone().into_iter());

        assert_eq!(self.expected, actual);
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
