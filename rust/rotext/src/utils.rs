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

#[cfg(feature = "block-id")]
pub fn write_usize(buf: &mut Vec<u8>, n: usize) {
    let mut buffer = itoa::Buffer::new();
    buf.extend(buffer.format(n).as_bytes());
}

pub fn render_escaped_html_text(buf: &mut Vec<u8>, input: &[u8]) {
    for char in input {
        match *char {
            b'<' => buf.extend(b"&lt;"),
            b'&' => buf.extend(b"&amp;"),
            char => buf.push(char),
        }
    }
}

pub fn render_escaped_double_quoted_attribute_value(buf: &mut Vec<u8>, input: &[u8]) {
    for char in input {
        match *char {
            b'"' => buf.extend(b"&quot;"),
            b'&' => buf.extend(b"&amp;"),
            char => buf.push(char),
        }
    }
}

pub fn render_empty_element(buf: &mut Vec<u8>, tag: &[u8], attrs: &[(&[u8], &[u8])]) {
    render_eopening_tag(buf, tag, attrs);
    render_closing_tag(buf, tag);
}

pub fn render_eopening_tag(buf: &mut Vec<u8>, tag: &[u8], attrs: &[(&[u8], &[u8])]) {
    buf.push(b'<');
    buf.extend(tag);
    for (name, value) in attrs {
        buf.push(b' ');
        buf.extend(*name);
        buf.extend(br#"=""#);
        render_escaped_double_quoted_attribute_value(buf, value);
        buf.push(b'"');
    }
    buf.push(b'>');
}

pub fn render_closing_tag(buf: &mut Vec<u8>, tag: &[u8]) {
    buf.extend(b"</");
    buf.extend(tag);
    buf.push(b'>');
}
