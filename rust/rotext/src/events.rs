#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    Undetermined = 1,
    LineFeed = 2,

    Exit = 1001,

    VerbatimEscaping = 10001,
    Comment = 10002,

    EnterParagraph = 20001,
    /// XXX: 数字是临时的。
    ThematicBreak = 20098,
    /// XXX: 数字是临时的。
    CodeBlock = 20099,
}

#[cfg(test)]
impl From<u32> for EventType {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
