use std::collections::{HashMap, HashSet};

use rotext::executing::extensions::{
    Extension, ExtensionElementMapper, ExtensionElementMapperParameter,
    ExtensionElementMapperParameterMappingTo, ExtensionElementMapperVerbatimParameter,
    ExtensionElementMapperVerbatimParameterMappingTo, ParameterWrapper,
};

#[derive(Debug, serde::Deserialize)]
pub enum ExtensionInput {
    ElementMapper(ExtensionElementMapperInput),
    Alias { name: String, to: String },
}

#[derive(Debug, serde::Deserialize)]
pub struct ExtensionElementMapperInput {
    pub name: String,
    pub tag_name: String,
    pub variant: Option<String>,
    pub parameters: HashMap<String, ParameterWrapperInput<ExtensionElementMapperParameterInput>>,
    pub verbatim_parameters:
        HashMap<String, ParameterWrapperInput<ExtensionElementMapperVerbatimParameterInput>>,
}
impl ExtensionElementMapperInput {
    fn convert(&self) -> Extension {
        let mut parameters: HashMap<&[u8], ParameterWrapper<ExtensionElementMapperParameter>> =
            HashMap::new();
        let mut required_parameters: HashSet<&[u8]> = HashSet::new();
        for (key, value) in &self.parameters {
            parameters.insert(key.as_bytes(), value.convert());
            if value.is_required_non_alias() {
                required_parameters.insert(key.as_bytes());
            }
        }

        let mut verbatim_parameters: HashMap<
            &[u8],
            ParameterWrapper<ExtensionElementMapperVerbatimParameter>,
        > = HashMap::new();
        let mut required_verbatim_parameters: HashSet<&[u8]> = HashSet::new();
        for (key, value) in &self.verbatim_parameters {
            verbatim_parameters.insert(key.as_bytes(), value.convert());
            if value.is_required_non_alias() {
                required_verbatim_parameters.insert(key.as_bytes());
            }
        }

        Extension::ElementMapper(Box::new(ExtensionElementMapper {
            tag_name: self.tag_name.as_bytes(),
            variant: self.variant.as_ref().map(|x| x.as_bytes()),
            parameters,
            required_parameters,
            verbatim_parameters,
            required_verbatim_parameters,
        }))
    }
}

#[derive(Debug, serde::Deserialize)]
pub enum ParameterWrapperInput<T> {
    Real(T),
    Alias(String),
}
impl ParameterWrapperInput<ExtensionElementMapperParameterInput> {
    fn convert(&self) -> ParameterWrapper<ExtensionElementMapperParameter> {
        match self {
            ParameterWrapperInput::Real(value) => ParameterWrapper::Real(value.convert()),
            ParameterWrapperInput::Alias(value) => ParameterWrapper::Alias(value.as_bytes()),
        }
    }
    fn is_required_non_alias(&self) -> bool {
        match self {
            ParameterWrapperInput::Real(value) => !value.is_optional,
            ParameterWrapperInput::Alias(_) => false,
        }
    }
}
impl ParameterWrapperInput<ExtensionElementMapperVerbatimParameterInput> {
    fn convert(&self) -> ParameterWrapper<ExtensionElementMapperVerbatimParameter> {
        match self {
            ParameterWrapperInput::Real(value) => ParameterWrapper::Real(value.convert()),
            ParameterWrapperInput::Alias(value) => ParameterWrapper::Alias(value.as_bytes()),
        }
    }
    fn is_required_non_alias(&self) -> bool {
        match self {
            ParameterWrapperInput::Real(value) => !value.is_optional,
            ParameterWrapperInput::Alias(_) => false,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ExtensionElementMapperParameterInput {
    pub is_optional: bool,
    pub mapping_to: ExtensionElementMapperParameterMappingToInput,
}
impl ExtensionElementMapperParameterInput {
    fn convert(&self) -> ExtensionElementMapperParameter {
        ExtensionElementMapperParameter {
            mapping_to: self.mapping_to.convert(),
        }
    }
}

/// XXX: 不能用 `#[serde(tag = …)]`，否则编译出的 WASM 文件大小会猛涨近 60KB。
#[derive(Debug, serde::Deserialize)]
pub enum ExtensionElementMapperParameterMappingToInput {
    NamedSlot { name: String },
    UnnamedSlot,
}
impl ExtensionElementMapperParameterMappingToInput {
    fn convert(&self) -> ExtensionElementMapperParameterMappingTo {
        match self {
            ExtensionElementMapperParameterMappingToInput::NamedSlot { name } => {
                ExtensionElementMapperParameterMappingTo::NamedSlot(name.as_bytes())
            }
            ExtensionElementMapperParameterMappingToInput::UnnamedSlot => {
                ExtensionElementMapperParameterMappingTo::UnnamedSlot
            }
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct ExtensionElementMapperVerbatimParameterInput {
    pub is_optional: bool,
    pub mapping_to_attribute: String,
}
impl ExtensionElementMapperVerbatimParameterInput {
    fn convert(&self) -> ExtensionElementMapperVerbatimParameter {
        ExtensionElementMapperVerbatimParameter {
            mapping_to: ExtensionElementMapperVerbatimParameterMappingTo::Attribute(
                self.mapping_to_attribute.as_bytes(),
            ),
        }
    }
}

pub fn convert_to_extension_map(
    list: &Vec<ExtensionInput>,
) -> Result<HashMap<&[u8], Extension>, String> {
    let mut result: HashMap<&[u8], Extension> = HashMap::new();

    let mut alias_map: HashMap<&[u8], &[u8]> = HashMap::new();

    for item in list {
        match item {
            ExtensionInput::ElementMapper(item) => {
                result.insert(item.name.as_bytes(), item.convert());
            }
            ExtensionInput::Alias { name, to } => {
                alias_map.insert(name.as_bytes(), to.as_bytes());
            }
        }
    }

    for (name, to) in &alias_map {
        if !result.contains_key(to) {
            return Err(format!(
                "alias `{}`'s target `{}` either does not exist or is an alias",
                String::from_utf8_lossy(name),
                String::from_utf8_lossy(to)
            ));
        }
    }

    for (name, to) in alias_map {
        result.insert(name, Extension::Alias { to });
    }

    Ok(result)
}
