/// 解析事件。
///
/// 各项文档开头标记的含义：
/// - “<…>” 在全局阶段， “{…}” 在块级阶段， “[…]” 在行内阶段；
/// - “+” 产出，“-” 消耗前一阶段，“--” 消耗前一阶段全部，“~” 传递。
/// - 某个阶段的括号为空代表那个阶段不感知对应事件。
#[derive(Debug)]
#[repr(u32)]
pub enum Event {
    /// <+> → {--+} → [-- ] 有待下个阶段决定。自闭合。
    Undetermined { start: usize, length: usize } = 1,

    /// <+> → {-- } → [   ] 注释。自闭合。
    Comment {
        content_start: usize,
        content_length: usize,
        is_closed_forcedly: bool,
    } = 10001,
    /// <+> → {- ~} → [-- ] 逐字文本转义。自闭合。
    ///
    /// NOTE: 内容包含开头和结尾各可能存在的一个空格。这些空格在渲染时才删去。
    VerbatimEscaping {
        content_start: usize,
        content_length: usize,
        is_closed_forcedly: bool,
    } = 10002,
}

impl Event {
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }
}
