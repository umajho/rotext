pub struct SequenceGenerator {
    next: usize,
}
impl SequenceGenerator {
    pub fn new(initial_next: usize) -> Self {
        Self { next: initial_next }
    }
    pub fn next(&mut self) -> usize {
        let result = self.next;
        self.next += 1;
        result
    }
}

pub fn write_escaped_html_text(buf: &mut Vec<u8>, input: &[u8]) {
    for char in input {
        match *char {
            b'<' => buf.extend(b"&lt;"),
            b'&' => buf.extend(b"&amp;"),
            char => buf.push(char),
        }
    }
}

pub fn write_escaped_double_quoted_attribute_value(buf: &mut Vec<u8>, input: &[u8]) {
    for char in input {
        match *char {
            b'"' => buf.extend(b"&quot;"),
            b'&' => buf.extend(b"&amp;"),
            char => buf.push(char),
        }
    }
}
