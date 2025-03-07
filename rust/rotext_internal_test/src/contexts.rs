use std::marker::PhantomData;

use rotext_core::{BlockParser, BlockStackEntry, Event, InlineStackEntry, Stack};

pub struct BlockContext<TStack: Stack<BlockStackEntry>> {
    phantom_stack: PhantomData<TStack>,
}
impl<TStack: Stack<BlockStackEntry>> BlockContext<TStack> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom_stack: PhantomData::<TStack>,
        }
    }
}

impl<TStack: Stack<BlockStackEntry>> crate::suites::block::Context for BlockContext<TStack> {
    /// 返回的事件都属于 `Block` 分组。
    fn parse(input: &str) -> impl Iterator<Item = rotext_core::Result<Event>> {
        let block_parser: BlockParser<TStack> = BlockParser::new(input.as_bytes());

        block_parser
    }
}

pub struct BlendContext<TBlockStack: Stack<BlockStackEntry>, TInlineStack: Stack<InlineStackEntry>>
{
    phantom_block_stack: PhantomData<TBlockStack>,
    phantom_inline_stack: PhantomData<TInlineStack>,
}
impl<TBlockStack: Stack<BlockStackEntry>, TInlineStack: Stack<InlineStackEntry>>
    BlendContext<TBlockStack, TInlineStack>
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom_block_stack: PhantomData::<TBlockStack>,
            phantom_inline_stack: PhantomData::<TInlineStack>,
        }
    }
}
impl<TBlockStack: Stack<BlockStackEntry>, TInlineStack: Stack<InlineStackEntry>>
    crate::suites::blend::Context for BlendContext<TBlockStack, TInlineStack>
{
    /// 返回的事件都属于 `Blend` 分组。
    fn parse(input: &str) -> impl Iterator<Item = rotext_core::Result<Event>> {
        rotext_core::parse::<TBlockStack, TInlineStack>(input.as_bytes())
    }
}

pub struct InlineContext<TBlockStack: Stack<BlockStackEntry>, TInlineStack: Stack<InlineStackEntry>>
{
    /// 由于目前针对行内解析的测试是通过调用完整的解析器来进行的，因此需要这个。
    phantom_block_stack: PhantomData<TBlockStack>,
    phantom_inline_stack: PhantomData<TInlineStack>,
}
impl<TBlockStack: Stack<BlockStackEntry>, TInlineStack: Stack<InlineStackEntry>>
    InlineContext<TBlockStack, TInlineStack>
{
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            phantom_block_stack: PhantomData::<TBlockStack>,
            phantom_inline_stack: PhantomData::<TInlineStack>,
        }
    }
}
impl<TBlockStack: Stack<BlockStackEntry>, TInlineStack: Stack<InlineStackEntry>>
    crate::suites::inline::Context for InlineContext<TBlockStack, TInlineStack>
{
    /// 返回的事件都属于 `Inline` 分组。
    fn parse(input: &str) -> Vec<Event> {
        // [parse] 返回的结果是一系列 `Blend` 分组的事件。
        let evs: rotext_core::Result<Vec<_>> =
            rotext_core::parse::<TBlockStack, TInlineStack>(input.as_bytes()).collect();
        let evs = match evs {
            Ok(evs) => evs,
            Err(_) => todo!("should yield err!"),
        };

        let evs = if !evs.is_empty() {
            if !matches!(evs.first(), Some(Event::EnterParagraph(_))) {
                panic!("the input should be a paragraph!")
            }
            if !matches!(evs.last(), Some(Event::ExitBlock(_))) {
                unreachable!()
            }
            evs[1..evs.len() - 1].to_vec()
        } else {
            evs
        };

        if evs.iter().any(|ev| matches!(ev, Event::ExitBlock(_))) {
            panic!("the input should be ONE paragraph! input: {:?}", evs)
        }

        evs.iter()
            .for_each(|item| debug_assert!(rotext_core::is_event_of!(Inline, item)));
        evs
    }
}
