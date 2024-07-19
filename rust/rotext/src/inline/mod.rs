mod events;

pub use events::{Event, EventFromBlockLevel};

use crate::blend;

pub struct Parser<'a> {
    input_stream: blend::WhileInlineSegment<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(input_stream: blend::WhileInlineSegment<'a>) -> Self {
        Self { input_stream }
    }

    #[inline(always)]
    fn next(&mut self) -> Option<Event> {
        let next = self.input_stream.next()?;

        let to_yield = match next {
            EventFromBlockLevel::Unparsed(content) => Event::Text(content),
            EventFromBlockLevel::LineFeed => Event::LineFeed,
            EventFromBlockLevel::Text(content) => Event::Text(content),
        };

        Some(to_yield)
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Self::Item> {
        self.next()
    }
}
