use crate::{common::Range, events::EventType};

#[repr(u32)]
pub enum EventFromBlockLevel {
    Unparsed(Range) = EventType::Unparsed as u32,
    LineFeed = EventType::LineFeed as u32,
    Text(Range) = EventType::Text as u32,
}

#[derive(Debug)]
#[repr(u32)]
pub enum Event {
    LineFeed = EventType::LineFeed as u32,
    Text(Range) = EventType::Text as u32,
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u32 {
        unsafe { *<*const _>::from(self).cast::<u32>() }
    }

    pub fn content<'a>(&self, input: &'a [u8]) -> Option<&'a str> {
        let result = match self {
            Event::LineFeed => return None,
            Event::Text(content) => content.content(input),
        };

        Some(result)
    }
}
