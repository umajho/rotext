use crate::{block, inline};

#[derive(Debug)]
pub enum Event {
    Block(block::Event),
    Inline(inline::Event),
}

impl Event {
    #[cfg(test)]
    pub fn discriminant(&self) -> u32 {
        match self {
            Event::Block(ev) => ev.discriminant(),
            Event::Inline(ev) => ev.discriminant(),
        }
    }

    pub fn content<'a>(&self, input: &'a [u8]) -> Option<&'a str> {
        match self {
            Event::Block(ev) => ev.content(input),
            Event::Inline(ev) => ev.content(input),
        }
    }
}

pub type EventForInlineLevel = inline::EventFromBlockLevel;
