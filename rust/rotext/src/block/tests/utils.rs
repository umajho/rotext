macro_rules! case {
    ($input_variants:expr, $expected:expr) => {
        $crate::block::tests::Case {
            input_variants: $input_variants,
            expected: $expected,
        }
    };
}

pub(crate) use case;
