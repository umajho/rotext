#[allow(unused_macros)]
macro_rules! case {
    ($input:expr, [$(($($ev:tt)*)),*,], $expected:expr,) => {
        $crate::rendering::tests::support::Case {
            input_events: vec![$(case!(_input_event, $($ev)*)),*],
            input: $input,
            expected: $expected,

            options: $crate::rendering::tests::support::CaseOptions {
                tag_name_map: Default::default(),
                #[cfg(feature = "block-id")]
                with_block_id: false,
            },
        }
    };
    (_input_event, VerbatimEscaping ($start:literal..$end:literal)) => {
        $crate::events::BlendEvent::VerbatimEscaping($crate::events::VerbatimEscaping {
            content: case!(_range, $start..$end),
            is_closed_forcedly: false,
        })
    };
    (_input_event, NewLine (..)) => {
        $crate::events::BlendEvent::NewLine($crate::events::NewLine {})
    };
    (_input_event, Text ($start:literal..$end:literal)) => {
        $crate::events::BlendEvent::Text(case!(_range, $start..$end))
    };
    (_input_event, IndicateCodeBlockCode ()) => {
        $crate::events::BlendEvent::IndicateCodeBlockCode
    };
    (_input_event, IndicateTableRow ()) => {
        $crate::events::BlendEvent::IndicateTableRow
    };
    (_input_event, IndicateTableHeaderCell ()) => {
        $crate::events::BlendEvent::IndicateTableHeaderCell
    };
    (_input_event, IndicateTableDataCell ()) => {
        $crate::events::BlendEvent::IndicateTableDataCell
    };
    (_input_event, ExitBlock (..)) => {
        $crate::events::BlendEvent::ExitBlock($crate::events::ExitBlock {})
    };
    (_input_event, ThematicBreak (..)) => {
        $crate::events::BlendEvent::ThematicBreak($crate::events::ThematicBreak {})
    };
    (_input_event, $v:tt (..)) => {
        $crate::events::BlendEvent::$v($crate::events::BlockWithID {})
    };
    (_input_event, $v:path, $($tt:tt)*) => {
        $v($($tt)*)
    };
    (_range, $start:literal..$end:literal) => {
        $crate::common::Range::new($start, $end - $start)
    }
}

pub(super) use case;

use super::*;

#[allow(dead_code)]
pub(super) struct CaseOptions<'a> {
    pub tag_name_map: TagNameMap<'a>,
    #[cfg(feature = "block-id")]
    pub with_block_id: bool,
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
            should_include_block_ids: self.options.with_block_id,
        };
        let renderer = HtmlRenderer::new(self.input.as_bytes(), opts);
        let actual = renderer.render(self.input_events.clone().into_iter());

        assert_eq!(self.expected, actual);
    }

    fn input(&self) -> String {
        format!("{:?}", self.input_events)
    }
}
