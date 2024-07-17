#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    // 在全局阶段产出，由块级阶段和行内阶段逐渐消耗。
    Unparsed = 1,

    // 换行相关，皆在全局阶段产出。其中 CR 在块级阶段消耗（可能转化为 LF），LF
    // 在块级阶段、行内阶段保留或消耗。
    CarriageReturn = 11,
    LineFeed = 12,

    // 在块级阶段与行内阶段产出。
    Exit = 1001,

    // 在全局阶段产出，在块级阶段消耗（转化为 [EventType::Text）。
    VerbatimEscaping = 10001,

    // 在块级阶段产出。
    EnterParagraph = 20001,
    ThematicBreak = 20011,
    /// XXX: 数字是临时的。
    EnterCodeBlock = 29011,
    /// XXX: 数字是临时的。
    EnterCodeBlockMeta = 29012,
    /// XXX: 数字是临时的。
    EnterCodeBlockContent = 29013,

    // 在块级阶段与行内阶段产出。
    Text = 30001,
}

#[cfg(test)]
impl From<u32> for EventType {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
