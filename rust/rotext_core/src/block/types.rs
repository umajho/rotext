use crate::{
    types::{LineNumber, Tym},
    Event,
};

/// 只提供读取与移动 cursor，以及读取及增加当前行数的上下文。
pub trait CursorContext {
    fn cursor(&self) -> usize;
    fn set_cursor(&mut self, cursor: usize);
    fn move_cursor_forward(&mut self, n: usize);

    fn current_line(&self) -> LineNumber;
    /// 如果 `is_significant` 为真，代表该换行会影响到整体的解析状态。在注释和逐
    /// 字转义中，换行不会有如此影响，因此应传入 `false`；而通常情况下的换行会有
    /// 如此影响，因此应传入 `true`。
    fn increase_current_line(&mut self, is_significant: bool);
}
pub trait YieldContext {
    /// `ev` 是属于 `Block` 分组的事件。
    #[must_use]
    fn r#yield(&mut self, ev: Event) -> Tym<1>;
}
