use std::collections::{HashMap, HashSet};

use crate::executing::extensions::{
    Extension, ExtensionElementMapper, ExtensionElementMapperParameter,
    ExtensionElementMapperParameterMappingTo, ExtensionElementMapperVerbatimParameter,
    ExtensionElementMapperVerbatimParameterMappingTo, ParameterWrapper,
};

pub fn new_block_extension_map() -> HashMap<&'static [u8], Extension<'static>> {
    let mut map: HashMap<&'static [u8], Extension<'static>> = HashMap::new();

    map.insert(
        b"AllOptional",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"all-optional",
            variant: None,
            parameters: new_parametr_map(),
            required_parameters: HashSet::new(),
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(
        b"SomeNormalRequired",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"some-normal-required",
            variant: None,
            parameters: new_parametr_map(),
            required_parameters: {
                let mut set: HashSet<&'static [u8]> = HashSet::new();
                set.insert(b"1");
                set
            },
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(
        b"SomeVerbatimRequired",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"some-verbatim-required",
            variant: None,
            parameters: new_parametr_map(),
            required_parameters: HashSet::new(),
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: {
                let mut set: HashSet<&'static [u8]> = HashSet::new();
                set.insert(b"bar");
                set
            },
        })),
    );

    map.insert(
        b"WithVariant",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"with-variant",
            variant: Some(b"var"),
            parameters: new_parametr_map(),
            required_parameters: HashSet::new(),
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(b"Alias", Extension::Alias { to: b"AllOptional" });

    map
}

pub fn new_inline_extension_map() -> HashMap<&'static [u8], Extension<'static>> {
    let mut map: HashMap<&'static [u8], Extension<'static>> = HashMap::new();

    map.insert(
        b"AllOptional",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"i-all-optional",
            variant: None,
            parameters: new_parametr_map(),
            required_parameters: HashSet::new(),
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(
        b"SomeNormalRequired",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"i-some-normal-required",
            variant: None,
            parameters: new_parametr_map(),
            required_parameters: {
                let mut set: HashSet<&'static [u8]> = HashSet::new();
                set.insert(b"1");
                set
            },
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(
        b"SomeVerbatimRequired",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"i-some-verbatim-required",
            variant: None,
            parameters: new_parametr_map(),
            required_parameters: HashSet::new(),
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: {
                let mut set: HashSet<&'static [u8]> = HashSet::new();
                set.insert(b"bar");
                set
            },
        })),
    );

    map.insert(
        b"WithVariant",
        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: b"i-with-variant",
            variant: Some(b"var"),
            parameters: new_parametr_map(),
            required_parameters: HashSet::new(),
            verbatim_parameters: new_verbatim_parametr_map(),
            required_verbatim_parameters: HashSet::new(),
        })),
    );

    map.insert(b"Alias", Extension::Alias { to: b"AllOptional" });

    map
}

fn new_parametr_map()
-> HashMap<&'static [u8], ParameterWrapper<'static, ExtensionElementMapperParameter<'static>>> {
    let mut map: HashMap<&'static [u8], ParameterWrapper<ExtensionElementMapperParameter>> =
        HashMap::new();
    map.insert(
        b"1",
        ParameterWrapper::Real(ExtensionElementMapperParameter {
            mapping_to: ExtensionElementMapperParameterMappingTo::UnnamedSlot,
        }),
    );
    map.insert(
        b"2",
        ParameterWrapper::Real(ExtensionElementMapperParameter {
            mapping_to: ExtensionElementMapperParameterMappingTo::NamedSlot(b"second"),
        }),
    );
    map.insert(
        b"foo",
        ParameterWrapper::Real(ExtensionElementMapperParameter {
            mapping_to: ExtensionElementMapperParameterMappingTo::NamedSlot(b"foo"),
        }),
    );
    map.insert(b"alias", ParameterWrapper::Alias(b"1"));
    map
}

fn new_verbatim_parametr_map() -> HashMap<
    &'static [u8],
    ParameterWrapper<'static, ExtensionElementMapperVerbatimParameter<'static>>,
> {
    let mut map: HashMap<&'static [u8], ParameterWrapper<ExtensionElementMapperVerbatimParameter>> =
        HashMap::new();
    map.insert(
        b"1",
        ParameterWrapper::Real(ExtensionElementMapperVerbatimParameter {
            mapping_to: ExtensionElementMapperVerbatimParameterMappingTo::Attribute(b"first"),
        }),
    );
    map.insert(
        b"bar",
        ParameterWrapper::Real(ExtensionElementMapperVerbatimParameter {
            mapping_to: ExtensionElementMapperVerbatimParameterMappingTo::Attribute(b"bar"),
        }),
    );
    map.insert(
        b"baz",
        ParameterWrapper::Real(ExtensionElementMapperVerbatimParameter {
            mapping_to: ExtensionElementMapperVerbatimParameterMappingTo::Attribute(b"baz"),
        }),
    );
    map.insert(b"bar_alias", ParameterWrapper::Alias(b"bar"));
    map
}
