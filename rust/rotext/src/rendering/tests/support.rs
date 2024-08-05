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

macro_rules! event {
    (VerbatimEscaping ($start:literal..$end:literal)) => {
        $crate::events::BlendEvent::VerbatimEscaping($crate::events::VerbatimEscaping {
            content: $start..$end,
            is_closed_forcedly: false,
            line_number_after: $crate::types::LineNumber::new(),
        })
    };
    (NewLine (..)) => {
        $crate::events::BlendEvent::NewLine($crate::events::NewLine {
            line_number_after: $crate::types::LineNumber::new(),
        })
    };
    (Text ($start:literal..$end:literal)) => {
        $crate::events::BlendEvent::Text($start..$end)
    };
    (IndicateCodeBlockCode ()) => {
        $crate::events::BlendEvent::IndicateCodeBlockCode
    };
    (IndicateTableCaption ()) => {
        $crate::events::BlendEvent::IndicateTableCaption
    };
    (IndicateTableRow ()) => {
        $crate::events::BlendEvent::IndicateTableRow
    };
    (IndicateTableHeaderCell ()) => {
        $crate::events::BlendEvent::IndicateTableHeaderCell
    };
    (IndicateTableDataCell ()) => {
        $crate::events::BlendEvent::IndicateTableDataCell
    };
    (ExitBlock (..)) => {
        $crate::events::BlendEvent::ExitBlock($crate::events::ExitBlock {
            id: $crate::types::BlockId::new(),
            start_line_number: $crate::types::LineNumber::new(),
            end_line_number: $crate::types::LineNumber::new(),
        })
    };
    (ExitBlock (.., id = $id:literal, lns = $ln_s:literal..$ln_e:literal)) => {
        $crate::events::BlendEvent::ExitBlock($crate::events::ExitBlock {
            id: $crate::types::BlockId::new($id),
            start_line_number: $crate::types::LineNumber::new($ln_s),
            end_line_number: $crate::types::LineNumber::new($ln_e),
        })
    };
    (ThematicBreak (..)) => {
        $crate::events::BlendEvent::ThematicBreak($crate::events::ThematicBreak {
            id: $crate::types::BlockId::new(),
            line_number: $crate::types::LineNumber::new(),
        })
    };
    (ThematicBreak (.., id = $id:literal, ln = $ln:literal)) => {
        $crate::events::BlendEvent::ThematicBreak($crate::events::ThematicBreak {
            id: $crate::types::BlockId::new($id),
            line_number: $crate::types::LineNumber::new($ln),
        })
    };
    ($v:tt (..)) => {
        $crate::events::BlendEvent::$v($crate::events::BlockWithId {
            id: $crate::types::BlockId::new(),
        })
    };
    ($v:tt (.., id = $id:literal)) => {
        $crate::events::BlendEvent::$v($crate::events::BlockWithId {
            id: $crate::types::BlockId::new($id),
        })
    };
}

macro_rules! events {
    ($(($($ev:tt)*)),*,) => {
        vec![$($crate::rendering::tests::support::event!($($ev)*)),*]
    };
}

pub(super) use case;
pub(super) use event;
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
    pub input_events: Vec<BlendEvent>,
    pub expected: &'static str,

    #[allow(dead_code)]
    pub options: CaseOptions<'a>,
}
impl<'a> test_support::Case for Case<'a> {
    fn assert_ok(&self) {
        let opts = NewHtmlRendererOptoins {
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
