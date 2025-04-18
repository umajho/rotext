use core::ops::Range;

pub fn count_continuous_character(input: &[u8], char: u8, since: usize) -> usize {
    let mut i = 0;
    while matches!(input.get(since+ i), Some(actual_char) if *actual_char == char) {
        i += 1;
    }

    i
}

pub fn count_continuous_character_with_maximum(
    input: &[u8],
    char: u8,
    since: usize,
    maximum: usize,
) -> usize {
    let mut i = 0;
    while i < maximum && matches!(input.get(since + i), Some(actual_char) if *actual_char == char) {
        i += 1;
    }

    i
}

pub fn count_continuous_whitespaces(input: &[u8], since: usize) -> usize {
    let mut i = 0;
    while matches!(input.get(since+ i), Some(actual_char) if is_whitespace!(actual_char)) {
        i += 1;
    }

    i
}

macro_rules! is_whitespace {
    ($char:expr) => {
        matches!($char, b' ' | b'\t')
    };
}

pub(crate) use is_whitespace;

pub fn trim_end(input: &[u8], range: Range<usize>) -> Range<usize> {
    let mut end: usize = range.end;
    while end > range.start && is_whitespace!(input[end - 1]) {
        end -= 1;
    }
    range.start..end
}
