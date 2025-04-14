use std::collections::{HashMap, HashSet};

pub enum Extension<'a> {
    ElementMapper(Box<ExtensionElementMapper<'a>>),
    Alias { to: &'a [u8] },
}

pub struct ExtensionElementMapper<'a> {
    /// XXX: 调用者需自行确保标签的名称不会导致 XSS。
    pub tag_name: &'a [u8],
    /// 若存在，则渲染的元素会有一个 `variant` 属性，其值为这里的值。
    pub variant: Option<&'a [u8]>,
    pub parameters: HashMap<&'a [u8], ParameterWrapper<'a, ExtensionElementMapperParameter<'a>>>,
    pub required_parameters: HashSet<&'a [u8]>,
    pub verbatim_parameters:
        HashMap<&'a [u8], ParameterWrapper<'a, ExtensionElementMapperVerbatimParameter<'a>>>,
    pub required_verbatim_parameters: HashSet<&'a [u8]>,
}
impl ExtensionElementMapper<'_> {
    pub fn get_real_parameter<'a>(
        &'a self,
        name: &'a [u8],
    ) -> Option<(&'a [u8], &'a ExtensionElementMapperParameter<'a>)> {
        self._get_real_parameter(name, false)
    }

    fn _get_real_parameter<'a>(
        &'a self,
        name: &'a [u8],
        is_in_recursion: bool,
    ) -> Option<(&'a [u8], &'a ExtensionElementMapperParameter<'a>)> {
        let param = self.parameters.get(name)?;
        match param {
            ParameterWrapper::Real(param) => Some((name, param)),
            ParameterWrapper::Alias(real_name) if !is_in_recursion => {
                self._get_real_parameter(real_name, true)
            }
            // 别名只应有一层。
            _ => unreachable!(),
        }
    }

    pub fn get_real_verbatim_parameter<'a>(
        &'a self,
        name: &'a [u8],
    ) -> Option<(&'a [u8], &'a ExtensionElementMapperVerbatimParameter<'a>)> {
        self._get_real_verbatim_parameter(name, false)
    }

    fn _get_real_verbatim_parameter<'a>(
        &'a self,
        name: &'a [u8],
        is_in_recursion: bool,
    ) -> Option<(&'a [u8], &'a ExtensionElementMapperVerbatimParameter<'a>)> {
        let param = self.verbatim_parameters.get(name)?;
        match param {
            ParameterWrapper::Real(param) => Some((name, param)),
            ParameterWrapper::Alias(real_name) if !is_in_recursion => {
                self._get_real_verbatim_parameter(real_name, true)
            }
            // 别名只应有一层。
            _ => unreachable!(),
        }
    }
}

pub enum ParameterWrapper<'a, T> {
    Real(T),
    Alias(&'a [u8]),
}

pub struct ExtensionElementMapperParameter<'a> {
    pub mapping_to: ExtensionElementMapperParameterMappingTo<'a>,
}

pub enum ExtensionElementMapperParameterMappingTo<'a> {
    NamedSlot(&'a [u8]),
    UnnamedSlot,
}

pub struct ExtensionElementMapperVerbatimParameter<'a> {
    pub mapping_to: ExtensionElementMapperVerbatimParameterMappingTo<'a>,
}
pub enum ExtensionElementMapperVerbatimParameterMappingTo<'a> {
    Attribute(&'a [u8]),
}

#[cfg(any(test, feature = "test"))]
pub fn new_demo_block_extension_map_for_test() -> HashMap<&'static [u8], Extension<'static>> {
    let mut map: HashMap<&'static [u8], Extension<'static>> = HashMap::new();

    map.insert(
        b"Div",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"div",
            variant: None,
            parameters: HashMap::new(),
            required_parameters: HashSet::new(),
            verbatim_parameters: HashMap::new(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(
        b"Collapse",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"x-collapse",
            variant: None,
            parameters: {
                let mut map: HashMap<
                    &'static [u8],
                    ParameterWrapper<ExtensionElementMapperParameter>,
                > = HashMap::new();
                map.insert(
                    b"1",
                    ParameterWrapper::Real(ExtensionElementMapperParameter {
                        mapping_to: ExtensionElementMapperParameterMappingTo::UnnamedSlot,
                    }),
                );
                map
            },
            required_parameters: {
                let mut set: HashSet<&'static [u8]> = HashSet::new();
                set.insert(b"1");
                set
            },
            verbatim_parameters: {
                let mut map: HashMap<
                    &'static [u8],
                    ParameterWrapper<ExtensionElementMapperVerbatimParameter>,
                > = HashMap::new();
                map.insert(
                    b"title",
                    ParameterWrapper::Real(ExtensionElementMapperVerbatimParameter {
                        mapping_to: ExtensionElementMapperVerbatimParameterMappingTo::Attribute(
                            b"title",
                        ),
                    }),
                );
                map.insert("标题".as_bytes(), ParameterWrapper::Alias(b"title"));
                map.insert(
                    b"open",
                    ParameterWrapper::Real(ExtensionElementMapperVerbatimParameter {
                        mapping_to: ExtensionElementMapperVerbatimParameterMappingTo::Attribute(
                            b"open-by-default",
                        ),
                    }),
                );
                map.insert("展开".as_bytes(), ParameterWrapper::Alias(b"open"));
                map
            },
            required_verbatim_parameters: HashSet::new(),
        })),
    );
    map.insert("折叠".as_bytes(), Extension::Alias { to: b"Collapse" });
    for (name, variant, alias) in [
        (&b"Note"[..], &b"note"[..], "注".as_bytes()),
        (b"Tip", b"tip", "提示".as_bytes()),
        (b"Important", b"important", "重要".as_bytes()),
        (b"Warning", b"warning", "警告".as_bytes()),
        (b"Caution", b"caution", "当心".as_bytes()),
    ] {
        map.insert(
            name,
            Extension::ElementMapper(Box::new(ExtensionElementMapper {
                tag_name: b"x-callout",
                variant: Some(variant),
                parameters: {
                    let mut map: HashMap<
                        &'static [u8],
                        ParameterWrapper<ExtensionElementMapperParameter>,
                    > = HashMap::new();
                    map.insert(
                        b"1",
                        ParameterWrapper::Real(ExtensionElementMapperParameter {
                            mapping_to: ExtensionElementMapperParameterMappingTo::UnnamedSlot,
                        }),
                    );
                    map
                },
                required_parameters: {
                    let mut set: HashSet<&'static [u8]> = HashSet::new();
                    set.insert(b"1");
                    set
                },
                verbatim_parameters: HashMap::new(),
                required_verbatim_parameters: HashSet::new(),
            })),
        );
        map.insert(alias, Extension::Alias { to: name });
    }

    map
}

#[cfg(any(test, feature = "test"))]
pub fn new_demo_inline_extension_map_for_test() -> HashMap<&'static [u8], Extension<'static>> {
    let mut map: HashMap<&'static [u8], Extension<'static>> = HashMap::new();

    map.insert(
        b"Span",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"span",
            variant: None,
            parameters: HashMap::new(),
            required_parameters: HashSet::new(),
            verbatim_parameters: HashMap::new(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(
        b"ScratchOff",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"x-scratch-off",
            variant: None,
            parameters: {
                let mut map: HashMap<
                    &'static [u8],
                    ParameterWrapper<ExtensionElementMapperParameter>,
                > = HashMap::new();
                map.insert(
                    b"1",
                    ParameterWrapper::Real(ExtensionElementMapperParameter {
                        mapping_to: ExtensionElementMapperParameterMappingTo::UnnamedSlot,
                    }),
                );
                map
            },
            required_parameters: {
                let mut set: HashSet<&'static [u8]> = HashSet::new();
                set.insert(b"1");
                set
            },
            verbatim_parameters: HashMap::new(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );
    map.insert("刮开".as_bytes(), Extension::Alias { to: b"ScratchOff" });

    map
}
