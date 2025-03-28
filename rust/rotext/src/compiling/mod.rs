mod renderer;

pub use renderer::TagNameMap;

use rotext_core::{BlockId, Event};

use renderer::StackEntryBox;

pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error {
    RecursionDepthExceeded,
}

impl Error {
    pub fn name(&self) -> &'static str {
        match self {
            Error::RecursionDepthExceeded => "RecursionDepthExceeded",
        }
    }
}

pub enum CompiledItem<'a> {
    Rendered(Vec<u8>),
    BlockTransclusion(BlockCall<'a>),
    BlockExtension(BlockCall<'a>),
}

pub struct BlockCall<'a> {
    pub name: &'a [u8],
    pub arguments: Vec<(ArgumentKey<'a>, Vec<CompiledItem<'a>>)>,
    pub verbatim_arguments: Vec<(&'a [u8], Vec<u8>)>,

    pub block_id: BlockId,
}

#[derive(Eq, Hash, PartialEq)]
pub enum ArgumentKey<'a> {
    Named(&'a [u8]),
    Anonymous(usize),
}
impl ArgumentKey<'_> {
    pub fn to_vec(&self) -> Vec<u8> {
        match self {
            ArgumentKey::Named(name) => name.to_vec(),
            ArgumentKey::Anonymous(index) => {
                let mut buffer = itoa::Buffer::new();
                buffer.format(*index).as_bytes().to_vec()
            }
        }
    }
}

pub struct NewCompileOptions<'a> {
    pub restrictions: Restrictions,

    pub tag_name_map: &'a TagNameMap<'a>,

    #[cfg(feature = "block-id")]
    pub should_include_block_ids: bool,
}

pub struct Restrictions {
    /// 单份文档中最多允许的调用（包括最外层的）的嵌套数量。
    pub max_call_depth_in_document: usize,
}

pub struct Compiler<'a> {
    restrictions: &'a Restrictions,

    renderer: renderer::Renderer<'a>,
}

impl<'a> Compiler<'a> {
    pub fn new(opts: &'a NewCompileOptions<'a>) -> Self {
        let renderer_opts = renderer::NewRendererOptions {
            tag_name_map: opts.tag_name_map,
            #[cfg(feature = "block-id")]
            should_include_block_ids: opts.should_include_block_ids,
        };

        Self {
            restrictions: &opts.restrictions,
            renderer: renderer::Renderer::new(renderer_opts),
        }
    }

    pub fn compile(&self, input: &'a [u8], evs: &[Event]) -> Result<Vec<CompiledItem<'a>>> {
        let (_, result) = self.compile_internal(1, input, evs, 0)?;
        Ok(result)
    }

    fn compile_internal(
        &self,
        depth: usize,
        input: &'a [u8],
        evs: &[Event],
        mut i: usize,
    ) -> Result<(usize, Vec<CompiledItem<'a>>)> {
        if depth > self.restrictions.max_call_depth_in_document {
            return Err(Error::RecursionDepthExceeded);
        }

        let mut result: Vec<CompiledItem> = vec![];
        let mut last_rendered: Option<Vec<u8>> = None;

        let mut stack: Vec<StackEntryBox> = vec![];

        loop {
            if i >= evs.len() {
                if let Some(last_rendered) = last_rendered.take() {
                    result.push(CompiledItem::Rendered(last_rendered));
                }
                return Ok((i, result));
            }

            match &evs[i] {
                Event::ExitBlock(_)
                | Event::IndicateCallNormalArgument(_)
                | Event::IndicateCallVerbatimArgument(_)
                    if stack.is_empty() =>
                {
                    if let Some(last_rendered) = last_rendered.take() {
                        result.push(CompiledItem::Rendered(last_rendered));
                    }
                    return Ok((i, result));
                }
                Event::IndicateCallNormalArgument(_) | Event::IndicateCallVerbatimArgument(_) => {
                    unreachable!()
                }
                Event::EnterCallOnTemplate(call) | Event::EnterCallOnExtension(call) => {
                    let is_transclusion = matches!(evs[i], Event::EnterCallOnTemplate(_));

                    if let Some(last_rendered) = last_rendered.take() {
                        result.push(CompiledItem::Rendered(last_rendered));
                    }

                    let mut call_compiled: BlockCall = BlockCall {
                        name: &input[call.name.clone()],
                        arguments: Vec::new(),
                        verbatim_arguments: Vec::new(),
                        block_id: call.id,
                    };

                    let mut anonymous_argument_name_generator =
                        crate::utils::SequenceGenerator::new(1);

                    i += 1;
                    loop {
                        match &evs[i] {
                            Event::ExitBlock(_) => {
                                result.push(if is_transclusion {
                                    CompiledItem::BlockTransclusion(call_compiled)
                                } else {
                                    CompiledItem::BlockExtension(call_compiled)
                                });
                                i += 1;
                                break;
                            }
                            Event::IndicateCallNormalArgument(arg_name) => {
                                let arg_name = if let Some(arg_name) = arg_name {
                                    ArgumentKey::Named(&input[arg_name.clone()])
                                } else {
                                    ArgumentKey::Anonymous(anonymous_argument_name_generator.next())
                                };

                                let value: Vec<CompiledItem>;
                                (i, value) = self.compile_internal(depth + 1, input, evs, i + 1)?;
                                call_compiled.arguments.push((arg_name, value));
                            }
                            Event::IndicateCallVerbatimArgument(arg_name) => {
                                let mut value: Vec<u8> = vec![];

                                loop {
                                    i += 1;
                                    match &evs[i] {
                                        Event::Text(content) => {
                                            value.extend(&input[content.clone()])
                                        }
                                        Event::NewLine(_) => value.push(b'\n'),
                                        _ => break,
                                    }
                                }
                                call_compiled
                                    .verbatim_arguments
                                    .push((&input[arg_name.clone()], value));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => {
                    let buf = last_rendered.get_or_insert_with(Vec::new);
                    i = self.renderer.render_event(buf, input, evs, i, &mut stack);
                }
            }
        }
    }
}
