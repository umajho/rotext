#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    Undetermined = 1,
    LineFeed = 2,

    Exit = 1001,

    VerbatimEscaping = 10001,

    EnterParagraph = 20001,
    /// XXX: 数字是临时的。
    ThematicBreak = 29001,
    /// XXX: 数字是临时的。
    EnterCodeBlock = 29011,
    /// XXX: 数字是临时的。
    EnterCodeBlockMeta = 29012,
    /// XXX: 数字是临时的。
    EnterCodeBlockContent = 29013,

    Text = 30001,
}

#[cfg(test)]
impl From<u32> for EventType {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
