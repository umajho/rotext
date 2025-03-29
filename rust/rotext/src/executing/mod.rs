use std::collections::{HashMap, HashSet};

mod renderer;

#[cfg(feature = "block-id")]
use rotext_core::BlockId;
use rotext_core::Event;

use crate::{compiling, CompiledItem};
pub use renderer::TagNameMap;

pub mod extensions;

pub struct NewExecutorOptions<'a> {
    pub tag_name_map: &'a TagNameMap<'a>,
    pub block_extension_map: &'a HashMap<&'a [u8], extensions::Extension<'a>>,

    #[cfg(feature = "block-id")]
    pub should_include_block_ids: bool,
}

pub struct Executor<'a> {
    tag_name_map: &'a TagNameMap<'a>,
    block_extension_map: &'a HashMap<&'a [u8], extensions::Extension<'a>>,

    #[cfg(feature = "block-id")]
    with_block_id: bool,

    renderer: renderer::Renderer<'a>,
}

enum CallType {
    Transclusion,
    Extension,
}
impl CallType {
    fn as_bytes(&self) -> &[u8] {
        match self {
            CallType::Transclusion => b"transclusion",
            CallType::Extension => b"extension",
        }
    }
}

enum CallError<'a> {
    /// 功能还没实现。TODO: 去掉此变体。
    Todo,

    /// 没有找到要调用的对象。
    UnknownCallee(&'a [u8]),
    BadParameters {
        normal: Option<Box<CallErrorBadParameters>>,
        verbatim: Option<Box<CallErrorBadParameters>>,
    },
}
impl CallError<'_> {
    fn destruct(self) -> (&'static [u8], Option<Vec<u8>>) {
        match self {
            CallError::Todo => (b"TODO", None),
            CallError::UnknownCallee(name) => (b"UnknownCallee", Some(name.to_vec())),
            CallError::BadParameters { normal, verbatim } => {
                let normal_vec = normal.map(|v| v.to_vec());
                let verbatim_vec = verbatim.map(|v| v.to_vec());
                let len = normal_vec.as_ref().map_or(0, |v| v.len())
                    + verbatim_vec.as_ref().map_or(0, |v| v.len());
                let mut buf = Vec::with_capacity(len + 1);
                if let Some(normal_vec) = normal_vec {
                    buf.extend_from_slice(&normal_vec);
                }
                buf.push(b';');
                if let Some(verbatim_vec) = verbatim_vec {
                    buf.extend_from_slice(&verbatim_vec);
                }

                (b"BadParameters", Some(buf))
            }
        }
    }
}

#[derive(Default)]
struct CallErrorBadParameters {
    missing: HashSet<Vec<u8>>,
    unknown: HashSet<Vec<u8>>,
    duplicated: HashSet<Vec<u8>>,
}
impl CallErrorBadParameters {
    /// 将存在问题的参数名统合成一个 `Vec<u8>` 字符串。
    ///
    /// 每一项参数名由 `,` 字符分隔，每一项的第一个字符代表该参数名存在的问题：
    /// - `!` 缺失了的必要参数。
    /// - `?` 未知参数。
    /// - `=` 重复参数。
    ///
    /// 参数名称中的 `~` 会被转义为 `~0`，`,` 会被转义为 `~1`，`;` 会被转义为 `~2`。
    fn to_vec(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        for name in &self.missing {
            buf.push(b'!');
            Self::render_escaped_parameter(&mut buf, name);
            buf.push(b',');
        }
        for name in &self.unknown {
            buf.push(b'?');
            Self::render_escaped_parameter(&mut buf, name);
            buf.push(b',');
        }
        for name in &self.duplicated {
            buf.push(b'=');
            Self::render_escaped_parameter(&mut buf, name);
            buf.push(b',');
        }
        buf.pop();
        buf
    }

    fn render_escaped_parameter(buf: &mut Vec<u8>, param: &[u8]) {
        for char in param {
            match *char {
                b'~' => buf.extend(b"~0"),
                b',' => buf.extend(b"~1"),
                b';' => buf.extend(b"~2"),
                char => buf.push(char),
            }
        }
    }
}

struct RenderCallErrorInput<'a> {
    call_type: CallType,
    call_name: &'a [u8],
    error: CallError<'a>,

    #[cfg(feature = "block-id")]
    block_id: BlockId,
}

impl<'a> Executor<'a> {
    pub fn new(opts: &NewExecutorOptions<'a>) -> Self {
        let renderer_opts = renderer::NewRendererOptions {
            tag_name_map: opts.tag_name_map,
            #[cfg(feature = "block-id")]
            should_include_block_ids: opts.should_include_block_ids,
        };

        Self {
            tag_name_map: opts.tag_name_map,
            block_extension_map: opts.block_extension_map,
            #[cfg(feature = "block-id")]
            with_block_id: opts.should_include_block_ids,
            renderer: renderer::Renderer::new(renderer_opts),
        }
    }

    pub fn execute(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        compiled: &[CompiledItem],
    ) {
        let mut stack: Vec<renderer::StackEntryBox> = vec![];

        for item in compiled {
            match item {
                CompiledItem::SimpleEvents(range) => {
                    let evs = &parsed[range.clone()];
                    self.renderer.render_events(buf, input, evs, &mut stack);
                }
                CompiledItem::BlockTransclusion(block_call) => {
                    self.render_call_error(
                        buf,
                        RenderCallErrorInput {
                            call_type: CallType::Transclusion,
                            call_name: block_call.name,
                            error: CallError::Todo,
                            #[cfg(feature = "block-id")]
                            block_id: block_call.block_id,
                        },
                    );
                }
                CompiledItem::BlockExtension(block_call) => {
                    self.render_block_extension(buf, input, parsed, block_call);
                }
            }
        }
    }

    fn render_block_extension(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        block_call: &crate::compiling::BlockCall<'a>,
    ) {
        let Some(ext) = self.block_extension_map.get(block_call.name) else {
            self.render_call_error(
                buf,
                RenderCallErrorInput {
                    call_type: CallType::Extension,
                    call_name: block_call.name,
                    error: CallError::UnknownCallee(block_call.name),
                    #[cfg(feature = "block-id")]
                    block_id: block_call.block_id,
                },
            );
            return;
        };

        let ext = if let extensions::Extension::Alias { to } = ext {
            self.block_extension_map.get(to).unwrap()
        } else {
            ext
        };

        match ext {
            extensions::Extension::ElementMapper(ext) => {
                self.render_block_element_mapper_extension(buf, input, parsed, block_call, ext);
            }
            extensions::Extension::Alias { .. } => unreachable!(),
        }
    }

    fn render_block_element_mapper_extension(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        block_call: &compiling::BlockCall<'a>,
        ext: &extensions::ExtensionElementMapper<'a>,
    ) {
        // 不记别名。
        let mut seen_params: HashSet<Vec<u8>> = HashSet::new();
        // 不记别名。
        let mut seen_verbatim_params: HashSet<Vec<u8>> = HashSet::new();

        let mut attrs: Vec<(&[u8], &[u8])> = vec![];
        let mut content: Vec<u8> = vec![];

        let mut bad: Option<CallErrorBadParameters> = None;
        let mut bad_verbatim: Option<CallErrorBadParameters> = None;

        for (key, value) in &block_call.arguments {
            self.process_block_element_mapper_extension_argument(
                &mut content,
                input,
                parsed,
                ProcessBlockElementMapperExtensionArgumentParameters {
                    ext,
                    key,
                    value,
                    seen: &mut seen_params,
                    bad: &mut bad,
                },
            );
        }

        for (key, value) in &block_call.verbatim_arguments {
            self.process_block_element_mapper_extension_verbatim_argument(
                &mut attrs,
                ext,
                key,
                value,
                &mut seen_verbatim_params,
                &mut bad_verbatim,
            );
        }

        let seen_params: HashSet<&[u8]> = seen_params.iter().map(|v| v.as_slice()).collect();
        let missing_params = ext
            .required_parameters
            .difference(&seen_params)
            .collect::<HashSet<_>>();
        if !missing_params.is_empty() {
            let bad = bad.get_or_insert_with(CallErrorBadParameters::default);
            bad.missing = missing_params.iter().map(|v| v.to_vec()).collect();
        }

        let seen_verbatim_params: HashSet<&[u8]> =
            seen_verbatim_params.iter().map(|v| v.as_slice()).collect();
        let missing_verbatim_params = ext
            .required_verbatim_parameters
            .difference(&seen_verbatim_params)
            .collect::<HashSet<_>>();
        if !missing_verbatim_params.is_empty() {
            let bad_verbatim = bad_verbatim.get_or_insert_with(CallErrorBadParameters::default);
            bad_verbatim.missing = missing_verbatim_params.iter().map(|v| v.to_vec()).collect();
        }

        if bad.is_some() || bad_verbatim.is_some() {
            self.render_call_error(
                buf,
                RenderCallErrorInput {
                    call_type: CallType::Extension,
                    call_name: block_call.name,
                    error: CallError::BadParameters {
                        normal: bad.map(Box::new),
                        verbatim: bad_verbatim.map(Box::new),
                    },
                    #[cfg(feature = "block-id")]
                    block_id: block_call.block_id,
                },
            );
            return;
        }

        if let Some(variant) = ext.variant {
            attrs.push((b"variant", variant));
        }

        #[cfg(feature = "block-id")]
        let mut buffer = if self.with_block_id {
            Some(itoa::Buffer::new())
        } else {
            None
        };
        #[cfg(feature = "block-id")]
        if let Some(buffer) = &mut buffer {
            attrs.push((
                b"data-block-id",
                buffer.format(block_call.block_id.value()).as_bytes(),
            ));
        }

        crate::utils::render_eopening_tag(buf, ext.tag_name, &attrs);
        buf.extend(&content);
        crate::utils::render_closing_tag(buf, ext.tag_name);
    }

    fn process_block_element_mapper_extension_argument(
        &self,
        content_buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        params: ProcessBlockElementMapperExtensionArgumentParameters<'a, '_>,
    ) {
        let key_vec = params.key.to_vec();
        let param = params.ext.get_real_parameter(&key_vec);
        let Some(param) = param else {
            let bad = params
                .bad
                .get_or_insert_with(CallErrorBadParameters::default);
            bad.unknown.insert(key_vec);
            return;
        };
        if params.seen.contains(&key_vec) {
            let bad = params
                .bad
                .get_or_insert_with(CallErrorBadParameters::default);
            bad.duplicated.insert(key_vec);
            return;
        } else {
            params.seen.insert(key_vec);
        }
        if params.bad.is_some() {
            return;
        }

        match param.mapping_to {
            extensions::ExtensionElementMapperParameterMappingTo::NamedSlot(slot_name) => {
                crate::utils::render_eopening_tag(content_buf, b"div", &[(b"slot", slot_name)]);
                self.execute(content_buf, input, parsed, params.value);
                crate::utils::render_closing_tag(content_buf, b"div");
            }
            extensions::ExtensionElementMapperParameterMappingTo::UnnamedSlot => {
                self.execute(content_buf, input, parsed, params.value);
            }
        }
    }

    fn process_block_element_mapper_extension_verbatim_argument(
        &self,
        attrs: &mut Vec<(&'a [u8], &'a [u8])>,
        ext: &'a extensions::ExtensionElementMapper<'a>,
        key: &'a [u8],
        value: &'a [u8],
        seen: &mut HashSet<Vec<u8>>,
        bad: &mut Option<CallErrorBadParameters>,
    ) {
        let key_vec = key.to_vec();
        let param = ext.get_real_verbatim_parameter(&key_vec);
        let Some(param) = param else {
            let bad = bad.get_or_insert_with(CallErrorBadParameters::default);
            bad.unknown.insert(key_vec);
            return;
        };
        if seen.contains(&key_vec) {
            let bad = bad.get_or_insert_with(CallErrorBadParameters::default);
            bad.duplicated.insert(key_vec);
            return;
        } else {
            seen.insert(key_vec);
        }
        if bad.is_some() {
            return;
        }

        attrs.push((param.mapping_to_attribute, value));
    }

    fn render_call_error(&self, buf: &mut Vec<u8>, input: RenderCallErrorInput<'_>) {
        let (error_type, ref error_value) = input.error.destruct();

        let mut attrs = vec![
            (&b"call-type"[..], input.call_type.as_bytes()),
            (b"call-name", input.call_name),
            (b"error-type", error_type),
        ];
        if let Some(error_value) = error_value {
            attrs.push((b"error-value", error_value));
        }

        #[cfg(feature = "block-id")]
        let mut buffer = if self.with_block_id {
            Some(itoa::Buffer::new())
        } else {
            None
        };
        #[cfg(feature = "block-id")]
        if let Some(buffer) = &mut buffer {
            attrs.push((
                b"data-block-id",
                buffer.format(input.block_id.value()).as_bytes(),
            ));
        }

        crate::utils::render_empty_element(buf, self.tag_name_map.block_call_error, &attrs);
    }
}

struct ProcessBlockElementMapperExtensionArgumentParameters<'a, 'b> {
    ext: &'b extensions::ExtensionElementMapper<'a>,
    key: &'b compiling::ArgumentKey<'a>,
    value: &'b Vec<CompiledItem<'b>>,
    seen: &'b mut HashSet<Vec<u8>>,
    bad: &'b mut Option<CallErrorBadParameters>,
}
