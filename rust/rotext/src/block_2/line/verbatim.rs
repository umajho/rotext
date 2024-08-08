use std::ops::Range;

use crate::block_2::types::CursorContext;

pub struct EndCondition {
    pub on_fence: Fence,
}
pub struct Fence {
    pub character: u8,
    pub minimum_count: usize,
}

pub enum End {
    Eof,
    NewLine,
    Fence,
}

pub fn parse<TCtx: CursorContext>(
    input: &[u8],
    inner: &mut TCtx,
    end_condition: EndCondition,
    content_before: usize,
) -> crate::Result<(Range<usize>, End)> {
    todo!()
}
