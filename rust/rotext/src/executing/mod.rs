mod renderer;

#[cfg(test)]
mod tests;

use std::collections::{HashMap, HashSet};

#[cfg(feature = "block-id")]
use rotext_core::BlockId;
use rotext_core::Event;

use crate::{CompiledItem, compiling};
pub use renderer::TagNameMap;

pub mod extensions;

pub struct NewExecutorOptions<'a> {
    pub tag_name_map: &'a TagNameMap<'a>,
    pub block_extension_map: &'a HashMap<&'a [u8], extensions::Extension<'a>>,
    pub inline_extension_map: &'a HashMap<&'a [u8], extensions::Extension<'a>>,

    #[cfg(feature = "block-id")]
    pub should_include_block_ids: bool,
}

pub struct Executor<'a> {
    tag_name_map: &'a TagNameMap<'a>,
    block_extension_map: &'a HashMap<&'a [u8], extensions::Extension<'a>>,
    inline_extension_map: &'a HashMap<&'a [u8], extensions::Extension<'a>>,

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
    is_block: bool,
    call_type: CallType,
    call_name: &'a [u8],
    error: CallError<'a>,

    #[cfg(feature = "block-id")]
    block_id: Option<BlockId>,
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
            inline_extension_map: opts.inline_extension_map,
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
                CompiledItem::BlockTransclusion(call) | CompiledItem::InlineTransclusion(call) => {
                    self.render_call_error(buf, RenderCallErrorInput {
                        is_block: matches!(item, CompiledItem::BlockTransclusion(_)),
                        call_type: CallType::Transclusion,
                        call_name: call.name,
                        error: CallError::Todo,
                        #[cfg(feature = "block-id")]
                        block_id: call.block_id,
                    });
                }
                CompiledItem::BlockExtension(call) => {
                    self.render_block_extension(buf, input, parsed, call);
                }
                CompiledItem::InlineExtension(call) => {
                    self.render_inline_extension(buf, input, parsed, call);
                }
            }
        }
    }

    fn render_block_extension(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        call: &crate::compiling::CompiledItemCall<'a>,
    ) {
        #[cfg(all(debug_assertions, feature = "block-id"))]
        {
            assert!(call.block_id.is_some());
        }

        let is_block = true;

        let Some(ext) = self.block_extension_map.get(call.name) else {
            self.render_call_error(buf, RenderCallErrorInput {
                is_block,
                call_type: CallType::Extension,
                call_name: call.name,
                error: CallError::UnknownCallee(call.name),
                #[cfg(feature = "block-id")]
                block_id: call.block_id,
            });
            return;
        };

        let ext = if let extensions::Extension::Alias { to } = ext {
            self.block_extension_map.get(to).unwrap()
        } else {
            ext
        };

        match ext {
            extensions::Extension::ElementMapper(ext) => {
                self.render_element_mapper_extension(buf, input, parsed, is_block, call, ext);
            }
            extensions::Extension::Alias { .. } => unreachable!(),
        }
    }

    fn render_inline_extension(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        call: &crate::compiling::CompiledItemCall<'a>,
    ) {
        let is_block = false;

        let Some(ext) = self.inline_extension_map.get(call.name) else {
            self.render_call_error(buf, RenderCallErrorInput {
                is_block,
                call_type: CallType::Extension,
                call_name: call.name,
                error: CallError::UnknownCallee(call.name),
                #[cfg(feature = "block-id")]
                block_id: None,
            });
            return;
        };

        let ext = if let extensions::Extension::Alias { to } = ext {
            self.inline_extension_map.get(to).unwrap()
        } else {
            ext
        };

        match ext {
            extensions::Extension::ElementMapper(ext) => {
                self.render_element_mapper_extension(buf, input, parsed, is_block, call, ext);
            }
            extensions::Extension::Alias { .. } => unreachable!(),
        }
    }

    fn render_element_mapper_extension(
        &self,
        buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        is_block: bool,
        call: &compiling::CompiledItemCall<'a>,
        ext: &extensions::ExtensionElementMapper<'a>,
    ) {
        // 不记别名。
        let mut seen_params: HashSet<Vec<u8>> = HashSet::new();
        // 不记别名。
        let mut seen_verbatim_params: HashSet<Vec<u8>> = HashSet::new();

        let mut attrs: Vec<(Vec<u8>, &[u8])> = vec![];
        let mut content: Vec<u8> = vec![];

        let mut bad: Option<CallErrorBadParameters> = None;
        let mut bad_verbatim: Option<CallErrorBadParameters> = None;

        for (key, value) in &call.arguments {
            self.process_element_mapper_extension_argument(
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

        for (key, value) in &call.verbatim_arguments {
            self.process_element_mapper_extension_verbatim_argument(
                &mut attrs,
                ProcessBlockElementMapperExtensionVerbatimArgumentParameters {
                    ext,
                    key,
                    value,
                    seen: &mut seen_verbatim_params,
                    bad: &mut bad_verbatim,
                },
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
            self.render_call_error(buf, RenderCallErrorInput {
                is_block,
                call_type: CallType::Extension,
                call_name: call.name,
                error: CallError::BadParameters {
                    normal: bad.map(Box::new),
                    verbatim: bad_verbatim.map(Box::new),
                },
                #[cfg(feature = "block-id")]
                block_id: call.block_id,
            });
            return;
        }

        if let Some(variant) = ext.variant {
            attrs.push((b"variant".to_vec(), variant));
        }

        #[allow(unused_mut)]
        let mut attrs = attrs
            .iter()
            .map(|(k, v)| (k.as_slice(), *v))
            .collect::<Vec<_>>();

        #[cfg(feature = "block-id")]
        {
            if let Some(block_id) = call.block_id {
                let mut buffer = if self.with_block_id {
                    Some(itoa::Buffer::new())
                } else {
                    None
                };
                if let Some(buffer) = &mut buffer {
                    attrs.push((b"data-block-id", buffer.format(block_id.value()).as_bytes()));
                }

                crate::utils::render_eopening_tag(buf, ext.tag_name, &attrs);
            } else {
                crate::utils::render_eopening_tag(buf, ext.tag_name, &attrs);
            }
        }
        #[cfg(not(feature = "block-id"))]
        {
            crate::utils::render_eopening_tag(buf, ext.tag_name, &attrs);
        }

        buf.extend(&content);
        crate::utils::render_closing_tag(buf, ext.tag_name);
    }

    fn process_element_mapper_extension_argument(
        &self,
        content_buf: &mut Vec<u8>,
        input: &'a [u8],
        parsed: &[Event],
        params: ProcessBlockElementMapperExtensionArgumentParameters<'a, '_>,
    ) {
        let key_vec = params.key.to_vec();
        let Some((key_real, param)) = params.ext.get_real_parameter(&key_vec) else {
            let bad = params
                .bad
                .get_or_insert_with(CallErrorBadParameters::default);
            bad.unknown.insert(key_vec);
            return;
        };
        if params.seen.contains(key_real) {
            let bad = params
                .bad
                .get_or_insert_with(CallErrorBadParameters::default);
            bad.duplicated.insert(key_real.to_vec());
            return;
        } else {
            params.seen.insert(key_real.to_vec());
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

    fn process_element_mapper_extension_verbatim_argument(
        &self,
        attrs: &mut Vec<(Vec<u8>, &'a [u8])>,
        params: ProcessBlockElementMapperExtensionVerbatimArgumentParameters<'a, '_>,
    ) {
        let key_vec = params.key.to_vec();
        let Some((key_real, param)) = params.ext.get_real_verbatim_parameter(&key_vec) else {
            let bad = params
                .bad
                .get_or_insert_with(CallErrorBadParameters::default);
            bad.unknown.insert(key_vec);
            return;
        };
        if params.seen.contains(key_real) {
            let bad = params
                .bad
                .get_or_insert_with(CallErrorBadParameters::default);
            bad.duplicated.insert(key_real.to_vec());
            return;
        } else {
            params.seen.insert(key_real.to_vec());
        }
        if params.bad.is_some() {
            return;
        }

        match param.mapping_to {
            extensions::ExtensionElementMapperVerbatimParameterMappingTo::Attribute(attr) => {
                attrs.push((attr.to_vec(), params.value));
            }
        }
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

        let tag = if input.is_block {
            self.tag_name_map.block_call_error
        } else {
            self.tag_name_map.inline_call_error
        };

        #[cfg(feature = "block-id")]
        {
            if let Some(block_id) = input.block_id {
                let mut buffer = if self.with_block_id {
                    Some(itoa::Buffer::new())
                } else {
                    None
                };
                if let Some(buffer) = &mut buffer {
                    attrs.push((b"data-block-id", buffer.format(block_id.value()).as_bytes()));
                }

                crate::utils::render_empty_element(buf, tag, &attrs);
            } else {
                crate::utils::render_empty_element(buf, tag, &attrs);
            }
        }
        #[cfg(not(feature = "block-id"))]
        {
            crate::utils::render_empty_element(buf, tag, &attrs);
        }
    }
}

struct ProcessBlockElementMapperExtensionArgumentParameters<'a, 'b> {
    ext: &'b extensions::ExtensionElementMapper<'a>,
    key: &'b compiling::ArgumentKey<'a>,
    value: &'b Vec<CompiledItem<'b>>,
    seen: &'b mut HashSet<Vec<u8>>,
    bad: &'b mut Option<CallErrorBadParameters>,
}

struct ProcessBlockElementMapperExtensionVerbatimArgumentParameters<'a, 'b> {
    ext: &'b extensions::ExtensionElementMapper<'a>,
    key: &'b compiling::ArgumentKey<'a>,
    value: &'a [u8],
    seen: &'b mut HashSet<Vec<u8>>,
    bad: &'b mut Option<CallErrorBadParameters>,
}
