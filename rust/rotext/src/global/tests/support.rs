/// event matcher
macro_rules! m {
    ($event_type:expr, $content:expr, $flags:expr) => {
        $crate::global::tests::support::EventMatcher {
            event_type: $event_type,
            content: $content,
            flags: $flags,
        }
    };
}

/// flag
macro_rules! f {
    ($($flag:expr),*) => {
        Some(HashSet::from([$($flag),*]))
    };
}

macro_rules! case {
    ($input:expr, $expected:expr) => {
        $crate::global::tests::support::Case {
            input: $input,
            expected: $expected,
        }
    };
}

pub(super) use case;
pub(super) use f;
pub(super) use m;

use super::*;

pub(super) struct Case {
    pub input: &'static str,
    pub expected: Vec<EventMatcher>,
}
impl test_support::Case for Case {
    fn assert_ok(&self) {
        let expected: Vec<EventMatcher> = if cfg!(not(feature = "line-number")) {
            self.expected
                .clone()
                .into_iter()
                .map(|mut m| {
                    if let Some(flags) = m.flags {
                        let new_flags: HashSet<&str> = flags
                            .into_iter()
                            .filter(|f| !f.starts_with(">ln:"))
                            .collect();
                        m.flags = if new_flags.is_empty() {
                            None
                        } else {
                            Some(new_flags)
                        };
                    }
                    m
                })
                .collect()
        } else {
            self.expected.clone()
        };

        let parser = Parser::new(self.input.as_bytes(), NewParserOptions::default());
        let actual: Vec<_> = parser
            .map(|ev| -> EventMatcher {
                let ev: Event = ev.into();
                EventMatcher {
                    event_type: EventType::from(ev.discriminant()),
                    content: ev.content(self.input.as_bytes()),
                    flags: ev.assertion_flags(),
                }
            })
            .collect();

        assert_eq!(expected, actual);
    }

    fn input(&self) -> String {
        self.input.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct EventMatcher {
    pub event_type: EventType,
    pub content: Option<&'static str>,
    pub flags: Option<HashSet<&'static str>>,
}
