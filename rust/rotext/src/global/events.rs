use crate::events::EventType;

/// 全局阶段的解析事件。
#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum Event {
    /// 有待下个阶段决定。
    Undetermined { start: usize, length: usize } = EventType::Undetermined as u32,

    /// 逐字文本转义。
    ///
    /// NOTE: 内容包含开头和结尾各可能存在的一个空格。这些空格在渲染时才删去。
    VerbatimEscaping {
        content_start: usize,
        content_length: usize,
        is_closed_forcedly: bool,
    } = EventType::VerbatimEscaping as u32,
    ///注释。
    Comment {
        content_start: usize,
        content_length: usize,
        is_closed_forcedly: bool,
    } = EventType::Comment as u32,
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }

    #[cfg(test)]
    pub fn content(&self, input: &[u8]) -> Option<String> {
        let slice = match *self {
            Event::Undetermined { start, length } => &input[start..start + length],
            Event::Comment {
                content_start,
                content_length,
                is_closed_forcedly: _,
            } => &input[content_start..content_start + content_length],
            Event::VerbatimEscaping {
                content_start,
                content_length,
                is_closed_forcedly: _,
            } => &input[content_start..content_start + content_length],
        };

        Some(String::from_utf8(slice.to_vec()).unwrap())
    }

    #[cfg(test)]
    pub fn assertion_flags(&self) -> Option<std::collections::HashSet<&'static str>> {
        let mut flags = std::collections::HashSet::new();

        match *self {
            Event::Undetermined {
                start: _,
                length: _,
            } => {}
            Event::Comment {
                content_start: _,
                content_length: _,
                is_closed_forcedly,
            } => {
                if is_closed_forcedly {
                    flags.insert("F");
                }
            }
            Event::VerbatimEscaping {
                content_start: _,
                content_length: _,
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
