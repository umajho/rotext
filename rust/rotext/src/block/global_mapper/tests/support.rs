macro_rules! new_line {
    ($line_number_after:expr) => {
        $crate::block::global_mapper::Mapped::NewLine($crate::block::global_mapper::NewLine {
            #[cfg(feature = "line-number")]
            line_number_after: $line_number_after,
        })
    };
}
macro_rules! verbatim_escaping {
    ($content:expr, $line_number_after:expr) => {
        $crate::block::global_mapper::Mapped::VerbatimEscaping(
            $crate::block::global_mapper::VerbatimEscaping {
                content: $content,
                is_closed_forcedly: false,
                #[cfg(feature = "line-number")]
                line_number_after: $line_number_after,
            },
        )
    };
    ($content:expr, $line_number_after:expr, "F") => {
        $crate::block::global_mapper::Mapped::VerbatimEscaping(
            $crate::block::global_mapper::VerbatimEscaping {
                content: $content,
                is_closed_forcedly: true,
                #[cfg(feature = "line-number")]
                line_number_after: $line_number_after,
            },
        )
    };
}

macro_rules! case {
    ($input:expr, $expected:expr) => {
        $crate::block::global_mapper::tests::support::Case {
            input: $input,
            expected: $expected,
        }
    };
}

pub(super) use case;
pub(super) use new_line;
pub(super) use verbatim_escaping;

use super::*;

pub(super) struct Case {
    pub input: &'static str,
    pub expected: Vec<Mapped>,
}
impl test_support::Case for Case {
    fn assert_ok(&self) {
        let global_parser =
            global::Parser::new(self.input.as_bytes(), global::NewParserOptions::default());
        let global_mapper = GlobalEventStreamMapper::new(self.input.as_bytes(), global_parser);

        let actual: Vec<_> = global_mapper.collect();

        assert_eq!(self.expected, actual);
    }

    fn input(&self) -> String {
        self.input.to_string()
    }
}
