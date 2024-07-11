#[derive(Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum EventType {
    Undetermined = 1,

    VerbatimEscaping = 10001,
    Comment = 10002,
}

#[cfg(test)]
impl From<u32> for EventType {
    fn from(value: u32) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}
