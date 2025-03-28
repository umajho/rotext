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
    pub fn get_real_parameter(&self, name: &[u8]) -> Option<&ExtensionElementMapperParameter> {
        self._get_real_parameter(name, false)
    }

    fn _get_real_parameter(
        &self,
        name: &[u8],
        is_in_recursion: bool,
    ) -> Option<&ExtensionElementMapperParameter> {
        let param = self.parameters.get(name)?;
        match param {
            ParameterWrapper::Real(param) => Some(param),
            ParameterWrapper::Alias(name) if !is_in_recursion => {
                self._get_real_parameter(name, true)
            }
            // 别名只应有一层。
            _ => unreachable!(),
        }
    }

    pub fn get_real_verbatim_parameter(
        &self,
        name: &[u8],
    ) -> Option<&ExtensionElementMapperVerbatimParameter> {
        self._get_real_verbatim_parameter(name, false)
    }

    fn _get_real_verbatim_parameter(
        &self,
        name: &[u8],
        is_in_recursion: bool,
    ) -> Option<&ExtensionElementMapperVerbatimParameter> {
        let param = self.verbatim_parameters.get(name)?;
        match param {
            ParameterWrapper::Real(param) => Some(param),
            ParameterWrapper::Alias(name) if !is_in_recursion => {
                self._get_real_verbatim_parameter(name, true)
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
    pub mapping_to_attribute: &'a [u8],
}
