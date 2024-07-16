use crate::{common::Range, events::EventType};

/// 全局阶段的解析事件。
#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Event {
    /// 留给下个阶段解析。
    Unparsed(Range) = EventType::Unparsed as u32,

    /// 逐字文本转义。
    ///
    /// NOTE: 内容包含开头和结尾各可能存在的一个空格，省略上述空格的处理是在块级
    /// 阶段将 VerbatimEscaping 变换为 Text 时进行。
    VerbatimEscaping {
        content: Range,
        is_closed_forcedly: bool,
    } = EventType::VerbatimEscaping as u32,

    CarriageReturn {
        index: usize,
    } = EventType::CarriageReturn as u32,
    LineFeed {
        index: usize,
    } = EventType::LineFeed as u32,
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }

    #[cfg(test)]
    pub fn content<'a>(&self, input: &'a [u8]) -> Option<&'a str> {
        let result = match *self {
            Event::Unparsed(content) => content.content(input),
            Event::VerbatimEscaping {
                content,
                is_closed_forcedly: _,
            } => content.content(input),
            Event::CarriageReturn { .. } | Event::LineFeed { .. } => return None,
        };

        Some(result)
    }

    #[cfg(test)]
    pub fn assertion_flags(&self) -> Option<std::collections::HashSet<&'static str>> {
        let mut flags = std::collections::HashSet::new();

        match *self {
            Event::Unparsed(_) | Event::CarriageReturn { .. } | Event::LineFeed { .. } => {}
            Event::VerbatimEscaping {
                content: _,
                is_closed_forcedly,
            } => {
                if is_closed_forcedly {
                    flags.insert("F");
                }
            }
        }

        if !flags.is_empty() {
            Some(flags)
        } else {
            None
        }
    }
}
