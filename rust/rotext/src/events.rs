#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    // 在全局阶段产出，由块级阶段和行内阶段逐渐消耗。
    Unparsed = 1,

    // 在全局阶段产出，在块级阶段消耗（转化为 [EventType::Text）。
    VerbatimEscaping = 101,

    // 换行相关，皆在全局阶段产出。其中 CR 在块级阶段消耗（可能转化为 LF），LF
    // 在块级阶段、行内阶段保留或消耗。
    CarriageReturn = 201,
    LineFeed = 202,

    // 在块级阶段与行内阶段产出。
    Text = 1001,
    Exit = 1011,
    Separator = 1021,

    // 在块级阶段产出。
    EnterParagraph = 10001,
    ThematicBreak = 10011,
    EnterHeading1 = 10021,
    EnterHeading2 = 10022,
    EnterHeading3 = 10023,
    EnterHeading4 = 10024,
    EnterHeading5 = 10025,
    EnterHeading6 = 10026,
    /// XXX: 数字是临时的。
    EnterCodeBlock = 19011,
}

#[cfg(test)]
impl From<u32> for EventType {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
